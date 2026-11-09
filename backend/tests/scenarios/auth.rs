//! 계정·인증 시나리오 (features/01, api.md §2).

use axum::Router;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{
    PASSWORD, SEED_PASSWORD, TestResponse, empty_req, json_req, login_as, send, session_cookie,
    signup, test_app,
};

async fn login(app: &Router, login_id: &str, password: &str, remember: bool) -> TestResponse {
    let body = json!({ "loginId": login_id, "password": password, "remember": remember });
    send(app, json_req("POST", "/api/v1/auth/login", None, body)).await
}

async fn me(app: &Router, cookie: &str) -> TestResponse {
    send(app, empty_req("GET", "/api/v1/me", Some(cookie))).await
}

async fn patch_nickname(app: &Router, cookie: &str, nickname: &str) -> TestResponse {
    let body = json!({ "nickname": nickname });
    send(app, json_req("PATCH", "/api/v1/me", Some(cookie), body)).await
}

fn set_cookie(res: &TestResponse) -> &str {
    res.headers["set-cookie"].to_str().unwrap()
}

#[sqlx::test]
async fn auth_signup_login_logout(db: PgPool) {
    let app = test_app(db.clone());

    // 가입: 아이디는 소문자로 저장, 가입 즉시 로그인(7일 쿠키)
    let body = json!({
        "loginId": "Alice01", "password": PASSWORD, "nickname": "앨리스",
        "agreeTerms": true, "agreePrivacy": true, "over14": true,
    });
    let res = send(&app, json_req("POST", "/api/v1/auth/signup", None, body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_eq!(res.json["loginId"], "alice01");
    assert_eq!(res.json["nickname"], "앨리스");
    assert_eq!(res.json["balance"], 0);
    assert_eq!(res.json["hasPassword"], true);
    assert!(res.json["nicknameChangeableAt"].is_null());
    assert!(res.json["liveBroadcastId"].is_null());
    let cookie = set_cookie(&res);
    for attr in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/",
        "Max-Age=604800",
    ] {
        assert!(cookie.contains(attr), "{attr} in {cookie}");
    }
    let signup_session = session_cookie(&res);

    // DB에는 토큰 원문이 아니라 SHA-256 해시만 있다
    let token = signup_session.trim_start_matches("flutch_session=");
    let stored: Vec<u8> = sqlx::query_scalar("SELECT token_hash FROM sessions")
        .fetch_one(&db)
        .await
        .unwrap();
    use sha2::Digest;
    assert_eq!(stored, sha2::Sha256::digest(token.as_bytes()).to_vec());

    let res = me(&app, &signup_session).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["loginId"], "alice01");

    // 로그인: 대소문자 무시, remember면 30일 쿠키
    let res = login(&app, "ALICE01", PASSWORD, true).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert!(set_cookie(&res).contains("Max-Age=2592000"));
    let login_session = session_cookie(&res);
    assert_eq!(me(&app, &login_session).await.status, StatusCode::OK);

    // 틀린 비밀번호·없는 아이디는 같은 에러
    for (id, pw) in [("alice01", "wrongpass1"), ("nobody99", PASSWORD)] {
        let res = login(&app, id, pw, false).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
    }

    // 로그아웃: 그 세션만 즉시 무효, 쿠키 만료
    let res = send(
        &app,
        empty_req("POST", "/api/v1/auth/logout", Some(&login_session)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert!(set_cookie(&res).contains("Max-Age=0"));
    let res = me(&app, &login_session).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.json["error"]["code"], "UNAUTHENTICATED");
    assert_eq!(me(&app, &signup_session).await.status, StatusCode::OK);

    // 세션 없이 로그아웃해도 204
    let res = send(&app, empty_req("POST", "/api/v1/auth/logout", None)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
}

#[sqlx::test]
async fn auth_signup_validation_and_duplicates(db: PgPool) {
    let app = test_app(db.clone());
    signup(&app, "taken01", "있는닉").await;

    let res = send(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/signup",
            None,
            json!({
                "loginId": "a_b", "password": "taken01x", "nickname": "관리자",
                "agreeTerms": false, "agreePrivacy": true, "over14": false,
            }),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "VALIDATION_FAILED");
    let fields = res.json["error"]["fields"].as_object().unwrap();
    for f in ["loginId", "nickname", "agreeTerms", "over14"] {
        assert!(fields.contains_key(f), "{f}: {fields:?}");
    }
    assert!(!fields.contains_key("agreePrivacy"));

    // 아이디와 같은 비밀번호, 흔한 비밀번호
    for (login_id, password) in [("samepw12", "SAMEPW12"), ("common01", "1qaz2wsx")] {
        let res = send(
            &app,
            json_req(
                "POST",
                "/api/v1/auth/signup",
                None,
                json!({
                    "loginId": login_id, "password": password, "nickname": "새닉네임",
                    "agreeTerms": true, "agreePrivacy": true, "over14": true,
                }),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{password}");
        assert!(res.json["error"]["fields"]["password"].is_string());
    }

    // JSON 형식 오류도 api.md 에러 형식
    let res = send(
        &app,
        json_req("POST", "/api/v1/auth/signup", None, json!({ "loginId": 1 })),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "VALIDATION_FAILED");

    // 중복: 아이디·닉네임 대소문자 무시
    let dup = |login_id: &str, nickname: &str| {
        json_req(
            "POST",
            "/api/v1/auth/signup",
            None,
            json!({
                "loginId": login_id, "password": PASSWORD, "nickname": nickname,
                "agreeTerms": true, "agreePrivacy": true, "over14": true,
            }),
        )
    };
    let res = send(&app, dup("TAKEN01", "다른닉")).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_LOGIN_ID_TAKEN");
    signup(&app, "bob01", "Bobby").await;
    let res = send(&app, dup("bob02", "BOBBY")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");

    // 탈퇴한 아이디는 영구 사용 불가, 닉네임은 탈퇴 30일 후 해제
    sqlx::query(
        "UPDATE users SET withdrawn_at = now() - interval '29 days' WHERE login_id = 'bob01'",
    )
    .execute(&db)
    .await
    .unwrap();
    let res = send(&app, dup("bob01", "새밥")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_LOGIN_ID_TAKEN");
    let res = send(&app, dup("bob03", "bobby")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");
    sqlx::query(
        "UPDATE users SET withdrawn_at = now() - interval '31 days' WHERE login_id = 'bob01'",
    )
    .execute(&db)
    .await
    .unwrap();
    signup(&app, "bob03", "bobby").await;
    // 탈퇴 계정은 로그인 불가
    let res = login(&app, "bob01", PASSWORD, false).await;
    assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
}

#[sqlx::test]
async fn auth_lockout(db: PgPool) {
    let app = test_app(db.clone());
    signup(&app, "locky01", "잠금이").await;

    // 실패 4회 → 성공하면 횟수 0으로
    for _ in 0..4 {
        let res = login(&app, "locky01", "wrongpass1", false).await;
        assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
    }
    assert_eq!(
        login(&app, "locky01", PASSWORD, false).await.status,
        StatusCode::OK
    );

    // 연속 5회 실패 → 5분 잠금
    for _ in 0..4 {
        let res = login(&app, "locky01", "wrongpass1", false).await;
        assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
    }
    let res = login(&app, "locky01", "wrongpass1", false).await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(res.json["error"]["code"], "AUTH_LOCKED");
    let left = res.json["error"]["retryAfterSec"].as_i64().unwrap();
    assert!((290..=300).contains(&left), "{left}");
    assert!(res.headers.contains_key("retry-after"));
    assert!(
        res.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("5분")
    );

    // 잠금 중엔 비밀번호가 맞아도 거절
    let res = login(&app, "locky01", PASSWORD, false).await;
    assert_eq!(res.json["error"]["code"], "AUTH_LOCKED");

    // 5분이 지나면(잠금 시각을 과거로 옮김) 로그인되고 실패 횟수 0
    sqlx::query(
        "UPDATE users SET locked_until = now() - interval '1 second' WHERE login_id = 'locky01'",
    )
    .execute(&db)
    .await
    .unwrap();
    assert_eq!(
        login(&app, "locky01", PASSWORD, false).await.status,
        StatusCode::OK
    );
    let (count, locked): (i32, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as(
        "SELECT failed_login_count, locked_until FROM users WHERE login_id = 'locky01'",
    )
    .fetch_one(&db)
    .await
    .unwrap();
    assert_eq!((count, locked), (0, None));
}

#[sqlx::test]
async fn auth_password_change_revokes_other_sessions(db: PgPool) {
    let app = test_app(db);
    let current = signup(&app, "pwuser01", "비번바꿈").await;
    let other = session_cookie(&login(&app, "pwuser01", PASSWORD, false).await);
    let change = |cookie: &str, cur: &str, new: &str| {
        json_req(
            "PUT",
            "/api/v1/me/password",
            Some(cookie),
            json!({ "currentPassword": cur, "newPassword": new }),
        )
    };

    let res = send(&app, change(&current, "wrongpass1", "newpass456")).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "AUTH_WRONG_PASSWORD");

    let res = send(&app, change(&current, PASSWORD, "short1")).await;
    assert_eq!(res.json["error"]["code"], "VALIDATION_FAILED");
    assert!(res.json["error"]["fields"]["newPassword"].is_string());

    let res = send(&app, change(&current, PASSWORD, "newpass456")).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);

    // 현재 세션은 유지, 다른 기기 세션은 만료
    assert_eq!(me(&app, &current).await.status, StatusCode::OK);
    assert_eq!(me(&app, &other).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        login(&app, "pwuser01", PASSWORD, false).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&app, "pwuser01", "newpass456", false).await.status,
        StatusCode::OK
    );
}

#[sqlx::test]
async fn auth_nickname_change_30day_limit(db: PgPool) {
    let app = test_app(db.clone());
    let cookie = signup(&app, "nick01", "처음닉").await;
    signup(&app, "nick02", "남의닉").await;

    let res = patch_nickname(&app, &cookie, "공백 닉").await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(res.json["error"]["fields"]["nickname"].is_string());
    let res = patch_nickname(&app, &cookie, "남의닉").await;
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");

    // 가입 시 설정은 제한에 안 들어가므로 바로 바꿀 수 있다
    let res = patch_nickname(&app, &cookie, "바꾼닉").await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["nickname"], "바꾼닉");
    assert!(res.json["nicknameChangeableAt"].is_string());

    let res = patch_nickname(&app, &cookie, "또바꿈").await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_CHANGE_TOO_SOON");
    assert!(
        res.json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("30일")
    );

    // 30일이 지나면 다시 바꿀 수 있다
    sqlx::query(
        "UPDATE users SET nickname_changed_at = now() - interval '30 days 1 second' WHERE login_id = 'nick01'",
    )
    .execute(&db)
    .await
    .unwrap();
    let res = me(&app, &cookie).await;
    assert!(res.json["nicknameChangeableAt"].is_null());
    let res = patch_nickname(&app, &cookie, "또바꿈").await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["nickname"], "또바꿈");
}

#[sqlx::test]
async fn auth_requires_login_and_acts_only_on_own_account(db: PgPool) {
    let app = test_app(db.clone());

    // 비로그인·가짜 쿠키 → 401
    for cookie in [None, Some("flutch_session=forged")] {
        let res = send(&app, empty_req("GET", "/api/v1/me", cookie)).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        assert_eq!(res.json["error"]["code"], "UNAUTHENTICATED");
        let res = send(
            &app,
            json_req(
                "PATCH",
                "/api/v1/me",
                cookie,
                json!({ "nickname": "몰래바꿈" }),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
        let res = send(
            &app,
            json_req(
                "PUT",
                "/api/v1/me/password",
                cookie,
                json!({ "currentPassword": PASSWORD, "newPassword": "newpass456" }),
            ),
        )
        .await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    }

    // 변경은 세션 주인의 계정에만 적용된다(요청에 대상 id가 없다)
    let alice = signup(&app, "alice02", "앨리스둘").await;
    let bob = signup(&app, "bob04", "밥넷").await;
    let res = patch_nickname(&app, &bob, "밥바꿈").await;
    assert_eq!(res.json["nickname"], "밥바꿈");
    assert_eq!(me(&app, &alice).await.json["nickname"], "앨리스둘");

    // 만료된 세션은 401, 단 LIVE 송출 중이면 유지
    sqlx::query("UPDATE sessions SET expires_at = now() - interval '1 second'")
        .execute(&db)
        .await
        .unwrap();
    assert_eq!(me(&app, &alice).await.status, StatusCode::UNAUTHORIZED);
    let alice_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE login_id = 'alice02'")
        .fetch_one(&db)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO broadcasts (owner_id, status, title, category, started_at)
         VALUES ($1, 'LIVE', '방송', 'game', now())",
    )
    .bind(alice_id)
    .execute(&db)
    .await
    .unwrap();
    let res = me(&app, &alice).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.json["liveBroadcastId"].is_i64());
    assert_eq!(me(&app, &bob).await.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn auth_rate_limits_by_ip(db: PgPool) {
    let app = test_app(db);
    let from = |mut req: axum::http::Request<axum::body::Body>, ip: &str| {
        req.headers_mut().insert(
            "x-forwarded-for",
            format!("{ip}, 10.0.0.1").parse().unwrap(),
        );
        req
    };

    // 가입: 같은 IP 1시간 5회
    for i in 0..6 {
        let body = json!({
            "loginId": format!("rate{i:02}"), "password": PASSWORD, "nickname": format!("레이트{i}"),
            "agreeTerms": true, "agreePrivacy": true, "over14": true,
        });
        let req = from(
            json_req("POST", "/api/v1/auth/signup", None, body),
            "1.2.3.4",
        );
        let res = send(&app, req).await;
        if i < 5 {
            assert_eq!(res.status, StatusCode::CREATED, "{i}: {}", res.json);
        } else {
            assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
            assert_eq!(res.json["error"]["code"], "RATE_LIMITED");
            assert!(res.json["error"]["retryAfterSec"].as_i64().unwrap() > 0);
            assert!(res.headers.contains_key("retry-after"));
        }
    }

    // 로그인: 같은 IP 1분 20회. 다른 IP는 영향 없음
    let attempt = |ip: &str| {
        let body = json!({ "loginId": "nobody99", "password": PASSWORD, "remember": false });
        from(json_req("POST", "/api/v1/auth/login", None, body), ip)
    };
    for _ in 0..20 {
        let res = send(&app, attempt("5.6.7.8")).await;
        assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
    }
    let res = send(&app, attempt("5.6.7.8")).await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(res.json["error"]["code"], "RATE_LIMITED");
    let res = send(&app, attempt("9.9.9.9")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
}

#[sqlx::test]
async fn auth_test_login_seed_accounts(db: PgPool) {
    let app = test_app(db.clone());

    let (cookie, id) = login_as(&app, "viewer").await;
    let res = me(&app, &cookie).await;
    assert_eq!(res.json["nickname"], "시청자");
    assert_eq!(res.json["loginId"], "viewer");

    // 다시 눌러도 같은 계정, 시드는 3개만 생긴다
    assert_eq!(login_as(&app, "viewer").await.1, id);
    let seeds: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(seeds, 3);

    // 시드 계정은 SEED_PASSWORD로도 로그인된다
    let res = login(&app, "streamer1", SEED_PASSWORD, false).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["nickname"], "스트리머1");

    let res = send(
        &app,
        json_req(
            "POST",
            "/api/v1/auth/test-login",
            None,
            json!({ "account": "alice01" }),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(res.json["error"]["fields"]["account"].is_string());
}

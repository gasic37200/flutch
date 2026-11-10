//! 계정·인증 시나리오 (features/01, api.md §2).

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

use crate::common::{
    FakeKakao, PASSWORD, SEED_PASSWORD, TestResponse, empty_req, json_req, login_as, send,
    session_cookie, signup, test_app, test_app_with_kakao,
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

#[sqlx::test]
async fn auth_seed_account_cannot_be_changed(db: PgPool) {
    let app = test_app(db);
    let (cookie, _) = login_as(&app, "streamer1").await;

    let res = patch_nickname(&app, &cookie, "바꾸기시도").await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.json["error"]["code"], "AUTH_SEED_ACCOUNT_LOCKED");

    let body = json!({ "currentPassword": SEED_PASSWORD, "newPassword": "Another1234" });
    let res = send(
        &app,
        json_req("PUT", "/api/v1/me/password", Some(&cookie), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.json["error"]["code"], "AUTH_SEED_ACCOUNT_LOCKED");

    // 세션도 비밀번호도 그대로
    assert_eq!(me(&app, &cookie).await.status, StatusCode::OK);
    assert_eq!(
        login(&app, "streamer1", SEED_PASSWORD, false).await.status,
        StatusCode::OK
    );
}

#[sqlx::test]
async fn auth_test_login_is_rate_limited_by_ip(db: PgPool) {
    let app = test_app(db);
    let mut last = StatusCode::OK;
    for _ in 0..21 {
        let mut req = json_req(
            "POST",
            "/api/v1/auth/test-login",
            None,
            json!({ "account": "viewer" }),
        );
        req.headers_mut()
            .insert("x-forwarded-for", "7.7.7.7, 10.0.0.1".parse().unwrap());
        last = send(&app, req).await.status;
    }
    assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
}

fn withdraw_req(
    cookie: Option<&str>,
    password: &str,
    confirm: bool,
) -> axum::http::Request<axum::body::Body> {
    let body = json!({ "password": password, "confirmForfeit": confirm });
    json_req("DELETE", "/api/v1/me", cookie, body)
}

async fn user_id(db: &PgPool, login_id: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE login_id = $1")
        .bind(login_id)
        .fetch_one(db)
        .await
        .unwrap()
}

/// 충전 주문을 만들어 `amount`비트를 적립한다.
async fn fund(db: &PgPool, user_id: i64, amount: i64) {
    let order: i64 = sqlx::query_scalar(
        "INSERT INTO payment_orders (order_id, user_id, purpose, amount, expires_at)
         VALUES (gen_random_uuid()::text, $1, 'charge', $2, now() + interval '30 minutes') RETURNING id",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(db)
    .await
    .unwrap();
    let mut tx = db.begin().await.unwrap();
    flutch::wallet::credit_charge(&mut tx, user_id, amount, order)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

async fn insert_broadcast(db: &PgPool, owner_id: i64, status: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO broadcasts (owner_id, status, title, category, started_at)
         VALUES ($1, $2, '방송', 'game', CASE WHEN $2 = 'READY' THEN NULL ELSE now() END) RETURNING id",
    )
    .bind(owner_id)
    .bind(status)
    .fetch_one(db)
    .await
    .unwrap()
}

#[sqlx::test]
async fn auth_withdraw_forfeits_balance(db: PgPool) {
    let app = test_app(db.clone());
    let current = signup(&app, "quit01", "떠날사람").await;
    let other = session_cookie(&login(&app, "quit01", PASSWORD, true).await);
    let uid = user_id(&db, "quit01").await;
    fund(&db, uid, 5000).await;
    let ready = insert_broadcast(&db, uid, "READY").await;

    let res = send(&app, withdraw_req(Some(&current), "wrongpass1", true)).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "AUTH_WRONG_PASSWORD");

    // 잔액이 있으면 확인 없이는 거절, 아무것도 바뀌지 않는다
    let res = send(&app, withdraw_req(Some(&current), PASSWORD, false)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_WITHDRAW_CONFIRM_FORFEIT");
    assert_eq!(
        res.json["error"]["message"],
        "남은 5,000비트는 사라지고 되돌릴 수 없어요."
    );
    let res = me(&app, &current).await;
    assert_eq!(res.json["balance"], 5000);
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM broadcasts WHERE id = $1)")
        .bind(ready)
        .fetch_one(&db)
        .await
        .unwrap();
    assert!(exists, "rejected withdraw must keep READY broadcast");

    let res = send(&app, withdraw_req(Some(&current), PASSWORD, true)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    assert!(set_cookie(&res).contains("Max-Age=0"));

    // 원장: 소멸 1행(-5000, 잔액 0), 지갑 0, 정합성 유지
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT type::text, amount, balance_after FROM ledger_entries WHERE user_id = $1 ORDER BY id",
    )
    .bind(uid)
    .fetch_all(&db)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            ("charge".to_owned(), 5000, 5000),
            ("withdraw_forfeit".to_owned(), -5000, 0)
        ]
    );
    let balance: i64 = sqlx::query_scalar("SELECT balance FROM wallets WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(balance, 0);
    assert_eq!(
        flutch::wallet::mismatched_wallets(&db).await.unwrap(),
        Vec::<i64>::new()
    );

    // 세션 전부 무효, 로그인 불가, 아이디 재가입 불가, READY 방송 삭제, 스트리머 페이지 404
    for cookie in [&current, &other] {
        assert_eq!(me(&app, cookie).await.status, StatusCode::UNAUTHORIZED);
    }
    let sessions: i64 = sqlx::query_scalar("SELECT count(*) FROM sessions WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(sessions, 0);
    let res = login(&app, "quit01", PASSWORD, false).await;
    assert_eq!(res.json["error"]["code"], "AUTH_INVALID_CREDENTIALS");
    let signup_body = |login_id: &str, nickname: &str| {
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
    let res = send(&app, signup_body("quit01", "새사람")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_LOGIN_ID_TAKEN");
    let res = send(
        &app,
        empty_req("GET", &format!("/api/v1/broadcasts/{ready}"), None),
    )
    .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    let res = send(
        &app,
        empty_req("GET", &format!("/api/v1/users/{uid}"), None),
    )
    .await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);

    // 닉네임은 탈퇴 30일 동안 보류, 그 뒤 해제
    let res = send(&app, signup_body("quit02", "떠날사람")).await;
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");
    sqlx::query("UPDATE users SET withdrawn_at = now() - interval '31 days' WHERE id = $1")
        .bind(uid)
        .execute(&db)
        .await
        .unwrap();
    signup(&app, "quit02", "떠날사람").await;
}

#[sqlx::test]
async fn auth_withdraw_rules(db: PgPool) {
    let app = test_app(db.clone());

    // 비로그인 401
    let res = send(&app, withdraw_req(None, PASSWORD, true)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);

    // 테스트 계정은 비밀번호가 맞아도 403
    let (seed, _) = login_as(&app, "viewer").await;
    let res = send(&app, withdraw_req(Some(&seed), SEED_PASSWORD, true)).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.json["error"]["code"], "AUTH_SEED_ACCOUNT_LOCKED");
    assert_eq!(me(&app, &seed).await.status, StatusCode::OK);

    // LIVE 중이면 409(확인했어도), 끝나면 탈퇴 가능. 잔액 0이면 원장 없이 탈퇴.
    let cookie = signup(&app, "live01", "방송중").await;
    let uid = user_id(&db, "live01").await;
    let live = insert_broadcast(&db, uid, "LIVE").await;
    let res = send(&app, withdraw_req(Some(&cookie), PASSWORD, true)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_WITHDRAW_WHILE_LIVE");
    assert_eq!(me(&app, &cookie).await.status, StatusCode::OK);

    sqlx::query("UPDATE broadcasts SET status = 'ENDED', ended_at = now() WHERE id = $1")
        .bind(live)
        .execute(&db)
        .await
        .unwrap();
    let res = send(&app, withdraw_req(Some(&cookie), PASSWORD, false)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    let ledger: i64 = sqlx::query_scalar("SELECT count(*) FROM ledger_entries WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(ledger, 0);
    assert_eq!(me(&app, &cookie).await.status, StatusCode::UNAUTHORIZED);
    // 끝난 방송은 남고 소유자는 "탈퇴한 사용자"
    let res = send(
        &app,
        empty_req("GET", &format!("/api/v1/broadcasts/{live}"), None),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["owner"]["nickname"], "탈퇴한 사용자");
}

// ---- 카카오 로그인 (T28) ----

const KAKAO_ID: i64 = 777_001;

/// `Set-Cookie`들 중 이름이 `name`인 것의 `이름=값`(없으면 None).
fn cookie_named(res: &TestResponse, name: &str) -> Option<String> {
    res.headers
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with(&format!("{name}=")))
        .and_then(|c| c.split(';').next())
        .map(str::to_owned)
}

fn location(res: &TestResponse) -> &str {
    res.headers["location"].to_str().unwrap()
}

/// start를 부르고 `(Location, state 쿠키 "이름=값", state 값)`.
async fn kakao_start(app: &Router) -> (String, String, String) {
    let res = send(app, empty_req("GET", "/api/v1/auth/kakao/start", None)).await;
    assert_eq!(res.status, StatusCode::FOUND);
    let cookie = cookie_named(&res, "flutch_kakao_state").expect("state cookie");
    let state = cookie.split_once('=').unwrap().1.to_owned();
    (location(&res).to_owned(), cookie, state)
}

async fn kakao_callback(
    app: &Router,
    state_cookie: Option<&str>,
    state_param: &str,
) -> TestResponse {
    let uri = format!("/api/v1/auth/kakao/callback?code=authcode&state={state_param}");
    send(app, empty_req("GET", &uri, state_cookie)).await
}

/// 카카오 로그인 → 닉네임 설정 화면까지 가서 임시 가입 쿠키("이름=값")를 돌려준다.
async fn kakao_pending_cookie(app: &Router) -> String {
    let (_, cookie, state) = kakao_start(app).await;
    let res = kakao_callback(app, Some(&cookie), &state).await;
    assert_eq!(res.status, StatusCode::FOUND);
    assert_eq!(location(&res), "/signup/kakao");
    cookie_named(&res, "flutch_kakao_signup").expect("signup cookie")
}

fn kakao_signup_req(
    cookie: Option<&str>,
    nickname: &str,
    agree: bool,
) -> axum::http::Request<axum::body::Body> {
    let body = json!({
        "nickname": nickname, "agreeTerms": agree, "agreePrivacy": true, "over14": true,
    });
    json_req("POST", "/api/v1/auth/kakao/signup", cookie, body)
}

#[sqlx::test]
async fn auth_kakao_new_user_signup(db: PgPool) {
    let kakao = Arc::new(FakeKakao::default());
    kakao.set_profile(KAKAO_ID, Some("카카오 곰"));
    let app = test_app_with_kakao(db.clone(), kakao.clone());

    // start: 닉네임만 요청, 임시 state 쿠키는 카카오 라우트에만, 콜백 주소는 설정값
    let res = send(&app, empty_req("GET", "/api/v1/auth/kakao/start", None)).await;
    assert_eq!(res.status, StatusCode::FOUND);
    let url = location(&res);
    assert!(
        url.starts_with("https://kauth.kakao.com/oauth/authorize?"),
        "{url}"
    );
    for part in [
        "client_id=kakao-rest",
        "response_type=code",
        "scope=profile_nickname",
        "redirect_uri=https%3A%2F%2Fflutch.test%2Fapi%2Fv1%2Fauth%2Fkakao%2Fcallback",
    ] {
        assert!(url.contains(part), "{part} in {url}");
    }
    let set = res.headers["set-cookie"].to_str().unwrap();
    for attr in [
        "HttpOnly",
        "Secure",
        "SameSite=Lax",
        "Path=/api/v1/auth/kakao",
        "Max-Age=600",
    ] {
        assert!(set.contains(attr), "{attr} in {set}");
    }
    let state_cookie = cookie_named(&res, "flutch_kakao_state").unwrap();
    let state = state_cookie.split_once('=').unwrap().1;
    assert!(url.contains(&format!("state={state}")));

    // callback: 처음이면 세션 없이 닉네임 설정 화면으로, 닉네임 후보는 규칙에 맞게 줄인다
    let res = kakao_callback(&app, Some(&state_cookie), state).await;
    assert_eq!(res.status, StatusCode::FOUND);
    assert_eq!(location(&res), "/signup/kakao");
    assert!(cookie_named(&res, "flutch_session").is_none());
    assert_eq!(*kakao.codes.lock().unwrap(), ["authcode"]);
    let pending = cookie_named(&res, "flutch_kakao_signup").unwrap();
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(count, 0, "가입 전에는 사용자를 만들지 않는다");

    let res = send(
        &app,
        empty_req("GET", "/api/v1/auth/kakao/signup", Some(&pending)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["nickname"], "카카오곰");

    // signup: 닉네임 확정 → 201 Me + 세션, 지갑 행 생성
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰", true)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert!(res.json["loginId"].is_null());
    assert_eq!(res.json["nickname"], "카카오곰");
    assert_eq!(res.json["hasPassword"], false);
    assert_eq!(res.json["balance"], 0);
    let session = cookie_named(&res, "flutch_session").unwrap();
    assert_eq!(me(&app, &session).await.json["id"], res.json["id"]);
    let uid = res.json["id"].as_i64().unwrap();
    let wallets: i64 = sqlx::query_scalar("SELECT count(*) FROM wallets WHERE user_id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(wallets, 1);
    let stored: i64 = sqlx::query_scalar("SELECT kakao_id FROM users WHERE id = $1")
        .bind(uid)
        .fetch_one(&db)
        .await
        .unwrap();
    assert_eq!(stored, KAKAO_ID);

    // 같은 임시 쿠키로 한 번 더(다른 탭): 이미 가입된 카카오 계정이라 409
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰둘", true)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_KAKAO_ALREADY_REGISTERED");

    // 카카오 계정은 비밀번호를 바꿀 수 없다
    let body = json!({ "currentPassword": PASSWORD, "newPassword": "newpass1234" });
    let res = send(
        &app,
        json_req("PUT", "/api/v1/me/password", Some(&session), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
}

#[sqlx::test]
async fn auth_kakao_existing_user_login(db: PgPool) {
    let kakao = Arc::new(FakeKakao::default());
    kakao.set_profile(KAKAO_ID, Some("카카오곰"));
    let app = test_app_with_kakao(db.clone(), kakao);
    let pending = kakao_pending_cookie(&app).await;
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰", true)).await;
    let uid = res.json["id"].as_i64().unwrap();

    // 다시 로그인하면 닉네임 화면 없이 같은 계정으로 세션 발급 후 홈으로
    let (_, cookie, state) = kakao_start(&app).await;
    let res = kakao_callback(&app, Some(&cookie), &state).await;
    assert_eq!(res.status, StatusCode::FOUND);
    assert_eq!(location(&res), "/");
    assert!(cookie_named(&res, "flutch_kakao_signup").is_none());
    let session = cookie_named(&res, "flutch_session").unwrap();
    let res = me(&app, &session).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["id"], uid);
}

#[sqlx::test]
async fn auth_kakao_state_mismatch_rejected(db: PgPool) {
    let kakao = Arc::new(FakeKakao::default());
    kakao.set_profile(KAKAO_ID, Some("카카오곰"));
    let app = test_app_with_kakao(db, kakao.clone());
    let (_, cookie, state) = kakao_start(&app).await;

    // state 값이 쿠키와 다름 / 쿠키 없음 / 동의 취소(code 없음) → 로그인 화면 에러, 카카오 호출 없음
    let bad = kakao_callback(&app, Some(&cookie), "forged").await;
    let no_cookie = kakao_callback(&app, None, &state).await;
    let denied = send(
        &app,
        empty_req(
            "GET",
            "/api/v1/auth/kakao/callback?error=access_denied",
            Some(&cookie),
        ),
    )
    .await;
    for res in [bad, no_cookie, denied] {
        assert_eq!(res.status, StatusCode::FOUND);
        assert_eq!(location(&res), "/login?kakao=error");
        assert!(cookie_named(&res, "flutch_session").is_none());
        assert!(cookie_named(&res, "flutch_kakao_signup").is_none());
    }
    assert!(kakao.codes.lock().unwrap().is_empty());

    // 카카오가 토큰 교환을 거절해도 같은 화면
    *kakao.profile.lock().unwrap() = None;
    let res = kakao_callback(&app, Some(&cookie), &state).await;
    assert_eq!(location(&res), "/login?kakao=error");
}

#[sqlx::test]
async fn auth_kakao_signup_rules_and_forged_cookie(db: PgPool) {
    let kakao = Arc::new(FakeKakao::default());
    kakao.set_profile(KAKAO_ID, Some("카카오곰"));
    let app = test_app_with_kakao(db.clone(), kakao);
    signup(&app, "other01", "먼저온사람").await;
    let pending = kakao_pending_cookie(&app).await;

    // 임시 쿠키가 없거나 위조·변조되면 401
    let res = send(&app, empty_req("GET", "/api/v1/auth/kakao/signup", None)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.json["error"]["code"], "AUTH_KAKAO_SIGNUP_EXPIRED");
    let (name_value, sig) = pending.rsplit_once('.').unwrap();
    // 서명 자리를 바꾸거나, 다른 카카오 id로 바꿔 서명을 새로 만들 수 없게 한다
    let tampered_sig = format!("{name_value}.{}", sig.chars().rev().collect::<String>());
    let other_id = pending.replacen("=777001.", "=1.", 1);
    for bad in [
        tampered_sig,
        other_id,
        "flutch_kakao_signup=x.y.z.w".to_owned(),
    ] {
        let res = send(&app, kakao_signup_req(Some(&bad), "카카오곰", true)).await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{bad}");
    }

    // 닉네임 규칙·동의 검사(아이디 가입과 같은 규칙), 아무것도 만들어지지 않는다
    for (nickname, agree) in [
        ("관리자", true),
        ("a", true),
        ("카카오 곰", true),
        ("카카오곰", false),
    ] {
        let res = send(&app, kakao_signup_req(Some(&pending), nickname, agree)).await;
        assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY, "{nickname}");
    }
    let res = send(&app, kakao_signup_req(Some(&pending), "먼저온사람", true)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");
    let kakao_users: i64 =
        sqlx::query_scalar("SELECT count(*) FROM users WHERE kakao_id IS NOT NULL")
            .fetch_one(&db)
            .await
            .unwrap();
    assert_eq!(kakao_users, 0);
}

#[sqlx::test]
async fn auth_kakao_withdraw_unlinks(db: PgPool) {
    let kakao = Arc::new(FakeKakao::default());
    kakao.set_profile(KAKAO_ID, Some("카카오곰"));
    let app = test_app_with_kakao(db.clone(), kakao.clone());
    let pending = kakao_pending_cookie(&app).await;
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰", true)).await;
    let session = cookie_named(&res, "flutch_session").unwrap();
    let uid = res.json["id"].as_i64().unwrap();
    fund(&db, uid, 3000).await;

    // 비밀번호 없이 잔액 소멸 확인만으로 탈퇴한다. 확인이 없으면 409.
    let body = json!({ "confirmForfeit": false });
    let res = send(&app, json_req("DELETE", "/api/v1/me", Some(&session), body)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_WITHDRAW_CONFIRM_FORFEIT");
    assert!(kakao.unlinked.lock().unwrap().is_empty());

    let body = json!({ "confirmForfeit": true });
    let res = send(&app, json_req("DELETE", "/api/v1/me", Some(&session), body)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT, "{}", res.json);
    assert_eq!(*kakao.unlinked.lock().unwrap(), [KAKAO_ID]);
    assert_eq!(me(&app, &session).await.status, StatusCode::UNAUTHORIZED);

    // 같은 카카오 계정으로 다시 로그인하면 새 계정 가입 화면. 탈퇴 닉네임은 30일 보류.
    let pending = kakao_pending_cookie(&app).await;
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰", true)).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.json["error"]["code"], "AUTH_NICKNAME_TAKEN");
    let res = send(&app, kakao_signup_req(Some(&pending), "카카오곰새", true)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    assert_ne!(res.json["id"].as_i64().unwrap(), uid);
}

#[sqlx::test]
async fn auth_password_account_withdraw_needs_password(db: PgPool) {
    let app = test_app(db);
    let cookie = signup(&app, "quit02", "떠날사람").await;
    let body = json!({ "confirmForfeit": true });
    let res = send(&app, json_req("DELETE", "/api/v1/me", Some(&cookie), body)).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.json["error"]["code"], "AUTH_WRONG_PASSWORD");
}

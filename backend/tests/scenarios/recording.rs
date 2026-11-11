//! 녹화·다시보기 시나리오 (features/08, api.md §8). Egress·저장소는 fake, 완료 통보는 `egress_ended` 웹훅으로 흉내 낸다.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::Router;
use axum::http::StatusCode;
use flutch::AppState;
use flutch::recording::cleanup;
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::common::{
    FakeLiveKit, FakeStore, empty_req, json_req, login_as, recording_config, send, test_state_with,
};
use crate::live::{create, go_live, participant, webhook};

const B: &str = "/api/v1/broadcasts";
const R: &str = "/api/v1/recordings";
const TAB: &str = "tab-aaaaaaaa";

struct Env {
    app: Router,
    state: Arc<AppState>,
    lk: Arc<FakeLiveKit>,
    store: Arc<FakeStore>,
}

fn setup(db: PgPool, max_total_bytes: i64) -> Env {
    let lk = Arc::new(FakeLiveKit::default());
    let store = Arc::new(FakeStore::default());
    let mut config = recording_config();
    config.max_total_bytes = max_total_bytes;
    let state = test_state_with(db, lk.clone(), store.clone(), Some(config));
    Env {
        app: flutch::app(state.clone()),
        state,
        lk,
        store,
    }
}

/// 송출 시작 + 송출자 접속 웹훅(녹화는 여기서 시작된다).
async fn live(env: &Env, cookie: &str, uid: i64, id: i64) {
    let res = go_live(&env.app, Some(cookie), id, json!({ "tabId": TAB })).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let identity = format!("u:{uid}:{TAB}");
    webhook(
        &env.app,
        participant("participant_joined", id, &identity, true),
    )
    .await;
}

async fn end(env: &Env, cookie: &str, id: i64) {
    let res = send(
        &env.app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

async fn recording_id(env: &Env, id: i64) -> i64 {
    let res = send(&env.app, empty_req("GET", &format!("{B}/{id}"), None)).await;
    res.json["recording"]["id"].as_i64().unwrap()
}

async fn egress_id(env: &Env, rid: i64) -> String {
    sqlx::query_scalar("SELECT egress_id FROM recordings WHERE id = $1")
        .bind(rid)
        .fetch_one(&env.state.db)
        .await
        .unwrap()
}

/// LiveKit `egress_ended` 웹훅(protojson: int64는 문자열, duration은 나노초).
fn egress_ended(egress_id: &str, status: &str, size: i64, secs: i64) -> Value {
    json!({
        "event": "egress_ended", "id": "EV_egress",
        "egressInfo": {
            "egressId": egress_id, "roomName": "broadcast-1", "status": status,
            "fileResults": [{ "filename": "x.mp4", "size": size.to_string(), "duration": (secs * 1_000_000_000).to_string() }],
        },
    })
}

/// 방송 하나를 처음부터 ready 녹화까지 만든다. `(방송 id, 녹화 id)`.
async fn make_ready(env: &Env, cookie: &str, uid: i64, size: i64) -> (i64, i64) {
    let id = create(&env.app, cookie).await;
    live(env, cookie, uid, id).await;
    end(env, cookie, id).await;
    let rid = recording_id(env, id).await;
    let eg = egress_id(env, rid).await;
    webhook(&env.app, egress_ended(&eg, "EGRESS_COMPLETE", size, 65)).await;
    (id, rid)
}

async fn get(env: &Env, path: &str) -> crate::common::TestResponse {
    send(&env.app, empty_req("GET", path, None)).await
}

async fn status_of(env: &Env, rid: i64) -> (String, Option<String>) {
    sqlx::query_as("SELECT status, object_key FROM recordings WHERE id = $1")
        .bind(rid)
        .fetch_one(&env.state.db)
        .await
        .unwrap()
}

#[sqlx::test]
async fn recording_saved_after_end_survives_restart(db: PgPool) {
    let env = setup(db.clone(), i64::MAX);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    let id = create(&env.app, &cookie).await;
    live(&env, &cookie, uid, id).await;
    // 같은 송출자의 접속 웹훅이 반복돼도 Egress는 한 번만 시작한다
    webhook(
        &env.app,
        participant("participant_joined", id, &format!("u:{uid}:{TAB}"), true),
    )
    .await;
    let key = format!("recordings/{id}.mp4");
    assert_eq!(
        *env.lk.started_recordings.lock().unwrap(),
        vec![(format!("broadcast-{id}"), key.clone())]
    );

    let rid = recording_id(&env, id).await;
    let res = get(&env, &format!("{R}/{rid}")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.json["status"], "recording");
    assert!(res.json["videoUrl"].is_null());
    // ready가 아니면 목록에 없다
    assert!(
        get(&env, R).await.json["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    end(&env, &cookie, id).await;
    assert_eq!(*env.lk.stopped_recordings.lock().unwrap(), vec!["EG_1"]);
    let ev = egress_ended("EG_1", "EGRESS_COMPLETE", 12_345_678, 65);
    webhook(&env.app, ev.clone()).await;
    webhook(&env.app, ev).await; // 반복 통보는 무해

    // 재시작: 같은 DB로 새 앱 인스턴스
    let restarted = setup(db, i64::MAX);
    let res = get(&restarted, &format!("{R}/{rid}")).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    assert_eq!(res.json["status"], "ready");
    assert_eq!(res.json["durationSec"], 65);
    assert_eq!(res.json["videoUrl"], format!("https://cdn.test/rec/{key}"));
    assert_eq!(res.json["owner"]["id"], uid);
    assert!(res.json["expiresAt"].is_string());
    let list = get(&restarted, R).await.json;
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["durationSec"], 65);
    let by_owner = get(
        &restarted,
        &format!("{R}?userId={uid}&category=game&q=송출"),
    )
    .await;
    assert_eq!(by_owner.json["items"].as_array().unwrap().len(), 1);
    let other = get(&restarted, &format!("{R}?category=music")).await;
    assert!(other.json["items"].as_array().unwrap().is_empty());
    assert_eq!(
        get(&restarted, &format!("{R}?limit=x")).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[sqlx::test]
async fn recording_failure_does_not_break_broadcast(db: PgPool) {
    let env = setup(db, i64::MAX);
    let (c1, u1) = login_as(&env.app, "streamer1").await;
    let (c2, u2) = login_as(&env.app, "streamer2").await;

    // Egress를 시작하지 못해도 방송은 정상
    env.lk.recording_start_fails.store(true, Ordering::SeqCst);
    let a = create(&env.app, &c1).await;
    live(&env, &c1, u1, a).await;
    let rid_a = recording_id(&env, a).await;
    assert_eq!(status_of(&env, rid_a).await.0, "failed");
    assert_eq!(get(&env, &format!("{B}/{a}")).await.json["status"], "LIVE");
    end(&env, &c1, a).await;
    let res = get(&env, &format!("{B}/{a}")).await;
    assert_eq!(res.json["status"], "ENDED");
    assert_eq!(res.json["recording"]["status"], "failed");

    // Egress가 실패로 끝나도 방송은 정상 ENDED, 다시보기는 없다
    env.lk.recording_start_fails.store(false, Ordering::SeqCst);
    let b = create(&env.app, &c2).await;
    live(&env, &c2, u2, b).await;
    end(&env, &c2, b).await;
    let rid_b = recording_id(&env, b).await;
    let eg = egress_id(&env, rid_b).await;
    webhook(&env.app, json!({ "event": "egress_ended", "egressInfo": { "egressId": eg, "status": "EGRESS_FAILED", "error": "boom" } })).await;
    assert_eq!(get(&env, &format!("{B}/{b}")).await.json["status"], "ENDED");
    assert_eq!(status_of(&env, rid_b).await.0, "failed");
    let res = get(&env, &format!("{R}/{rid_b}")).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.json["videoUrl"].is_null());
    assert!(
        get(&env, R).await.json["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[sqlx::test]
async fn recording_deleted_with_broadcast_hide(db: PgPool) {
    let env = setup(db, i64::MAX);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    let (id, rid) = make_ready(&env, &cookie, uid, 1000).await;
    assert_eq!(
        get(&env, &format!("{R}/{rid}")).await.status,
        StatusCode::OK
    );

    // 파일 삭제가 실패해도 방송 삭제는 성공하고, 목록·상세엔 나오지 않는다. 파일은 표시가 남아 다시 시도된다
    env.store.delete_fails.store(true, Ordering::SeqCst);
    let res = send(
        &env.app,
        empty_req("DELETE", &format!("{B}/{id}"), Some(&cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&env, &format!("{R}/{rid}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert!(
        get(&env, R).await.json["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let (status, key) = status_of(&env, rid).await;
    assert_eq!((status.as_str(), key.is_some()), ("deleted", true));
    assert!(env.store.deleted.lock().unwrap().is_empty());

    env.store.delete_fails.store(false, Ordering::SeqCst);
    cleanup(&env.state).await.unwrap();
    assert_eq!(
        *env.store.deleted.lock().unwrap(),
        vec![format!("recordings/{id}.mp4")]
    );
    assert_eq!(status_of(&env, rid).await, ("deleted".into(), None));

    // 정상 경로: 삭제와 함께 파일도 바로 지워진다
    let (id2, rid2) = make_ready(&env, &cookie, uid, 1000).await;
    let res = send(
        &env.app,
        empty_req("DELETE", &format!("{B}/{id2}"), Some(&cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(status_of(&env, rid2).await, ("deleted".into(), None));
    assert_eq!(env.store.deleted.lock().unwrap().len(), 2);
}

#[sqlx::test]
async fn recording_retention_cleanup(db: PgPool) {
    // 합계 100바이트까지
    let env = setup(db.clone(), 100);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    let (_, old) = make_ready(&env, &cookie, uid, 10).await;
    let (_, mid) = make_ready(&env, &cookie, uid, 60).await;
    let (_, new) = make_ready(&env, &cookie, uid, 60).await;

    // 30일이 지난 것(시간 조작)
    sqlx::query("UPDATE recordings SET expires_at = now() - interval '1 second' WHERE id = $1")
        .bind(old)
        .execute(&db)
        .await
        .unwrap();
    cleanup(&env.state).await.unwrap();

    // 만료 → 삭제, 용량 초과 → 오래된 것부터 삭제, 가장 최신만 남는다
    assert_eq!(status_of(&env, old).await.0, "deleted");
    assert_eq!(status_of(&env, mid).await.0, "deleted");
    assert_eq!(status_of(&env, new).await.0, "ready");
    assert_eq!(env.store.deleted.lock().unwrap().len(), 2);
    let res = get(&env, &format!("{R}/{old}")).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.json["error"]["code"], "RECORDING_EXPIRED");
    let list = get(&env, R).await.json;
    assert_eq!(list["items"].as_array().unwrap().len(), 1);
    assert_eq!(list["items"][0]["id"], new);

    // 종료 통보가 끝내 오지 않은 녹화는 실패로 정리
    let id = create(&env.app, &cookie).await;
    live(&env, &cookie, uid, id).await;
    let rid = recording_id(&env, id).await;
    sqlx::query("UPDATE recordings SET created_at = now() - interval '5 hours' WHERE id = $1")
        .bind(rid)
        .execute(&db)
        .await
        .unwrap();
    cleanup(&env.state).await.unwrap();
    assert_eq!(status_of(&env, rid).await.0, "failed");
}

#[sqlx::test]
async fn recording_off_without_record_flag(db: PgPool) {
    let env = setup(db, i64::MAX);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    let res = send(
        &env.app,
        json_req(
            "POST",
            B,
            Some(&cookie),
            json!({ "title": "녹화 없음", "category": "game", "record": false }),
        ),
    )
    .await;
    let id = res.json["id"].as_i64().unwrap();
    live(&env, &cookie, uid, id).await;
    end(&env, &cookie, id).await;
    assert!(env.lk.started_recordings.lock().unwrap().is_empty());
    assert!(get(&env, &format!("{B}/{id}")).await.json["recording"].is_null());
}

#[sqlx::test]
async fn recording_views_once_per_day(db: PgPool) {
    let env = setup(db.clone(), i64::MAX);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    let (_, rid) = make_ready(&env, &cookie, uid, 1000).await;
    let path = format!("{R}/{rid}/views");
    let view = |cookie: Option<&str>, body: Value| {
        let req = json_req("POST", &path, cookie, body);
        send(&env.app, req)
    };
    let views = || async {
        get(&env, &format!("{R}/{rid}")).await.json["views"]
            .as_i64()
            .unwrap()
    };

    assert_eq!(
        view(None, json!({})).await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        view(None, json!({ "guestId": "guest-aaaa1111" }))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        view(None, json!({ "guestId": "guest-aaaa1111" }))
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(views().await, 1);
    view(None, json!({ "guestId": "guest-bbbb2222" })).await;
    view(Some(&cookie), json!({})).await;
    view(Some(&cookie), json!({})).await;
    assert_eq!(views().await, 3);

    // 다음 날(KST)이면 다시 센다
    sqlx::query("UPDATE recording_views SET date_kst = date_kst - 1")
        .execute(&db)
        .await
        .unwrap();
    view(None, json!({ "guestId": "guest-aaaa1111" })).await;
    assert_eq!(views().await, 4);
    assert_eq!(
        send(
            &env.app,
            json_req(
                "POST",
                &format!("{R}/999999/views"),
                None,
                json!({ "guestId": "guest-aaaa1111" })
            )
        )
        .await
        .status,
        StatusCode::NOT_FOUND
    );
}

const CLIPS: &str = "/api/v1/clips";

/// 방송 시작 + `secs`초 뒤에 SUPER CLUTCH가 발동했다고 기록한다(시간 조작). super_clutch id.
async fn super_at(db: &PgPool, broadcast_id: i64, secs: i32) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO super_clutches (broadcast_id, triggered_at, expires_at, rotation_order)
         SELECT id, started_at + $2 * interval '1 second', started_at + $2 * interval '1 second' + interval '5 minutes', 0
         FROM broadcasts WHERE id = $1 RETURNING id",
    )
    .bind(broadcast_id)
    .bind(secs)
    .fetch_one(db)
    .await
    .unwrap()
}

#[sqlx::test]
async fn clips_from_ready_recordings(db: PgPool) {
    let env = setup(db.clone(), i64::MAX);
    let (cookie, uid) = login_as(&env.app, "streamer1").await;
    // 녹화 길이 65초인 ready 방송 네 개 + 아직 녹화 중인 방송 하나
    let (b1, r1) = make_ready(&env, &cookie, uid, 10).await;
    let (b2, _) = make_ready(&env, &cookie, uid, 10).await;
    let (b3, r3) = make_ready(&env, &cookie, uid, 10).await;
    let (b4, _) = make_ready(&env, &cookie, uid, 10).await;
    let b5 = create(&env.app, &cookie).await;
    live(&env, &cookie, uid, b5).await;
    sqlx::query("UPDATE broadcasts SET started_at = now() - interval '1 hour'")
        .execute(&db)
        .await
        .unwrap();

    let early = super_at(&db, b1, 10).await; // 30초 전이 방송 시작 전 → startSec 0
    let late = super_at(&db, b1, 60).await; // 30 + 45초가 영상(65초)을 넘음 → 35초로 자름
    let beyond = super_at(&db, b1, 100).await; // 녹화가 끝난 뒤 발동 → 구간 없음
    let hidden = super_at(&db, b2, 40).await;
    let deleted = super_at(&db, b3, 40).await;
    let rec = super_at(&db, b5, 40).await;
    let shown = super_at(&db, b4, 40).await;
    sqlx::query("UPDATE broadcasts SET hidden_at = now() WHERE id = $1")
        .bind(b2)
        .execute(&db)
        .await
        .unwrap();
    sqlx::query("UPDATE recordings SET status = 'deleted' WHERE id = $1")
        .bind(r3)
        .execute(&db)
        .await
        .unwrap();
    let _ = (hidden, deleted, rec, beyond, r1);

    // 누구나 볼 수 있고 최신순, ready만
    let res = get(&env, CLIPS).await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
    let items = res.json["items"].as_array().unwrap();
    let ids: Vec<i64> = items.iter().map(|c| c["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, vec![shown, late, early]);
    assert_eq!(
        (
            items[0]["startSec"].as_i64(),
            items[0]["durationSec"].as_i64()
        ),
        (Some(10), Some(45))
    );
    assert_eq!(
        (
            items[1]["startSec"].as_i64(),
            items[1]["durationSec"].as_i64()
        ),
        (Some(30), Some(35))
    );
    assert_eq!(
        (
            items[2]["startSec"].as_i64(),
            items[2]["durationSec"].as_i64()
        ),
        (Some(0), Some(45))
    );
    assert_eq!(
        items[2]["videoUrl"],
        format!("https://cdn.test/rec/recordings/{b1}.mp4")
    );
    assert_eq!(items[2]["broadcast"]["id"], b1);

    // 페이지
    let page = get(&env, &format!("{CLIPS}?limit=2")).await.json;
    assert_eq!(page["items"].as_array().unwrap().len(), 2);
    let next = page["nextCursor"].as_str().unwrap();
    let page2 = get(&env, &format!("{CLIPS}?limit=2&cursor={next}"))
        .await
        .json;
    assert_eq!(page2["items"][0]["id"], early);
    assert!(page2["nextCursor"].is_null());
}

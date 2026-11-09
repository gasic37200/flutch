//! WebSocket 이벤트 시나리오 (api.md §11, architecture §6). 서버를 임시 포트에 띄우고 실제 WebSocket으로 접속한다.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use flutch::AppState;
use flutch::broadcast::{BroadcastEnded, EndReason};
use flutch::realtime::{Event, WsEvent};
use futures_util::StreamExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Error as WsError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::common::{FakeLiveKit, ORIGIN, empty_req, json_req, login_as, send, test_state};

const B: &str = "/api/v1/broadcasts";
pub type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// REST용 앱과 WebSocket용 서버 주소. 둘은 같은 상태(허브)를 쓴다.
async fn setup(db: PgPool) -> (Router, Arc<AppState>, SocketAddr) {
    let state = test_state(db, Arc::new(FakeLiveKit::default()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = flutch::app(state.clone());
    tokio::spawn(async move { axum::serve(listener, server).await.unwrap() });
    (flutch::app(state.clone()), state, addr)
}

pub async fn connect(
    addr: SocketAddr,
    id: i64,
    origin: Option<&str>,
    cookie: Option<&str>,
) -> Result<Ws, u16> {
    let mut req = format!("ws://{addr}{B}/{id}/events")
        .into_client_request()
        .unwrap();
    if let Some(o) = origin {
        req.headers_mut().insert("origin", o.parse().unwrap());
    }
    if let Some(c) = cookie {
        req.headers_mut().insert("cookie", c.parse().unwrap());
    }
    match tokio_tungstenite::connect_async(req).await {
        Ok((ws, _)) => Ok(ws),
        Err(WsError::Http(res)) => Err(res.status().as_u16()),
        Err(e) => panic!("ws connect: {e}"),
    }
}

/// 다음 텍스트 메시지(JSON). 5초 안에 없으면 실패.
pub async fn next_event(ws: &mut Ws) -> Value {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("ws event timeout")
            .expect("ws closed")
            .expect("ws error");
        if let Message::Text(t) = msg {
            return serde_json::from_str(&t).unwrap();
        }
    }
}

async fn create(app: &Router, cookie: &str) -> i64 {
    let body = json!({ "title": "실시간 테스트", "category": "game" });
    let res = send(app, json_req("POST", B, Some(cookie), body)).await;
    assert_eq!(res.status, StatusCode::CREATED, "{}", res.json);
    res.json["id"].as_i64().unwrap()
}

async fn go_live(app: &Router, cookie: &str, id: i64) {
    let body = json!({ "tabId": "tab-aaaaaaaa" });
    let res = send(
        app,
        json_req("POST", &format!("{B}/{id}/live"), Some(cookie), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{}", res.json);
}

fn ended(reason: EndReason) -> WsEvent {
    WsEvent::BroadcastEnded(BroadcastEnded {
        ended_at: chrono::Utc::now(),
        peak_viewers: 0,
        duration_sec: 0,
        reason,
        recording: None,
    })
}

#[sqlx::test]
async fn ws_origin_rejected(db: PgPool) {
    let (app, _, addr) = setup(db).await;
    let (cookie, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &cookie).await;

    let other = connect(addr, id, Some("https://evil.example"), Some(&cookie)).await;
    assert_eq!(other.err(), Some(403));
    assert_eq!(connect(addr, id, None, None).await.err(), Some(403));
    assert!(connect(addr, id, Some(ORIGIN), None).await.is_ok());
}

#[sqlx::test]
async fn ws_unknown_or_hidden_broadcast_404(db: PgPool) {
    let (app, _, addr) = setup(db).await;
    let (cookie, _) = login_as(&app, "streamer1").await;
    assert_eq!(
        connect(addr, 999_999, Some(ORIGIN), None).await.err(),
        Some(404)
    );

    // LIVE 중 삭제 → 종료 후 숨김
    let id = create(&app, &cookie).await;
    go_live(&app, &cookie, id).await;
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{id}"), Some(&cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(connect(addr, id, Some(ORIGIN), None).await.err(), Some(404));
}

#[sqlx::test]
async fn ws_receives_broadcast_updated_and_ended(db: PgPool) {
    let (app, _, addr) = setup(db).await;
    let (cookie, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &cookie).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    // READY → LIVE
    go_live(&app, &cookie, id).await;
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.updated");
    assert_eq!(ev["data"]["id"], id);
    assert_eq!(ev["data"]["status"], "LIVE");

    // PATCH
    let body = json!({ "title": "바뀐 제목" });
    let res = send(
        &app,
        json_req("PATCH", &format!("{B}/{id}"), Some(&cookie), body),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.updated");
    assert_eq!(ev["data"]["title"], "바뀐 제목");
    assert!(
        ev["data"].get("viewer").is_none(),
        "공개 이벤트에 요청자 정보 없음"
    );

    // 종료
    let res = send(
        &app,
        empty_req("DELETE", &format!("{B}/{id}/live"), Some(&cookie)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    let ev = next_event(&mut ws).await;
    assert_eq!(ev["type"], "broadcast.ended");
    assert_eq!(ev["data"]["reason"], "owner");
    assert_eq!(ev["data"]["endedAt"], res.json["endedAt"]);
    assert!(ev["data"]["durationSec"].as_i64().unwrap() >= 0);
    assert_eq!(ev["data"]["peakViewers"], 0);
    assert!(ev["data"]["recording"].is_null());
}

#[sqlx::test]
async fn ws_split_event_hides_anonymous_from_public(db: PgPool) {
    let (app, state, addr) = setup(db).await;
    let (owner_cookie, _) = login_as(&app, "streamer1").await;
    let (viewer_cookie, _) = login_as(&app, "viewer").await;
    let id = create(&app, &owner_cookie).await;
    let mut owner = connect(addr, id, Some(ORIGIN), Some(&owner_cookie))
        .await
        .unwrap();
    let mut viewer = connect(addr, id, Some(ORIGIN), Some(&viewer_cookie))
        .await
        .unwrap();
    let mut guest = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    // split: 공개형(익명) / 소유자형(실명) 대신 reason으로 두 형태를 구분한다
    let hub = &state.realtime;
    hub.publish(
        id,
        Event::split(ended(EndReason::Owner), ended(EndReason::Deleted)),
    );
    // owner_only 다음에 공개 표시 이벤트: 소유자 아닌 연결은 owner_only를 건너뛰고 표시 이벤트를 받아야 한다
    hub.publish(id, Event::owner_only(ended(EndReason::MaxDuration)));
    hub.publish(id, Event::public(ended(EndReason::Disconnect)));

    let reasons = |evs: [Value; 2]| evs.map(|e| e["data"]["reason"].as_str().unwrap().to_owned());
    let o = [next_event(&mut owner).await, next_event(&mut owner).await];
    assert_eq!(reasons(o), ["deleted", "max_duration"]);
    assert_eq!(next_event(&mut owner).await["data"]["reason"], "disconnect");
    for ws in [&mut viewer, &mut guest] {
        let e = [next_event(ws).await, next_event(ws).await];
        assert_eq!(reasons(e), ["owner", "disconnect"]);
    }
}

#[sqlx::test]
async fn ws_lagged_sends_resync(db: PgPool) {
    let (app, state, addr) = setup(db).await;
    let (cookie, _) = login_as(&app, "streamer1").await;
    let id = create(&app, &cookie).await;
    let mut ws = connect(addr, id, Some(ORIGIN), None).await.unwrap();

    // 채널 크기(256)보다 많이 한꺼번에 발행 → 연결이 밀려 Lagged
    for _ in 0..1000 {
        state
            .realtime
            .publish(id, Event::public(ended(EndReason::Owner)));
    }
    let mut seen = 0;
    loop {
        let ev = next_event(&mut ws).await;
        if ev["type"] == "resync" {
            assert_eq!(ev["data"], json!({}));
            break;
        }
        seen += 1;
        assert!(seen < 1000, "resync를 받지 못함");
    }
    // 끊지 않고 계속 받는다
    state
        .realtime
        .publish(id, Event::public(ended(EndReason::Deleted)));
    loop {
        if next_event(&mut ws).await["data"]["reason"] == "deleted" {
            break;
        }
    }
}

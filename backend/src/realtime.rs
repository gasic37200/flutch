//! 방송별 실시간 이벤트 허브 + WebSocket `GET /broadcasts/{id}/events` (architecture §6, api.md §11).
//! 방송마다 크기 제한 `tokio::broadcast` 채널(구독자가 0이 되면 정리), **DB 커밋 후** `Hub::publish`.
//! 이벤트는 공개형·소유자형을 발행 시점에 같이 만들고, 연결이 자기 종류(소유자인지)로 고른다.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use axum::Router;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::broadcast::{self, error::RecvError};
use ts_rs::TS;

use crate::AppState;
use crate::broadcast::{Broadcast, BroadcastEnded, RecordingRef};
use crate::chat::Chat;
use crate::error::AppError;
use crate::extract::OptionalUser;
use crate::middleware::origin_allowed;
use crate::wallet::DonationCreated;

/// 방송당 버퍼. 느린 연결이 이만큼 밀리면 `Lagged` → `resync`.
const CHANNEL_CAPACITY: usize = 256;
/// 끊김 감지: 이 주기로 ping, 다음 주기까지 pong이 없으면 끊는다.
const PING_INTERVAL: Duration = Duration::from_secs(30);

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/broadcasts/{id}/events", get(events))
}

/// WebSocket 메시지 `{type, data}`. 이벤트를 추가하는 task가 여기에 변형 하나를 더한다(api.md §11 표의 이름 그대로).
#[derive(Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum WsEvent {
    #[serde(rename = "broadcast.updated")]
    BroadcastUpdated(Broadcast),
    #[serde(rename = "broadcast.ended")]
    BroadcastEnded(BroadcastEnded),
    /// 송출자 끊김, 60초 유예 시작
    #[serde(rename = "broadcast.reconnecting")]
    BroadcastReconnecting { since: DateTime<Utc> },
    #[serde(rename = "broadcast.resumed")]
    BroadcastResumed {},
    /// 자동 종료 10분 전(소유자만)
    #[serde(rename = "broadcast.duration_warning")]
    BroadcastDurationWarning {
        #[serde(rename = "endsAt")]
        ends_at: DateTime<Utc>,
    },
    /// 시청자 수(사용자 단위, 소유자 제외)
    #[serde(rename = "viewer.count")]
    ViewerCount { count: i64 },
    #[serde(rename = "chat.created")]
    ChatCreated(Chat),
    #[serde(rename = "chat.updated")]
    ChatUpdated(Chat),
    #[serde(rename = "chat.deleted")]
    ChatDeleted { id: i64 },
    /// 녹화 상태 변경(다시보기 준비 완료·실패)
    #[serde(rename = "recording.updated")]
    RecordingUpdated(RecordingRef),
    /// split으로만 발행(`wallet::publish_donation`) — 공개형은 익명 donor=null
    #[serde(rename = "donation.created")]
    DonationCreated(DonationCreated),
    /// 소유자가 표시 중인 후원 알림을 넘김
    #[serde(rename = "donation.skipped")]
    DonationSkipped { id: i64 },
    /// 서버 채널 지연으로 이벤트를 놓침 → 상세·채팅을 다시 조회
    #[serde(rename = "resync")]
    Resync {},
}

/// 발행할 이벤트. 필드가 비공개라 아래 세 생성자로만 만든다 — 소유자형이 공개 연결로 갈 길이 없다.
pub struct Event {
    /// 소유자가 아닌 연결(익명 포함)에 보낼 메시지. `None`이면 보내지 않음.
    public: Option<Utf8Bytes>,
    /// 소유자 연결에 보낼 메시지.
    owner: Option<Utf8Bytes>,
}

impl Event {
    /// 모든 연결에 같은 내용.
    pub fn public(e: WsEvent) -> Self {
        let m = encode(&e);
        Self {
            public: m.clone(),
            owner: m,
        }
    }

    /// 방송 소유자 연결만.
    pub fn owner_only(e: WsEvent) -> Self {
        Self {
            public: None,
            owner: encode(&e),
        }
    }

    /// 소유자 연결엔 `owner`, 나머지엔 `public`(예: 익명 후원자 실명을 뺀 형태).
    pub fn split(public: WsEvent, owner: WsEvent) -> Self {
        Self {
            public: encode(&public),
            owner: encode(&owner),
        }
    }

    fn for_connection(&self, is_owner: bool) -> Option<&Utf8Bytes> {
        if is_owner {
            self.owner.as_ref()
        } else {
            self.public.as_ref()
        }
    }
}

fn encode(e: &WsEvent) -> Option<Utf8Bytes> {
    match serde_json::to_string(e) {
        Ok(s) => Some(s.into()),
        Err(err) => {
            // 우리 타입 직렬화라 사실상 없음. 이벤트 누락은 허용되므로 로그만 남기고 버린다.
            tracing::error!(error = %err, "ws event encode failed");
            None
        }
    }
}

type Channels = HashMap<i64, broadcast::Sender<Arc<Event>>>;

#[derive(Default)]
pub struct Hub {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    channels: Mutex<Channels>,
    connections: AtomicUsize,
}

impl Inner {
    fn channels(&self) -> MutexGuard<'_, Channels> {
        // 잠금 중 패닉이 나도 맵은 일관적이다(삽입·삭제 한 번씩뿐).
        self.channels.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Hub {
    /// 방송의 연결들에 보낸다. **DB 커밋 후** 호출한다. 연결이 없으면 아무 일도 없다.
    pub fn publish(&self, broadcast_id: i64, event: Event) {
        if let Some(tx) = self.inner.channels().get(&broadcast_id) {
            // 실패 = 수신자 0(마지막 연결이 막 끊김). 보낼 곳이 없을 뿐이다.
            if tx.send(Arc::new(event)).is_err() {
                tracing::debug!(broadcast_id, "ws publish: no subscribers");
            }
        }
    }

    /// 지금 열려 있는 WebSocket 연결 수(T13 메트릭용).
    pub fn connections(&self) -> usize {
        self.inner.connections.load(Ordering::Relaxed)
    }

    fn subscribe(&self, broadcast_id: i64) -> Subscription {
        let rx = self
            .inner
            .channels()
            .entry(broadcast_id)
            .or_insert_with(|| broadcast::channel(CHANNEL_CAPACITY).0)
            .subscribe();
        self.inner.connections.fetch_add(1, Ordering::Relaxed);
        Subscription {
            inner: self.inner.clone(),
            broadcast_id,
            rx,
        }
    }
}

/// 연결 하나의 구독. 버려지면(정상 종료·패닉·업그레이드 실패) 마지막 구독일 때 채널을 정리한다.
struct Subscription {
    inner: Arc<Inner>,
    broadcast_id: i64,
    rx: broadcast::Receiver<Arc<Event>>,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.inner.connections.fetch_sub(1, Ordering::Relaxed);
        let mut channels = self.inner.channels();
        // 아직 살아 있는 self.rx 하나만 남았으면 마지막 연결이다. 잠금 안이라 새 구독과 겹치지 않는다.
        if channels
            .get(&self.broadcast_id)
            .is_some_and(|tx| tx.receiver_count() <= 1)
        {
            channels.remove(&self.broadcast_id);
        }
    }
}

/// 업그레이드 요청: Origin(CSWSH 방지, GET이라 CSRF 미들웨어가 안 거른다) → 방송 존재(숨김 404) → 쿠키로 사용자 식별.
async fn events(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    OptionalUser(user): OptionalUser,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    if !origin_allowed(&headers, &state.config.allowed_origins) {
        tracing::warn!(broadcast_id = id, "ws origin rejected");
        return Err(AppError::Forbidden);
    }
    let owner_id = crate::broadcast::detail_of(&state, id, None)
        .await?
        .broadcast
        .owner
        .id;
    let user_id = user.map(|u| u.id);
    let is_owner = user_id == Some(owner_id);
    // 응답 전에 구독한다: 클라이언트가 101을 받은 직후 발행된 이벤트도 받게.
    let sub = state.realtime.subscribe(id);
    Ok(ws.on_upgrade(move |socket| async move {
        tracing::debug!(broadcast_id = id, ?user_id, is_owner, "ws connected");
        run(socket, sub, is_owner).await;
        tracing::debug!(broadcast_id = id, ?user_id, "ws disconnected");
    }))
}

async fn run(mut socket: WebSocket, mut sub: Subscription, is_owner: bool) {
    let id = sub.broadcast_id;
    let mut ping = tokio::time::interval(PING_INTERVAL);
    ping.tick().await; // 첫 tick은 즉시 끝난다
    let mut awaiting_pong = false;
    loop {
        let out = tokio::select! {
            r = sub.rx.recv() => match r {
                Ok(ev) => match ev.for_connection(is_owner) {
                    Some(text) => Message::Text(text.clone()),
                    None => continue,
                },
                Err(RecvError::Lagged(skipped)) => {
                    tracing::warn!(broadcast_id = id, skipped, "ws lagged, sending resync");
                    match encode(&WsEvent::Resync {}) {
                        Some(text) => Message::Text(text),
                        None => continue,
                    }
                }
                // 이 구독이 살아 있는 동안 송신자는 맵에 남아 있어 사실상 없음
                Err(RecvError::Closed) => break,
            },
            m = socket.recv() => match m {
                // 클라이언트 → 서버 메시지는 없다. ping에 대한 pong은 라이브러리가 자동으로 보낸다.
                Some(Ok(Message::Pong(_))) => {
                    awaiting_pong = false;
                    continue;
                }
                Some(Ok(Message::Close(_))) | None => break,
                Some(Ok(_)) => continue,
                Some(Err(e)) => {
                    tracing::debug!(broadcast_id = id, error = %e, "ws recv error");
                    break;
                }
            },
            _ = ping.tick() => {
                if awaiting_pong {
                    tracing::debug!(broadcast_id = id, "ws pong timeout");
                    break;
                }
                awaiting_pong = true;
                Message::Ping(Default::default())
            }
        };
        if let Err(e) = socket.send(out).await {
            tracing::debug!(broadcast_id = id, error = %e, "ws send failed");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_only_skips_public_connections() {
        let e = Event::owner_only(WsEvent::Resync {});
        assert!(e.for_connection(true).is_some());
        assert!(e.for_connection(false).is_none());
    }

    #[test]
    fn last_subscription_drop_removes_channel() {
        let hub = Hub::default();
        let a = hub.subscribe(1);
        let b = hub.subscribe(1);
        assert_eq!(hub.connections(), 2);
        drop(a);
        assert!(hub.inner.channels().contains_key(&1));
        drop(b);
        assert!(hub.inner.channels().is_empty());
        assert_eq!(hub.connections(), 0);
    }
}

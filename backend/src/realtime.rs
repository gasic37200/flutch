//! 방송별 실시간 이벤트 허브 + WebSocket `/broadcasts/{id}/events` 핸들러 자리.
//! 설계는 architecture §6: 방송마다 크기 제한 `tokio::broadcast` 채널, DB 커밋 후 발행,
//! 이벤트는 `public`/`owner_only` 두 형태를 만들어 연결별로 골라 보낸다. WebSocket task가 채운다.

#[derive(Default)]
pub struct Hub {}

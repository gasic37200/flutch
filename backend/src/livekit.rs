//! LiveKit 연동. 운영은 `LiveKitHttp`, 시나리오 테스트는 fake 구현을 `AppState.livekit`에 넣는다.
//! 메서드(Egress·웹훅 검증)는 live/recording task가 trait에 추가한다.
//! 토큰은 livekit-api crate 대신 HS256 JWT를 직접 서명한다(protobuf·aws-lc를 끌고 오지 않으려고).

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::{Value, json};
use sha2::Sha256;

use crate::config::LiveKitConfig;

/// 클라이언트 접속 토큰 유효 시간(api.md §3-1). 방송 최대 3시간보다 길게.
pub const ACCESS_TOKEN_TTL_SECS: i64 = 4 * 60 * 60;

#[derive(Debug, thiserror::Error)]
pub enum LiveKitError {
    #[error("livekit request: {0}")]
    Http(#[from] reqwest::Error),
    #[error("livekit status {0}")]
    Status(u16),
    #[error("livekit signing key rejected")]
    Key,
}

#[async_trait]
pub trait LiveKitApi: Send + Sync {
    /// 방을 닫아 모든 참가자를 내보낸다. 방이 이미 없으면 성공.
    async fn delete_room(&self, room: &str) -> Result<(), LiveKitError>;
    /// 참가자 한 명을 내보낸다(기기 전환). 방이나 참가자가 이미 없으면 성공.
    async fn remove_participant(&self, room: &str, identity: &str) -> Result<(), LiveKitError>;
}

/// 클라이언트 접속 토큰. `publish`면 송출(카메라·마이크·화면·화면 소리)+구독, 아니면 구독만.
/// LiveKit은 `canPublish`·`canPublishData`가 없으면 true로 보므로 항상 명시한다.
pub fn access_token(
    config: &LiveKitConfig,
    room: &str,
    identity: &str,
    publish: bool,
    now: i64,
) -> Result<String, LiveKitError> {
    let mut video = json!({
        "room": room, "roomJoin": true, "canSubscribe": true,
        "canPublish": publish, "canPublishData": false,
    });
    if publish {
        video["canPublishSources"] =
            json!(["camera", "microphone", "screen_share", "screen_share_audio"]);
    }
    sign(
        &config.api_secret,
        &json!({
            "iss": config.api_key, "sub": identity,
            "nbf": now, "exp": now + ACCESS_TOKEN_TTL_SECS, "video": video,
        }),
    )
}

/// HS256 JWT 서명.
fn sign(secret: &str, claims: &Value) -> Result<String, LiveKitError> {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut mac =
        Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| LiveKitError::Key)?;
    mac.update(format!("{header}.{payload}").as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{header}.{payload}.{sig}"))
}

pub struct LiveKitHttp {
    pub config: LiveKitConfig,
    pub http: reqwest::Client,
}

impl LiveKitHttp {
    pub fn new(config: LiveKitConfig, http: reqwest::Client) -> Self {
        Self { config, http }
    }

    /// 서버 API용 관리 토큰(60초). `video`는 호출하는 API가 요구하는 권한.
    fn admin_token(&self, now: i64, video: Value) -> Result<String, LiveKitError> {
        sign(
            &self.config.api_secret,
            &json!({ "iss": self.config.api_key, "nbf": now, "exp": now + 60, "video": video }),
        )
    }

    /// `ws(s)://` 설정값을 서버 API용 `http(s)://`로.
    fn http_base(&self) -> String {
        let url = self.config.url.trim_end_matches('/');
        match url.split_once("://") {
            Some(("wss", rest)) => format!("https://{rest}"),
            Some(("ws", rest)) => format!("http://{rest}"),
            _ => url.to_owned(),
        }
    }

    /// RoomService Twirp 호출. Twirp는 없는 방·참가자에 404(not_found)를 준다 — 이미 없는 것이므로 성공.
    async fn room_service(
        &self,
        method: &str,
        video: Value,
        body: Value,
    ) -> Result<(), LiveKitError> {
        let token = self.admin_token(chrono::Utc::now().timestamp(), video)?;
        let res = self
            .http
            .post(format!(
                "{}/twirp/livekit.RoomService/{method}",
                self.http_base()
            ))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;
        match res.status() {
            s if s.is_success() || s.as_u16() == 404 => Ok(()),
            s => Err(LiveKitError::Status(s.as_u16())),
        }
    }
}

#[async_trait]
impl LiveKitApi for LiveKitHttp {
    async fn delete_room(&self, room: &str) -> Result<(), LiveKitError> {
        self.room_service(
            "DeleteRoom",
            json!({ "roomCreate": true }),
            json!({ "room": room }),
        )
        .await
    }

    async fn remove_participant(&self, room: &str, identity: &str) -> Result<(), LiveKitError> {
        self.room_service(
            "RemoveParticipant",
            json!({ "roomAdmin": true, "room": room }),
            json!({ "room": room, "identity": identity }),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_of(token: &str, secret: &[u8]) -> Value {
        let parts: Vec<_> = token.split('.').collect();
        assert_eq!(parts.len(), 3);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
        mac.update(format!("{}.{}", parts[0], parts[1]).as_bytes());
        assert_eq!(
            parts[2],
            URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
        );
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap()
    }

    #[test]
    fn admin_token_is_signed_jwt_and_url_is_http() {
        let lk = LiveKitHttp::new(
            LiveKitConfig {
                url: "wss://x.livekit.cloud/".into(),
                api_key: "key".into(),
                api_secret: "secret".into(),
            },
            reqwest::Client::new(),
        );
        assert_eq!(lk.http_base(), "https://x.livekit.cloud");
        let t = lk.admin_token(1000, json!({ "roomCreate": true })).unwrap();
        let claims = claims_of(&t, b"secret");
        assert_eq!(claims["iss"], "key");
        assert_eq!(claims["video"]["roomCreate"], true);
    }
}

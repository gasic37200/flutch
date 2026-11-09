//! LiveKit 연동. 운영은 `LiveKitHttp`, 시나리오 테스트는 fake 구현을 `AppState.livekit`에 넣는다.
//! 메서드(토큰 발급·Egress·웹훅 검증)는 live/recording task가 trait에 추가한다.

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::config::LiveKitConfig;

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
}

pub struct LiveKitHttp {
    pub config: LiveKitConfig,
    pub http: reqwest::Client,
}

impl LiveKitHttp {
    pub fn new(config: LiveKitConfig, http: reqwest::Client) -> Self {
        Self { config, http }
    }

    /// 서버 API용 관리 토큰(HS256 JWT, 60초). livekit-api crate는 protobuf·웹소켓까지 끌고 와서
    /// 방 삭제 하나에는 과해 직접 서명한다. 토큰 발급이 필요한 T08a가 필요하면 crate 도입을 재검토.
    fn admin_token(&self, now: i64) -> Result<String, LiveKitError> {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let claims = serde_json::json!({
            "iss": self.config.api_key, "nbf": now, "exp": now + 60,
            "video": { "roomCreate": true },
        });
        let payload = URL_SAFE_NO_PAD.encode(claims.to_string());
        let mut mac = Hmac::<Sha256>::new_from_slice(self.config.api_secret.as_bytes())
            .map_err(|_| LiveKitError::Key)?;
        mac.update(format!("{header}.{payload}").as_bytes());
        let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
        Ok(format!("{header}.{payload}.{sig}"))
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
}

#[async_trait]
impl LiveKitApi for LiveKitHttp {
    async fn delete_room(&self, room: &str) -> Result<(), LiveKitError> {
        let token = self.admin_token(chrono::Utc::now().timestamp())?;
        let res = self
            .http
            .post(format!(
                "{}/twirp/livekit.RoomService/DeleteRoom",
                self.http_base()
            ))
            .bearer_auth(token)
            .json(&serde_json::json!({ "room": room }))
            .send()
            .await?;
        // Twirp는 없는 방에 404(not_found)를 준다 — 이미 닫힌 것이므로 성공.
        match res.status() {
            s if s.is_success() || s.as_u16() == 404 => Ok(()),
            s => Err(LiveKitError::Status(s.as_u16())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let t = lk.admin_token(1000).unwrap();
        let parts: Vec<_> = t.split('.').collect();
        assert_eq!(parts.len(), 3);
        let claims: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();
        assert_eq!(claims["iss"], "key");
        assert_eq!(claims["video"]["roomCreate"], true);
        let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
        mac.update(format!("{}.{}", parts[0], parts[1]).as_bytes());
        assert_eq!(
            parts[2],
            URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
        );
    }
}

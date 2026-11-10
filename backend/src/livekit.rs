//! LiveKit 연동. 운영은 `LiveKitHttp`, 시나리오 테스트는 fake 구현을 `AppState.livekit`에 넣는다.
//! 메서드(Egress)는 recording task가 trait에 추가한다.
//! 토큰은 livekit-api crate 대신 HS256 JWT를 직접 서명한다(protobuf·aws-lc를 끌고 오지 않으려고).

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::config::{LiveKitConfig, S3Config};

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
    #[error("livekit response: {0}")]
    Json(#[from] serde_json::Error),
}

/// LiveKit `ParticipantInfo`에서 쓰는 필드. 웹훅은 camelCase, 서버 API(Twirp JSON)는 snake_case로 온다.
#[derive(Debug, Clone, Deserialize)]
pub struct Participant {
    pub identity: String,
    /// 접속 1회마다 새로 붙는 id. 같은 신원 재접속을 구분한다.
    #[serde(default)]
    pub sid: String,
    #[serde(default)]
    pub permission: Permission,
}

/// protojson 웹훅은 false를 생략한다 → 없으면 false.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Permission {
    #[serde(default, rename = "canPublish", alias = "can_publish")]
    pub can_publish: bool,
}

#[async_trait]
pub trait LiveKitApi: Send + Sync {
    /// 방을 닫아 모든 참가자를 내보낸다. 방이 이미 없으면 성공.
    async fn delete_room(&self, room: &str) -> Result<(), LiveKitError>;
    /// 참가자 한 명을 내보낸다(기기 전환). 방이나 참가자가 이미 없으면 성공.
    async fn remove_participant(&self, room: &str, identity: &str) -> Result<(), LiveKitError>;
    /// 방에 지금 접속한 참가자(hidden 포함). 방이 없으면 빈 목록.
    async fn list_participants(&self, room: &str) -> Result<Vec<Participant>, LiveKitError>;
    /// 방 전체(RoomComposite) MP4 녹화를 시작해 S3 호환 저장소의 `key`로 올린다. 돌려주는 값은 egress id.
    /// 방이 있어야 시작되므로 송출자가 접속한 뒤에 부른다.
    async fn start_recording(
        &self,
        room: &str,
        s3: &S3Config,
        key: &str,
    ) -> Result<String, LiveKitError>;
    /// Egress를 끝낸다(파일을 마무리해 올린다). 이미 끝났으면 성공.
    async fn stop_recording(&self, egress_id: &str) -> Result<(), LiveKitError>;
}

/// 웹훅 검증 시계 오차 허용(초).
const WEBHOOK_LEEWAY_SECS: i64 = 60;

/// LiveKit 웹훅 검증: `Authorization` 헤더의 JWT가 API secret(HS256)으로 서명됐고 `iss`가 API key이며
/// 만료 전이고, `sha256` 클레임이 본문 SHA-256의 표준 base64와 같아야 한다(livekit/protocol webhook 방식).
pub fn verify_webhook(config: &LiveKitConfig, token: &str, body: &[u8], now: i64) -> bool {
    let mut parts = token.split('.');
    let (Some(header), Some(payload), Some(sig), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let Ok(sig) = URL_SAFE_NO_PAD.decode(sig) else {
        return false;
    };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(config.api_secret.as_bytes()) else {
        return false;
    };
    mac.update(format!("{header}.{payload}").as_bytes());
    if mac.verify_slice(&sig).is_err() {
        return false;
    }
    let Some(claims) = URL_SAFE_NO_PAD
        .decode(payload)
        .ok()
        .and_then(|p| serde_json::from_slice::<Value>(&p).ok())
    else {
        return false;
    };
    let body_hash = STANDARD.encode(Sha256::digest(body));
    claims["iss"] == config.api_key.as_str()
        && claims["exp"]
            .as_i64()
            .is_some_and(|exp| exp + WEBHOOK_LEEWAY_SECS >= now)
        && claims["nbf"]
            .as_i64()
            .is_none_or(|nbf| nbf - WEBHOOK_LEEWAY_SECS <= now)
        && claims["sha256"] == body_hash.as_str()
}

/// 클라이언트 접속 토큰. `publish`면 송출(카메라·마이크·화면·화면 소리)+구독, 아니면 구독만.
/// LiveKit은 `canPublish`·`canPublishData`가 없으면 true로 보므로 항상 명시한다.
/// 시청자는 `hidden`: 다른 참가자에게 신원이 보이지 않는다(웹훅·서버 API에는 그대로 나온다).
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
    } else {
        video["hidden"] = json!(true);
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

    /// RoomService Twirp 호출, 응답 JSON. Twirp는 없는 방·참가자에 404(not_found)를 준다 — 이미 없는 것이므로 성공(`Null`).
    async fn room_service(
        &self,
        method: &str,
        video: Value,
        body: Value,
    ) -> Result<Value, LiveKitError> {
        self.twirp("RoomService", method, video, body).await
    }

    async fn twirp(
        &self,
        service: &str,
        method: &str,
        video: Value,
        body: Value,
    ) -> Result<Value, LiveKitError> {
        let token = self.admin_token(chrono::Utc::now().timestamp(), video)?;
        let res = self
            .http
            .post(format!(
                "{}/twirp/livekit.{service}/{method}",
                self.http_base()
            ))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;
        match res.status() {
            s if s.is_success() => Ok(res.json().await?),
            s if s.as_u16() == 404 => Ok(Value::Null),
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
        .map(drop)
    }

    async fn remove_participant(&self, room: &str, identity: &str) -> Result<(), LiveKitError> {
        self.room_service(
            "RemoveParticipant",
            json!({ "roomAdmin": true, "room": room }),
            json!({ "room": room, "identity": identity }),
        )
        .await
        .map(drop)
    }

    async fn list_participants(&self, room: &str) -> Result<Vec<Participant>, LiveKitError> {
        let mut res = self
            .room_service(
                "ListParticipants",
                json!({ "roomAdmin": true, "room": room }),
                json!({ "room": room }),
            )
            .await?;
        match res.get_mut("participants") {
            Some(list) => Ok(serde_json::from_value(list.take())?),
            None => Ok(Vec::new()),
        }
    }

    async fn start_recording(
        &self,
        room: &str,
        s3: &S3Config,
        key: &str,
    ) -> Result<String, LiveKitError> {
        let res = self
            .twirp(
                "Egress",
                "StartRoomCompositeEgress",
                json!({ "roomRecord": true }),
                json!({
                    "room_name": room,
                    "file_outputs": [{
                        "file_type": "MP4",
                        "filepath": key,
                        "s3": {
                            "access_key": s3.access_key, "secret": s3.secret_key,
                            "region": s3.region, "endpoint": s3.endpoint,
                            "bucket": s3.bucket, "force_path_style": true,
                        },
                    }],
                }),
            )
            .await?;
        res["egress_id"]
            .as_str()
            .or(res["egressId"].as_str())
            .map(str::to_owned)
            .ok_or(LiveKitError::Status(404))
    }

    async fn stop_recording(&self, egress_id: &str) -> Result<(), LiveKitError> {
        // 이미 끝난 Egress를 멈추면 Twirp 에러(failed_precondition 412 등) — 끝난 것이므로 성공으로 본다.
        match self
            .twirp(
                "Egress",
                "StopEgress",
                json!({ "roomRecord": true }),
                json!({ "egress_id": egress_id }),
            )
            .await
        {
            Err(LiveKitError::Status(412)) => Ok(()),
            r => r.map(drop),
        }
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

    /// 로컬 LiveKit v1.13 실측 형식: 웹훅은 camelCase·false 생략, ListParticipants는 snake_case.
    #[test]
    fn participant_parses_webhook_and_server_api() {
        let hook: Participant = serde_json::from_value(json!({
            "sid": "PA_1", "identity": "u:1:t", "permission": { "canSubscribe": true, "canPublish": true }
        }))
        .unwrap();
        assert!(hook.permission.can_publish);
        let hidden: Participant = serde_json::from_value(json!({
            "sid": "PA_2", "identity": "u:2:t", "permission": { "canSubscribe": true, "hidden": true }
        }))
        .unwrap();
        assert!(!hidden.permission.can_publish);
        let api: Participant = serde_json::from_value(json!({
            "sid": "PA_3", "identity": "u:1:t", "permission": { "can_subscribe": true, "can_publish": true, "hidden": false }
        }))
        .unwrap();
        assert_eq!(
            (api.sid.as_str(), api.permission.can_publish),
            ("PA_3", true)
        );
    }

    #[test]
    fn webhook_signature_and_body_hash_checked() {
        let config = LiveKitConfig {
            url: String::new(),
            api_key: "key".into(),
            api_secret: "secret".into(),
        };
        let body = br#"{"event":"room_finished"}"#;
        let token = |secret: &str, iss: &str, body: &[u8], exp: i64| {
            let hash = STANDARD.encode(Sha256::digest(body));
            sign(secret, &json!({ "iss": iss, "exp": exp, "sha256": hash })).unwrap()
        };
        assert!(verify_webhook(
            &config,
            &token("secret", "key", body, 2000),
            body,
            1000
        ));
        assert!(!verify_webhook(
            &config,
            &token("other", "key", body, 2000),
            body,
            1000
        ));
        assert!(!verify_webhook(
            &config,
            &token("secret", "nope", body, 2000),
            body,
            1000
        ));
        assert!(!verify_webhook(
            &config,
            &token("secret", "key", b"{}", 2000),
            body,
            1000
        ));
        assert!(!verify_webhook(
            &config,
            &token("secret", "key", body, 900),
            body,
            1000
        ));
        assert!(!verify_webhook(&config, "", body, 1000));
    }
}

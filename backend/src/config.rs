//! 환경변수 설정. 도메인 설정은 하위 구조체(`config.livekit` 등)로 추가한다:
//! `XxxConfig` + `from_env()`를 만들고 `Config`에 필드 하나, `Config::from_env`에 한 줄.
//! 비밀값 필드가 있는 구조체는 `Debug`를 직접 구현해 값을 가린다(로그 유출 방지).

use std::env;
use std::fmt;
use std::path::PathBuf;

pub struct Config {
    pub backend_port: u16,
    /// 비밀번호가 들어 있으므로 `Debug`에서 가린다.
    pub database_url: String,
    /// 변경 요청·WebSocket `Origin` 허용 목록 (쉼표 구분).
    pub allowed_origins: Vec<String>,
    pub livekit: LiveKitConfig,
    pub auth: AuthConfig,
    pub live: LiveConfig,
    pub broadcast: BroadcastConfig,
    pub recording: RecordingConfig,
}

/// Egress가 올리고 백엔드가 지우는 S3 호환 버킷(path-style).
#[derive(Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl fmt::Debug for S3Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("access_key", &"***")
            .field("secret_key", &"***")
            .finish()
    }
}

/// 녹화 설정(features/08). `s3`가 없으면 녹화를 끈다(방송은 정상).
#[derive(Debug, Clone)]
pub struct RecordingConfig {
    pub s3: Option<S3Config>,
    /// `RECORDING_PUBLIC_BASE_URL`: 객체 공개 읽기 URL의 앞부분. `videoUrl` = 이 값 + `/` + object_key
    pub public_base_url: String,
    pub retention_days: i64,
    pub max_total_bytes: i64,
}

impl RecordingConfig {
    pub fn from_env() -> Self {
        let get = |n: &'static str| required(n).ok();
        let s3 = match (
            get("RECORDING_S3_ENDPOINT"),
            get("RECORDING_S3_BUCKET"),
            get("RECORDING_S3_ACCESS_KEY"),
            get("RECORDING_S3_SECRET_KEY"),
        ) {
            (Some(endpoint), Some(bucket), Some(access_key), Some(secret_key)) => Some(S3Config {
                endpoint: endpoint.trim_end_matches('/').to_owned(),
                bucket,
                region: get("RECORDING_S3_REGION").unwrap_or_else(|| "us-east-1".into()),
                access_key,
                secret_key,
            }),
            _ => None,
        };
        let public_base_url = get("RECORDING_PUBLIC_BASE_URL")
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_owned();
        if s3.is_none() || public_base_url.is_empty() {
            tracing::warn!("RECORDING_S3_* / RECORDING_PUBLIC_BASE_URL 미설정: 녹화를 끕니다");
        }
        Self {
            s3: s3.filter(|_| !public_base_url.is_empty()),
            public_base_url,
            retention_days: 30,
            max_total_bytes: 16 * 1024 * 1024 * 1024,
        }
    }
}

/// 송출 시간 규칙(features/03). 수동 확인용으로 환경변수로 줄일 수 있다.
#[derive(Debug, Clone)]
pub struct LiveConfig {
    /// `LIVE_GRACE_SECS`: 송출자 끊김 후 자동 종료까지(기본 60초)
    pub grace_secs: i64,
    /// `LIVE_MAX_DURATION_SECS`: LIVE 최대 길이(기본 3시간). 10분 전에 경고한다.
    pub max_duration_secs: i64,
}

impl LiveConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            grace_secs: secs_or("LIVE_GRACE_SECS", 60)?,
            max_duration_secs: secs_or("LIVE_MAX_DURATION_SECS", 3 * 60 * 60)?,
        })
    }
}

/// 방송 도메인 설정.
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// `THUMB_DIR`: 썸네일 JPEG 저장 폴더(기본 `./thumbs`). 운영은 nginx와 공유하는 볼륨.
    pub thumb_dir: PathBuf,
}

impl BroadcastConfig {
    pub fn from_env() -> Self {
        Self {
            thumb_dir: required("THUMB_DIR")
                .unwrap_or_else(|_| "./thumbs".into())
                .into(),
        }
    }
}

/// 선택 환경변수(양의 정수 초). 없으면 기본값.
fn secs_or(name: &'static str, default: i64) -> Result<i64, ConfigError> {
    match required(name) {
        Err(_) => Ok(default),
        Ok(v) => v
            .trim()
            .parse()
            .ok()
            .filter(|n: &i64| *n > 0)
            .ok_or(ConfigError(name, "양의 정수(초)여야 합니다")),
    }
}

pub struct AuthConfig {
    /// `SEED_PASSWORD`: 테스트 계정 3개의 비밀번호. 비우면 테스트 로그인 버튼으로만 들어간다.
    pub seed_password: Option<String>,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            seed_password: required("SEED_PASSWORD").ok(),
        }
    }
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthConfig")
            .field("seed_password", &self.seed_password.as_ref().map(|_| "***"))
            .finish()
    }
}

#[derive(Clone)]
pub struct LiveKitConfig {
    pub url: String,
    pub api_key: String,
    pub api_secret: String,
}

#[derive(Debug, thiserror::Error)]
#[error("환경변수 {0} 확인 필요: {1}")]
pub struct ConfigError(&'static str, &'static str);

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            backend_port: required("BACKEND_PORT")?
                .parse()
                .map_err(|_| ConfigError("BACKEND_PORT", "숫자여야 합니다"))?,
            database_url: required("DATABASE_URL")?,
            allowed_origins: parse_list(&required("ALLOWED_ORIGINS")?),
            livekit: LiveKitConfig::from_env()?,
            auth: AuthConfig::from_env(),
            live: LiveConfig::from_env()?,
            broadcast: BroadcastConfig::from_env(),
            recording: RecordingConfig::from_env(),
        })
    }
}

impl LiveKitConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            url: required("LIVEKIT_URL")?,
            api_key: required("LIVEKIT_API_KEY")?,
            api_secret: required("LIVEKIT_API_SECRET")?,
        })
    }
}

fn required(name: &'static str) -> Result<String, ConfigError> {
    env::var(name)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .ok_or(ConfigError(name, "값이 없습니다"))
}

fn parse_list(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("backend_port", &self.backend_port)
            .field("database_url", &"***")
            .field("allowed_origins", &self.allowed_origins)
            .field("livekit", &self.livekit)
            .field("auth", &self.auth)
            .field("live", &self.live)
            .field("broadcast", &self.broadcast)
            .field("recording", &self.recording)
            .finish()
    }
}

impl fmt::Debug for LiveKitConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveKitConfig")
            .field("url", &self.url)
            .field("api_key", &self.api_key)
            .field("api_secret", &"***")
            .finish()
    }
}

//! 환경변수 설정. 도메인 설정은 하위 구조체(`config.livekit` 등)로 추가한다:
//! `XxxConfig` + `from_env()`를 만들고 `Config`에 필드 하나, `Config::from_env`에 한 줄.
//! 비밀값 필드가 있는 구조체는 `Debug`를 직접 구현해 값을 가린다(로그 유출 방지).

use std::env;
use std::fmt;

pub struct Config {
    pub backend_port: u16,
    /// 비밀번호가 들어 있으므로 `Debug`에서 가린다.
    pub database_url: String,
    /// 변경 요청·WebSocket `Origin` 허용 목록 (쉼표 구분).
    pub allowed_origins: Vec<String>,
    pub livekit: LiveKitConfig,
    pub auth: AuthConfig,
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

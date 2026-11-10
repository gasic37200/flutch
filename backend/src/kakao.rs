//! 카카오 로그인 연동. 운영은 `KakaoHttp`, 시나리오 테스트는 fake 구현을 `AppState.kakao`에 넣는다.
//! 공식 문서(developers.kakao.com › 카카오 로그인 › REST API): 토큰 `POST kauth.kakao.com/oauth/token`,
//! 사용자 정보 `GET kapi.kakao.com/v2/user/me`(Bearer), 연결 끊기 `POST kapi.kakao.com/v1/user/unlink`(앱 어드민 키).

use async_trait::async_trait;
use serde::Deserialize;

use crate::config::KakaoConfig;

const TOKEN_URL: &str = "https://kauth.kakao.com/oauth/token";
const ME_URL: &str = "https://kapi.kakao.com/v2/user/me";
const UNLINK_URL: &str = "https://kapi.kakao.com/v1/user/unlink";

#[derive(Debug, Clone)]
pub struct KakaoProfile {
    /// 앱별 사용자 고유 번호(탈퇴·재연결해도 같다)
    pub id: i64,
    pub nickname: Option<String>,
}

/// 메시지에는 URL·코드·토큰을 넣지 않는다(로그에 남는다).
#[derive(Debug, Clone, thiserror::Error)]
#[error("kakao: {0}")]
pub struct KakaoError(pub String);

#[async_trait]
pub trait KakaoApi: Send + Sync {
    /// 인가 코드 → 카카오 사용자 정보(토큰 교환 + 사용자 정보 조회). 토큰은 저장하지 않는다.
    async fn profile(&self, code: &str) -> Result<KakaoProfile, KakaoError>;
    /// 연결 끊기. 설정(어드민 키)이 없으면 아무것도 하지 않는다.
    async fn unlink(&self, kakao_id: i64) -> Result<(), KakaoError>;
}

pub struct KakaoHttp {
    http: reqwest::Client,
    config: Option<KakaoConfig>,
}

impl KakaoHttp {
    pub fn new(http: reqwest::Client, config: Option<KakaoConfig>) -> Self {
        Self { http, config }
    }
}

fn net(e: reqwest::Error) -> KakaoError {
    KakaoError(e.without_url().to_string())
}

#[async_trait]
impl KakaoApi for KakaoHttp {
    async fn profile(&self, code: &str) -> Result<KakaoProfile, KakaoError> {
        let Some(c) = &self.config else {
            return Err(KakaoError("카카오 설정 없음".into()));
        };
        #[derive(Deserialize)]
        struct Token {
            access_token: String,
        }
        let mut form = vec![
            ("grant_type", "authorization_code"),
            ("client_id", c.rest_key.as_str()),
            ("redirect_uri", c.redirect_uri.as_str()),
            ("code", code),
        ];
        if let Some(secret) = &c.client_secret {
            form.push(("client_secret", secret));
        }
        let res = self
            .http
            .post(TOKEN_URL)
            .form(&form)
            .send()
            .await
            .map_err(net)?;
        if !res.status().is_success() {
            return Err(KakaoError(format!("token status {}", res.status())));
        }
        let token: Token = res.json().await.map_err(net)?;

        #[derive(Deserialize)]
        struct Me {
            id: i64,
            kakao_account: Option<Account>,
            properties: Option<Properties>,
        }
        #[derive(Deserialize)]
        struct Account {
            profile: Option<Properties>,
        }
        #[derive(Deserialize)]
        struct Properties {
            nickname: Option<String>,
        }
        let res = self
            .http
            .get(ME_URL)
            .bearer_auth(token.access_token)
            .send()
            .await
            .map_err(net)?;
        if !res.status().is_success() {
            return Err(KakaoError(format!("user/me status {}", res.status())));
        }
        let me: Me = res.json().await.map_err(net)?;
        let nickname = me
            .kakao_account
            .and_then(|a| a.profile)
            .and_then(|p| p.nickname)
            .or(me.properties.and_then(|p| p.nickname));
        Ok(KakaoProfile {
            id: me.id,
            nickname,
        })
    }

    async fn unlink(&self, kakao_id: i64) -> Result<(), KakaoError> {
        let Some(admin_key) = self.config.as_ref().and_then(|c| c.admin_key.as_ref()) else {
            return Err(KakaoError("KAKAO_ADMIN_KEY 없음".into()));
        };
        let res = self
            .http
            .post(UNLINK_URL)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("KakaoAK {admin_key}"),
            )
            .form(&[
                ("target_id_type", "user_id"),
                ("target_id", &kakao_id.to_string()),
            ])
            .send()
            .await
            .map_err(net)?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(KakaoError(format!("unlink status {}", res.status())))
        }
    }
}

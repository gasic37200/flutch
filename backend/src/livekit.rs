//! LiveKit 연동. 운영은 `LiveKitHttp`, 시나리오 테스트는 fake 구현을 `AppState.livekit`에 넣는다.
//! 메서드(토큰 발급·방 삭제·Egress·웹훅 검증)는 live/recording task가 trait에 추가한다.

use async_trait::async_trait;

use crate::config::LiveKitConfig;

#[async_trait]
pub trait LiveKitApi: Send + Sync {}

pub struct LiveKitHttp {
    pub config: LiveKitConfig,
    pub http: reqwest::Client,
}

impl LiveKitHttp {
    pub fn new(config: LiveKitConfig, http: reqwest::Client) -> Self {
        Self { config, http }
    }
}

#[async_trait]
impl LiveKitApi for LiveKitHttp {}

//! 토스페이먼츠 연동. 운영은 `TossHttp`, 시나리오 테스트는 fake 구현을 `AppState.toss`에 넣는다.
//! 메서드(승인·결제 조회)와 설정(`config.toss`)은 결제 task가 추가한다.

use async_trait::async_trait;

#[async_trait]
pub trait TossApi: Send + Sync {}

pub struct TossHttp {
    pub http: reqwest::Client,
}

impl TossHttp {
    pub fn new(http: reqwest::Client) -> Self {
        Self { http }
    }
}

#[async_trait]
impl TossApi for TossHttp {}

//! 토스페이먼츠 연동. 운영은 `TossHttp`, 시나리오 테스트는 fake 구현을 `AppState.toss`에 넣는다.
//! API: 결제 승인 `POST /v1/payments/confirm`, 결제 조회 `GET /v1/payments/{paymentKey}`,
//! 인증은 시크릿 키(결제위젯 `test_gsk_…`) + `:`을 base64로 감싼 Basic 헤더.

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::StatusCode;
use serde::Deserialize;

use crate::config::TossConfig;

const API: &str = "https://api.tosspayments.com/v1/payments";

/// 토스 Payment 객체 중 확정에 쓰는 필드.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TossPayment {
    pub payment_key: String,
    pub order_id: String,
    /// `DONE`이면 결제 완료. 그 외(`IN_PROGRESS`·`WAITING_FOR_DEPOSIT`·`CANCELED`·`ABORTED`·`EXPIRED` …)
    pub status: String,
    pub total_amount: i64,
    /// 웹훅 본문의 `secret`과 대조하는 값
    pub secret: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TossError {
    /// 토스가 요청을 거절(4xx, 토스 에러 코드). 예: 카드 거절, `ALREADY_PROCESSED_PAYMENT`, `NOT_FOUND_PAYMENT`
    #[error("toss rejected: {code}")]
    Rejected { code: String, message: String },
    /// 통신 실패·타임아웃·5xx·키 없음·응답 해석 실패 — 승인됐는지 알 수 없다.
    #[error("toss unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait TossApi: Send + Sync {
    /// 결제 승인
    async fn confirm(
        &self,
        payment_key: &str,
        order_id: &str,
        amount: i64,
    ) -> Result<TossPayment, TossError>;
    /// 결제 조회(웹훅 재확인, 승인 결과를 모를 때 복구)
    async fn payment(&self, payment_key: &str) -> Result<TossPayment, TossError>;
}

pub struct TossHttp {
    pub http: reqwest::Client,
    secret_key: Option<String>,
}

impl TossHttp {
    /// 공용 골격(main.rs)을 그대로 두려고 시크릿 키는 환경변수에서 직접 읽는다(`Config.toss`와 같은 값).
    pub fn new(http: reqwest::Client) -> Self {
        Self {
            http,
            secret_key: TossConfig::from_env().secret_key,
        }
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<TossPayment, TossError> {
        let Some(secret) = &self.secret_key else {
            return Err(TossError::Unavailable("TOSS_SECRET_KEY 없음".into()));
        };
        let auth = format!("Basic {}", STANDARD.encode(format!("{secret}:")));
        // without_url: 조회 URL에 paymentKey 전체가 들어 있어 로그에 남기지 않는다.
        let res = req
            .header(reqwest::header::AUTHORIZATION, auth)
            .send()
            .await
            .map_err(|e| TossError::Unavailable(e.without_url().to_string()))?;
        let status = res.status();
        if status.is_success() {
            return res
                .json()
                .await
                .map_err(|e| TossError::Unavailable(format!("응답 해석: {}", e.without_url())));
        }
        // 401은 우리 키 문제라 사용자 결제 거절이 아니다.
        if status.is_client_error() && status != StatusCode::UNAUTHORIZED {
            #[derive(Deserialize, Default)]
            struct Body {
                #[serde(default)]
                code: String,
                #[serde(default)]
                message: String,
            }
            let b: Body = res.json().await.unwrap_or_default();
            return Err(TossError::Rejected {
                code: b.code,
                message: b.message,
            });
        }
        Err(TossError::Unavailable(format!("status {status}")))
    }
}

#[async_trait]
impl TossApi for TossHttp {
    async fn confirm(
        &self,
        payment_key: &str,
        order_id: &str,
        amount: i64,
    ) -> Result<TossPayment, TossError> {
        let body =
            serde_json::json!({ "paymentKey": payment_key, "orderId": order_id, "amount": amount });
        self.send(self.http.post(format!("{API}/confirm")).json(&body))
            .await
    }

    async fn payment(&self, payment_key: &str) -> Result<TossPayment, TossError> {
        // paymentKey는 토스가 만든 값이지만 웹훅 본문에서 오므로 경로 문자를 막는다.
        if !payment_key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        {
            return Err(TossError::Rejected {
                code: "INVALID_PAYMENT_KEY".into(),
                message: String::new(),
            });
        }
        self.send(self.http.get(format!("{API}/{payment_key}")))
            .await
    }
}

/// 웹훅 `PAYMENT_STATUS_CHANGED` 본문(`{eventType, createdAt, data: Payment}`)에서 `(paymentKey, secret)`.
/// 다른 이벤트·형식이 틀린 본문은 `None`(무시). 본문 내용은 믿지 않고 조회 API로 다시 확인한다.
pub fn parse_webhook(body: &[u8]) -> Option<(String, Option<String>)> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        payment_key: String,
        secret: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Event {
        event_type: String,
        data: Data,
    }
    let e: Event = serde_json::from_slice(body).ok()?;
    (e.event_type == "PAYMENT_STATUS_CHANGED").then_some((e.data.payment_key, e.data.secret))
}

/// 로그용 paymentKey 앞부분(전체는 남기지 않는다).
pub fn key_hint(payment_key: &str) -> String {
    payment_key.chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn webhook_body_parsing() {
        let ok = br#"{"eventType":"PAYMENT_STATUS_CHANGED","createdAt":"2026-11-10T10:00:00.000000","data":{"paymentKey":"pk_1","orderId":"o1","status":"DONE","secret":"s"}}"#;
        assert_eq!(
            parse_webhook(ok),
            Some(("pk_1".to_owned(), Some("s".to_owned())))
        );
        let other = br#"{"eventType":"DEPOSIT_CALLBACK","data":{"paymentKey":"pk_1"}}"#;
        assert_eq!(parse_webhook(other), None);
        assert_eq!(parse_webhook(b"not json"), None);
        assert_eq!(key_hint("abcdefghijk"), "abcdefgh");
    }
}

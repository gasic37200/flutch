//! 토스 결제: 주문 → 승인(confirm) → 확정, 웹훅 재확인(api.md §6).
//! 확정은 `finalize` 하나로 모은다(승인 응답·재시도·웹훅이 모두 같은 길). 주문 행 `FOR UPDATE`로 같은
//! 주문의 확정을 한 줄로 세우고, 원장 `payment_order_id`·주문 `payment_key` 유니크가 이중 적립을 막는다.

use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::db::{self, NewOrder, OrderRow};
use super::{
    AMOUNT_MIN, DebitKind, Donation, DonationError, DonationKind, NewDonation, WALLET_MAX,
    WalletError, check_amount, check_donation_input, donation_target,
};
use crate::AppState;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody};
use crate::toss::{self, TossApi, TossError, TossPayment, key_hint};

const NOTICE_ENDED: &str = "방송이 종료되어 잔액으로 충전했어요";
const NOTICE_BANNED: &str = "이 채널에서는 후원할 수 없어 잔액으로 충전했어요";
const NOTICE_INSUFFICIENT: &str = "잔액이 부족해 후원하지 못하고 충전만 했어요";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PaymentPurpose {
    Charge,
    Donation,
}

/// POST /payments/orders. 결제 금액은 충전이면 `amount`, 후원이면 서버가 계산한다.
#[derive(Debug, Deserialize, TS)]
#[serde(tag = "purpose", rename_all = "snake_case")]
#[ts(export)]
pub enum CreateOrderRequest {
    Charge {
        amount: i64,
    },
    #[serde(rename_all = "camelCase")]
    Donation {
        broadcast_id: i64,
        donation: DonationInput,
    },
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct DonationInput {
    pub amount: i64,
    #[ts(optional)]
    pub message: Option<String>,
    pub anonymous: bool,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PaymentOrder {
    pub order_id: String,
    /// 결제할 금액
    pub amount: i64,
    /// 예: "1,000비트 충전"
    pub order_name: String,
    pub expires_at: DateTime<Utc>,
    /// 토스 결제위젯 클라이언트 키(`TOSS_CLIENT_KEY`). 서버에 없으면 null
    pub client_key: Option<String>,
}

/// POST /payments/confirm — 토스 성공 리다이렉트 쿼리 그대로
#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConfirmPaymentRequest {
    pub payment_key: String,
    pub order_id: String,
    pub amount: i64,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PaymentResult {
    pub order_id: String,
    pub purpose: PaymentPurpose,
    pub charged: i64,
    pub balance: i64,
    /// 후원 주문이 후원까지 확정됐을 때(내 후원이라 실명)
    pub donation: Option<Donation>,
    /// 예: "방송이 종료되어 잔액으로 충전했어요"
    pub notice: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("잔액으로 바로 후원할 수 있어요.")]
    NotNeeded,
    #[error("주문을 찾을 수 없어요.")]
    OrderNotFound,
    #[error("내 주문만 확정할 수 있어요.")]
    NotOwner,
    #[error("결제 금액이 주문과 달라요.")]
    AmountMismatch,
    #[error("주문 시간이 지났어요. 다시 시도해 주세요.")]
    Expired,
    #[error("이미 처리된 주문이에요.")]
    AlreadyConfirmed,
    #[error("결제 승인에 실패했어요.")]
    ApprovalFailed,
    #[error("결제 서비스에 연결할 수 없어요. 잠시 후 다시 시도해 주세요.")]
    ProviderUnavailable,
}

impl From<PaymentError> for AppError {
    fn from(e: PaymentError) -> Self {
        let (status, code) = match e {
            PaymentError::NotNeeded => (StatusCode::CONFLICT, "PAYMENT_NOT_NEEDED"),
            PaymentError::OrderNotFound => (StatusCode::NOT_FOUND, "PAYMENT_ORDER_NOT_FOUND"),
            PaymentError::NotOwner => (StatusCode::FORBIDDEN, "PAYMENT_NOT_OWNER"),
            PaymentError::AmountMismatch => (StatusCode::CONFLICT, "PAYMENT_AMOUNT_MISMATCH"),
            PaymentError::Expired => (StatusCode::CONFLICT, "PAYMENT_ORDER_EXPIRED"),
            PaymentError::AlreadyConfirmed => (StatusCode::CONFLICT, "PAYMENT_ALREADY_CONFIRMED"),
            PaymentError::ApprovalFailed => (StatusCode::CONFLICT, "PAYMENT_APPROVAL_FAILED"),
            PaymentError::ProviderUnavailable => {
                (StatusCode::BAD_GATEWAY, "PAYMENT_PROVIDER_UNAVAILABLE")
            }
        };
        AppError::domain(status, code, e.to_string())
    }
}

/// 1234567 → "1,234,567"
pub fn bits(n: i64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

pub async fn create_order(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    JsonBody(req): JsonBody<CreateOrderRequest>,
) -> Result<(StatusCode, Json<PaymentOrder>), AppError> {
    let (amount, donation, order_name) = match req {
        CreateOrderRequest::Charge { amount } => {
            check_amount(amount)?;
            if db::balance(&state.db, user.id).await? + amount > WALLET_MAX {
                return Err(WalletError::BalanceLimit.into());
            }
            (amount, None, format!("{}비트 충전", bits(amount)))
        }
        CreateOrderRequest::Donation {
            broadcast_id,
            donation: d,
        } => {
            let status = donation_target(&state.db, user.id, broadcast_id).await?;
            let message = check_donation_input(d.amount, d.message.as_deref())?;
            if status != "LIVE" {
                return Err(DonationError::NotLive.into());
            }
            let balance = db::balance(&state.db, user.id).await?;
            if balance >= d.amount {
                return Err(PaymentError::NotNeeded.into());
            }
            // 최소 충전 단위 때문에 남는 비트는 잔액에 남는다.
            let n = (d.amount - balance).max(AMOUNT_MIN);
            let name = format!("{}비트 충전 후 후원", bits(n));
            (
                n,
                Some((broadcast_id, d.amount, d.anonymous, message)),
                name,
            )
        }
    };
    let order_id = uuid::Uuid::new_v4().simple().to_string(); // 32자 hex, 122비트 랜덤
    let expires_at = db::insert_order(
        &state.db,
        &NewOrder {
            order_id: &order_id,
            user_id: user.id,
            amount,
            donation: donation
                .as_ref()
                .map(|(b, a, anon, m)| (*b, *a, *anon, m.as_deref())),
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(PaymentOrder {
            order_id,
            amount,
            order_name,
            expires_at,
            client_key: state.config.toss.client_key.clone(),
        }),
    ))
}

pub async fn confirm(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    JsonBody(req): JsonBody<ConfirmPaymentRequest>,
) -> Result<Json<PaymentResult>, AppError> {
    let order = db::find_order(&state.db, &req.order_id)
        .await?
        .ok_or(PaymentError::OrderNotFound)?;
    if order.user_id != user.id {
        return Err(PaymentError::NotOwner.into());
    }
    if req.amount != order.amount {
        return Err(PaymentError::AmountMismatch.into());
    }
    // 확정된 주문은 만료와 상관없이 같은 결과(멱등)
    if order.status == "done" {
        return if order.payment_key.as_deref() == Some(req.payment_key.as_str()) {
            Ok(Json(result_of(&state, &order).await?))
        } else {
            Err(PaymentError::AlreadyConfirmed.into())
        };
    }
    if order.status != "pending" || order.expires_at <= Utc::now() {
        return Err(PaymentError::Expired.into());
    }
    // 토스가 승인한 뒤에 한도로 적립을 못 하는 일이 없게 승인 전에 확인한다.
    if db::balance(&state.db, user.id).await? + order.amount > WALLET_MAX {
        return Err(WalletError::BalanceLimit.into());
    }
    let payment = approve(state.toss.as_ref(), &req.payment_key, &order).await?;
    Ok(Json(finalize(&state, &payment).await?))
}

/// 토스 승인. 결과를 모를 때(통신 실패)·이미 승인됐다고 할 때(앞선 요청이 승인했는데 응답을 못 받음)는
/// 결제 조회로 실제 상태를 확인한다. 승인 응답의 키·주문·금액·상태를 다시 대조한다.
async fn approve(
    toss: &dyn TossApi,
    payment_key: &str,
    order: &OrderRow,
) -> Result<TossPayment, AppError> {
    let res = match toss
        .confirm(payment_key, &order.order_id, order.amount)
        .await
    {
        Err(e)
            if matches!(&e, TossError::Unavailable(_))
                || matches!(&e, TossError::Rejected { code, .. } if code == "ALREADY_PROCESSED_PAYMENT") =>
        {
            match toss.payment(payment_key).await {
                Ok(p) if p.status == "DONE" => Ok(p),
                _ => Err(e),
            }
        }
        r => r,
    };
    match res {
        Ok(p)
            if p.status == "DONE"
                && p.payment_key == payment_key
                && p.order_id == order.order_id
                && p.total_amount == order.amount =>
        {
            Ok(p)
        }
        Ok(p) => {
            tracing::error!(
                order_id = %order.order_id, toss_order_id = %p.order_id, status = %p.status,
                amount = p.total_amount, "toss approval does not match order"
            );
            Err(PaymentError::ApprovalFailed.into())
        }
        Err(TossError::Rejected { code, message }) => {
            tracing::warn!(order_id = %order.order_id, code, message, "toss approval rejected");
            Err(PaymentError::ApprovalFailed.into())
        }
        Err(TossError::Unavailable(e)) => {
            // 주문은 pending 그대로: 실제로 승인됐다면 웹훅(조회 재확인)이나 같은 confirm 재시도가 확정한다.
            tracing::error!(order_id = %order.order_id, payment_key = %key_hint(payment_key), error = %e, "toss unavailable");
            Err(PaymentError::ProviderUnavailable.into())
        }
    }
}

/// 토스가 DONE이라고 확인한 결제를 확정한다(승인·웹훅 공통, 멱등). 만료 시각은 보지 않는다 — 토스가
/// 승인했으면 돈이 나갔으니 적립한다. 커밋 전에는 응답·이벤트를 내지 않는다.
async fn finalize(state: &Arc<AppState>, p: &TossPayment) -> Result<PaymentResult, AppError> {
    let mut tx = state.db.begin().await?;
    let order = db::lock_order(&mut tx, &p.order_id)
        .await?
        .ok_or(PaymentError::OrderNotFound)?;
    if order.status == "done" {
        tx.rollback().await?;
        return if order.payment_key.as_deref() == Some(p.payment_key.as_str()) {
            result_of(state, &order).await
        } else {
            Err(PaymentError::AlreadyConfirmed.into())
        };
    }
    // 결제 직후 탈퇴: 탈퇴 지갑에 적립하지 않는다. 주문은 pending으로 남고 운영자가 수동 환불한다.
    if db::user_withdrawn(&mut tx, order.user_id).await? {
        tx.rollback().await?;
        tracing::error!(order_id = %order.order_id, amount = order.amount, "payment for withdrawn user: manual refund needed");
        return Err(PaymentError::OrderNotFound.into());
    }
    if p.total_amount != order.amount {
        tracing::error!(order_id = %order.order_id, toss_amount = p.total_amount, amount = order.amount, "toss amount mismatch");
        return Err(PaymentError::AmountMismatch.into());
    }
    // 잠금 순서: 주문 → 사용자 → 방송 FOR SHARE → 지갑. 후원·클러치(방송 → 지갑)와 같은 순서라 교착이 없다.
    let live = match order.broadcast_id {
        Some(b) => db::broadcast_status_for_share(&mut tx, b).await?,
        None => None,
    };
    match super::credit_charge(&mut tx, order.user_id, order.amount, order.id).await {
        Ok(_) => {}
        Err(WalletError::AlreadyCredited) => {
            // 유니크 위반으로 tx가 깨졌을 수 있어 롤백하고 커밋된 상태로 다시 답한다.
            tx.rollback().await?;
            let order = db::find_order(&state.db, &p.order_id)
                .await?
                .ok_or(PaymentError::OrderNotFound)?;
            return result_of(state, &order).await;
        }
        Err(WalletError::BalanceLimit) => {
            // shortcut: 승인 전 한도 확인 뒤 다른 충전이 끼면 승인된 결제를 적립 못 한다, 토스 취소 API를 붙일 때 자동 취소
            tracing::error!(order_id = %order.order_id, amount = order.amount, "toss approved but balance limit blocks credit; manual refund needed");
            return Err(WalletError::BalanceLimit.into());
        }
        Err(e) => return Err(e.into()),
    }
    db::mark_done(&mut tx, order.id, &p.payment_key).await?;

    // 후원까지 했으면 (방송, 게이지 결과) — 커밋 뒤 발행
    let mut donated = false;
    if let (Some(broadcast_id), Some(amount), Some(anonymous)) = (
        order.broadcast_id,
        order.donation_amount,
        order.donation_anonymous,
    ) && let Some((owner_id, status)) = live
        && status == "LIVE"
        && !crate::chat::ban_state(&mut *tx, owner_id, order.user_id)
            .await?
            .is_some_and(|b| b.permanent)
    {
        match super::debit(
            &mut tx,
            order.user_id,
            amount,
            DebitKind::Donation { broadcast_id },
        )
        .await
        {
            Ok(posted) => {
                super::insert_donation(
                    &mut tx,
                    &NewDonation {
                        broadcast_id,
                        donor_id: order.user_id,
                        kind: DonationKind::Donation,
                        amount,
                        anonymous,
                        message: order.donation_message.as_deref(),
                        ledger_entry_id: posted.ledger_entry_id,
                    },
                )
                .await?;
                donated = true;
            }
            // 주문 뒤 잔액을 다른 데 써서 모자라면 충전만 한다(notice).
            Err(WalletError::Insufficient) => {}
            Err(e) => return Err(e.into()),
        }
    }
    tx.commit().await?;
    tracing::info!(order_id = %order.order_id, amount = order.amount, purpose = %order.purpose, donated, "payment confirmed");
    let result = result_of(state, &order).await?;
    // 이번 확정이 후원까지 했을 때만(재시도·웹훅 재확정은 위에서 이미 돌아가 다시 발행하지 않는다)
    if donated {
        metrics::counter!("donations_total").increment(1);
        if let Some(d) = &result.donation {
            super::publish_donation(state, d.clone()).await;
        }
    }
    Ok(result)
}

/// 확정된 주문의 결과. 처음 확정 응답과 재시도·웹훅 뒤 재조회가 같은 계산을 탄다(같은 결과).
async fn result_of(state: &AppState, order: &OrderRow) -> Result<PaymentResult, AppError> {
    let is_donation = order.purpose == "donation";
    let donation = match is_donation {
        true => db::order_donation(&state.db, order.id)
            .await?
            .map(|r| r.into_donation())
            .transpose()?,
        false => None,
    };
    let notice = match (is_donation, &donation, order.broadcast_id) {
        (true, None, Some(b)) => Some(fallback_notice(state, b, order.user_id).await?.to_owned()),
        _ => None,
    };
    Ok(PaymentResult {
        order_id: order.order_id.clone(),
        purpose: if is_donation {
            PaymentPurpose::Donation
        } else {
            PaymentPurpose::Charge
        },
        charged: order.amount,
        balance: db::balance(&state.db, order.user_id).await?,
        donation,
        notice,
    })
}

/// 후원 주문이 충전만 된 이유. 종료는 되돌릴 수 없어 다시 계산해도 같다.
async fn fallback_notice(
    state: &AppState,
    broadcast_id: i64,
    user_id: i64,
) -> Result<&'static str, AppError> {
    Ok(match db::broadcast_info(&state.db, broadcast_id).await? {
        Some((owner_id, status)) if status == "LIVE" => {
            if crate::chat::ban_state(&state.db, owner_id, user_id)
                .await?
                .is_some_and(|b| b.permanent)
            {
                NOTICE_BANNED
            } else {
                NOTICE_INSUFFICIENT
            }
        }
        _ => NOTICE_ENDED,
    })
}

/// 토스 웹훅. 서명이 없어 본문은 믿지 않고 `paymentKey`로 결제 조회 API를 다시 불러 확인한다.
/// 처리·무시 모두 200, 다시 시도하면 될 수 있는 실패(조회 통신 실패·DB 오류)만 500(토스가 재전송).
pub async fn webhook(State(state): State<Arc<AppState>>, body: Bytes) -> StatusCode {
    let Some((payment_key, secret)) = toss::parse_webhook(&body) else {
        tracing::debug!("toss webhook ignored: not PAYMENT_STATUS_CHANGED");
        return StatusCode::OK;
    };
    let hint = key_hint(&payment_key);
    let p = match state.toss.payment(&payment_key).await {
        Ok(p) => p,
        Err(TossError::Rejected { code, .. }) => {
            tracing::warn!(payment_key = %hint, code, "toss webhook ignored: payment lookup rejected");
            return StatusCode::OK;
        }
        Err(TossError::Unavailable(e)) => {
            tracing::error!(payment_key = %hint, error = %e, "toss webhook: payment lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    if p.secret != secret || p.payment_key != payment_key {
        tracing::warn!(payment_key = %hint, "toss webhook ignored: secret mismatch");
        return StatusCode::OK;
    }
    if p.status != "DONE" {
        tracing::info!(payment_key = %hint, status = %p.status, "toss webhook ignored: not DONE");
        return StatusCode::OK;
    }
    match finalize(&state, &p).await {
        Ok(_) => StatusCode::OK,
        Err(e @ (AppError::Database(_) | AppError::Internal(_))) => {
            tracing::error!(order_id = %p.order_id, error = %e, "toss webhook: finalize failed");
            StatusCode::INTERNAL_SERVER_ERROR
        }
        Err(e) => {
            tracing::warn!(order_id = %p.order_id, error = %e, "toss webhook not applied");
            StatusCode::OK
        }
    }
}

#[cfg(test)]
mod tests {
    use super::bits;

    #[test]
    fn bits_thousands_separator() {
        for (n, s) in [
            (0, "0"),
            (999, "999"),
            (1000, "1,000"),
            (1_000_000, "1,000,000"),
            (123_456, "123,456"),
        ] {
            assert_eq!(bits(n), s);
        }
    }
}

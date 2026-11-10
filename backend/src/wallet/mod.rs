//! 지갑·결제·후원 도메인 — docs/features/06-wallet-donation.md
//!
//! 잔액을 바꾸는 길은 `debit`·`credit_charge` 두 개뿐이다. 둘 다 "지갑 갱신 + 원장 추가"를
//! SQL 한 문장(CTE)으로 해서, 트랜잭션 밖에서 불러도 잔액과 원장이 어긋나지 않는다.
//! 후원 기록 등과 함께 묶으려면 호출자가 연 트랜잭션의 커넥션(`&mut *tx`)을 넘긴다.

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::get;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use ts_rs::TS;

use crate::AppState;
use crate::error::AppError;
use crate::scheduler::Scheduler;

mod db;
mod handlers;

/// 보유 상한(비트). DB `wallets.balance` CHECK와 같은 값.
pub const WALLET_MAX: i64 = 3_000_000;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/me/wallet", get(handlers::get_wallet))
}

/// 주기 작업: 만료 결제 대기 주문 정리(1분)
pub fn register(_s: &mut Scheduler) {}

/// 차감 유형. 방송을 참조하는 유형은 방송 id를 함께 가진다(원장 CHECK와 같은 규칙).
#[derive(Debug, Clone, Copy)]
pub enum DebitKind {
    Donation { broadcast_id: i64 },
    Clutch { broadcast_id: i64 },
    Burst { broadcast_id: i64 },
    WithdrawForfeit,
}

impl DebitKind {
    fn parts(self) -> (&'static str, Option<i64>) {
        match self {
            Self::Donation { broadcast_id } => ("donation", Some(broadcast_id)),
            Self::Clutch { broadcast_id } => ("clutch", Some(broadcast_id)),
            Self::Burst { broadcast_id } => ("burst", Some(broadcast_id)),
            Self::WithdrawForfeit => ("withdraw_forfeit", None),
        }
    }
}

/// 원장에 추가된 행.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Posted {
    /// `donations.ledger_entry_id`에 넣을 값
    pub ledger_entry_id: i64,
    pub balance_after: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("비트가 부족해요.")]
    Insufficient,
    #[error("보유 한도(3,000,000비트)를 넘을 수 없어요.")]
    BalanceLimit,
    /// 이 주문은 이미 적립됐다(멱등 신호). 유니크 위반으로 왔으면 트랜잭션이 깨져 있으니 rollback한다.
    #[error("이미 처리된 주문이에요.")]
    AlreadyCredited,
    /// 0 이하 금액 — 호출자 버그
    #[error("invalid wallet amount {0}")]
    InvalidAmount(i64),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl From<WalletError> for AppError {
    fn from(e: WalletError) -> Self {
        let code = match e {
            WalletError::Insufficient => "WALLET_INSUFFICIENT_BALANCE",
            WalletError::BalanceLimit => "WALLET_BALANCE_LIMIT",
            WalletError::AlreadyCredited => "PAYMENT_ALREADY_CONFIRMED",
            WalletError::InvalidAmount(_) => return AppError::Internal(e.to_string()),
            WalletError::Database(e) => return AppError::Database(e),
        };
        AppError::domain(StatusCode::CONFLICT, code, e.to_string())
    }
}

/// 조건부 차감 + 원장 추가. 잔액이 모자라면(지갑 행이 없어도) `Insufficient`, 잔액·원장은 그대로.
pub async fn debit(
    conn: &mut PgConnection,
    user_id: i64,
    amount: i64,
    kind: DebitKind,
) -> Result<Posted, WalletError> {
    if amount <= 0 {
        return Err(WalletError::InvalidAmount(amount));
    }
    let (ty, broadcast_id) = kind.parts();
    db::debit(conn, user_id, amount, ty, broadcast_id)
        .await?
        .ok_or(WalletError::Insufficient)
}

/// 충전 적립 + 원장 `charge`. 상한을 넘으면 `BalanceLimit`, 같은 주문이 이미 적립됐으면 `AlreadyCredited`.
/// 주문의 주인·금액·상태 확인은 호출자(결제 확정) 몫. 지갑 행이 없으면 만든다.
pub async fn credit_charge(
    conn: &mut PgConnection,
    user_id: i64,
    amount: i64,
    payment_order_id: i64,
) -> Result<Posted, WalletError> {
    if amount <= 0 {
        return Err(WalletError::InvalidAmount(amount));
    }
    if amount > WALLET_MAX {
        return Err(WalletError::BalanceLimit);
    }
    match db::credit(conn, user_id, amount, payment_order_id).await {
        Ok(Some(p)) => Ok(p),
        Ok(None) if db::is_credited(conn, payment_order_id).await? => {
            Err(WalletError::AlreadyCredited)
        }
        Ok(None) => Err(WalletError::BalanceLimit),
        Err(e)
            if e.as_database_error().and_then(|d| d.constraint())
                == Some("ledger_entries_payment_order_id_key") =>
        {
            Err(WalletError::AlreadyCredited)
        }
        Err(e) => Err(e.into()),
    }
}

/// 잔액 일치 점검: `wallets.balance`가 원장 마지막 `balance_after`·원장 합계와 다른 사용자 id(정상이면 빈 목록).
pub async fn mismatched_wallets(db: &PgPool) -> Result<Vec<i64>, sqlx::Error> {
    db::mismatched_wallets(db).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum LedgerType {
    Charge,
    Donation,
    Clutch,
    Burst,
    WithdrawForfeit,
}

/// 지갑 내역·후원 통계용 방송 요약. hidden=true면 화면은 "삭제된 방송".
#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BroadcastRef {
    pub id: i64,
    pub title: String,
    pub hidden: bool,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LedgerEntry {
    /// 묶음이면 묶음의 마지막 원장 id
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: LedgerType,
    /// charge만 +, 나머지 -
    pub amount: i64,
    pub balance_after: i64,
    /// clutch 묶음의 건수, 그 외 1
    pub count: i64,
    pub broadcast: Option<BroadcastRef>,
    pub created_at: DateTime<Utc>,
}

/// GET /me/wallet 응답
#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Wallet {
    pub balance: i64,
    pub items: Vec<LedgerEntry>,
    pub next_cursor: Option<String>,
}

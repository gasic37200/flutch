//! 지갑·결제·후원 도메인 — docs/features/06-wallet-donation.md
//!
//! 잔액을 바꾸는 길은 `debit`·`credit_charge` 두 개뿐이다. 둘 다 "지갑 갱신 + 원장 추가"를
//! SQL 한 문장(CTE)으로 해서, 트랜잭션 밖에서 불러도 잔액과 원장이 어긋나지 않는다.
//! 후원 기록 등과 함께 묶으려면 호출자가 연 트랜잭션의 커넥션(`&mut *tx`)을 넘긴다.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{get, post};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool};
use ts_rs::TS;

use crate::AppState;
use crate::auth::{UserSummary, contains_banned_word, mask_profanity};
use crate::error::AppError;
use crate::realtime::{Event, WsEvent};
use crate::scheduler::Scheduler;

mod db;
mod donation;
mod handlers;
mod payment;

pub use payment::{
    ConfirmPaymentRequest, CreateOrderRequest, DonationInput, PaymentError, PaymentOrder,
    PaymentPurpose, PaymentResult, bits,
};

/// 보유 상한(비트). DB `wallets.balance` CHECK와 같은 값.
pub const WALLET_MAX: i64 = 3_000_000;
/// 충전·후원 1회 금액 범위(비트)
pub const AMOUNT_MIN: i64 = 1_000;
pub const AMOUNT_MAX: i64 = 1_000_000;
const DONATION_MESSAGE_MAX: usize = 100;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me/wallet", get(handlers::get_wallet))
        .route("/payments/orders", post(payment::create_order))
        .route("/payments/confirm", post(payment::confirm))
        .route("/payments/webhook", post(payment::webhook))
        .route(
            "/broadcasts/{id}/donations",
            get(donation::list).post(donation::donate),
        )
        .route("/donations/{id}/skip", post(donation::skip))
        .route("/me/donation-stats", get(donation::stats))
}

/// 주기 작업: 만료 결제 대기 주문 정리(1분)
pub fn register(s: &mut Scheduler) {
    s.every(
        "payment_order_expiry",
        Duration::from_secs(60),
        |state| async move {
            let n = expire_pending_orders(&state.db).await?;
            if n > 0 {
                tracing::info!(count = n, "payment orders expired");
            }
            Ok(())
        },
    );
}

/// 만료 시각이 지난 대기 주문 → `expired`. 바꾼 개수.
pub async fn expire_pending_orders(db: &PgPool) -> Result<u64, sqlx::Error> {
    db::expire_orders(db).await
}

/// 1,000~1,000,000 정수가 아니면 422 `fields.amount`.
pub fn check_amount(amount: i64) -> Result<(), AppError> {
    if (AMOUNT_MIN..=AMOUNT_MAX).contains(&amount) {
        return Ok(());
    }
    Err(AppError::Validation(BTreeMap::from([(
        "amount".to_owned(),
        "1,000~1,000,000비트로 입력해 주세요.".to_owned(),
    )])))
}

#[derive(Debug, thiserror::Error)]
pub enum DonationError {
    #[error("내 방송에는 후원할 수 없어요.")]
    SelfDonation,
    #[error("이 채널에서는 후원할 수 없어요.")]
    Banned,
    #[error("FLUTCH 정책상 사용할 수 없는 표현이 포함되어 있어요.")]
    BannedWord,
    #[error("방송 중일 때만 할 수 있어요.")]
    NotLive,
}

impl From<DonationError> for AppError {
    fn from(e: DonationError) -> Self {
        let (status, code) = match e {
            DonationError::SelfDonation => (StatusCode::FORBIDDEN, "DONATION_SELF"),
            DonationError::Banned => (StatusCode::FORBIDDEN, "MODERATION_BANNED"),
            DonationError::BannedWord => (StatusCode::UNPROCESSABLE_ENTITY, "DONATION_BANNED_WORD"),
            DonationError::NotLive => (StatusCode::CONFLICT, "BROADCAST_NOT_LIVE"),
        };
        AppError::domain(status, code, e.to_string())
    }
}

/// 후원 대상 확인(후원 주문·잔액 후원 공통): 없는·숨긴 방송 404 → 소유자 `DONATION_SELF` → 영구 밴
/// `MODERATION_BANNED`. 방송 상태를 돌려준다 — LIVE 확인(409)은 입력 검사(422) 뒤에 호출자가 한다.
pub async fn donation_target(
    db: &PgPool,
    user_id: i64,
    broadcast_id: i64,
) -> Result<String, AppError> {
    let (owner_id, status) = db::broadcast_info(db, broadcast_id)
        .await?
        .ok_or(AppError::NotFound)?;
    if owner_id == user_id {
        return Err(DonationError::SelfDonation.into());
    }
    if crate::chat::ban_state(db, owner_id, user_id)
        .await?
        .is_some_and(|b| b.permanent)
    {
        return Err(DonationError::Banned.into());
    }
    Ok(status)
}

/// 후원 금액(1,000~1,000,000)·메시지(0~100자, FLUTCH 금지어 불가) 검사. 메시지는 앞뒤 공백을 잘라 비면 `None`.
/// 욕설은 원문 저장, 응답에서 가린다.
pub fn check_donation_input(
    amount: i64,
    message: Option<&str>,
) -> Result<Option<String>, AppError> {
    check_amount(amount)?;
    let message = message.map(str::trim).filter(|m| !m.is_empty());
    if let Some(m) = message {
        if m.chars().count() > DONATION_MESSAGE_MAX {
            return Err(AppError::Validation(BTreeMap::from([(
                "message".to_owned(),
                "100자 이하로 입력해 주세요.".to_owned(),
            )])));
        }
        if contains_banned_word(m) {
            return Err(DonationError::BannedWord.into());
        }
    }
    Ok(message.map(str::to_owned))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DonationKind {
    Donation,
    /// 유료 클러치 — 합계에만 들어가고 목록·알림에는 나오지 않는다
    Clutch,
    Burst,
}

impl DonationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Donation => "donation",
            Self::Clutch => "clutch",
            Self::Burst => "burst",
        }
    }
}

/// 후원 한 건. `donor`는 공개형이면 익명일 때 null, 소유자형(·내 후원)이면 항상 실제 사용자.
#[derive(Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Donation {
    pub id: i64,
    pub broadcast_id: i64,
    pub kind: DonationKind,
    pub amount: i64,
    /// 욕설 가림 적용
    pub message: Option<String>,
    pub anonymous: bool,
    pub donor: Option<UserSummary>,
    pub created_at: DateTime<Utc>,
}

impl db::DonationRow {
    fn into_donation(self) -> Result<Donation, AppError> {
        let kind = match self.kind.as_str() {
            "donation" => DonationKind::Donation,
            "clutch" => DonationKind::Clutch,
            "burst" => DonationKind::Burst,
            k => return Err(AppError::Internal(format!("donation kind {k}"))),
        };
        Ok(Donation {
            id: self.id,
            broadcast_id: self.broadcast_id,
            kind,
            amount: self.amount,
            message: self.message.as_deref().map(mask_profanity),
            anonymous: self.anonymous,
            donor: Some(UserSummary {
                id: self.donor_id,
                nickname: self.nickname,
            }),
            created_at: self.created_at,
        })
    }
}

impl Donation {
    /// 공개형/소유자형 변환은 이것 하나로 한다(목록·이벤트 모두). 익명 후원은 소유자가 아니면 `donor`를 지운다.
    pub fn visible_to(mut self, is_owner: bool) -> Self {
        if self.anonymous && !is_owner {
            self.donor = None;
        }
        self
    }
}

/// `donation.created` 이벤트 data
#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub struct DonationCreated {
    pub donation: Donation,
    /// 이 방송이 받은 비트 합계(donation + clutch + burst)
    pub total: i64,
}

/// `donation.created`를 split으로 발행한다(**커밋 뒤**, `d`는 실명형). 합계 조회가 실패하면 이벤트만
/// 건너뛴다 — 후원은 이미 확정됐고 이벤트 누락은 허용(재조회로 회복).
pub async fn publish_donation(state: &AppState, d: Donation) {
    let total = match db::donation_totals(&state.db, d.broadcast_id).await {
        Ok(t) => t.total,
        Err(e) => {
            tracing::warn!(donation_id = d.id, error = %e, "donation.created skipped: totals query failed");
            return;
        }
    };
    let broadcast_id = d.broadcast_id;
    let public = DonationCreated {
        donation: d.clone().visible_to(false),
        total,
    };
    let owner = DonationCreated {
        donation: d.visible_to(true),
        total,
    };
    state.realtime.publish(
        broadcast_id,
        Event::split(
            WsEvent::DonationCreated(public),
            WsEvent::DonationCreated(owner),
        ),
    );
}

/// 후원 기록 입력. `ledger_entry_id`는 같은 트랜잭션의 `debit` 결과.
pub struct NewDonation<'a> {
    pub broadcast_id: i64,
    pub donor_id: i64,
    pub kind: DonationKind,
    pub amount: i64,
    pub anonymous: bool,
    /// 원문(검사는 `check_donation_input`)
    pub message: Option<&'a str>,
    pub ledger_entry_id: i64,
}

/// 후원 기록 추가 — 차감(`debit`)과 같은 트랜잭션에서 부른다. 돌려주는 `Donation`은 실명(소유자형).
pub async fn insert_donation(
    conn: &mut PgConnection,
    d: &NewDonation<'_>,
) -> Result<Donation, AppError> {
    db::insert_donation(conn, d).await?.into_donation()
}

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
    /// 테스트 지급(+). 로컬 QA 시드(backend/scripts/seed_qa.sql)만 넣는다.
    Grant,
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
    /// charge·grant만 +, 나머지 -
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

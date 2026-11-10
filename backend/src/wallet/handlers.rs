use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use chrono::{DateTime, FixedOffset, Months, NaiveDate, Utc};
use serde::Deserialize;

use super::db::{self, Filter};
use super::{BroadcastRef, LedgerEntry, LedgerType, Wallet};
use crate::AppState;
use crate::error::AppError;
use crate::extract::CurrentUser;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

#[derive(Deserialize)]
pub struct WalletQuery {
    #[serde(rename = "type")]
    ty: Option<String>,
    month: Option<String>,
    cursor: Option<String>,
    limit: Option<String>,
}

/// `YYYY-MM`(KST 달력) → 그 달의 [시작, 다음 달 시작) UTC.
fn kst_month(m: &str) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    if m.len() != 7 {
        return None;
    }
    let first = NaiveDate::parse_from_str(&format!("{m}-01"), "%Y-%m-%d").ok()?;
    let kst = FixedOffset::east_opt(9 * 3600)?; // KST는 서머타임이 없어 고정 +09:00
    let utc = |d: NaiveDate| {
        d.and_hms_opt(0, 0, 0)?
            .and_local_timezone(kst)
            .single()
            .map(|t| t.to_utc())
    };
    Some((utc(first)?, utc(first.checked_add_months(Months::new(1))?)?))
}

pub async fn get_wallet(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(q): Query<WalletQuery>,
) -> Result<Json<Wallet>, AppError> {
    let mut fields = BTreeMap::new();
    let mut f = Filter {
        user_id: user.id,
        ty: None,
        from: None,
        until: None,
        before: None,
        limit: DEFAULT_LIMIT,
    };
    if let Some(t) = q.ty {
        match serde_json::from_value::<LedgerType>(t.clone().into()) {
            Ok(_) => f.ty = Some(t),
            Err(_) => {
                fields.insert(
                    "type".to_owned(),
                    "올바르지 않은 내역 유형이에요.".to_owned(),
                );
            }
        }
    }
    if let Some(m) = q.month {
        match kst_month(&m) {
            Some((from, until)) => (f.from, f.until) = (Some(from), Some(until)),
            None => {
                fields.insert(
                    "month".to_owned(),
                    "YYYY-MM 형식으로 입력해 주세요.".to_owned(),
                );
            }
        }
    }
    if let Some(c) = q.cursor {
        match c.parse::<i64>() {
            Ok(id) => f.before = Some(id),
            Err(_) => {
                fields.insert("cursor".to_owned(), "올바르지 않은 커서예요.".to_owned());
            }
        }
    }
    if let Some(l) = q.limit {
        match l.parse::<i64>() {
            Ok(n) => f.limit = n.clamp(1, MAX_LIMIT),
            Err(_) => {
                fields.insert("limit".to_owned(), "숫자로 입력해 주세요.".to_owned());
            }
        }
    }
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }

    let limit = f.limit;
    f.limit += 1;
    let mut rows = db::entries(&state.db, &f).await?;
    let next_cursor = if rows.len() as i64 > limit {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.first_id.to_string())
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|r| {
            Ok(LedgerEntry {
                id: r.id,
                kind: serde_json::from_value(r.ty.into())
                    .map_err(|e| AppError::Internal(format!("ledger type: {e}")))?,
                amount: r.amount,
                balance_after: r.balance_after,
                count: r.count,
                broadcast: r.broadcast_id.map(|id| BroadcastRef {
                    id,
                    title: r.broadcast_title.unwrap_or_default(),
                    hidden: r.broadcast_hidden.unwrap_or(false),
                }),
                created_at: r.created_at,
            })
        })
        .collect::<Result<_, AppError>>()?;
    Ok(Json(Wallet {
        balance: db::balance(&state.db, user.id).await?,
        items,
        next_cursor,
    }))
}

#[cfg(test)]
mod tests {
    use super::kst_month;

    #[test]
    fn kst_month_bounds() {
        let (from, until) = kst_month("2026-12").unwrap();
        assert_eq!(from.to_rfc3339(), "2026-11-30T15:00:00+00:00");
        assert_eq!(until.to_rfc3339(), "2026-12-31T15:00:00+00:00");
        for bad in ["2026-13", "2026-1", "2026-01-01", "abcd-ef"] {
            assert!(kst_month(bad).is_none(), "{bad}");
        }
    }
}

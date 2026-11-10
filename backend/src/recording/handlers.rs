use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::db::{self, Filter, Row};
use super::parse_status;
use crate::AppState;
use crate::auth::UserSummary;
use crate::broadcast::{Category, Page, RecordingStatus};
use crate::error::AppError;
use crate::extract::OptionalUser;

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

#[derive(Debug, thiserror::Error)]
pub enum RecordingError {
    #[error("보관 기간이 지난 다시보기예요.")]
    Expired,
}

impl From<RecordingError> for AppError {
    fn from(e: RecordingError) -> Self {
        AppError::domain(StatusCode::NOT_FOUND, "RECORDING_EXPIRED", e.to_string())
    }
}

/// 카드·상세가 같이 쓰는 필드.
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RecordingInfo {
    pub id: i64,
    pub broadcast_id: i64,
    pub owner: UserSummary,
    pub title: String,
    pub category: Category,
    /// 방송의 마지막 라이브 캡처 재사용
    pub thumbnail_url: Option<String>,
    pub peak_viewers: i32,
    pub views: i64,
    pub started_at: DateTime<Utc>,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingCard {
    #[serde(flatten)]
    #[ts(flatten)]
    pub info: RecordingInfo,
    pub duration_sec: i32,
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordingDetail {
    #[serde(flatten)]
    #[ts(flatten)]
    pub info: RecordingInfo,
    pub status: RecordingStatus,
    pub duration_sec: Option<i32>,
    /// `ready`만. Object Storage 공개 URL
    pub video_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

fn info(r: &Row) -> Result<RecordingInfo, AppError> {
    Ok(RecordingInfo {
        id: r.id,
        broadcast_id: r.broadcast_id,
        owner: UserSummary {
            id: r.owner_id,
            nickname: r.owner_nickname.clone(),
        },
        title: r.title.clone(),
        category: serde_json::from_value(r.category.clone().into())
            .map_err(|_| AppError::Internal(format!("unexpected category {:?}", r.category)))?,
        thumbnail_url: r
            .thumbnail_path
            .as_ref()
            .map(|v| format!("/media/thumbs/{}.jpg?v={v}", r.broadcast_id)),
        peak_viewers: r.peak_viewers,
        views: r.views,
        started_at: r.started_at,
    })
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    category: Option<String>,
    q: Option<String>,
    #[serde(rename = "userId")]
    user_id: Option<String>,
    cursor: Option<String>,
    limit: Option<String>,
}

fn bad(field: &str, msg: &str) -> AppError {
    AppError::Validation(BTreeMap::from([(field.to_owned(), msg.to_owned())]))
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Page<RecordingCard>>, AppError> {
    let num = |field: &str, v: &Option<String>| -> Result<Option<i64>, AppError> {
        v.as_deref()
            .map(|s| s.parse().map_err(|_| bad(field, "숫자로 입력해 주세요.")))
            .transpose()
    };
    let limit = num("limit", &q.limit)?
        .unwrap_or(DEFAULT_LIMIT)
        .clamp(1, MAX_LIMIT);
    let mut f = Filter {
        ready_only: true,
        before: num("cursor", &q.cursor)?,
        owner_id: num("userId", &q.user_id)?,
        limit: limit + 1,
        ..Filter::default()
    };
    if let Some(c) = &q.category {
        serde_json::from_value::<Category>(c.clone().into())
            .map_err(|_| bad("category", "올바르지 않은 카테고리예요."))?;
        f.category = Some(c.clone());
    }
    if let Some(text) = q.q.as_deref().map(str::trim) {
        if !(1..=30).contains(&text.chars().count()) {
            return Err(bad("q", "1~30자로 입력해 주세요."));
        }
        let esc = text
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        f.q = Some(format!("%{esc}%"));
    }

    let mut rows = db::find(&state.db, &f).await?;
    // 한 건 더 읽어 다음 페이지 유무를 안다
    let next_cursor = if rows.len() as i64 > limit {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.id.to_string())
    } else {
        None
    };
    let items = rows
        .iter()
        .map(|r| {
            Ok(RecordingCard {
                info: info(r)?,
                duration_sec: r.duration_sec.unwrap_or(0),
            })
        })
        .collect::<Result<_, AppError>>()?;
    Ok(Json(Page { items, next_cursor }))
}

/// 숨기지 않은 방송의 녹화 한 건. 없거나 숨김이면 404, 보관 기간이 지났으면 `RECORDING_EXPIRED`.
async fn load(state: &AppState, id: i64) -> Result<Row, AppError> {
    let f = Filter {
        id: Some(id),
        limit: 1,
        ..Filter::default()
    };
    let row = db::find(&state.db, &f)
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound)?;
    if row.status == "deleted" {
        return Err(RecordingError::Expired.into());
    }
    Ok(row)
}

pub async fn get_one(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<RecordingDetail>, AppError> {
    let r = load(&state, id).await?;
    let status = parse_status(&r.status)?;
    let video_url = match (status, &r.object_key) {
        (RecordingStatus::Ready, Some(key)) => {
            Some(format!("{}/{key}", state.config.recording.public_base_url))
        }
        _ => None,
    };
    Ok(Json(RecordingDetail {
        info: info(&r)?,
        status,
        duration_sec: r.duration_sec,
        video_url,
        expires_at: r.expires_at,
    }))
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct ViewRequest {
    guest_id: Option<String>,
}

pub async fn add_view(
    State(state): State<Arc<AppState>>,
    OptionalUser(user): OptionalUser,
    Path(id): Path<i64>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let req: ViewRequest = if body.is_empty() {
        ViewRequest::default()
    } else {
        serde_json::from_slice(&body).map_err(|_| bad("body", "요청 형식이 올바르지 않습니다."))?
    };
    let viewer_key = match (&user, &req.guest_id) {
        (Some(u), _) => format!("u:{}", u.id),
        (None, Some(g))
            if (8..=64).contains(&g.len())
                && g.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') =>
        {
            format!("g:{g}")
        }
        _ => return Err(bad("guestId", "비로그인은 guestId가 필요해요.")),
    };
    if load(&state, id).await?.status != "ready" {
        return Err(AppError::NotFound);
    }
    db::add_view(&state.db, id, &viewer_key).await?;
    Ok(StatusCode::NO_CONTENT)
}

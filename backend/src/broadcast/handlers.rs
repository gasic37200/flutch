use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use serde::Deserialize;

use super::db::{self, Filter, Patch};
use super::{
    BROADCAST_READY_LIMIT, BURST_PRICE, Broadcast, BroadcastDetail, BroadcastError,
    BroadcastStatus, Category, CreateBroadcastRequest, EndReason, Page, UpdateBroadcastRequest,
    ViewerState, end_live, enum_str, parse_enum,
};
use crate::AppState;
use crate::auth::{contains_banned_word, contains_profanity};
use crate::clutch;
use crate::error::AppError;
use crate::extract::{CurrentUser, JsonBody, OptionalUser};
use crate::live::LiveError;
use crate::realtime::{Event, WsEvent};

const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

type Fields = BTreeMap<String, String>;

#[derive(Deserialize, Default)]
pub struct ListQuery {
    category: Option<String>,
    q: Option<String>,
    cursor: Option<String>,
    limit: Option<String>,
    status: Option<String>,
}

/// 쿼리 → 조회 조건. 형식이 틀리면 422.
fn filter_from(q: &ListQuery) -> Result<(Filter, i64), AppError> {
    let mut fields = Fields::new();
    let mut f = Filter::default();

    let limit = match q.limit.as_deref().map(str::parse::<i64>) {
        None => DEFAULT_LIMIT,
        Some(Ok(n)) => n.clamp(1, MAX_LIMIT),
        Some(Err(_)) => {
            fields.insert("limit".into(), "숫자로 입력해 주세요.".into());
            DEFAULT_LIMIT
        }
    };
    if let Some(c) = &q.cursor {
        match c.parse::<i64>() {
            Ok(id) => f.before = Some(id),
            Err(_) => {
                fields.insert("cursor".into(), "올바르지 않은 커서예요.".into());
            }
        }
    }
    if let Some(c) = &q.category {
        match serde_json::from_value::<Category>(c.clone().into()) {
            Ok(c) => f.category = Some(enum_str(&c)?),
            Err(_) => {
                fields.insert("category".into(), "올바르지 않은 카테고리예요.".into());
            }
        }
    }
    if let Some(text) = q.q.as_deref().map(str::trim) {
        if (1..=30).contains(&text.chars().count()) {
            let esc = text
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_");
            f.q = Some(format!("%{esc}%"));
        } else {
            fields.insert("q".into(), "1~30자로 입력해 주세요.".into());
        }
    }
    f.status = match q.status.as_deref() {
        None => None,
        Some("READY") => Some("READY"),
        Some("LIVE") => Some("LIVE"),
        Some("ENDED") => Some("ENDED"),
        Some(_) => {
            fields.insert(
                "status".into(),
                "READY, LIVE, ENDED 중 하나여야 해요.".into(),
            );
            None
        }
    };
    if fields.is_empty() {
        Ok((f, limit))
    } else {
        Err(AppError::Validation(fields))
    }
}

pub(super) fn into_broadcast(state: &AppState, row: db::Row) -> Result<Broadcast, AppError> {
    let n = state.live.viewer_count(row.id);
    row.into_broadcast(n)
}

async fn page(
    state: &AppState,
    mut f: Filter,
    limit: i64,
) -> Result<Json<Page<Broadcast>>, AppError> {
    f.limit = limit + 1;
    let mut rows = db::find(&state.db, &f).await?;
    let next_cursor = (rows.len() as i64 > limit).then(|| {
        rows.truncate(limit as usize);
        rows.last().map(|r| r.id.to_string())
    });
    let items = rows
        .into_iter()
        .map(|r| into_broadcast(state, r))
        .collect::<Result<_, _>>()?;
    Ok(Json(Page {
        items,
        next_cursor: next_cursor.flatten(),
    }))
}

/// LIVE 목록, 시청자 수 내림차순(동률 최신순). 시청자 수는 메모리 값이라 DB가 정렬할 수 없어
/// 조건에 맞는 LIVE를 전부 읽어 정렬하고, 커서는 순위 위치(offset)다.
/// shortcut: 페이지 사이에 시청자 수가 바뀌면 경계에서 중복·누락될 수 있다(홈은 10초 폴링이라 허용),
/// LIVE가 수백 개를 넘으면 시청자 수를 DB에 두고 DB에서 정렬한다.
pub async fn list_live(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Page<Broadcast>>, AppError> {
    let (mut f, limit) = filter_from(&q)?;
    let offset = usize::try_from(f.before.take().unwrap_or(0)).unwrap_or(0);
    f.status = Some("LIVE");
    f.limit = i64::MAX;
    let mut items = db::find(&state.db, &f)
        .await?
        .into_iter()
        .map(|r| into_broadcast(&state, r))
        .collect::<Result<Vec<_>, _>>()?;
    items.sort_by_key(|b| std::cmp::Reverse((b.viewer_count, b.id)));
    let end = offset.saturating_add(limit as usize);
    let next_cursor = (items.len() > end).then(|| end.to_string());
    let items = items
        .into_iter()
        .skip(offset)
        .take(limit as usize)
        .collect();
    Ok(Json(Page { items, next_cursor }))
}

pub async fn list_mine(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Query(q): Query<ListQuery>,
) -> Result<Json<Page<Broadcast>>, AppError> {
    let (mut f, limit) = filter_from(&q)?;
    f.owner_id = Some(user.id);
    page(&state, f, limit).await
}

pub async fn list_by_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Page<Broadcast>>, AppError> {
    sqlx::query_scalar!("SELECT id FROM users WHERE id = $1", user_id)
        .fetch_one(&state.db)
        .await?;
    let (mut f, limit) = filter_from(&q)?;
    f.owner_id = Some(user_id);
    f.status = Some("ENDED");
    page(&state, f, limit).await
}

pub(super) async fn load(state: &AppState, id: i64) -> Result<db::Row, AppError> {
    let f = Filter {
        id: Some(id),
        limit: 1,
        ..Filter::default()
    };
    db::find(&state.db, &f)
        .await?
        .into_iter()
        .next()
        .ok_or(AppError::NotFound)
}

/// 상세 응답. shortcut: burstPrice는 기본값, T24가 이 함수에서 채운다.
pub(super) async fn detail(
    state: &AppState,
    row: db::Row,
    viewer: Option<&CurrentUser>,
) -> Result<BroadcastDetail, AppError> {
    let id = row.id;
    let is_owner = viewer.is_some_and(|u| u.id == row.owner_id);
    let cfg = &state.config.clutch;
    let gauge = if row.status == "LIVE" {
        clutch::gauge_of(&state.db, cfg, id).await?
    } else {
        clutch::idle_gauge()
    };
    let free_clutches_left = match viewer {
        Some(u) => clutch::free_left(&state.db, cfg, u.id, id).await?,
        None => 0,
    };
    let ban = match viewer {
        Some(u) => crate::chat::ban_state(&state.db, row.owner_id, u.id).await?,
        None => None,
    };
    Ok(BroadcastDetail {
        broadcast: into_broadcast(state, row)?,
        gauge,
        burst_price: BURST_PRICE,
        viewer: viewer.map(|_| ViewerState {
            is_owner,
            ban,
            free_clutches_left,
        }),
    })
}

pub async fn get_one(
    State(state): State<Arc<AppState>>,
    OptionalUser(user): OptionalUser,
    Path(id): Path<i64>,
) -> Result<Json<BroadcastDetail>, AppError> {
    Ok(Json(
        detail(&state, load(&state, id).await?, user.as_ref()).await?,
    ))
}

fn check_title(title: &str) -> Result<String, &'static str> {
    let title = title.trim();
    if !(2..=50).contains(&title.chars().count()) {
        return Err("2~50자로 입력해 주세요.");
    }
    if contains_banned_word(title) || contains_profanity(title) {
        return Err("사용할 수 없는 표현이 포함되어 있어요.");
    }
    Ok(title.to_owned())
}

fn check_tags(tags: &[String]) -> Result<Vec<String>, &'static str> {
    let mut out: Vec<String> = Vec::new();
    for t in tags {
        let t = t.trim();
        if !(1..=10).contains(&t.chars().count()) {
            return Err("태그는 1~10자로 입력해 주세요.");
        }
        if contains_banned_word(t) || contains_profanity(t) {
            return Err("사용할 수 없는 표현이 포함되어 있어요.");
        }
        if !out.iter().any(|o| o == t) {
            out.push(t.to_owned());
        }
    }
    if out.len() > 3 {
        return Err("태그는 최대 3개까지 쓸 수 있어요.");
    }
    Ok(out)
}

fn field_err<T>(fields: &mut Fields, name: &str, r: Result<T, &'static str>) -> Option<T> {
    r.map_err(|reason| fields.insert(name.to_owned(), reason.to_owned()))
        .ok()
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    JsonBody(req): JsonBody<CreateBroadcastRequest>,
) -> Result<(StatusCode, Json<BroadcastDetail>), AppError> {
    let mut fields = Fields::new();
    let title = field_err(&mut fields, "title", check_title(&req.title));
    let tags = field_err(&mut fields, "tags", check_tags(&req.tags));
    let (Some(title), Some(tags)) = (title, tags) else {
        return Err(AppError::Validation(fields));
    };
    let id = db::insert_ready(
        &state.db,
        user.id,
        &title,
        &enum_str(&req.category)?,
        &tags,
        req.record,
        BROADCAST_READY_LIMIT,
    )
    .await?
    .ok_or(BroadcastError::ReadyLimit)?;
    tracing::info!(broadcast_id = id, owner_id = user.id, "broadcast created");
    let d = detail(&state, load(&state, id).await?, Some(&user)).await?;
    Ok((StatusCode::CREATED, Json(d)))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    JsonBody(req): JsonBody<UpdateBroadcastRequest>,
) -> Result<Json<BroadcastDetail>, AppError> {
    let row = load(&state, id).await?;
    if row.owner_id != user.id {
        return Err(BroadcastError::NotOwner.into());
    }
    let status: BroadcastStatus = parse_enum(&row.status)?;

    let mut fields = Fields::new();
    let title = req
        .title
        .as_deref()
        .and_then(|t| field_err(&mut fields, "title", check_title(t)));
    let tags = req
        .tags
        .as_deref()
        .and_then(|t| field_err(&mut fields, "tags", check_tags(t)));
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }

    // 상태별 수정 가능 필드(features/02): READY 전부, LIVE는 녹화 여부 제외, ENDED는 제목만
    let locked = match status {
        BroadcastStatus::Ready => false,
        BroadcastStatus::Live => req.record.is_some(),
        BroadcastStatus::Ended => {
            req.category.is_some()
                || req.tags.is_some()
                || req.record.is_some()
                || req.pip_corner.is_some()
        }
    };
    if locked {
        return Err(BroadcastError::FieldLocked.into());
    }

    let patch = Patch {
        title,
        category: req.category.as_ref().map(enum_str).transpose()?,
        tags,
        record: req.record,
        pip_corner: req.pip_corner.as_ref().map(enum_str).transpose()?,
    };
    if !db::update_fields(&state.db, id, status, &patch).await? {
        return Err(BroadcastError::InvalidTransition.into());
    }
    let d = detail(&state, load(&state, id).await?, Some(&user)).await?;
    let event = WsEvent::BroadcastUpdated(d.broadcast.clone());
    state.realtime.publish(id, Event::public(event));
    Ok(Json(d))
}

const MAX_THUMBNAIL_BYTES: usize = 512 * 1024;

fn thumbnail_file(state: &AppState, id: i64) -> std::path::PathBuf {
    state.config.broadcast.thumb_dir.join(format!("{id}.jpg"))
}

/// 송출 중 캡처 1장을 `{id}.jpg`로 덮어쓴다(임시 파일 → rename으로 nginx가 반쯤 쓴 파일을 읽지 않게).
pub async fn put_thumbnail(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let row = load(&state, id).await?;
    if row.owner_id != user.id {
        return Err(BroadcastError::NotOwner.into());
    }
    if row.status != "LIVE" {
        return Err(LiveError::NotLive.into());
    }
    let is_jpeg_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.trim().eq_ignore_ascii_case("image/jpeg"));
    if !is_jpeg_type || body.len() > MAX_THUMBNAIL_BYTES || !body.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Err(BroadcastError::ThumbnailInvalid.into());
    }

    let path = thumbnail_file(&state, id);
    let tmp = path.with_extension("jpg.tmp");
    let write = async {
        tokio::fs::create_dir_all(&state.config.broadcast.thumb_dir).await?;
        tokio::fs::write(&tmp, &body).await?;
        tokio::fs::rename(&tmp, &path).await
    };
    write
        .await
        .map_err(|e| AppError::Internal(format!("thumbnail write failed: {e}")))?;

    let version = chrono::Utc::now().timestamp_millis().to_string();
    if !db::set_thumbnail(&state.db, id, &version).await? {
        return Err(LiveError::NotLive.into());
    }
    let d = detail(&state, load(&state, id).await?, Some(&user)).await?;
    let event = WsEvent::BroadcastUpdated(d.broadcast);
    state.realtime.publish(id, Event::public(event));
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let row = load(&state, id).await?;
    if row.owner_id != user.id {
        return Err(BroadcastError::NotOwner.into());
    }
    match parse_enum::<BroadcastStatus>(&row.status)? {
        BroadcastStatus::Ready => {
            if !db::delete_ready(&state.db, id).await? {
                return Err(BroadcastError::InvalidTransition.into());
            }
        }
        BroadcastStatus::Live => {
            // 송출을 먼저 끝낸 뒤 ENDED 규칙(숨김)을 적용한다.
            end_live(&state, id, EndReason::Deleted).await?;
            db::hide_ended(&state.db, id).await?;
        }
        BroadcastStatus::Ended => {
            if !db::hide_ended(&state.db, id).await? {
                return Err(AppError::NotFound);
            }
        }
    }
    // 숨김 처리된 방송의 녹화 파일 삭제. 실패해도 삭제는 성공(deleted 표시가 남아 1시간 주기 작업이 다시 지운다).
    if let Err(e) = crate::recording::purge(&state).await {
        tracing::error!(broadcast_id = id, error = %e, "recording purge failed");
    }
    // 숨김 처리된 방송의 썸네일 파일은 더 쓰이지 않는다. 실패해도 삭제는 성공(없으면 정상).
    let _ = tokio::fs::remove_file(thumbnail_file(&state, id)).await;
    tracing::info!(broadcast_id = id, owner_id = user.id, "broadcast deleted");
    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Row, SqlitePool,
};
use std::{path::PathBuf, str::FromStr, time::Duration};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub studio: StudioVerifier,
}

/// Premium operations are authorized by Sociobot at the API boundary.  The
/// browser's cached verdict is only a UI optimization and is never trusted for
/// writes that create a paid entitlement.
#[derive(Clone)]
pub enum StudioVerifier {
    Billing {
        client: reqwest::Client,
        base_url: String,
    },
    Static {
        valid_license: String,
    },
}

impl StudioVerifier {
    pub fn billing(base_url: String) -> Self {
        Self::Billing {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("build billing HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    async fn verify(&self, license: &str) -> Result<bool, ()> {
        match self {
            Self::Static { valid_license } => Ok(license == valid_license),
            Self::Billing { client, base_url } => {
                let response = client
                    .get(format!(
                        "{base_url}/api/v1/products/rubric-revision-loop/verify"
                    ))
                    .query(&[("license", license)])
                    .send()
                    .await
                    .map_err(|_| ())?;
                if !response.status().is_success() {
                    return Err(());
                }
                #[derive(Deserialize)]
                struct LicenseVerdict {
                    valid: bool,
                }
                response
                    .json::<LicenseVerdict>()
                    .await
                    .map(|verdict| verdict.valid)
                    .map_err(|_| ())
            }
        }
    }
}

pub struct AppConfig {
    pub dist_dir: PathBuf,
}

pub async fn open_database(url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(if url.contains(":memory:") { 1 } else { 8 })
        .connect_with(options)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}

pub fn build_app(state: AppState, config: AppConfig) -> Router {
    let request_id = HeaderName::from_static("x-request-id");
    let api = Router::new()
        .route("/health", get(health))
        .route("/rubrics", get(list_rubrics).post(create_rubric))
        .route("/rubrics/{id}", delete(delete_rubric))
        .route("/loops", get(list_loops).post(create_loop))
        .route("/loops/{id}", delete(delete_loop))
        .route("/loops/{id}/review", patch(review_loop))
        .route("/student/{token}", get(student_loop))
        .route("/student/{token}/revision", post(submit_revision))
        .route("/packs", post(create_pack))
        .route("/packs/{token}/import", post(import_pack))
        .route("/export", get(export_workspace))
        .route("/workspace", delete(delete_workspace))
        .fallback(api_not_found)
        .layer(DefaultBodyLimit::max(64 * 1024));
    let static_files = ServeDir::new(&config.dist_dir)
        .append_index_html_on_directories(true)
        .fallback(ServeFile::new(config.dist_dir.join("index.html")));
    Router::new()
        .nest("/api", api)
        .fallback_service(static_files)
        .with_state(state)
        .layer(SetResponseHeaderLayer::if_not_present(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")))
        .layer(SetResponseHeaderLayer::if_not_present(header::REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin")))
        .layer(SetResponseHeaderLayer::if_not_present(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY")))
        .layer(SetResponseHeaderLayer::if_not_present(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.sociobot.in https://pilot-api.sociobot.in; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in")))
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(cache_control))
}

/// Keep deploys fresh while allowing content-addressed production assets to stay cached.
/// Student and teacher API responses contain private classroom data, so they are never
/// browser-cacheable.
async fn cache_control(request: axum::extract::Request, next: Next) -> Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let value = if path.starts_with("/api/") {
        "no-store"
    } else if path.starts_with("/assets/") {
        "public, max-age=31536000, immutable"
    } else {
        // HTML and the service-worker entry point must be revalidated so a new
        // deployment can take control immediately.
        "no-cache"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    response
}

#[derive(Debug, Serialize)]
struct Health<'a> {
    status: &'a str,
    build_sha: &'a str,
}

async fn health() -> Json<Health<'static>> {
    Json(Health {
        status: "ok",
        build_sha: option_env!("BUILD_SHA").unwrap_or("dev"),
    })
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("API route not found.")
}

#[derive(Debug, Serialize, Clone)]
pub struct RubricCode {
    pub id: i64,
    pub code: String,
    pub title: String,
    pub guidance: String,
    pub next_step: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct RubricInput {
    code: String,
    title: String,
    guidance: String,
    next_step: String,
}

#[derive(Debug, Serialize)]
struct ListResponse<T> {
    items: Vec<T>,
}

async fn list_rubrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListResponse<RubricCode>>, ApiError> {
    let key = workspace_key(&headers)?;
    let rows = sqlx::query("SELECT id, code, title, guidance, next_step, created_at FROM rubric_codes WHERE workspace_key = ? ORDER BY code")
        .bind(key).fetch_all(&state.pool).await?;
    Ok(Json(ListResponse {
        items: rows.into_iter().map(rubric_from_row).collect(),
    }))
}

async fn create_rubric(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RubricInput>,
) -> Result<(StatusCode, Json<RubricCode>), ApiError> {
    let key = workspace_key(&headers)?;
    let code = clean_required("Code", &input.code, 2, 12)?.to_uppercase();
    if !code
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return Err(ApiError::validation(
            "Code can use letters, numbers, hyphens, and periods.",
        ));
    }
    let title = clean_required("Criterion name", &input.title, 2, 80)?;
    let guidance = clean_required("Student guidance", &input.guidance, 8, 600)?;
    let next_step = clean_required("Revision prompt", &input.next_step, 8, 300)?;
    let result = sqlx::query("INSERT INTO rubric_codes (workspace_key, code, title, guidance, next_step) VALUES (?, ?, ?, ?, ?)")
        .bind(&key).bind(&code).bind(&title).bind(&guidance).bind(&next_step).execute(&state.pool).await;
    let id = match result {
        Ok(r) => r.last_insert_rowid(),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            return Err(ApiError::conflict(
                "That rubric code already exists in this workspace.",
            ))
        }
        Err(e) => return Err(e.into()),
    };
    let row = sqlx::query(
        "SELECT id, code, title, guidance, next_step, created_at FROM rubric_codes WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok((StatusCode::CREATED, Json(rubric_from_row(row))))
}

async fn delete_rubric(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let key = workspace_key(&headers)?;
    let linked: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM loop_codes lc JOIN feedback_loops fl ON fl.id = lc.loop_id WHERE lc.rubric_id = ? AND fl.workspace_key = ? AND fl.deleted_at IS NULL)",
    )
    .bind(id)
    .bind(&key)
    .fetch_one(&state.pool)
    .await?;
    if linked {
        return Err(ApiError::conflict(
            "This code is used in a feedback link. Delete that link first.",
        ));
    }
    let mut tx = state.pool.begin().await?;
    // Older deployments soft-deleted links without removing their join rows.
    // Those links are unreachable, so clear their stale relationships before
    // deleting the reusable code they used to reference.
    sqlx::query(
        "DELETE FROM loop_codes WHERE rubric_id = ? AND loop_id IN (SELECT id FROM feedback_loops WHERE deleted_at IS NOT NULL)",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    let result = sqlx::query("DELETE FROM rubric_codes WHERE id = ? AND workspace_key = ?")
        .bind(id)
        .bind(key)
        .execute(&mut *tx)
        .await;
    match result {
        Ok(r) if r.rows_affected() == 0 => Err(ApiError::not_found("Rubric code not found.")),
        Ok(_) => {
            tx.commit().await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(sqlx::Error::Database(e)) if e.is_foreign_key_violation() => Err(ApiError::conflict(
            "This code is used in a feedback link. Delete that link first.",
        )),
        Err(e) => Err(e.into()),
    }
}

fn rubric_from_row(row: sqlx::sqlite::SqliteRow) -> RubricCode {
    RubricCode {
        id: row.get("id"),
        code: row.get("code"),
        title: row.get("title"),
        guidance: row.get("guidance"),
        next_step: row.get("next_step"),
        created_at: row.get("created_at"),
    }
}

#[derive(Debug, Deserialize)]
struct LoopInput {
    assignment_title: String,
    student_label: Option<String>,
    teacher_note: Option<String>,
    rubric_ids: Vec<i64>,
    retention_days: Option<i64>,
}

#[derive(Debug, Serialize)]
struct CreatedLoop {
    id: i64,
    token: String,
    status: String,
    created_at: String,
}

async fn create_loop(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<LoopInput>,
) -> Result<(StatusCode, Json<CreatedLoop>), ApiError> {
    let key = workspace_key(&headers)?;
    let title = clean_required("Assignment title", &input.assignment_title, 2, 120)?;
    let label = clean_optional(input.student_label.as_deref().unwrap_or(""), 80)?;
    let note = clean_optional(input.teacher_note.as_deref().unwrap_or(""), 800)?;
    if input.rubric_ids.is_empty() || input.rubric_ids.len() > 12 {
        return Err(ApiError::validation(
            "Choose between 1 and 12 rubric codes.",
        ));
    }
    let retention = input.retention_days.unwrap_or(30);
    if !(7..=365).contains(&retention) {
        return Err(ApiError::validation(
            "Retention must be between 7 and 365 days.",
        ));
    }
    if retention > 30 {
        require_studio(&state, &headers).await?;
    }
    let mut tx = state.pool.begin().await?;
    let placeholders = std::iter::repeat_n("?", input.rubric_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let check_sql = format!("SELECT COUNT(*) AS count FROM rubric_codes WHERE workspace_key = ? AND id IN ({placeholders})");
    let mut check = sqlx::query(&check_sql).bind(&key);
    for id in &input.rubric_ids {
        check = check.bind(id);
    }
    let count: i64 = check.fetch_one(&mut *tx).await?.get("count");
    if count != input.rubric_ids.len() as i64 {
        return Err(ApiError::validation(
            "One or more rubric codes no longer exist.",
        ));
    }
    let token = Uuid::new_v4().simple().to_string();
    let result = sqlx::query("INSERT INTO feedback_loops (token, workspace_key, student_label, assignment_title, teacher_note, retention_days) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&token).bind(&key).bind(&label).bind(&title).bind(&note).bind(retention).execute(&mut *tx).await?;
    let id = result.last_insert_rowid();
    for rubric_id in &input.rubric_ids {
        sqlx::query("INSERT INTO loop_codes (loop_id, rubric_id) VALUES (?, ?)")
            .bind(id)
            .bind(rubric_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    let created_at: String =
        sqlx::query_scalar("SELECT created_at FROM feedback_loops WHERE id = ?")
            .bind(id)
            .fetch_one(&state.pool)
            .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreatedLoop {
            id,
            token,
            status: "awaiting".into(),
            created_at,
        }),
    ))
}

#[derive(Debug, Serialize)]
pub struct FeedbackLoop {
    id: i64,
    token: String,
    student_label: String,
    assignment_title: String,
    teacher_note: String,
    status: String,
    before_excerpt: Option<String>,
    after_excerpt: Option<String>,
    explanation: Option<String>,
    checklist: Vec<i64>,
    retention_days: i64,
    created_at: String,
    submitted_at: Option<String>,
    reviewed_at: Option<String>,
    rubrics: Vec<RubricCode>,
}

async fn list_loops(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ListResponse<FeedbackLoop>>, ApiError> {
    let key = workspace_key(&headers)?;
    let rows = sqlx::query("SELECT * FROM feedback_loops WHERE workspace_key = ? AND deleted_at IS NULL ORDER BY CASE status WHEN 'submitted' THEN 0 WHEN 'awaiting' THEN 1 ELSE 2 END, created_at DESC")
        .bind(key).fetch_all(&state.pool).await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        items.push(loop_from_row(&state.pool, row).await?);
    }
    Ok(Json(ListResponse { items }))
}

async fn loop_from_row(
    pool: &SqlitePool,
    row: sqlx::sqlite::SqliteRow,
) -> Result<FeedbackLoop, ApiError> {
    let id: i64 = row.get("id");
    let rubrics = rubric_rows_for_loop(pool, id).await?;
    let checklist_raw: String = row.get("checklist_json");
    Ok(FeedbackLoop {
        id,
        token: row.get("token"),
        student_label: row.get("student_label"),
        assignment_title: row.get("assignment_title"),
        teacher_note: row.get("teacher_note"),
        status: row.get("status"),
        before_excerpt: row.get("before_excerpt"),
        after_excerpt: row.get("after_excerpt"),
        explanation: row.get("explanation"),
        checklist: serde_json::from_str(&checklist_raw).unwrap_or_default(),
        retention_days: row.get("retention_days"),
        created_at: row.get("created_at"),
        submitted_at: row.get("submitted_at"),
        reviewed_at: row.get("reviewed_at"),
        rubrics,
    })
}

async fn rubric_rows_for_loop(pool: &SqlitePool, id: i64) -> Result<Vec<RubricCode>, ApiError> {
    let rows = sqlx::query("SELECT r.id, r.code, r.title, r.guidance, r.next_step, r.created_at FROM rubric_codes r JOIN loop_codes lc ON lc.rubric_id = r.id WHERE lc.loop_id = ? ORDER BY r.code")
        .bind(id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(rubric_from_row).collect())
}

async fn delete_loop(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    let key = workspace_key(&headers)?;
    let mut tx = state.pool.begin().await?;
    let result = sqlx::query("UPDATE feedback_loops SET deleted_at = datetime('now') WHERE id = ? AND workspace_key = ? AND deleted_at IS NULL")
        .bind(id)
        .bind(key)
        .execute(&mut *tx)
        .await?;
    if result.rows_affected() == 0 {
        Err(ApiError::not_found("Feedback link not found."))
    } else {
        // A deleted link must no longer hold a rubric hostage. The soft-deleted
        // loop remains for deletion bookkeeping, while its no-longer-reachable
        // assignment relationships are removed.
        sqlx::query("DELETE FROM loop_codes WHERE loop_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(StatusCode::NO_CONTENT)
    }
}

#[derive(Debug, Deserialize)]
struct ReviewInput {
    reviewed: bool,
}

async fn review_loop(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
    Json(input): Json<ReviewInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = workspace_key(&headers)?;
    let status = if input.reviewed {
        "reviewed"
    } else {
        "submitted"
    };
    let result = sqlx::query("UPDATE feedback_loops SET status = ?, reviewed_at = CASE WHEN ? = 'reviewed' THEN datetime('now') ELSE NULL END WHERE id = ? AND workspace_key = ? AND submitted_at IS NOT NULL AND deleted_at IS NULL")
        .bind(status).bind(status).bind(id).bind(key).execute(&state.pool).await?;
    if result.rows_affected() == 0 {
        return Err(ApiError::conflict(
            "Only a submitted revision can be reviewed.",
        ));
    }
    Ok(Json(serde_json::json!({ "status": status })))
}

#[derive(Debug, Serialize)]
struct StudentView {
    assignment_title: String,
    teacher_note: String,
    status: String,
    before_excerpt: Option<String>,
    after_excerpt: Option<String>,
    explanation: Option<String>,
    checklist: Vec<i64>,
    rubrics: Vec<RubricCode>,
    expires_at: String,
}

async fn student_loop(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Json<StudentView>, ApiError> {
    validate_token(&token)?;
    let row = sqlx::query("SELECT * FROM feedback_loops WHERE token = ? AND deleted_at IS NULL")
        .bind(&token)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("This revision link was not found or has been deleted.")
        })?;
    let created_at: String = row.get("created_at");
    let retention: i64 = row.get("retention_days");
    let expires_at: String = sqlx::query_scalar("SELECT datetime(?, '+' || ? || ' days')")
        .bind(&created_at)
        .bind(retention)
        .fetch_one(&state.pool)
        .await?;
    let expired: i64 = sqlx::query_scalar("SELECT datetime('now') > datetime(?)")
        .bind(&expires_at)
        .fetch_one(&state.pool)
        .await?;
    if expired == 1 {
        return Err(ApiError::gone(
            "This revision link has expired. Ask your teacher for a new link.",
        ));
    }
    let checklist_raw: String = row.get("checklist_json");
    Ok(Json(StudentView {
        assignment_title: row.get("assignment_title"),
        teacher_note: row.get("teacher_note"),
        status: row.get("status"),
        before_excerpt: row.get("before_excerpt"),
        after_excerpt: row.get("after_excerpt"),
        explanation: row.get("explanation"),
        checklist: serde_json::from_str(&checklist_raw).unwrap_or_default(),
        rubrics: rubric_rows_for_loop(&state.pool, row.get("id")).await?,
        expires_at,
    }))
}

#[derive(Debug, Deserialize)]
struct RevisionInput {
    before_excerpt: String,
    after_excerpt: String,
    explanation: String,
    checklist: Vec<i64>,
}

async fn submit_revision(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(input): Json<RevisionInput>,
) -> Result<Json<serde_json::Value>, ApiError> {
    validate_token(&token)?;
    let before = clean_required("Before excerpt", &input.before_excerpt, 1, 4000)?;
    let after = clean_required("After excerpt", &input.after_excerpt, 1, 4000)?;
    let explanation = clean_required("Revision explanation", &input.explanation, 8, 2000)?;
    let row = sqlx::query("SELECT id, status, created_at, retention_days FROM feedback_loops WHERE token = ? AND deleted_at IS NULL AND datetime(created_at, '+' || retention_days || ' days') >= datetime('now')").bind(&token).fetch_optional(&state.pool).await?.ok_or_else(|| ApiError::not_found("This revision link was not found, has expired, or was deleted."))?;
    if row.get::<String, _>("status") == "reviewed" {
        return Err(ApiError::conflict(
            "This revision has already been reviewed. Ask your teacher to reopen it.",
        ));
    }
    let id: i64 = row.get("id");
    let mut assigned_ids: Vec<i64> =
        sqlx::query_scalar("SELECT rubric_id FROM loop_codes WHERE loop_id = ?")
            .bind(id)
            .fetch_all(&state.pool)
            .await?;
    let mut checklist_ids = input.checklist.clone();
    assigned_ids.sort_unstable();
    checklist_ids.sort_unstable();
    if checklist_ids != assigned_ids {
        return Err(ApiError::validation(
            "Check each rubric step before submitting.",
        ));
    }
    let checklist = serde_json::to_string(&checklist_ids).map_err(|_| ApiError::internal())?;
    sqlx::query("UPDATE feedback_loops SET before_excerpt = ?, after_excerpt = ?, explanation = ?, checklist_json = ?, status = 'submitted', submitted_at = datetime('now'), reviewed_at = NULL WHERE id = ?")
        .bind(before).bind(after).bind(explanation).bind(checklist).bind(id).execute(&state.pool).await?;
    Ok(Json(serde_json::json!({ "status": "submitted" })))
}

#[derive(Debug, Serialize)]
struct WorkspaceExport {
    exported_at: String,
    rubrics: Vec<RubricCode>,
    feedback_loops: Vec<FeedbackLoop>,
}

async fn export_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let key = workspace_key(&headers)?;
    let rubric_rows = sqlx::query("SELECT id, code, title, guidance, next_step, created_at FROM rubric_codes WHERE workspace_key = ? ORDER BY code").bind(&key).fetch_all(&state.pool).await?;
    let loop_rows = sqlx::query("SELECT * FROM feedback_loops WHERE workspace_key = ? AND deleted_at IS NULL ORDER BY created_at DESC").bind(key).fetch_all(&state.pool).await?;
    let mut loops = Vec::new();
    for row in loop_rows {
        loops.push(loop_from_row(&state.pool, row).await?);
    }
    let exported_at: String = sqlx::query_scalar("SELECT datetime('now')")
        .fetch_one(&state.pool)
        .await?;
    let bytes = serde_json::to_vec_pretty(&WorkspaceExport {
        exported_at,
        rubrics: rubric_rows.into_iter().map(rubric_from_row).collect(),
        feedback_loops: loops,
    })
    .map_err(|_| ApiError::internal())?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=revision-loop-export.json",
            ),
        ],
        bytes,
    )
        .into_response())
}

#[derive(Debug, Deserialize)]
struct PackInput {
    rubric_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PackRubric {
    code: String,
    title: String,
    guidance: String,
    next_step: String,
}

async fn create_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<PackInput>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let key = workspace_key(&headers)?;
    require_studio(&state, &headers).await?;
    if input.rubric_ids.is_empty() || input.rubric_ids.len() > 100 {
        return Err(ApiError::validation(
            "Choose between 1 and 100 rubric codes for a team pack.",
        ));
    }
    let placeholders = std::iter::repeat_n("?", input.rubric_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!("SELECT code, title, guidance, next_step FROM rubric_codes WHERE workspace_key = ? AND id IN ({placeholders}) ORDER BY code");
    let mut query = sqlx::query(&sql).bind(&key);
    for id in &input.rubric_ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(&state.pool).await?;
    if rows.len() != input.rubric_ids.len() {
        return Err(ApiError::validation(
            "One or more rubric codes no longer exist.",
        ));
    }
    let pack: Vec<PackRubric> = rows
        .into_iter()
        .map(|row| PackRubric {
            code: row.get("code"),
            title: row.get("title"),
            guidance: row.get("guidance"),
            next_step: row.get("next_step"),
        })
        .collect();
    let token = Uuid::new_v4().simple().to_string();
    sqlx::query("INSERT INTO rubric_packs (token, workspace_key, rubric_json) VALUES (?, ?, ?)")
        .bind(&token)
        .bind(key)
        .bind(serde_json::to_string(&pack).map_err(|_| ApiError::internal())?)
        .execute(&state.pool)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "token": token })),
    ))
}

async fn import_pack(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let key = workspace_key(&headers)?;
    require_studio(&state, &headers).await?;
    validate_token(&token)?;
    let raw: String = sqlx::query_scalar("SELECT rubric_json FROM rubric_packs WHERE token = ? AND datetime(created_at, '+30 days') >= datetime('now')")
        .bind(token)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::not_found("This team pack was not found or has expired."))?;
    let pack: Vec<PackRubric> = serde_json::from_str(&raw).map_err(|_| ApiError::internal())?;
    let mut imported = 0_u64;
    let mut tx = state.pool.begin().await?;
    for rubric in pack {
        imported += sqlx::query("INSERT OR IGNORE INTO rubric_codes (workspace_key, code, title, guidance, next_step) VALUES (?, ?, ?, ?, ?)")
            .bind(&key)
            .bind(rubric.code)
            .bind(rubric.title)
            .bind(rubric.guidance)
            .bind(rubric.next_step)
            .execute(&mut *tx)
            .await?
            .rows_affected();
    }
    tx.commit().await?;
    Ok(Json(serde_json::json!({ "imported": imported })))
}

async fn delete_workspace(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let key = workspace_key(&headers)?;
    if headers
        .get("x-confirm-delete")
        .and_then(|v| v.to_str().ok())
        != Some("delete my workspace")
    {
        return Err(ApiError::validation(
            "Workspace deletion was not confirmed.",
        ));
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query("DELETE FROM loop_codes WHERE loop_id IN (SELECT id FROM feedback_loops WHERE workspace_key = ?)").bind(&key).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM feedback_loops WHERE workspace_key = ?")
        .bind(&key)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM rubric_codes WHERE workspace_key = ?")
        .bind(&key)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM rubric_packs WHERE workspace_key = ?")
        .bind(&key)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

fn workspace_key(headers: &HeaderMap) -> Result<String, ApiError> {
    let key = headers
        .get("x-workspace-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !(32..=128).contains(&key.len())
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiError::unauthorized(
            "This browser does not have a valid workspace key.",
        ));
    }
    Ok(key.to_string())
}

async fn require_studio(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let license = headers
        .get("x-studio-license")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ApiError::forbidden("A valid Studio license is required for this feature.")
        })?;
    match state.studio.verify(license).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ApiError::forbidden(
            "This Studio license is not active. Restore or purchase a license to continue.",
        )),
        Err(()) => Err(ApiError::service_unavailable(
            "Studio verification is temporarily unavailable. Try again shortly.",
        )),
    }
}

fn validate_token(token: &str) -> Result<(), ApiError> {
    if token.len() != 32 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
        Err(ApiError::not_found("This revision link is not valid."))
    } else {
        Ok(())
    }
}

fn clean_required(label: &str, value: &str, min: usize, max: usize) -> Result<String, ApiError> {
    let clean = value.trim();
    if clean.chars().count() < min {
        return Err(ApiError::validation(&format!(
            "{label} must be at least {min} characters."
        )));
    }
    if clean.chars().count() > max {
        return Err(ApiError::validation(&format!(
            "{label} must be {max} characters or fewer."
        )));
    }
    Ok(clean.to_string())
}

fn clean_optional(value: &str, max: usize) -> Result<String, ApiError> {
    let clean = value.trim();
    if clean.chars().count() > max {
        return Err(ApiError::validation(&format!(
            "This field must be {max} characters or fewer."
        )));
    }
    Ok(clean.to_string())
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn validation(message: &str) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: message.into(),
        }
    }
    fn unauthorized(message: &str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }
    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
    fn gone(message: &str) -> Self {
        Self {
            status: StatusCode::GONE,
            message: message.into(),
        }
    }
    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }
    fn forbidden(message: &str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }
    fn service_unavailable(message: &str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            message: message.into(),
        }
    }
    fn internal() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Something went wrong while saving. Try again.".into(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(?error, "database_error");
        Self::internal()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let pool = open_database("sqlite::memory:").await.unwrap();
        build_app(
            AppState {
                pool,
                studio: StudioVerifier::Static {
                    valid_license: "studio-test-license".into(),
                },
            },
            AppConfig {
                dist_dir: PathBuf::from("dist"),
            },
        )
    }

    async fn json_response(app: Router, request: Request<Body>) -> (StatusCode, serde_json::Value) {
        let response = app.oneshot(request).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    fn api(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("x-workspace-key", "teacher_workspace_key_1234567890")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn studio_api(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header("x-workspace-key", "teacher_workspace_key_1234567890")
            .header("x-studio-license", "studio-test-license")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    #[tokio::test]
    async fn complete_revision_loop() {
        let app = test_app().await;
        let (status, rubric) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"EV-1","title":"Use evidence","guidance":"Connect a quotation to the claim you make.","next_step":"Add one sentence explaining how the quotation proves the claim."}))).await;
        assert_eq!(status, StatusCode::CREATED);
        let rubric_id = rubric["id"].as_i64().unwrap();
        let (status, loop_json) = json_response(app.clone(), api("POST", "/api/loops", serde_json::json!({"assignment_title":"Argument paragraph","student_label":"Student 12","rubric_ids":[rubric_id]}))).await;
        assert_eq!(status, StatusCode::CREATED);
        let token = loop_json["token"].as_str().unwrap();
        let request = Request::builder()
            .uri(format!("/api/student/{token}"))
            .body(Body::empty())
            .unwrap();
        let (status, view) = json_response(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(view["rubrics"][0]["code"], "EV-1");
        let request = Request::builder().method("POST").uri(format!("/api/student/{token}/revision")).header("content-type", "application/json").body(Body::from(serde_json::json!({"before_excerpt":"This proves the point.","after_excerpt":"The detail shows the policy failed because costs rose.","explanation":"I explained the connection between evidence and claim.","checklist":[rubric_id]}).to_string())).unwrap();
        let (status, _) = json_response(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        let (status, queue) = json_response(
            app.clone(),
            api("GET", "/api/loops", serde_json::Value::Null),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(queue["items"][0]["status"], "submitted");
    }

    #[tokio::test]
    async fn rejects_unknown_rubric_and_bad_workspace() {
        let app = test_app().await;
        let (status, _) = json_response(
            app.clone(),
            api(
                "POST",
                "/api/loops",
                serde_json::json!({"assignment_title":"Essay","rubric_ids":[999]}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        let request = Request::builder()
            .uri("/api/rubrics")
            .body(Body::empty())
            .unwrap();
        let (status, _) = json_response(app, request).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn shares_rubric_pack_without_exposing_source_workspace() {
        let app = test_app().await;
        let (_, rubric) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"ORG-2","title":"Organize reasons","guidance":"Arrange reasons so each one builds on the last.","next_step":"Move one sentence and explain why the new order is clearer."}))).await;
        let (_, pack) = json_response(
            app.clone(),
            studio_api(
                "POST",
                "/api/packs",
                serde_json::json!({"rubric_ids":[rubric["id"]]}),
            ),
        )
        .await;
        let token = pack["token"].as_str().unwrap();
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/packs/{token}/import"))
            .header("x-workspace-key", "second_workspace_key_1234567890_ab")
            .header("x-studio-license", "studio-test-license")
            .body(Body::empty())
            .unwrap();
        let (status, imported) = json_response(app.clone(), request).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(imported["imported"], 1);
        let request = Request::builder()
            .uri("/api/rubrics")
            .header("x-workspace-key", "second_workspace_key_1234567890_ab")
            .body(Body::empty())
            .unwrap();
        let (_, library) = json_response(app, request).await;
        assert_eq!(library["items"][0]["code"], "ORG-2");
    }

    #[tokio::test]
    async fn rejects_unlicensed_studio_retention_and_team_pack_writes() {
        let app = test_app().await;
        let (_, rubric) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"EV-1","title":"Use evidence","guidance":"Connect a quotation to the claim you make.","next_step":"Add one sentence explaining how the quotation proves the claim."}))).await;
        let rubric_id = rubric["id"].as_i64().unwrap();

        let (status, body) = json_response(
            app.clone(),
            api(
                "POST",
                "/api/loops",
                serde_json::json!({"assignment_title":"Paid bypass","rubric_ids":[rubric_id],"retention_days":365}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(
            body["error"],
            "A valid Studio license is required for this feature."
        );

        let (status, body) = json_response(
            app.clone(),
            api(
                "POST",
                "/api/packs",
                serde_json::json!({"rubric_ids":[rubric_id]}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(
            body["error"],
            "A valid Studio license is required for this feature."
        );

        let (status, _) = json_response(
            app.clone(),
            studio_api(
                "POST",
                "/api/loops",
                serde_json::json!({"assignment_title":"Licensed retention","rubric_ids":[rubric_id],"retention_days":365}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        let (status, _) = json_response(
            app,
            studio_api(
                "POST",
                "/api/packs",
                serde_json::json!({"rubric_ids":[rubric_id]}),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn rejects_incomplete_or_duplicate_checklists_without_storing_a_revision() {
        let app = test_app().await;
        let (_, first) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"EV-1","title":"Use evidence","guidance":"Connect a quotation to the claim you make.","next_step":"Add one sentence explaining how the quotation proves the claim."}))).await;
        let (_, second) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"ORG-2","title":"Organize reasons","guidance":"Arrange reasons so each one builds on the last.","next_step":"Move one sentence and explain why the new order is clearer."}))).await;
        let (_, loop_json) = json_response(app.clone(), api("POST", "/api/loops", serde_json::json!({"assignment_title":"Argument paragraph","rubric_ids":[first["id"], second["id"]]}))).await;
        let token = loop_json["token"].as_str().unwrap();
        let revision = |checklist: serde_json::Value| {
            Request::builder().method("POST").uri(format!("/api/student/{token}/revision")).header("content-type", "application/json").body(Body::from(serde_json::json!({"before_excerpt":"Before.","after_excerpt":"After.","explanation":"I made the evidence and organization clearer.","checklist":checklist}).to_string())).unwrap()
        };

        let (status, body) =
            json_response(app.clone(), revision(serde_json::json!([first["id"]]))).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["error"], "Check each rubric step before submitting.");
        let (status, _) = json_response(
            app.clone(),
            revision(serde_json::json!([first["id"], first["id"]])),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        let (status, student) = json_response(
            app,
            Request::builder()
                .uri(format!("/api/student/{token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(student["status"], "awaiting");
        assert_eq!(student["checklist"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn linked_rubric_returns_recoverable_conflict_then_deletes_after_link_removal() {
        let app = test_app().await;
        let (_, rubric) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"EV-1","title":"Use evidence","guidance":"Connect a quotation to the claim you make.","next_step":"Add one sentence explaining how the quotation proves the claim."}))).await;
        let (_, loop_json) = json_response(app.clone(), api("POST", "/api/loops", serde_json::json!({"assignment_title":"Argument paragraph","rubric_ids":[rubric["id"]]}))).await;
        let rubric_id = rubric["id"].as_i64().unwrap();
        let loop_id = loop_json["id"].as_i64().unwrap();

        let (status, body) = json_response(
            app.clone(),
            api(
                "DELETE",
                &format!("/api/rubrics/{rubric_id}"),
                serde_json::Value::Null,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(
            body["error"],
            "This code is used in a feedback link. Delete that link first."
        );
        let (status, _) = json_response(
            app.clone(),
            api(
                "DELETE",
                &format!("/api/loops/{loop_id}"),
                serde_json::Value::Null,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, _) = json_response(
            app,
            api(
                "DELETE",
                &format!("/api/rubrics/{rubric_id}"),
                serde_json::Value::Null,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn deletes_a_rubric_after_cleaning_a_legacy_soft_deleted_link() {
        let pool = open_database("sqlite::memory:").await.unwrap();
        let app = build_app(
            AppState {
                pool: pool.clone(),
                studio: StudioVerifier::Static {
                    valid_license: "studio-test-license".into(),
                },
            },
            AppConfig {
                dist_dir: PathBuf::from("dist"),
            },
        );
        let (_, rubric) = json_response(app.clone(), api("POST", "/api/rubrics", serde_json::json!({"code":"EV-1","title":"Use evidence","guidance":"Connect a quotation to the claim you make.","next_step":"Add one sentence explaining how the quotation proves the claim."}))).await;
        let (_, loop_json) = json_response(app.clone(), api("POST", "/api/loops", serde_json::json!({"assignment_title":"Argument paragraph","rubric_ids":[rubric["id"]]}))).await;
        sqlx::query("UPDATE feedback_loops SET deleted_at = datetime('now') WHERE id = ?")
            .bind(loop_json["id"].as_i64().unwrap())
            .execute(&pool)
            .await
            .unwrap();

        let (status, _) = json_response(
            app,
            api(
                "DELETE",
                &format!("/api/rubrics/{}", rubric["id"].as_i64().unwrap()),
                serde_json::Value::Null,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn applies_cache_policy_for_private_api_assets_and_service_worker() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("assets")).unwrap();
        std::fs::write(
            temp.path().join("index.html"),
            "<html><body>Revision loop</body></html>",
        )
        .unwrap();
        std::fs::write(temp.path().join("assets/app-abc123.js"), "export {};").unwrap();
        std::fs::write(temp.path().join("sw.js"), "// worker").unwrap();
        let pool = open_database("sqlite::memory:").await.unwrap();
        let app = build_app(
            AppState {
                pool,
                studio: StudioVerifier::Static {
                    valid_license: "studio-test-license".into(),
                },
            },
            AppConfig {
                dist_dir: temp.path().to_path_buf(),
            },
        );

        for (uri, expected) in [
            ("/api/health", "no-store"),
            (
                "/assets/app-abc123.js",
                "public, max-age=31536000, immutable",
            ),
            ("/sw.js", "no-cache"),
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(
                response.headers().get(header::CACHE_CONTROL).unwrap(),
                expected
            );
        }
    }
}

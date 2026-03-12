use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::AppError;
use crate::models::{
    Annotation, AnnotationType, Coverage, CoverageStatus, Requirement, ScanResult, Task, TaskStatus,
};
use crate::scanner::scan_project;

#[derive(Clone)]
pub struct AppState {
    root: PathBuf,
    requirements_path: PathBuf,
    tasks_path: PathBuf,
    scan_state: Arc<RwLock<ScanState>>,
}

impl AppState {
    /// Create application state with explicit input paths.
    pub fn new(root: PathBuf, requirements_path: PathBuf, tasks_path: PathBuf) -> Self {
        Self {
            root,
            requirements_path,
            tasks_path,
            scan_state: Arc::new(RwLock::new(ScanState::new())),
        }
    }
}

#[derive(Debug, Clone)]
struct ScanState {
    status: ScanStatus,
    last_result: Option<ScanResult>,
    last_error: Option<String>,
}

impl ScanState {
    fn new() -> Self {
        Self {
            status: ScanStatus::Idle,
            last_result: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScanStatus {
    Idle,
    Scanning,
    Complete,
    Failed,
}

impl ScanStatus {
    fn as_str(self) -> &'static str {
        match self {
            ScanStatus::Idle => "idle",
            ScanStatus::Scanning => "scanning",
            ScanStatus::Complete => "complete",
            ScanStatus::Failed => "failed",
        }
    }
}

/// Build the API router with all endpoints.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/stats", get(stats))
        .route("/requirements", get(list_requirements))
        .route("/requirements/:requirement_id", get(requirement_detail))
        .route("/annotations", get(list_annotations))
        .route("/tasks", get(list_tasks))
        .route("/scan", post(start_scan).get(scan_status))
        .with_state(state)
}

#[derive(Serialize)]
struct HealthcheckResponse {
    status: &'static str,
    version: &'static str,
}

/// @req SCS-SCAN-012
/// Return service health and version.
async fn healthcheck() -> impl IntoResponse {
    Json(HealthcheckResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
struct RequirementsByStatus {
    covered: usize,
    partial: usize,
    missing: usize,
}

#[derive(Serialize)]
struct AnnotationStats {
    #[serde(rename = "impl")]
    impl_count: usize,
    #[serde(rename = "test")]
    test_count: usize,
    orphans: usize,
}

#[derive(Serialize)]
struct TaskStats {
    open: usize,
    in_progress: usize,
    done: usize,
    orphans: usize,
}

#[derive(Serialize)]
struct StatsResponse {
    coverage_percent: usize,
    requirements_by_status: RequirementsByStatus,
    requirements_by_type: std::collections::BTreeMap<String, RequirementsByStatus>,
    annotation_counts: AnnotationStats,
    task_counts: TaskStats,
}

/// @req SCS-SCAN-013
/// Return aggregated project metrics for requirements, annotations, and tasks.
async fn stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let result = ensure_scan(&state).await?;
    let stats = result.stats;

    let requirements_by_status = RequirementsByStatus {
        covered: stats.covered,
        partial: stats.partial,
        missing: stats.missing,
    };

    let requirements_by_type = stats
        .requirements_by_type
        .into_iter()
        .map(|(key, counts)| {
            (
                key,
                RequirementsByStatus {
                    covered: counts.covered,
                    partial: counts.partial,
                    missing: counts.missing,
                },
            )
        })
        .collect();

    let annotation_counts = AnnotationStats {
        impl_count: stats.annotation_counts.impl_count,
        test_count: stats.annotation_counts.test_count,
        orphans: stats.annotation_counts.orphan_count,
    };

    let task_counts = TaskStats {
        open: stats.task_counts.open,
        in_progress: stats.task_counts.in_progress,
        done: stats.task_counts.done,
        orphans: stats.task_counts.orphan_count,
    };

    Ok(Json(StatsResponse {
        coverage_percent: stats.coverage_percent,
        requirements_by_status,
        requirements_by_type,
        annotation_counts,
        task_counts,
    }))
}

#[derive(Debug, Deserialize)]
struct RequirementsQuery {
    status: Option<String>,
    #[serde(rename = "type")]
    type_filter: Option<String>,
    sort: Option<String>,
    order: Option<String>,
}

#[derive(Serialize)]
struct RequirementListItem {
    requirement: Requirement,
    coverage: Coverage,
}

/// @req SCS-SCAN-014
/// List requirements with optional filtering and sorting.
async fn list_requirements(
    State(state): State<AppState>,
    Query(query): Query<RequirementsQuery>,
) -> Result<Json<Vec<RequirementListItem>>, ApiError> {
    let result = ensure_scan(&state).await?;
    let mut items: Vec<RequirementListItem> = result
        .requirements
        .iter()
        .map(|requirement| RequirementListItem {
            requirement: requirement.clone(),
            coverage: result
                .coverage
                .get(&requirement.id)
                .cloned()
                .unwrap_or(Coverage {
                    impl_count: 0,
                    test_count: 0,
                    status: CoverageStatus::Missing,
                }),
        })
        .collect();

    if let Some(status) = query.status.as_deref().and_then(parse_coverage_status) {
        items.retain(|item| item.coverage.status == status);
    }

    if let Some(type_filter) = query.type_filter.as_deref() {
        let prefix = format!("{}-", type_filter);
        items.retain(|item| item.requirement.id.starts_with(&prefix));
    }

    match query.sort.as_deref() {
        Some("title") => items.sort_by(|a, b| a.requirement.title.cmp(&b.requirement.title)),
        Some("status") => items
            .sort_by(|a, b| status_rank(&a.coverage.status).cmp(&status_rank(&b.coverage.status))),
        _ => items.sort_by(|a, b| a.requirement.id.cmp(&b.requirement.id)),
    }

    if query.order.as_deref() == Some("desc") {
        items.reverse();
    }

    Ok(Json(items))
}

#[derive(Serialize)]
struct RequirementDetail {
    requirement: Requirement,
    coverage: Coverage,
    annotations: Vec<Annotation>,
    tasks: Vec<Task>,
}

/// @req SCS-SCAN-015
/// Fetch a single requirement with linked annotations and tasks.
async fn requirement_detail(
    State(state): State<AppState>,
    Path(requirement_id): Path<String>,
) -> Result<Json<RequirementDetail>, ApiError> {
    let result = ensure_scan(&state).await?;
    let requirement = result
        .requirements
        .iter()
        .find(|requirement| requirement.id == requirement_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("requirement not found"))?;

    let annotations = result
        .annotations
        .iter()
        .filter(|annotation| annotation.requirement_id == requirement_id)
        .cloned()
        .collect();
    let tasks = result
        .tasks
        .iter()
        .filter(|task| task.requirement_id == requirement_id)
        .cloned()
        .collect();

    let coverage = result
        .coverage
        .get(&requirement_id)
        .cloned()
        .unwrap_or(Coverage {
            impl_count: 0,
            test_count: 0,
            status: CoverageStatus::Missing,
        });

    Ok(Json(RequirementDetail {
        requirement,
        coverage,
        annotations,
        tasks,
    }))
}

#[derive(Debug, Deserialize)]
struct AnnotationQuery {
    #[serde(rename = "type")]
    annotation_type: Option<String>,
    orphans: Option<bool>,
}

/// @req SCS-SCAN-016
/// List detected annotations with optional filters.
async fn list_annotations(
    State(state): State<AppState>,
    Query(query): Query<AnnotationQuery>,
) -> Result<Json<Vec<Annotation>>, ApiError> {
    let result = ensure_scan(&state).await?;
    let mut annotations = if query.orphans == Some(true) {
        result.orphan_annotations
    } else {
        result.annotations
    };

    if let Some(annotation_type) = query
        .annotation_type
        .as_deref()
        .and_then(parse_annotation_type)
    {
        annotations.retain(|annotation| annotation.annotation_type == annotation_type);
    }

    Ok(Json(annotations))
}

#[derive(Debug, Deserialize)]
struct TasksQuery {
    status: Option<String>,
    orphans: Option<bool>,
}

/// @req SCS-SCAN-017
/// List tasks with optional filters.
async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TasksQuery>,
) -> Result<Json<Vec<Task>>, ApiError> {
    let result = ensure_scan(&state).await?;
    let mut tasks = if query.orphans == Some(true) {
        result.orphan_tasks
    } else {
        result.tasks
    };

    if let Some(status) = query.status.as_deref().and_then(parse_task_status) {
        tasks.retain(|task| task.status == status);
    }

    Ok(Json(tasks))
}

#[derive(Serialize)]
struct ScanStatusResponse {
    status: String,
    error: Option<String>,
}

/// @req SCS-SCAN-018
/// Trigger a new project scan and return immediately.
async fn start_scan(State(state): State<AppState>) -> impl IntoResponse {
    let should_start = {
        let mut guard = state.scan_state.write().await;
        if guard.status == ScanStatus::Scanning {
            false
        } else {
            guard.status = ScanStatus::Scanning;
            guard.last_error = None;
            true
        }
    };

    if should_start {
        let state_clone = state.clone();
        tokio::spawn(async move {
            let _ = run_scan(&state_clone).await;
        });
    }

    (
        StatusCode::ACCEPTED,
        Json(ScanStatusResponse {
            status: ScanStatus::Scanning.as_str().to_string(),
            error: None,
        }),
    )
}

/// @req SCS-SCAN-019
/// Return the current scan status.
async fn scan_status(State(state): State<AppState>) -> impl IntoResponse {
    let guard = state.scan_state.read().await;
    Json(ScanStatusResponse {
        status: guard.status.as_str().to_string(),
        error: guard.last_error.clone(),
    })
}

async fn ensure_scan(state: &AppState) -> Result<ScanResult, ApiError> {
    {
        let guard = state.scan_state.read().await;
        if let Some(result) = &guard.last_result {
            return Ok(result.clone());
        }
        if guard.status == ScanStatus::Scanning {
            return Err(ApiError::conflict("scan in progress"));
        }
    }

    let result = run_scan(state).await?;
    Ok(result)
}

async fn run_scan(state: &AppState) -> Result<ScanResult, ApiError> {
    {
        let mut guard = state.scan_state.write().await;
        guard.status = ScanStatus::Scanning;
        guard.last_error = None;
    }

    let root = state.root.clone();
    let requirements_path = state.requirements_path.clone();
    let tasks_path = state.tasks_path.clone();
    let scan_result =
        tokio::task::spawn_blocking(move || scan_project(&root, &requirements_path, &tasks_path))
            .await
            .map_err(|err| ApiError::from_app(AppError::internal(err.to_string())))?
            .map_err(ApiError::from_app);

    match scan_result {
        Ok(result) => {
            let mut guard = state.scan_state.write().await;
            guard.status = ScanStatus::Complete;
            guard.last_result = Some(result.clone());
            guard.last_error = None;
            Ok(result)
        }
        Err(err) => {
            let mut guard = state.scan_state.write().await;
            guard.status = ScanStatus::Failed;
            guard.last_error = Some(err.message.clone());
            Err(err)
        }
    }
}

fn parse_coverage_status(value: &str) -> Option<CoverageStatus> {
    match value {
        "covered" => Some(CoverageStatus::Covered),
        "partial" => Some(CoverageStatus::Partial),
        "missing" => Some(CoverageStatus::Missing),
        _ => None,
    }
}

fn parse_annotation_type(value: &str) -> Option<AnnotationType> {
    match value {
        "impl" => Some(AnnotationType::Impl),
        "test" => Some(AnnotationType::Test),
        _ => None,
    }
}

fn parse_task_status(value: &str) -> Option<TaskStatus> {
    match value {
        "open" => Some(TaskStatus::Open),
        "in_progress" => Some(TaskStatus::InProgress),
        "done" => Some(TaskStatus::Done),
        _ => None,
    }
}

fn status_rank(status: &CoverageStatus) -> usize {
    match status {
        CoverageStatus::Covered => 0,
        CoverageStatus::Partial => 1,
        CoverageStatus::Missing => 2,
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

/// @req SCS-SCAN-020
/// Map structured errors to HTTP responses.
#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn from_app(err: AppError) -> Self {
        let (status, message) = match err {
            AppError::MissingFile { path } => (
                StatusCode::NOT_FOUND,
                format!("missing file: {}", path.display()),
            ),
            AppError::Yaml {
                path,
                message,
                line,
            } => {
                let detail = if let Some(line) = line {
                    format!("malformed YAML at {}:{}: {}", path.display(), line, message)
                } else {
                    format!("malformed YAML at {}: {}", path.display(), message)
                };
                (StatusCode::BAD_REQUEST, detail)
            }
            AppError::Validation { path, message } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("validation error in {}: {}", path.display(), message),
            ),
            AppError::Io { path, source } => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("io error reading {}: {}", path.display(), source),
            ),
            AppError::Internal { message } => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };

        Self { status, message }
    }

    fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    fn conflict(message: &str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ErrorResponse {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

fn tasks_path_for(root: &StdPath) -> PathBuf {
    let yaml = root.join("tasks.yaml");
    if yaml.exists() {
        yaml
    } else {
        root.join("tasks.yml")
    }
}

/// Build API state from a project root, resolving input paths.
pub fn state_from_root(root: PathBuf) -> AppState {
    let requirements_path = root.join("requirements.yaml");
    let tasks_path = tasks_path_for(&root);
    AppState::new(root, requirements_path, tasks_path)
}

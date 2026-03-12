use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use hyper::body;
use serde_json::Value;
use std::fs;
use tempfile::TempDir;
use tower::ServiceExt;

use ssd_navigator::api;

mod support;
use support::write_temp_file;

fn setup_state() -> (TempDir, Router) {
    let dir = tempfile::tempdir().expect("tempdir");

    write_temp_file(
        &dir,
        "requirements.yaml",
        r#"
requirements:
  - id: SCS-SCAN-001
    title: Parse requirements.yaml
    description: "Load requirements"
  - id: FR-AUTH-001
    title: User login
    description: "Authenticate users"
"#,
    );

    write_temp_file(
        &dir,
        "tasks.yaml",
        r#"
tasks:
  - id: TASK-001
    requirementId: SCS-SCAN-001
    title: Implement requirements parser
    status: open
  - id: TASK-002
    requirementId: FR-AUTH-001
    title: Implement login
    status: done
"#,
    );

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src");
    write_temp_file(
        &dir,
        "src/lib.rs",
        r#"
// @req SCS-SCAN-001
pub fn parse_requirements() {}
"#,
    );

    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests");
    write_temp_file(
        &dir,
        "tests/parser_tests.rs",
        r#"
// @req SCS-SCAN-001
#[test]
fn test_parser() {}
"#,
    );

    let state = api::state_from_root(dir.path().to_path_buf());
    let app = api::router(state);
    (dir, app)
}

fn setup_state_with_orphans() -> (TempDir, Router) {
    let dir = tempfile::tempdir().expect("tempdir");

    write_temp_file(
        &dir,
        "requirements.yaml",
        r#"
requirements:
  - id: FR-ORPH-001
    title: Known requirement
    description: "Known"
"#,
    );

    write_temp_file(
        &dir,
        "tasks.yaml",
        r#"
tasks:
  - id: TASK-001
    requirementId: FR-ORPH-001
    title: Implement known
    status: open
  - id: TASK-999
    requirementId: FR-ORPH-999
    title: Orphan task
    status: done
"#,
    );

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src");
    write_temp_file(
        &dir,
        "src/lib.rs",
        r#"
// @req FR-ORPH-001
// @req FR-ORPH-999
pub fn parse_requirements() {}
"#,
    );

    let state = api::state_from_root(dir.path().to_path_buf());
    let app = api::router(state);
    (dir, app)
}

async fn json_response(app: &Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let body = body::to_bytes(response.into_body())
        .await
        .expect("body bytes");
    let value = serde_json::from_slice(&body).expect("json");
    (status, value)
}

#[tokio::test]
/// @req SCS-SCAN-012
/// Healthcheck returns status and version.
async fn healthcheck_ok() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/healthcheck")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
/// @req SCS-SCAN-014
/// Requirements endpoint supports filtering and sorting.
async fn requirements_filter_and_sort() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/requirements?type=FR&sort=id&order=desc")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["requirement"]["id"], "FR-AUTH-001");
}

#[tokio::test]
/// @req SCS-SCAN-014
/// Requirements endpoint filters by coverage status.
async fn requirements_filter_by_status() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/requirements?status=missing")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["requirement"]["id"], "FR-AUTH-001");
}

#[tokio::test]
/// @req SCS-SCAN-016
/// Annotations endpoint filters orphan annotations.
async fn annotations_orphans_filter() {
    let (_dir, app) = setup_state_with_orphans();
    let request = Request::builder()
        .uri("/annotations?orphans=true")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["requirementId"], "FR-ORPH-999");
}

#[tokio::test]
/// @req SCS-SCAN-017
/// Tasks endpoint filters orphan tasks.
async fn tasks_orphans_filter() {
    let (_dir, app) = setup_state_with_orphans();
    let request = Request::builder()
        .uri("/tasks?orphans=true")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], "TASK-999");
}

#[tokio::test]
/// @req SCS-SCAN-015
/// Requirement detail includes linked annotations and tasks.
async fn requirement_detail_includes_links() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/requirements/SCS-SCAN-001")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["requirement"]["id"], "SCS-SCAN-001");
    assert!(body["annotations"].as_array().unwrap().len() >= 1);
    assert_eq!(body["tasks"].as_array().unwrap().len(), 1);
}

#[tokio::test]
/// @req SCS-SCAN-016
/// Annotations endpoint filters by type.
async fn annotations_filter_by_type() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/annotations?type=test")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["type"], "test");
}

#[tokio::test]
/// @req SCS-SCAN-017
/// Tasks endpoint filters by status.
async fn tasks_filter_by_status() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/tasks?status=done")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    let list = body.as_array().expect("array response");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["status"], "done");
}

#[tokio::test]
/// @req SCS-SCAN-018
/// @req SCS-SCAN-019
/// Scan endpoint returns 202 and starts a scan.
async fn scan_triggers_and_status_updates() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .method("POST")
        .uri("/scan")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["status"], "scanning");

    let status_request = Request::builder().uri("/scan").body(Body::empty()).unwrap();
    let (status, body) = json_response(&app, status_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["status"] == "scanning" || body["status"] == "complete");
}

#[tokio::test]
/// @req SCS-SCAN-013
/// Stats endpoint returns aggregate metrics.
async fn stats_endpoint_returns_metrics() {
    let (_dir, app) = setup_state();
    let request = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .unwrap();
    let (status, body): (StatusCode, Value) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["coverage_percent"], 50);
    assert_eq!(body["requirements_by_status"]["covered"], 1);
    assert_eq!(body["requirements_by_status"]["missing"], 1);
    assert_eq!(body["annotation_counts"]["impl"], 1);
    assert_eq!(body["annotation_counts"]["test"], 1);
    assert_eq!(body["annotation_counts"]["orphans"], 0);
    assert_eq!(body["task_counts"]["open"], 1);
    assert_eq!(body["task_counts"]["done"], 1);
    assert_eq!(body["task_counts"]["orphans"], 0);
    assert_eq!(body["requirements_by_type"]["SCS"]["covered"], 1);
    assert_eq!(body["requirements_by_type"]["FR"]["missing"], 1);
}

#[tokio::test]
/// @req SCS-SCAN-020
/// Missing input files return a structured error.
async fn missing_file_returns_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).expect("create src");
    write_temp_file(
        &dir,
        "src/lib.rs",
        r#"
// @req SCS-SCAN-001
pub fn parse_requirements() {}
"#,
    );

    let state = api::state_from_root(dir.path().to_path_buf());
    let app = api::router(state);
    let request = Request::builder()
        .uri("/stats")
        .body(Body::empty())
        .unwrap();
    let (status, body) = json_response(&app, request).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("missing file"));
}

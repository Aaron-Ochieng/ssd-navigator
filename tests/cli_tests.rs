use ssd_navigator::cli::parse_args;
use ssd_navigator::cli::{ScanArgs, run_scan};
use ssd_navigator::errors::AppError;
use std::fs;

mod support;
use support::write_temp_file;

fn base_tasks_yaml(requirement_id: &str) -> String {
    format!(
        r#"
tasks:
  - id: TASK-001
    requirementId: {requirement_id}
    title: Implement requirement
    status: open
"#
    )
}

fn base_requirements_yaml(requirement_id: &str) -> String {
    format!(
        r#"
requirements:
  - id: {requirement_id}
    title: CLI requirement
    description: "Must be covered"
"#
    )
}

#[test]
/// @req SCS-SCAN-008
/// @req SCS-SCAN-022
fn strict_scan_fails_when_partial() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requirement_id = "FR-CLI-001";

    write_temp_file(
        &dir,
        "requirements.yaml",
        &base_requirements_yaml(requirement_id),
    );
    write_temp_file(&dir, "tasks.yaml", &base_tasks_yaml(requirement_id));

    write_temp_file(
        &dir,
        "src/lib.rs",
        r#"
// @req FR-CLI-001
pub fn feature() {}
"#,
    );

    let args = ScanArgs {
        root: dir.path().to_path_buf(),
        requirements_path: dir.path().join("requirements.yaml"),
        tasks_path: dir.path().join("tasks.yaml"),
        strict: true,
        json: true,
    };

    let err = run_scan(args).expect_err("strict scan should fail");
    assert!(matches!(err, AppError::Internal { .. }));
}

#[test]
/// @req SCS-SCAN-007
/// @req SCS-SCAN-022
fn strict_scan_succeeds_when_fully_covered() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requirement_id = "FR-CLI-002";

    write_temp_file(
        &dir,
        "requirements.yaml",
        &base_requirements_yaml(requirement_id),
    );
    write_temp_file(&dir, "tasks.yaml", &base_tasks_yaml(requirement_id));

    write_temp_file(
        &dir,
        "src/lib.rs",
        r#"
// @req FR-CLI-002
pub fn feature() {}
"#,
    );
    write_temp_file(
        &dir,
        "tests/feature_test.rs",
        r#"
// @req FR-CLI-002
#[test]
fn feature_test() {}
"#,
    );

    let args = ScanArgs {
        root: dir.path().to_path_buf(),
        requirements_path: dir.path().join("requirements.yaml"),
        tasks_path: dir.path().join("tasks.yaml"),
        strict: true,
        json: true,
    };

    run_scan(args).expect("strict scan should succeed");
}

#[test]
/// CLI rejects flags missing required values.
fn scan_args_missing_value_is_error() {
    let args = vec![
        "ssd-navigator".to_string(),
        "scan".to_string(),
        "--tests".to_string(),
        "--strict".to_string(),
    ];

    assert!(parse_args(&args).is_err());
}

#[test]
/// CLI rejects non-existent tests paths.
fn scan_args_missing_tests_path_is_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing-tests");
    let args = vec![
        "ssd-navigator".to_string(),
        "scan".to_string(),
        "--tests".to_string(),
        missing.to_string_lossy().to_string(),
    ];

    assert!(parse_args(&args).is_err());
}

#[test]
/// CLI rejects tests directories without supported files.
fn scan_args_empty_tests_dir_is_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    let args = vec![
        "ssd-navigator".to_string(),
        "scan".to_string(),
        "--tests".to_string(),
        tests_dir.to_string_lossy().to_string(),
    ];

    assert!(parse_args(&args).is_err());
}

use ssd_navigator::errors::AppError;
use ssd_navigator::models::TaskStatus;
use ssd_navigator::parser::{load_requirements, load_tasks};

mod support;
use support::write_temp_file;

#[test]
/// @req SCS-SCAN-001
/// Loads a valid requirements YAML file.
fn valid_requirements_yaml_loads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_temp_file(
        &dir,
        "requirements.yaml",
        r#"
requirements:
  - id: FR-AUTH-001
    title: User login
    description: "System MUST authenticate users."
"#,
    );

    let requirements = load_requirements(&path).expect("requirements load");
    assert_eq!(requirements.len(), 1);
    assert_eq!(requirements[0].id, "FR-AUTH-001");
}

#[test]
/// @req SCS-SCAN-002
/// Loads a valid tasks YAML file.
fn valid_tasks_yaml_loads() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_temp_file(
        &dir,
        "tasks.yaml",
        r#"
tasks:
  - id: TASK-001
    requirementId: FR-AUTH-001
    title: Implement login
    status: in_progress
"#,
    );

    let tasks = load_tasks(&path).expect("tasks load");
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].status, TaskStatus::InProgress);
}

#[test]
/// @req SCS-SCAN-002
/// Rejects tasks missing required fields.
fn missing_task_fields_cause_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_temp_file(
        &dir,
        "tasks.yaml",
        r#"
tasks:
  - id: TASK-001
    requirementId: FR-AUTH-001
    status: open
"#,
    );

    let err = load_tasks(&path).expect_err("missing fields should error");
    match err {
        AppError::Validation { message, .. } => {
            assert!(message.contains("title"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
/// @req SCS-SCAN-001
/// Rejects requirements missing required fields.
fn missing_requirement_fields_cause_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_temp_file(
        &dir,
        "requirements.yaml",
        r#"
requirements:
  - id: FR-AUTH-001
    description: "System MUST authenticate users."
"#,
    );

    let err = load_requirements(&path).expect_err("missing fields should error");
    match err {
        AppError::Validation { message, .. } => {
            assert!(message.contains("title"));
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
/// @req SCS-SCAN-020
/// Reports malformed YAML with line information.
fn malformed_yaml_returns_error_with_line() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_temp_file(
        &dir,
        "requirements.yaml",
        "requirements:\n  - id: FR-AUTH-001\n    title: \"User login\"\n    description: \"oops\"\n    - invalid\n",
    );

    let err = load_requirements(&path).expect_err("malformed yaml should error");
    match err {
        AppError::Yaml {
            line: Some(line), ..
        } => {
            assert!(line > 0);
        }
        other => panic!("expected yaml error with line, got {other:?}"),
    }
}

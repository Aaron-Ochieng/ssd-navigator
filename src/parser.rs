use crate::errors::AppError;
use crate::models::{Requirement, Task, TaskStatus};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RequirementsFile {
    requirements: Option<Vec<RequirementSeed>>,
}

#[derive(Debug, Deserialize)]
struct RequirementSeed {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TasksFile {
    tasks: Option<Vec<TaskSeed>>,
}

#[derive(Debug, Deserialize)]
struct TaskSeed {
    id: Option<String>,
    #[serde(rename = "requirementId")]
    requirement_id: Option<String>,
    title: Option<String>,
    status: Option<String>,
}

/// @req SCS-SCAN-001
pub fn load_requirements(path: &Path) -> Result<Vec<Requirement>, AppError> {
    let contents = read_to_string(path)?;
    let parsed: RequirementsFile =
        serde_yaml::from_str(&contents).map_err(|err| map_yaml_error(path, err))?;
    let seeds = parsed.requirements.ok_or_else(|| {
        AppError::validation(path, "missing top-level `requirements` list".to_string())
    })?;

    let mut requirements = Vec::with_capacity(seeds.len());
    for (index, seed) in seeds.into_iter().enumerate() {
        requirements.push(seed.into_requirement(path, index)?);
    }

    Ok(requirements)
}

/// @req SCS-SCAN-002
pub fn load_tasks(path: &Path) -> Result<Vec<Task>, AppError> {
    let contents = read_to_string(path)?;
    let parsed: TasksFile =
        serde_yaml::from_str(&contents).map_err(|err| map_yaml_error(path, err))?;
    let seeds = parsed
        .tasks
        .ok_or_else(|| AppError::validation(path, "missing top-level `tasks` list".to_string()))?;

    let mut tasks = Vec::with_capacity(seeds.len());
    for (index, seed) in seeds.into_iter().enumerate() {
        tasks.push(seed.into_task(path, index)?);
    }

    Ok(tasks)
}

fn read_to_string(path: &Path) -> Result<String, AppError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(AppError::missing_file(path)),
        Err(err) => Err(AppError::io(path, err)),
    }
}

/// @req SCS-SCAN-020
fn map_yaml_error(path: &Path, err: serde_yaml::Error) -> AppError {
    let line = err.location().map(|location| location.line() + 1);
    AppError::yaml(path, err.to_string(), line)
}

fn required_string(
    value: Option<String>,
    field: &str,
    path: &Path,
    item: &str,
    index: usize,
) -> Result<String, AppError> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(AppError::validation(
            path,
            format!(
                "{} at index {} missing required field `{}`",
                item,
                index + 1,
                field
            ),
        )),
    }
}

impl RequirementSeed {
    fn into_requirement(self, path: &Path, index: usize) -> Result<Requirement, AppError> {
        Ok(Requirement {
            id: required_string(self.id, "id", path, "requirement", index)?,
            title: required_string(self.title, "title", path, "requirement", index)?,
            description: required_string(
                self.description,
                "description",
                path,
                "requirement",
                index,
            )?,
        })
    }
}

impl TaskSeed {
    fn into_task(self, path: &Path, index: usize) -> Result<Task, AppError> {
        let status_raw = required_string(self.status, "status", path, "task", index)?;
        let status = parse_task_status(&status_raw, path, index)?;

        Ok(Task {
            id: required_string(self.id, "id", path, "task", index)?,
            requirement_id: required_string(
                self.requirement_id,
                "requirementId",
                path,
                "task",
                index,
            )?,
            title: required_string(self.title, "title", path, "task", index)?,
            status,
        })
    }
}

fn parse_task_status(value: &str, path: &Path, index: usize) -> Result<TaskStatus, AppError> {
    match value {
        "open" => Ok(TaskStatus::Open),
        "in_progress" => Ok(TaskStatus::InProgress),
        "done" => Ok(TaskStatus::Done),
        _ => Err(AppError::validation(
            path,
            format!(
                "task at index {} has invalid status `{}`; expected open, in_progress, or done",
                index + 1,
                value
            ),
        )),
    }
}

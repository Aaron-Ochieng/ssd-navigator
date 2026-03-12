use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A single requirement defined in requirements.yaml.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    pub id: String,
    pub title: String,
    pub description: String,
}

/// A task mapped to a requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "requirementId")]
    pub requirement_id: String,
    pub title: String,
    pub status: TaskStatus,
}

/// Task lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
}

/// A detected @req annotation in source code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    #[serde(rename = "requirementId")]
    pub requirement_id: String,
    pub file: String,
    pub line: usize,
    #[serde(rename = "type")]
    pub annotation_type: AnnotationType,
}

/// Annotation classification based on file type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationType {
    Impl,
    Test,
}

/// Coverage counts and status for a requirement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub impl_count: usize,
    pub test_count: usize,
    pub status: CoverageStatus,
}

/// Coverage status derived from implementation and test counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Covered,
    Partial,
    Missing,
}

/// Aggregate annotation counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationCounts {
    #[serde(rename = "impl")]
    pub impl_count: usize,
    #[serde(rename = "test")]
    pub test_count: usize,
    #[serde(rename = "orphans")]
    pub orphan_count: usize,
    pub total: usize,
}

/// Aggregate task counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskCounts {
    pub open: usize,
    pub in_progress: usize,
    pub done: usize,
    #[serde(rename = "orphans")]
    pub orphan_count: usize,
    pub total: usize,
}

/// Coverage counts by status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementStatusCounts {
    pub total: usize,
    pub covered: usize,
    pub partial: usize,
    pub missing: usize,
}

/// Project-level summary metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_requirements: usize,
    pub covered: usize,
    pub partial: usize,
    pub missing: usize,
    pub coverage_percent: usize,
    pub requirements_by_type: BTreeMap<String, RequirementStatusCounts>,
    pub annotation_counts: AnnotationCounts,
    pub task_counts: TaskCounts,
    pub orphan_annotations: usize,
    pub orphan_tasks: usize,
}

/// A non-fatal scan warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanWarning {
    pub path: String,
    pub message: String,
}

/// Full scan output including derived metrics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScanResult {
    pub requirements: Vec<Requirement>,
    pub tasks: Vec<Task>,
    pub annotations: Vec<Annotation>,
    pub coverage: BTreeMap<String, Coverage>,
    pub orphan_annotations: Vec<Annotation>,
    pub orphan_tasks: Vec<Task>,
    pub stats: ProjectStats,
    pub warnings: Vec<ScanWarning>,
}

use crate::coverage::compute_coverage;
use crate::errors::AppError;
use crate::models::{
    Annotation, AnnotationCounts, AnnotationType, ProjectStats, Requirement,
    RequirementStatusCounts, ScanResult, ScanWarning, Task, TaskCounts, TaskStatus,
};
use crate::parser::{load_requirements, load_tasks};
use regex::Regex;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// @req SCS-SCAN-003
/// @req SCS-SCAN-021
/// Scan a repository for @req annotations and coverage.
pub fn scan_project(
    root: &Path,
    requirements_path: &Path,
    tasks_path: &Path,
) -> Result<ScanResult, AppError> {
    let requirements = load_requirements(requirements_path)?;
    let tasks = load_tasks(tasks_path)?;
    let mut warnings = Vec::new();
    let annotations = scan_annotations(root, &mut warnings)?;

    let requirement_ids: HashSet<String> = requirements.iter().map(|req| req.id.clone()).collect();
    let orphan_annotations = find_orphan_annotations(&annotations, &requirement_ids);
    let orphan_tasks = find_orphan_tasks(&tasks, &requirement_ids);

    let coverage = compute_coverage(&requirements, &annotations);
    let stats = compute_stats(
        &requirements,
        &tasks,
        &annotations,
        &coverage,
        orphan_annotations.len(),
        orphan_tasks.len(),
    );

    Ok(ScanResult {
        requirements,
        tasks,
        annotations,
        coverage,
        orphan_annotations,
        orphan_tasks,
        stats,
        warnings,
    })
}

/// @req SCS-SCAN-003
fn scan_annotations(
    root: &Path,
    warnings: &mut Vec<ScanWarning>,
) -> Result<Vec<Annotation>, AppError> {
    if !root.exists() {
        return Err(AppError::missing_file(root));
    }

    let regex = Regex::new(r"@req\s+([A-Za-z0-9_-]+)").expect("static regex should compile");
    let mut annotations = Vec::new();
    let mut found_supported = false;

    for entry in WalkDir::new(root).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                push_walkdir_warning(err, warnings);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let comment_prefix = match comment_prefix_for(path) {
            Some(prefix) => prefix,
            None => continue,
        };
        found_supported = true;

        let annotation_type = if is_test_file(path, root) {
            AnnotationType::Test
        } else {
            AnnotationType::Impl
        };

        annotations.extend(scan_file(
            path,
            root,
            comment_prefix,
            annotation_type,
            &regex,
            warnings,
        ));
    }

    if !found_supported {
        warnings.push(ScanWarning {
            path: display_path(root, root),
            message: "empty source directory".to_string(),
        });
    }

    Ok(annotations)
}

/// @req SCS-SCAN-004
/// Return true when a file extension is supported by the scanner.
pub(crate) fn is_supported_file(path: &Path) -> bool {
    comment_prefix_for(path).is_some()
}

fn comment_prefix_for(path: &Path) -> Option<&'static str> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs" | "ts" | "js" | "go" | "dart") => Some("//"),
        Some("py") => Some("#"),
        _ => None,
    }
}

/// @req SCS-SCAN-006
pub(crate) fn is_test_file(path: &Path, root: &Path) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    if relative
        .components()
        .any(|component| component.as_os_str() == "tests")
    {
        return true;
    }

    let filename = match relative.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return false,
    };

    filename.starts_with("test_") || filename.contains("_test.") || filename.contains(".test.")
}

/// @req SCS-SCAN-005
fn scan_file(
    path: &Path,
    root: &Path,
    comment_prefix: &str,
    annotation_type: AnnotationType,
    regex: &Regex,
    warnings: &mut Vec<ScanWarning>,
) -> Vec<Annotation> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(err) => {
            warnings.push(ScanWarning {
                path: display_path(path, root),
                message: err.to_string(),
            });
            return Vec::new();
        }
    };

    let mut annotations = Vec::new();
    let file = display_path(path, root);

    for (line_index, line) in contents.lines().enumerate() {
        let Some(comment_index) = line.find(comment_prefix) else {
            continue;
        };
        let comment = &line[comment_index + comment_prefix.len()..];
        for capture in regex.captures_iter(comment) {
            if let Some(id) = capture.get(1) {
                annotations.push(Annotation {
                    requirement_id: id.as_str().to_string(),
                    file: file.clone(),
                    line: line_index + 1,
                    annotation_type: annotation_type.clone(),
                });
            }
        }
    }

    annotations
}

/// @req SCS-SCAN-009
fn find_orphan_annotations(
    annotations: &[Annotation],
    requirement_ids: &HashSet<String>,
) -> Vec<Annotation> {
    annotations
        .iter()
        .filter(|annotation| !requirement_ids.contains(&annotation.requirement_id))
        .cloned()
        .collect()
}

/// @req SCS-SCAN-010
fn find_orphan_tasks(tasks: &[Task], requirement_ids: &HashSet<String>) -> Vec<Task> {
    tasks
        .iter()
        .filter(|task| !requirement_ids.contains(&task.requirement_id))
        .cloned()
        .collect()
}

/// @req SCS-SCAN-011
fn compute_stats(
    requirements: &[Requirement],
    tasks: &[Task],
    annotations: &[Annotation],
    coverage: &std::collections::BTreeMap<String, crate::models::Coverage>,
    orphan_annotations: usize,
    orphan_tasks: usize,
) -> ProjectStats {
    let total_requirements = requirements.len();
    let mut covered = 0;
    let mut partial = 0;
    let mut missing = 0;

    for entry in coverage.values() {
        match entry.status {
            crate::models::CoverageStatus::Covered => covered += 1,
            crate::models::CoverageStatus::Partial => partial += 1,
            crate::models::CoverageStatus::Missing => missing += 1,
        }
    }

    let coverage_percent = if total_requirements == 0 {
        0
    } else {
        covered * 100 / total_requirements
    };

    let mut requirements_by_type: BTreeMap<String, RequirementStatusCounts> = BTreeMap::new();
    for requirement in requirements {
        let type_key = requirement_type(&requirement.id);
        let entry = requirements_by_type
            .entry(type_key)
            .or_insert(RequirementStatusCounts {
                total: 0,
                covered: 0,
                partial: 0,
                missing: 0,
            });
        entry.total += 1;
        match coverage
            .get(&requirement.id)
            .map(|item| &item.status)
            .unwrap_or(&crate::models::CoverageStatus::Missing)
        {
            crate::models::CoverageStatus::Covered => entry.covered += 1,
            crate::models::CoverageStatus::Partial => entry.partial += 1,
            crate::models::CoverageStatus::Missing => entry.missing += 1,
        }
    }

    let mut impl_count = 0;
    let mut test_count = 0;
    for annotation in annotations {
        match annotation.annotation_type {
            AnnotationType::Impl => impl_count += 1,
            AnnotationType::Test => test_count += 1,
        }
    }

    let annotation_counts = AnnotationCounts {
        impl_count,
        test_count,
        orphan_count: orphan_annotations,
        total: impl_count + test_count,
    };

    let mut open = 0;
    let mut in_progress = 0;
    let mut done = 0;
    for task in tasks {
        match task.status {
            TaskStatus::Open => open += 1,
            TaskStatus::InProgress => in_progress += 1,
            TaskStatus::Done => done += 1,
        }
    }

    let task_counts = TaskCounts {
        open,
        in_progress,
        done,
        orphan_count: orphan_tasks,
        total: open + in_progress + done,
    };

    ProjectStats {
        total_requirements,
        covered,
        partial,
        missing,
        coverage_percent,
        requirements_by_type,
        annotation_counts,
        task_counts,
        orphan_annotations,
        orphan_tasks,
    }
}

fn push_walkdir_warning(err: walkdir::Error, warnings: &mut Vec<ScanWarning>) {
    let path = err
        .path()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("<unknown>"));
    warnings.push(ScanWarning {
        path: path.display().to_string(),
        message: err.to_string(),
    });
}

fn display_path(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn requirement_type(id: &str) -> String {
    id.split('-').next().unwrap_or("UNKNOWN").to_string()
}

use crate::scanner::{is_supported_file, scan_project};
use crate::{errors::AppError, models::CoverageStatus};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Parsed CLI command selection.
pub enum Command {
    Serve,
    Scan(ScanArgs),
    Stats(StatsArgs),
}

/// Arguments for the scan command.
pub struct ScanArgs {
    pub root: PathBuf,
    pub requirements_path: PathBuf,
    pub tasks_path: PathBuf,
    pub strict: bool,
    pub json: bool,
}

/// Arguments for the stats command.
pub struct StatsArgs {
    pub root: PathBuf,
    pub requirements_path: PathBuf,
    pub tasks_path: PathBuf,
    pub json: bool,
}

/// Parse CLI arguments into a command.
pub fn parse_args(args: &[String]) -> Result<Command, String> {
    if args.len() <= 1 {
        return Ok(Command::Serve);
    }

    match args[1].as_str() {
        "serve" => Ok(Command::Serve),
        "scan" => parse_scan_args(args),
        "stats" => parse_stats_args(args),
        _ => Err(usage()),
    }
}

/// @req SCS-SCAN-022
/// Execute a scan command and emit results to stdout/stderr.
pub fn run_scan(args: ScanArgs) -> Result<(), AppError> {
    let result = scan_project(&args.root, &args.requirements_path, &args.tasks_path)?;
    if args.strict {
        let mut missing = Vec::new();
        for (id, coverage) in &result.coverage {
            if coverage.status != CoverageStatus::Covered {
                missing.push(id.clone());
            }
        }
        if !missing.is_empty() {
            missing.sort();
            if args.json {
                let payload = json!({
                    "status": "failed",
                    "missing_requirements": missing,
                });
                println!("{}", payload);
            } else {
                eprintln!(
                    "strict scan failed: {} requirement(s) are not fully covered",
                    missing.len()
                );
                for id in missing {
                    eprintln!("- {}", id);
                }
            }
            return Err(AppError::internal("strict coverage failure".to_string()));
        }
    }

    if args.json {
        let payload = json!({
            "status": "ok",
            "stats": result.stats,
            "warnings": result.warnings,
        });
        println!("{}", payload);
    } else {
        print_stats(&result.stats);
        if !result.warnings.is_empty() {
            eprintln!("warnings: {}", result.warnings.len());
        }
    }

    Ok(())
}

/// Execute a stats command and emit results to stdout.
pub fn run_stats(args: StatsArgs) -> Result<(), AppError> {
    let result = scan_project(&args.root, &args.requirements_path, &args.tasks_path)?;
    if args.json {
        println!("{}", serde_json::to_string(&result.stats).unwrap());
    } else {
        print_stats(&result.stats);
    }
    Ok(())
}

fn parse_scan_args(args: &[String]) -> Result<Command, String> {
    let mut root = current_root();
    let mut root_override = false;
    let mut requirements_path = None;
    let mut tasks_path = None;
    let mut source_path = None;
    let mut tests_path = None;
    let mut strict = false;
    let mut json = false;
    let mut idx = 2;

    while idx < args.len() {
        match args[idx].as_str() {
            "--strict" => {
                strict = true;
                idx += 1;
            }
            "--json" => {
                json = true;
                idx += 1;
            }
            "--root" => {
                let value = value_after(args, &mut idx, "--root")?;
                root = PathBuf::from(value);
                root_override = true;
            }
            "--requirements" => {
                let value = value_after(args, &mut idx, "--requirements")?;
                requirements_path = Some(PathBuf::from(value));
            }
            "--tasks" => {
                let value = value_after(args, &mut idx, "--tasks")?;
                tasks_path = Some(PathBuf::from(value));
            }
            "--source" => {
                let value = value_after(args, &mut idx, "--source")?;
                source_path = Some(PathBuf::from(value));
            }
            "--tests" => {
                let value = value_after(args, &mut idx, "--tests")?;
                tests_path = Some(PathBuf::from(value));
            }
            _ => return Err(usage()),
        }
    }

    let root = resolve_root(root, root_override, source_path, tests_path)?;
    let requirements_path = requirements_path.unwrap_or_else(|| root.join("requirements.yaml"));
    let tasks_path = tasks_path.unwrap_or_else(|| default_tasks_path(&root));

    Ok(Command::Scan(ScanArgs {
        root,
        requirements_path,
        tasks_path,
        strict,
        json,
    }))
}

fn parse_stats_args(args: &[String]) -> Result<Command, String> {
    let mut root = current_root();
    let mut root_override = false;
    let mut requirements_path = None;
    let mut tasks_path = None;
    let mut source_path = None;
    let mut tests_path = None;
    let mut json = false;
    let mut idx = 2;

    while idx < args.len() {
        match args[idx].as_str() {
            "--json" => {
                json = true;
                idx += 1;
            }
            "--root" => {
                let value = value_after(args, &mut idx, "--root")?;
                root = PathBuf::from(value);
                root_override = true;
            }
            "--requirements" => {
                let value = value_after(args, &mut idx, "--requirements")?;
                requirements_path = Some(PathBuf::from(value));
            }
            "--tasks" => {
                let value = value_after(args, &mut idx, "--tasks")?;
                tasks_path = Some(PathBuf::from(value));
            }
            "--source" => {
                let value = value_after(args, &mut idx, "--source")?;
                source_path = Some(PathBuf::from(value));
            }
            "--tests" => {
                let value = value_after(args, &mut idx, "--tests")?;
                tests_path = Some(PathBuf::from(value));
            }
            _ => return Err(usage()),
        }
    }

    let root = resolve_root(root, root_override, source_path, tests_path)?;
    let requirements_path = requirements_path.unwrap_or_else(|| root.join("requirements.yaml"));
    let tasks_path = tasks_path.unwrap_or_else(|| default_tasks_path(&root));

    Ok(Command::Stats(StatsArgs {
        root,
        requirements_path,
        tasks_path,
        json,
    }))
}

fn value_after<'a>(args: &'a [String], idx: &mut usize, flag: &str) -> Result<&'a str, String> {
    let value = args
        .get(*idx + 1)
        .ok_or_else(|| format!("missing value for {}.\n{}", flag, usage()))?;
    if value.starts_with('-') {
        return Err(format!("missing value for {}.\n{}", flag, usage()));
    }
    *idx += 2;
    Ok(value.as_str())
}

/// Resolve the scan root using --root overrides or --source/--tests hints.
fn resolve_root(
    root: PathBuf,
    root_override: bool,
    source_path: Option<PathBuf>,
    tests_path: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let cwd = current_root();
    let mut resolved_dirs = Vec::new();
    let mut hint_paths = Vec::new();
    let mut tests_hint = None;
    if let Some(path) = source_path {
        let (absolute, is_dir) = validate_hint_path(&cwd, path, "--source")?;
        hint_paths.push(absolute.clone());
        resolved_dirs.push(hint_dir(&absolute, is_dir));
    }
    if let Some(path) = tests_path {
        let (absolute, is_dir) = validate_hint_path(&cwd, path, "--tests")?;
        hint_paths.push(absolute.clone());
        resolved_dirs.push(hint_dir(&absolute, is_dir));
        tests_hint = Some((absolute, is_dir));
    }

    let root = if !resolved_dirs.is_empty() && root_override {
        let root_abs = resolve_path(&cwd, root.clone());
        for path in hint_paths {
            if path.strip_prefix(&root_abs).is_err() {
                return Err(format!(
                    "path {} must be within root {}",
                    path.display(),
                    root_abs.display()
                ));
            }
        }
        root
    } else if resolved_dirs.is_empty() {
        root
    } else {
        common_ancestor(&resolved_dirs)
            .ok_or_else(|| "no common root for --source/--tests".to_string())?
    };

    if let Some((tests_path, is_dir)) = tests_hint {
        ensure_tests_present(&tests_path, is_dir)?;
    }

    Ok(root)
}

/// Resolve a path against a base directory when it is not absolute.
fn resolve_path(base: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

fn validate_hint_path(base: &Path, path: PathBuf, flag: &str) -> Result<(PathBuf, bool), String> {
    let absolute = resolve_path(base, path);
    let metadata = fs::metadata(&absolute).map_err(|_| {
        format!(
            "path for {} does not exist: {}.\n{}",
            flag,
            absolute.display(),
            usage()
        )
    })?;
    Ok((absolute, metadata.is_dir()))
}

fn hint_dir(path: &Path, is_dir: bool) -> PathBuf {
    if is_dir {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    }
}

fn ensure_tests_present(tests_path: &Path, is_dir: bool) -> Result<(), String> {
    if is_dir {
        let mut found = false;
        for entry in WalkDir::new(tests_path).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() && is_supported_file(entry.path()) {
                found = true;
                break;
            }
        }
        if !found {
            return Err(format!(
                "no supported test files found under {}.\n{}",
                tests_path.display(),
                usage()
            ));
        }
    } else if !is_supported_file(tests_path) {
        return Err(format!(
            "tests path {} is not a supported source file.\n{}",
            tests_path.display(),
            usage()
        ));
    }
    Ok(())
}

/// Compute the common ancestor directory for a non-empty path set.
fn common_ancestor(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut iter = paths.iter();
    let first = iter.next()?.components().collect::<Vec<_>>();
    let mut common = first;

    for path in iter {
        let components = path.components().collect::<Vec<_>>();
        let mut idx = 0;
        while idx < common.len() && idx < components.len() && common[idx] == components[idx] {
            idx += 1;
        }
        common.truncate(idx);
        if common.is_empty() {
            return None;
        }
    }

    let mut root = PathBuf::new();
    for component in common {
        root.push(component.as_os_str());
    }
    Some(root)
}

fn current_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn default_tasks_path(root: &Path) -> PathBuf {
    let yaml = root.join("tasks.yaml");
    if yaml.exists() {
        yaml
    } else {
        root.join("tasks.yml")
    }
}

fn print_stats(stats: &crate::models::ProjectStats) {
    println!("total_requirements: {}", stats.total_requirements);
    println!("covered: {}", stats.covered);
    println!("partial: {}", stats.partial);
    println!("missing: {}", stats.missing);
    println!("coverage_percent: {}", stats.coverage_percent);
    println!("annotations_total: {}", stats.annotation_counts.total);
    println!("tasks_total: {}", stats.task_counts.total);
    println!("orphan_annotations: {}", stats.orphan_annotations);
    println!("orphan_tasks: {}", stats.orphan_tasks);
    for (req_type, counts) in &stats.requirements_by_type {
        println!(
            "requirements_by_type.{}: total={} covered={} partial={} missing={}",
            req_type, counts.total, counts.covered, counts.partial, counts.missing
        );
    }
}

fn usage() -> String {
    [
        "usage:",
        "  sdd-coverage|ssd-navigator serve",
        "  sdd-coverage|ssd-navigator scan [--strict] [--json] [--root PATH] [--requirements PATH] [--tasks PATH] [--source PATH] [--tests PATH]",
        "  sdd-coverage|ssd-navigator stats [--json] [--root PATH] [--requirements PATH] [--tasks PATH] [--source PATH] [--tests PATH]",
    ]
    .join("\n")
}

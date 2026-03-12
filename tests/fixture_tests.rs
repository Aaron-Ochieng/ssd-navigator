use std::path::PathBuf;

use ssd_navigator::scanner::scan_project;

#[test]
/// @req SCS-SCAN-011
/// @req SCS-SCAN-021
fn fixture_project_scan_summary() {
    let root = PathBuf::from("tests/fixtures/sample_project");
    let result = scan_project(
        &root,
        &root.join("requirements.yaml"),
        &root.join("tasks.yaml"),
    )
    .expect("scan fixture project");

    assert_eq!(result.stats.total_requirements, 2);
    assert_eq!(result.stats.covered, 1);
    assert_eq!(result.stats.missing, 1);
    assert_eq!(result.stats.coverage_percent, 50);
    assert_eq!(result.stats.requirements_by_type["FR"].total, 2);
}

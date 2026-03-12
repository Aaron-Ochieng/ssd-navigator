use ssd_navigator::coverage::{compute_coverage, coverage_status};
use ssd_navigator::models::{Annotation, AnnotationType, CoverageStatus, Requirement};

fn requirement(id: &str) -> Requirement {
    Requirement {
        id: id.to_string(),
        title: format!("{id} title"),
        description: "desc".to_string(),
    }
}

fn annotation(requirement_id: &str, annotation_type: AnnotationType) -> Annotation {
    Annotation {
        requirement_id: requirement_id.to_string(),
        file: "src/lib.rs".to_string(),
        line: 1,
        annotation_type,
    }
}

#[test]
/// @req SCS-SCAN-008
fn coverage_status_rules() {
    assert_eq!(coverage_status(1, 1), CoverageStatus::Covered);
    assert_eq!(coverage_status(1, 0), CoverageStatus::Partial);
    assert_eq!(coverage_status(0, 1), CoverageStatus::Missing);
    assert_eq!(coverage_status(0, 0), CoverageStatus::Missing);
}

#[test]
/// @req SCS-SCAN-007
fn compute_coverage_counts() {
    let requirements = vec![
        requirement("FR-COV-001"),
        requirement("FR-COV-002"),
        requirement("FR-COV-003"),
    ];

    let annotations = vec![
        annotation("FR-COV-001", AnnotationType::Impl),
        annotation("FR-COV-001", AnnotationType::Impl),
        annotation("FR-COV-001", AnnotationType::Test),
        annotation("FR-COV-002", AnnotationType::Impl),
    ];

    let coverage = compute_coverage(&requirements, &annotations);
    let cov1 = coverage.get("FR-COV-001").expect("coverage 1");
    let cov2 = coverage.get("FR-COV-002").expect("coverage 2");
    let cov3 = coverage.get("FR-COV-003").expect("coverage 3");

    assert_eq!(cov1.impl_count, 2);
    assert_eq!(cov1.test_count, 1);
    assert_eq!(cov1.status, CoverageStatus::Covered);
    assert_eq!(cov2.impl_count, 1);
    assert_eq!(cov2.test_count, 0);
    assert_eq!(cov2.status, CoverageStatus::Partial);
    assert_eq!(cov3.impl_count, 0);
    assert_eq!(cov3.test_count, 0);
    assert_eq!(cov3.status, CoverageStatus::Missing);
}

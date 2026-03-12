use crate::models::{Annotation, AnnotationType, Coverage, CoverageStatus, Requirement};
use std::collections::BTreeMap;

/// @req SCS-SCAN-007
pub fn compute_coverage(
    requirements: &[Requirement],
    annotations: &[Annotation],
) -> BTreeMap<String, Coverage> {
    let mut coverage = BTreeMap::new();
    for requirement in requirements {
        coverage.insert(
            requirement.id.clone(),
            Coverage {
                impl_count: 0,
                test_count: 0,
                status: CoverageStatus::Missing,
            },
        );
    }

    for annotation in annotations {
        if let Some(entry) = coverage.get_mut(&annotation.requirement_id) {
            match annotation.annotation_type {
                AnnotationType::Impl => entry.impl_count += 1,
                AnnotationType::Test => entry.test_count += 1,
            }
        }
    }

    for entry in coverage.values_mut() {
        entry.status = coverage_status(entry.impl_count, entry.test_count);
    }

    coverage
}

/// @req SCS-SCAN-008
pub fn coverage_status(impl_count: usize, test_count: usize) -> CoverageStatus {
    if impl_count > 0 && test_count > 0 {
        CoverageStatus::Covered
    } else if impl_count > 0 {
        CoverageStatus::Partial
    } else {
        CoverageStatus::Missing
    }
}

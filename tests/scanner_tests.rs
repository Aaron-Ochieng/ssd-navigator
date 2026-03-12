use std::collections::HashMap;

use ssd_navigator::models::AnnotationType;
use ssd_navigator::scanner::scan_project;

mod support;
use support::{write_requirements, write_tasks, write_temp_file};

#[test]
/// @req SCS-SCAN-003
/// @req SCS-SCAN-004
/// Detects @req annotations across all supported languages.
fn scan_detects_supported_languages() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requirement_id = "FR-LANG-001";
    write_requirements(&dir, &[requirement_id]);
    write_tasks(&dir, &[requirement_id]);

    write_temp_file(
        &dir,
        "src/langs/lib.rs",
        "// @req FR-LANG-001\npub fn rust() {}\n",
    );
    write_temp_file(
        &dir,
        "src/langs/app.ts",
        "// @req FR-LANG-001\nexport const ts = true;\n",
    );
    write_temp_file(
        &dir,
        "src/langs/app.js",
        "// @req FR-LANG-001\nexport const js = true;\n",
    );
    write_temp_file(
        &dir,
        "src/langs/app.go",
        "// @req FR-LANG-001\npackage main\n",
    );
    write_temp_file(
        &dir,
        "src/langs/app.dart",
        "// @req FR-LANG-001\nvoid main() {}\n",
    );
    write_temp_file(
        &dir,
        "src/langs/app.py",
        "# @req FR-LANG-001\ndef run():\n    pass\n",
    );

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    let count = result
        .annotations
        .iter()
        .filter(|annotation| annotation.requirement_id == requirement_id)
        .count();
    assert_eq!(count, 6);
}

#[test]
/// @req SCS-SCAN-005
/// Parses @req annotations from comment prefixes.
fn scan_parses_comment_prefixes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requirement_id = "FR-COMMENT-001";
    write_requirements(&dir, &[requirement_id]);
    write_tasks(&dir, &[requirement_id]);

    write_temp_file(
        &dir,
        "src/lib.rs",
        "let value = 42; // @req FR-COMMENT-001\n",
    );
    write_temp_file(
        &dir,
        "src/app.py",
        "# @req FR-COMMENT-001\ndef run():\n    pass\n",
    );

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    let count = result
        .annotations
        .iter()
        .filter(|annotation| annotation.requirement_id == requirement_id)
        .count();
    assert_eq!(count, 2);
}

#[test]
/// @req SCS-SCAN-006
/// Classifies test files via naming and directory patterns.
fn scan_classifies_test_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let requirement_id = "FR-TEST-001";
    write_requirements(&dir, &[requirement_id]);
    write_tasks(&dir, &[requirement_id]);

    write_temp_file(
        &dir,
        "src/impl.rs",
        "// @req FR-TEST-001\npub fn feature() {}\n",
    );
    write_temp_file(
        &dir,
        "tests/impl_test.rs",
        "// @req FR-TEST-001\n#[test]\nfn test_feature() {}\n",
    );
    write_temp_file(
        &dir,
        "src/test_sample.rs",
        "// @req FR-TEST-001\npub fn test_sample() {}\n",
    );
    write_temp_file(
        &dir,
        "src/sample_test.rs",
        "// @req FR-TEST-001\npub fn sample_test() {}\n",
    );
    write_temp_file(
        &dir,
        "src/sample.test.rs",
        "// @req FR-TEST-001\npub fn sample_dot_test() {}\n",
    );

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    let mut types: HashMap<String, AnnotationType> = HashMap::new();
    for annotation in result
        .annotations
        .iter()
        .filter(|annotation| annotation.requirement_id == requirement_id)
    {
        types.insert(annotation.file.clone(), annotation.annotation_type.clone());
    }

    assert_eq!(types["src/impl.rs"], AnnotationType::Impl);
    assert_eq!(types["tests/impl_test.rs"], AnnotationType::Test);
    assert_eq!(types["src/test_sample.rs"], AnnotationType::Test);
    assert_eq!(types["src/sample_test.rs"], AnnotationType::Test);
    assert_eq!(types["src/sample.test.rs"], AnnotationType::Test);
}

#[test]
/// @req SCS-SCAN-009
/// Detects annotations that reference missing requirements.
fn scan_detects_orphan_annotations() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_requirements(&dir, &["FR-ORPH-001"]);
    write_tasks(&dir, &["FR-ORPH-001"]);
    write_temp_file(
        &dir,
        "src/orphan.rs",
        "// @req FR-ORPH-999\npub fn orphan() {}\n",
    );

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    assert_eq!(result.orphan_annotations.len(), 1);
}

#[test]
/// @req SCS-SCAN-010
/// Detects tasks that reference missing requirements.
fn scan_detects_orphan_tasks() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_requirements(&dir, &["FR-ORPH-001"]);
    write_tasks(&dir, &["FR-ORPH-999"]);
    write_temp_file(&dir, "src/impl.rs", "// @req FR-ORPH-001\npub fn ok() {}\n");

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    assert_eq!(result.orphan_tasks.len(), 1);
}

#[test]
/// @req SCS-SCAN-020
/// Emits a warning for empty source directories.
fn empty_source_directory_warns() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_requirements(&dir, &["FR-EMPTY-001"]);
    write_tasks(&dir, &["FR-EMPTY-001"]);

    let result = scan_project(
        dir.path(),
        &dir.path().join("requirements.yaml"),
        &dir.path().join("tasks.yaml"),
    )
    .expect("scan project");

    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("empty source directory"))
    );
}

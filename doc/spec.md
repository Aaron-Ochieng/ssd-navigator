# SDD Navigator Service — Specification

## Overview

The **SDD Navigator Service** is a Rust HTTP service that scans a project codebase for `@req` annotations, cross-references them with `requirements.yaml` and `tasks.yaml`, computes coverage metrics, and exposes the results through a REST API.

The service follows **Specification-Driven Development (SDD)** principles:

* **Traceability** — requirements link to code and tests through `@req` annotations
* **DRY** — shared models and logic used across CLI and HTTP layers
* **Deterministic Enforcement** — automated checks via `cargo fmt`, `clippy`, `tests`, and strict coverage scan
* **Parsimony** — minimal dependencies and simple architecture

The service must also be **self-hosting**, meaning it can scan its own repository.

---

# Project Structure

```
sdd-navigator/
│
├── Cargo.toml
├── requirements.yaml
├── tasks.yaml
│
├── docs
│   └── spec.md
│
├── src
│   ├── main.rs
│   ├── api.rs
│   ├── scanner.rs
│   ├── parser.rs
│   ├── coverage.rs
│   ├── models.rs
│   └── errors.rs
│
├── tests
│   ├── scanner_tests.rs
│   ├── api_tests.rs
│   └── fixtures/
│
├── README.md
└── PROCESS.md
```

---

# Core Inputs

## requirements.yaml

Defines the system requirements.

Example:

```yaml
requirements:
  - id: FR-AUTH-001
    title: User login
    description: "System MUST authenticate users with email and password and return a signed JWT token."
```

Each requirement must contain:

* `id`
* `title`
* `description`

---

## tasks.yaml

Defines work items mapped to requirements.

Example:

```yaml
tasks:
  - id: TASK-001
    requirementId: FR-AUTH-001
    title: Implement JWT login handler
    status: done
```

Fields:

* id
* requirementId
* title
* status (`open | in_progress | done`)

---

# Annotation Format

Source files reference requirements using `@req`.

Example:

```rust
/// @req FR-AUTH-001
fn login(credentials: &Credentials) -> Result<Token, AuthError> { }
```

Annotations appear in comments.

Supported comment styles:

| Language   | Comment |
| ---------- | ------- |
| Rust       | `//`    |
| TypeScript | `//`    |
| JavaScript | `//`    |
| Go         | `//`    |
| Dart       | `//`    |
| Python     | `#`     |

---

# Supported Languages

The scanner must detect annotations in:

```
.rs
.ts
.js
.py
.go
.dart
```

File detection must be extension-based.

---

# Test File Detection

The scanner must classify files as **implementation** or **test**.

A file is considered a test if it matches:

```
*_test.*
*.test.*
test_*
/tests/
```

Otherwise it is an implementation file.

---

# Coverage Calculation

Each requirement must compute coverage metrics.

Metrics:

```
impl_count
test_count
status
```

Coverage rules:

```
impl > 0 AND test > 0 -> covered
impl > 0 AND test = 0 -> partial
impl = 0 -> missing
```

---

# Orphan Detection

Two orphan types must be detected.

## Orphan Annotation

Occurs when an annotation references a requirement ID not defined in `requirements.yaml`.

Example:

```
@req FR-UNKNOWN-999
```

---

## Orphan Task

Occurs when a task references a requirement ID not defined in `requirements.yaml`.

---

# Project Summary Metrics

The system must compute:

```
total_requirements
covered
partial
missing
coverage_percent
annotation_counts
task_counts
orphan_annotations
orphan_tasks
```

---

# REST API

The service exposes a REST API.

Framework recommendation: **axum**

---

## GET /healthcheck

Returns service status.

Example response:

```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

---

## GET /stats

Returns project metrics.

Example:

```json
{
  "total_requirements": 12,
  "covered": 8,
  "partial": 2,
  "missing": 2,
  "coverage_percent": 66
}
```

---

## GET /requirements

Returns list of requirements.

Supports filtering:

```
?status=missing
?type=FR
```

Supports sorting:

```
?sort=id&order=asc
```

---

## GET /requirements/{requirementId}

Returns a single requirement with:

* linked annotations
* linked tasks
* coverage status

---

## GET /annotations

Returns all detected annotations.

Supports filters:

```
?type=impl
?type=test
?orphans=true
```

---

## GET /tasks

Returns tasks from `tasks.yaml`.

Filters:

```
?status=open
?status=in_progress
?status=done
?orphans=true
```

---

## POST /scan

Triggers a new scan.

Response:

```
202 Accepted
```

Example:

```json
{
  "status": "scanning"
}
```

---

## GET /scan

Returns scan progress.

Example:

```json
{
  "status": "complete"
}
```

---

# Error Handling

The service must handle errors without crashing.

Scenarios:

| Error                     | Behavior               |
| ------------------------- | ---------------------- |
| Missing requirements.yaml | return clear error     |
| Malformed YAML            | include line number    |
| Permission errors         | skip file with warning |
| Empty source directory    | warning only           |

The service must **never panic under user input**.

---

# Scanner Workflow

1. Load `requirements.yaml`
2. Load `tasks.yaml`
3. Walk project directory
4. Parse source files
5. Extract `@req` annotations
6. Classify annotations as `impl` or `test`
7. Compute coverage
8. Detect orphan annotations
9. Detect orphan tasks
10. Compute project summary

---

# Dependencies

Dependencies should remain minimal.

Recommended:

```
serde
serde_yaml
serde_json
regex
walkdir
axum
tokio
thiserror
```

Avoid unnecessary libraries.

---

# Testing Requirements

Tests must reference requirements via annotations.

Example:

```rust
/// @req SCS-SCAN-003
#[test]
fn test_annotation_detection() {}
```

Testing categories:

### Unit Tests

* YAML parsing
* annotation scanning
* coverage calculation
* filtering
* sorting

### Integration Tests

Use fixture projects.

Example:

```
tests/fixtures/sample_project/
```

Contains:

```
requirements.yaml
tasks.yaml
src/
tests/
```

---

# Self-Hosting Requirement

The service must scan **its own repository**.

This requires:

* repository contains `requirements.yaml`
* source code contains `@req` annotations
* tests reference requirement IDs

Verification command:

```
./target/release/sdd-coverage scan --strict
```

Strict mode requires every requirement to have:

```
implementation
AND
tests
```

---

# Deterministic Verification

Before submission, the following commands must pass:

```
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

---

# Deliverables

The repository must include:

* full Rust source code
* `requirements.yaml`
* `tasks.yaml`
* test fixtures
* `README.md`
* `PROCESS.md`

---

# PROCESS.md

This document describes the AI-assisted development process.

Required sections:

```
Tools Used
Conversation Log
Timeline
Key Decisions
Developer Control
Course Corrections
Self Assessment
```

The document should describe how AI tools were used and what the developer verified manually.

---

# Goal

The final system must allow developers to understand **which requirements are implemented, tested, or missing**, enabling traceability across the entire codebase.

# PROCESS

## Tools Used

- OpenAI Codex (terminal-based coding agent) for code edits and test execution.
- Rust toolchain via `cargo` for formatting, linting, tests, and builds.
- Command-line utilities: `rg`, `sed`, `cat` for search and file inspection.
- Zed editor for local code review and edits.
- Antigravity for auxiliary workflow support.

## Conversation Log

Note: Chat metadata timestamps are not available in this environment, so exact start/end times cannot be extracted. Sessions are listed in order with placeholders.

1. Session: Initial feature build (timestamps unavailable)
   - Topic: YAML parsing, scanner, models, coverage, API endpoints, CLI.
   - Developer asked for: YAML parsing with validation, error handling, scanner, API endpoints, tests, and spec compliance.
   - Accepted: New parser, models, coverage computation, scanner, API routes, tests, and CLI behavior.
   - Rejected/corrected: Adjusted stats response shape; fixed test classification; added missing annotations; added binary alias and CLI flags to match spec.

2. Session: Self-hosting and verification (timestamps unavailable)
   - Topic: Deterministic checks and self-hosting strict scan.
   - Developer asked for: Verify strict scan, align CLI with required command.
   - Accepted: Added `sdd-coverage` binary alias, `--source`/`--tests` flags, and completed strict self-hosting scan.
   - Rejected/corrected: Original binary name and CLI flags did not match the required spec command.

## Timeline

Timestamps unavailable; sequence below is chronological.

1. Implemented YAML parsing with validation and structured errors.
2. Added scanner, coverage computation, and project stats.
3. Built API endpoints with filtering/sorting and scan status handling.
4. Added comprehensive tests and shared test helpers for DRY.
5. Verified deterministic checks and fixed strict self-hosting failures by adding missing `@req` coverage.
6. Added `sdd-coverage` binary alias and CLI compatibility for `--source`/`--tests`.

## Key Decisions

- Chose `axum` for the HTTP API due to simplicity and compatibility with async Rust.
- Implemented a single `scan_project` path shared by CLI and API to avoid duplicate logic.
- Used `serde_yaml` for YAML parsing and a custom `AppError` for structured errors.
- Computed stats in a deterministic order using `BTreeMap` for stable outputs.
- Added a separate `sdd-coverage` binary with a shared `app::run` entrypoint to satisfy the required CLI command without duplicating logic.
- Implemented `--source` and `--tests` CLI flags to derive the scan root, matching spec expectations.

## What the Developer Controlled

- Reviewed and accepted changes in:
  - `src/parser.rs` (YAML parsing and validation)
  - `src/scanner.rs` (annotation scan and classification)
  - `src/coverage.rs` (coverage calculations)
  - `src/api.rs` (endpoints and stats)
  - `src/cli.rs` (CLI parsing and strict scan behavior)
  - `src/app.rs`, `src/main.rs`, `src/bin/sdd-coverage.rs` (entrypoints)
  - `tests/*` (API, scanner, parser, coverage, CLI, fixtures)
  - `requirements.yaml`, `tasks.yml`, `Cargo.toml`
- Verification steps executed:
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
  - `cargo build --release`
  - `./target/release/sdd-coverage scan --requirements requirements.yaml --source ./src --tests ./tests --strict`

## Course Corrections

- Updated `/stats` output shape to include `requirements_by_type` and correct annotation/task counts when mismatches were found.
- Fixed test file classification to use relative paths for `tests/` directory detection.
- Added missing `@req` annotations and tests to satisfy strict self-hosting coverage.
- Added `sdd-coverage` binary alias and CLI flags to match the required spec command.
- Refactored entrypoints to avoid clippy warnings for shared `src/main.rs` across multiple bins.

## Self-Assessment (SDD Pillars)

- Traceability: PASS. Requirements are listed in `requirements.yaml`, and code/tests include `@req` annotations. Strict self-hosting scan passes.
- DRY: PASS. Shared scan logic across CLI/API, shared test helpers, centralized stats and coverage.
- Deterministic Enforcement: PASS. All verification commands run; strict scan available and used.
- Parsimony: PARTIAL. Dependencies are minimal, but some helper logic (CLI root resolution) could be simplified further if spec constraints change.

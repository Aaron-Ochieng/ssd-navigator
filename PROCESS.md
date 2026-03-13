# PROCESS

## Tools Used

- OpenAI Codex (GPT-5, terminal-based coding agent) for code edits, tests, and refactors.
- Git CLI for atomic commits and history rewrites.
- Rust toolchain via `cargo` for formatting, linting, tests, and builds.
- Command-line utilities: `rg`, `sed`, `cat` for search and file inspection.
- Zed editor for local code review and edits.
- Antigravity for auxiliary workflow support.


## Conversation Log

Note: Chat metadata timestamps are not available in this environment, so exact start/end times cannot be extracted. Sessions are listed in order with placeholders and should be backfilled from chat metadata.

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

3. Session: DRY and CLI validation fixes (timestamps unavailable)
   - Topic: DRY compliance and CLI argument validation.
   - Developer asked for: Remove duplicate binary entrypoints; fix `cargo run`; validate `--tests` paths.
   - Accepted: Shared entrypoint, `default-run`, CLI validation for missing values and nonexistent/empty tests paths.
   - Rejected/corrected: Prior CLI allowed `--tests` without value and accepted invalid test paths.

4. Session: Process/docs/commit hygiene (timestamps unavailable)
   - Topic: README/PROCESS, committer skill fix, and commit bodies.
   - Developer asked for: Update PROCESS/README, fix skill frontmatter, and add commit bodies.
   - Accepted: SKILL frontmatter fix and amended commits with detailed bodies.
   - Rejected/corrected: Commits without body detail; skill YAML missing frontmatter delimiters.

5. Session: Fixture scan exclusion (timestamps unavailable)
   - Topic: Avoid scanning `tests/fixtures` during project scans.
   - Developer asked for: Exclude fixture examples from annotation scanning.
   - Accepted: Skip `tests/fixtures` paths and added a unit test to enforce it.

## Timeline

Timestamps unavailable; sequence below is chronological.

1. Implemented YAML parsing with validation and structured errors.
2. Added scanner, coverage computation, and project stats.
3. Built API endpoints with filtering/sorting and scan status handling.
4. Added comprehensive tests and shared test helpers for DRY.
5. Verified deterministic checks and fixed strict self-hosting failures by adding missing `@req` coverage.
6. Added `sdd-coverage` binary alias and CLI compatibility for `--source`/`--tests`.
7. Enforced CLI validation for missing values and invalid/empty tests paths.
8. Removed duplicated binary entrypoints and set `default-run` for `cargo run`.
9. Updated README/PROCESS and amended commits with descriptive bodies.
10. Excluded `tests/fixtures` from annotation scanning and added a unit test.

## Key Decisions

- Chose `axum` for the HTTP API due to simplicity and compatibility with async Rust.
- Implemented a single `scan_project` path shared by CLI and API to avoid duplicate logic.
- Used `serde_yaml` for YAML parsing and a custom `AppError` for structured errors.
- Computed stats in a deterministic order using `BTreeMap` for stable outputs.
- Added a separate `sdd-coverage` binary with a shared `app::run` entrypoint to satisfy the required CLI command without duplicating logic.
- Implemented `--source` and `--tests` CLI flags to derive the scan root, matching spec expectations.
- Enforced strict CLI validation for `--tests` to avoid silent strict-mode bypasses.
- Set `default-run` to keep `cargo run` deterministic.
- Skipped `tests/fixtures` during scans to prevent example data from polluting real scans.

## What the Developer Controlled

- Reviewed and accepted changes in:
  - `src/parser.rs` (YAML parsing and validation)
  - `src/scanner.rs` (annotation scan and classification)
  - `src/coverage.rs` (coverage calculations)
  - `src/api.rs` (endpoints and stats)
  - `src/cli.rs` (CLI parsing and strict scan behavior)
  - `src/app.rs`, `src/main.rs`, `src/shared_main.rs`, `src/bin/sdd-coverage.rs` (entrypoints)
  - `tests/*` (API, scanner, parser, coverage, CLI, fixtures)
  - `requirements.yaml`, `tasks.yml`, `Cargo.toml`, `README.md`, `PROCESS.md`, `doc/spec.md`
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
- Refactored entrypoints to eliminate duplicate `main` logic and set `default-run`.
- Tightened CLI parsing to reject missing values and invalid/empty tests paths.
- Amended commits to include descriptive bodies for review traceability.
- Excluded `tests/fixtures` from annotation scanning to avoid scanning sample data.

## Self-Assessment (SDD Pillars)

- Traceability: PASS. Requirements are listed in `requirements.yaml`, and code/tests include `@req` annotations. Strict self-hosting scan passes.
- DRY: PASS. Shared scan logic across CLI/API, shared test helpers, centralized stats and coverage, and shared entrypoint for binaries.
- Deterministic Enforcement: PASS. `fmt`, `clippy`, `test`, `build`, and strict scan were executed; no separate script yet.
- Parsimony: PARTIAL. Dependencies are minimal, but CLI root resolution and validation logic adds complexity driven by spec needs.

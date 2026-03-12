# SDD Navigator Service

A Rust service that scans a repository for `@req` annotations, loads `requirements.yaml` and `tasks.yaml`/`tasks.yml`, computes coverage, and serves results via CLI or HTTP.

## What It Does

- Loads requirements and tasks from YAML.
- Scans source and test files for `@req` annotations.
- Computes coverage (covered/partial/missing) per requirement.
- Detects orphan annotations and orphan tasks.
- Serves results via CLI or HTTP API.

## Build

```bash
cargo build --release
```

## Run

### Server mode

```bash
cargo run -- serve
# or
./target/release/ssd-navigator serve
```

By default the server listens on port `3000`. Override with `PORT`:

```bash
PORT=8080 ./target/release/ssd-navigator serve
```

### CLI scan

Spec-compatible command:

```bash
./target/release/sdd-coverage scan --requirements requirements.yaml --source ./src --tests ./tests --strict
```

Other options:

```bash
./target/release/ssd-navigator scan --root . --requirements requirements.yaml --tasks tasks.yml --strict
./target/release/ssd-navigator scan --json
```

### CLI stats

```bash
./target/release/sdd-coverage stats --root .
```

## API

Endpoints:

- `GET /healthcheck`
- `GET /stats`
- `GET /requirements` with `?type=FR&status=missing` and `?sort=id&order=asc`
- `GET /requirements/{requirementId}`
- `GET /annotations` with `?type=impl|test` and `?orphans=true`
- `GET /tasks` with `?status=open|in_progress|done` and `?orphans=true`
- `POST /scan` (returns 202, starts scan)
- `GET /scan` (returns current scan status)

API specification reference:

```
https://api.pdd.foreachpartners.com/spec/sdd-coverage-api.yaml
```

## Inputs

- `requirements.yaml`
- `tasks.yaml` or `tasks.yml`

## Test Fixtures

Sample projects for integration tests live in:

- `tests/fixtures/sample_project`

## Verification

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
./target/release/sdd-coverage scan --requirements requirements.yaml --source ./src --tests ./tests --strict
```

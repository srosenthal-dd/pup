# Changelog

All notable changes to Pup will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.40.0] - 2026-03-30

### Added
- **Runbooks: Reusable step templates** — define shared step logic in `~/.config/pup/runbooks/_templates/<name>.yaml` and reference it with `template: <name>` in any runbook step; step-level fields override template fields
- **Runbooks: Full HTTP method support** — `http` steps now support POST, PUT, PATCH, DELETE, HEAD, and OPTIONS with `method`, `body`, and `headers` fields
- **Runbooks: Multi-format HTTP body** — `content_type` field selects body encoding (`application/json`, `application/x-www-form-urlencoded`, `text/plain`, etc.); `body_file` loads body from a file path
- **Runbooks: Response capture and output** — `output_file` saves HTTP response body to disk; `accept` header controls response format

### Changed
- **Runbooks: `--arg` flag** replaces `--set` for passing runtime variables to runbook execution (e.g., `pup runbooks run deploy --arg SERVICE=api --arg VERSION=1.2.3`)

## [0.39.0] - 2026-03-28

### Added
- **Runbooks execution engine** — `pup runbooks` command for executing YAML-defined runbooks stored in `~/.config/pup/runbooks/`. Supports step types: `pup`, `shell`, `http`, `datadog-workflow`, and `confirm`. Features: variable interpolation, conditional steps (`when`), failure handling (`on_failure`), polling (`poll`), assertions (`assert`), output capture (`capture`), timestamped output with labeled stdout/stderr, and next-step hints on failure
- **`pup runbooks` subcommands**: `list`, `describe`, `run`, `import`, `validate`
- **Runbook example files** in `docs/examples/runbooks/`: `deploy-service.yaml`, `incident-triage.yaml`, `maintenance-window.yaml`

### Changed
- **DDSQL async querying** — `ddsql table` and `ddsql time-series` now use async query polling, improving reliability for long-running queries
- **Site normalization** — `DD_SITE` values with `app.` prefixes (e.g., `app.datadoghq.com`, `app.us3.datadoghq.com`) are now automatically normalized to their canonical form (`datadoghq.com`, `us3.datadoghq.com`)

### Fixed
- WASM build: unified typed Datadog API client across native and WASM targets, eliminating duplicate implementations
- WASM build: restored `cfg` guards on completions and native-only infrastructure functions

## [0.38.0] - 2026-03-27

### Added
- **Test harness**: 77 new catalog entries and `id_from` resolution for end-to-end parity testing
- Shape-shot snapshots checked in for API output regression testing

## [0.1.0] - Initial Development

### Added
- Initial Go-based CLI wrapper for Datadog APIs
- Basic commands for monitors, dashboards, SLOs, and incidents
- API key authentication support
- OAuth2 authentication with PKCE support
- Keychain storage for secure token management (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- Multi-architecture release builds (Linux, macOS, Windows on amd64/arm64)
- SBOM (Software Bill of Materials) generation
- Code signing with cosign
- APM services/entities API commands (`apm services`, `apm entities`, `apm dependencies`, `apm flow-map`)
- Automatic fallback from OAuth2 to API key authentication
- Comprehensive LLM-friendly help text

### Changed
- Project renamed from "fetch" to "pup"
- Configuration directory moved from `~/.config/fetch` to `~/.config/pup`

### Security
- OAuth2 PKCE flow for secure authentication
- Secure token storage in OS keychain
- Signed release artifacts

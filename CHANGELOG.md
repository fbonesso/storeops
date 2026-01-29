# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-29

### Added

- Full Apple App Store Connect API coverage: apps, versions, builds, TestFlight, metadata, screenshots, previews, pricing, age ratings, phased releases, in-app purchases, subscriptions, availability, reviews, devices, analytics, and submit.
- Full Google Play Developer API coverage: apps, tracks, builds, testers, listings, images, in-app products, availability, reviews, reports, and submit.
- Profile-based authentication with environment variable fallback.
- JSON, table, and markdown output formats with `--pretty` flag.
- Pagination support with `--limit`, `--next`, and `--paginate` flags.
- Interactive REPL console with ASCII banner when running with no arguments.
- Command history persisted across sessions.
- Self-update command (`storeops update`) with background version check.
- Cross-platform binaries (Linux x64/ARM64, macOS x64/ARM64, Windows x64).
- Curl-based install script (`install.sh`).
- GitHub Actions CI and release workflows.
- Agent Skills integration (`.skills/storeops/`).
- AGENTS.md for AI agent usage documentation.

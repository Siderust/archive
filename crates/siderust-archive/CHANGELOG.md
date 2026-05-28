# Changelog

All notable changes to the `siderust-archive` crate are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Renamed crate from `siderust-archive-data` to `siderust-archive`.
- Reorganised features: `default` is now manifest + checksum only. The IERS
  time module is gated behind a new `time` feature; the runtime download
  manager remains behind `fetch` (which implies `time`).
- Crate metadata polished for publication on crates.io
  (`publish` enabled, `keywords`, `categories`, `documentation` populated).

### Added

- `siderust-archive-update-time-data` maintenance binary (requires `--features
  fetch`) that refreshes `time/eop/raw/` and its provenance JSON.
- GitHub Actions workflows:
  - `ci.yml` — fmt, clippy, test matrix, and `cargo publish --dry-run`
    verification on every PR / push.
  - `update-time-data.yml` — weekly IERS refresh + automated patch publish.
  - `publish.yml` — tag-based / manual release to crates.io.

## [0.1.0] - 2026-05-28

### Added

- Initial release.
- `manifest` module: schema-v1 TOML model for `MANIFEST.toml` and per-family
  manifests with provenance, units, validity, checksums.
- `checksum` module: SHA-256 hex helpers.
- `time` module: IERS UTC-TAI / ΔT / Earth Orientation Parameters types,
  parsers, `TimeDataBundle`, and (behind `fetch`) `TimeDataManager` for
  runtime download / verification / atomic swap of the bundle.

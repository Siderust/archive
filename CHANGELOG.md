# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog, and this project uses Semantic Versioning.

## [Unreleased]

### Added

### Changed

### Fixed

## [0.1.3] - 2026-06-01

### Changed
- Refreshed IERS/USNO time-data snapshot:
  - `UTC-TAI.history`
  - `deltat.data`
  - `deltat.preds`
  - `finals2000A.all`
- Regenerated bundled time-data snapshot in `src/time/bundled/snapshot.rs`.

## [0.1.2] - 2026-05-30

### Added
- lagrange generator

## [0.1.1] - 2026-05-30

### Changed
- Bind vsop lagrange points

## [0.1.0] - 2026-05-30

### Added
- Initial `siderust-archive` crate.
- TOML manifest model and archive validator.
- BSD-3-Clause licensing.
- IERS/USNO time-data runtime fetch support behind `fetch`.
- Bundled time-data snapshot support.
- Dataset-family manifests for VSOP87, ELP2000, nutation, JPL kernels, Lagrange kernels, gravity, atmosphere, frames, constants, and Pluto.

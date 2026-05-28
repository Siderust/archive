# siderust-archive

[![Crates.io](https://img.shields.io/crates/v/siderust-archive.svg)](https://crates.io/crates/siderust-archive)
[![Docs.rs](https://docs.rs/siderust-archive/badge.svg)](https://docs.rs/siderust-archive)

Reusable Rust bindings for the [Siderust Archive](https://github.com/Siderust/archive)
— the canonical, repo-agnostic store for scientific datasets (IERS time data,
SPICE-style kernels, planetary theories, …).

The crate is **published to crates.io** so downstream projects don't need to
vendor the archive as a git submodule.

## Installation

```toml
# Library that only needs to parse the manifest schema:
siderust-archive = "0.1"

# Application that wants the IERS time data bundled at runtime:
siderust-archive = { version = "0.1", features = ["fetch"] }

# Only the time-data types + parsers (offline, e.g. tempoch):
siderust-archive = { version = "0.1", features = ["time"] }
```

## Features

| feature   | effect |
|-----------|--------|
| `default` | manifest + checksum APIs only; zero network/crypto dependencies. |
| `time`    | IERS time-scale data: UTC-TAI, ΔT, Earth Orientation Parameters — types, parsers, provenance, typed `TimeDataBundle`. |
| `fetch`   | implies `time`. Runtime download (`ureq`), SHA-256 verification (`sha2`), provenance JSON (`serde_json`). |

## Modules

| module     | feature   | purpose |
|------------|-----------|---------|
| `manifest` | always on | TOML manifest schema v1: top-level archive registry and per-family dataset manifests (provenance, units, validity, checksums). |
| `checksum` | always on | SHA-256 hex helpers. |
| `time`     | `time`    | IERS time-scale data: source URLs, parsers, `TimeDataBundle`, `TimeDataProvenance`, and (with `fetch`) a `TimeDataManager` that downloads / verifies / caches and atomically swaps the bundle. |

## Runtime data layout

`TimeDataManager` resolves its cache directory from `TEMPOCH_DATA_DIR` (kept for
backwards compatibility with tempoch), falling back to `~/.tempoch/data`. It
downloads the raw upstream IERS/USNO files, verifies their SHA-256, writes a
`time_data.provenance.json`, and atomically swaps the new bundle into place. No
scientific data is embedded in the crate itself.

## Maintenance binary

The crate also ships a maintenance binary, `siderust-archive-update-time-data`,
that refreshes the canonical raw files inside the archive repository and writes
a fresh provenance record. It is used by the archive repository's
`update-time-data.yml` GitHub Actions workflow to keep the dataset current.

```bash
cargo run --release --features fetch \
    --bin siderust-archive-update-time-data -- --archive-root ./archive
```

## Tests

```bash
# Unit tests (offline, tiny inline fixtures):
cargo test --features fetch

# Live network integration test (contacts IERS/USNO):
cargo test --features fetch -- --ignored
```

## License

AGPL-3.0-only.

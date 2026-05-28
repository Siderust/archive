# siderust-archive-data

Reusable Rust bindings for the [Siderust Archive](https://github.com/Siderust/archive).

The archive is the canonical, repo-agnostic store for scientific datasets
(kernels, IERS time data, planetary theories, …). Because the archive is
consumed as a **git submodule**, any repository can reuse this crate through a
path dependency into the submodule — keeping each consuming crate free of large
embedded data arrays and duplicated download/parse/provenance logic.

## Using it from another repository

1. Add the archive as a submodule and initialise it:

   ```bash
   git submodule add https://github.com/Siderust/archive.git archive
   git submodule update --init --recursive
   ```

2. Depend on the crate via a path into the submodule:

   ```toml
   [dependencies]
   siderust-archive-data = { path = "archive/crates/siderust-archive-data", features = ["fetch"] }
   ```

   > This crate declares an empty `[workspace]` table so it is **not** absorbed
   > into the consuming repository's cargo workspace.

## Modules

| module     | purpose |
|------------|---------|
| `manifest` | TOML manifest model (schema v1): top-level registry + per-family dataset manifests (provenance, units, validity, checksums). |
| `checksum` | SHA-256 integrity helpers (`fetch` feature). |
| `time`     | IERS time-scale data: leap seconds (UTC-TAI), ΔT, Earth Orientation Parameters. Source URLs, parsers, typed `TimeDataBundle`, and a runtime `TimeDataManager` (`fetch` feature) that downloads, verifies, caches, and records provenance. |

## Features

| feature   | effect |
|-----------|--------|
| `default` | pure parsing + manifest access; no network or crypto dependencies. |
| `fetch`   | runtime downloading (`ureq`), SHA-256 verification (`sha2`), provenance (de)serialisation (`serde_json`). |

## Runtime data layout

`TimeDataManager` resolves its cache directory from `TEMPOCH_DATA_DIR`, falling
back to `~/.tempoch/data`. It downloads the raw upstream IERS/USNO files,
verifies their SHA-256, writes a `time_data.provenance.json`, and atomically
swaps the new bundle into place. No scientific data is embedded in the crate.

## Tests

```bash
# Unit tests (offline, tiny inline fixtures):
cargo test --features fetch

# Live network integration test (contacts IERS/USNO):
cargo test --features fetch -- --ignored
```

## License

AGPL-3.0-only.

# siderust-archive

[![Crates.io](https://img.shields.io/crates/v/siderust-archive)](https://crates.io/crates/siderust-archive)
[![docs.rs](https://img.shields.io/docsrs/siderust-archive)](https://docs.rs/siderust-archive)
[![CI](https://github.com/Siderust/archive/actions/workflows/ci.yml/badge.svg)](https://github.com/Siderust/archive/actions/workflows/ci.yml)
[![Update Time Data](https://github.com/Siderust/archive/actions/workflows/update-time-data.yml/badge.svg)](https://github.com/Siderust/archive/actions/workflows/update-time-data.yml)
[![License: BSD-3-Clause](https://img.shields.io/badge/license-BSD--3--Clause-blue)](LICENSE)

**Scientific dataset manifests, provenance, checksums, and Rust accessors for
the Siderust ecosystem.**

`siderust-archive` is the small Rust crate and repository layout used to keep
Siderust scientific data out of downstream application code. It stores dataset
metadata in TOML, keeps raw payloads in their native formats, and exposes only
the Rust bindings needed to parse, verify, fetch, or bundle those datasets.

> **Pre-1.0 archive:** manifest shape and feature-gated APIs may still change.
> Release notes live in [CHANGELOG.md](CHANGELOG.md).

## Scope

- TOML archive and family manifests.
- SHA-256 checksum helpers and provenance records.
- IERS/USNO time-data parsing, runtime fetch, and bundled offline snapshots.
- Build-time generated accessors for committed coefficient tables.
- Dataset-family manifests for VSOP87, ELP2000, nutation, JPL kernels,
  Lagrange kernels, gravity, atmosphere, frames, constants, and Pluto.

## Non-goals

- Hiding upstream data provenance.
- Fetching large or mutable datasets from downstream build scripts.
- Silently replacing version-pinned datasets.
- Re-implementing Siderust scientific algorithms in the archive crate.

## Installation

```toml
[dependencies]
siderust-archive = "0.1"
```

IERS time-data parsers and the bundled offline snapshot:

```toml
[dependencies]
siderust-archive = { version = "0.1", features = ["time"] }
```

Runtime download/cache support for current IERS/USNO time data and JPL kernel
metadata:

```toml
[dependencies]
siderust-archive = { version = "0.1", features = ["fetch"] }
```

Planetary and geophysical tables are opt-in:

```toml
[dependencies]
siderust-archive = { version = "0.1", features = ["vsop", "elp", "nutation"] }
```

The crate declares its own `[workspace]`, so it is not absorbed into a parent
workspace when used through a local `path` dependency.

## Feature Flags

| Feature | Description |
|---------|-------------|
| default | Manifest, checksum, provenance, and shared error APIs only. |
| `time` | IERS UTC-TAI, Delta T, and EOP types/parsers; includes `bundled-time`. |
| `bundled-time` | Compiled UTC-TAI and Delta T fallback snapshot. |
| `fetch` | Runtime network download/cache support; implies `time` and `jpl`. |
| `jpl` | JPL DE440/DE441 ephemeris metadata and cache manager. |
| `vsop` | VSOP87A/E planetary theory tables. |
| `elp` | ELP2000-82B lunar theory tables. |
| `nutation` | IAU 2000A/2000B nutation coefficient tables. |
| `gravity` | Low-degree EGM2008 geopotential coefficients. |
| `atmosphere` | NRLMSISE-00 lite atmosphere-density table. |
| `frames` | SPICE-style frame definitions placeholder. |
| `constants` | SPICE-style body constants placeholder. |
| `lagrange` | Sun-Earth Lagrange Chebyshev kernel references. |
| `pluto` | Meeus 1998 abbreviated Pluto series. |

## Archive Layout

```text
archive/
├── CHANGELOG.md
├── Cargo.toml
├── LICENSE
├── MANIFEST.toml                # top-level dataset registry
├── README.md
├── schema/                      # manifest and binary-format specs
├── src/                         # crate sources and committed datasets
├── tools/validate/              # structural manifest validator
└── reports/validation/          # validation output, when present
```

Start with [`MANIFEST.toml`](MANIFEST.toml). Each `[[family]]` entry points to
a `src/<family>/manifest.toml` file that records source, generator, validity,
units, references, payload files, byte counts, and SHA-256 checksums.

## Manifest Conventions

- Metadata is TOML. JSON is not used for archive manifests or provenance.
- Payload files keep their authoritative upstream format where practical.
- Generated Rust tables are build artifacts under Cargo `OUT_DIR`; do not
  commit them.
- Dataset generators must record source, generator version, generated time,
  references, byte counts, and checksums.

Validate the registry and all referenced family manifests:

```bash
cargo run -p archive-validate -- MANIFEST.toml
```

## Adding Data

1. Put upstream payloads under `src/<family>/raw/` or the family-specific
   payload directory.
2. Update `src/<family>/manifest.toml`.
3. Register new families in `MANIFEST.toml`.
4. Run `cargo run -p archive-validate -- MANIFEST.toml`.
5. Build with the relevant feature enabled, for example
   `cargo check --features nutation`.

Large mutable datasets need a reviewed process before they become automated.

## Time-Data Maintenance

IERS/USNO time data are operational data. The weekly
[`Update Time Data`](https://github.com/Siderust/archive/actions/workflows/update-time-data.yml)
workflow may refresh only:

- `src/time/eop/raw/`
- `src/time/eop/manifest.toml`
- `src/time/bundled/snapshot.rs`

The snapshot carries TOML provenance in `time_data.provenance.toml`. After
refreshing raw files, the updater rewrites the managed `[[files]]` block in
`src/time/eop/manifest.toml` with current SHA-256 checksums and byte counts
so `archive-validate` can verify every committed payload. The workflow
validates manifests before committing or publishing. Runtime fetch is still
available behind `features = ["fetch"]` for users who want current upstream
data without waiting for the next committed snapshot.

Other archive families are manual/version-pinned. The time-data workflow must
not replace JPL kernels, VSOP/ELP, nutation, gravity, atmosphere, frames,
constants, or derived kernels.

## Releases

Manual releases should update [CHANGELOG.md](CHANGELOG.md) before tagging.
Automated IERS/USNO data-refresh patch releases add a generated changelog
entry for the new patch version and only publish when validation, tests, the
WIP guard, and crates.io credentials all pass.

## License

BSD-3-Clause — see [LICENSE](LICENSE).

Each upstream dataset keeps the license and attribution declared by its
original source. Check the relevant family manifest before redistributing data
payloads outside this repository.

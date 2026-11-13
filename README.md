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

For `no_std` + `alloc` (manifest, checksum, provenance, and scientific tables):

```toml
[dependencies]
siderust-archive = { version = "0.1", default-features = false }
```

IERS time-data parsers and the bundled offline snapshot:

```toml
[dependencies]
siderust-archive = { version = "0.1", features = ["time"] }
```

Runtime download/cache support for current IERS/USNO time data and JPL kernel
metadata (`fetch` requires `std`):

```toml
[dependencies]
siderust-archive = { version = "0.1", features = ["fetch"] }
```

Planetary and geophysical tables are opt-in and work with or without `std`:

```toml
[dependencies]
siderust-archive = { version = "0.1", default-features = false, features = ["vsop", "elp", "nutation"] }
```

The crate declares its own `[workspace]`, so it is not absorbed into a parent
workspace when used through a local `path` dependency.

## Feature Flags

| Feature | Description |
|---------|-------------|
| default | Enables `std`. |
| `std` | Standard library support (filesystem, network, env, OS I/O errors). Absent ⇒ `no_std` + `alloc`. |
| `time` | IERS UTC-TAI, Delta T, and EOP types/parsers; includes `bundled-time`. |
| `bundled-time` | Compiled UTC-TAI and Delta T fallback snapshot. |
| `fetch` | Runtime network download/cache support; requires `std`; implies `time` and `jpl`. |
| `jpl` | JPL DE440/DE441 ephemeris metadata; download/cache needs `fetch`. |
| `vsop` | Full VSOP87A/E planetary theory tables. |
| `vsop-compact` | Compact VSOP87E positions for the Sun and eight planets, 2020–2050. |
| `elp` | Full ELP2000-82B lunar theory tables. |
| `elp-compact` | Compact Earth-centered lunar position, 2020–2050. |
| `nutation` | IAU 2000A/2000B nutation coefficient tables. |
| `gravity` | Low-degree EGM2008 geopotential coefficients. |
| `atmosphere` | NRLMSISE-00 lite atmosphere-density table. |
| `frames` | SPICE-style frame definitions placeholder. |
| `constants` | SPICE-style body constants placeholder. |
| `lagrange` | Sun-Earth Lagrange Chebyshev kernel references. |
| `pluto` | Meeus 1998 abbreviated Pluto series. |

## Compact embedded planetary positions

```toml
siderust-archive = { version = "0.1.7", default-features = false, features = ["vsop-compact"] }
```

`siderust_archive::vsop_compact` provides a checked function per body, such as
`try_mars(jd_tdb)`. It returns `CompactEphemerisError` for non-finite dates or
dates outside `[FIRST_JD_TDB, END_JD_TDB)` (2020-01-01 through 2050-12-31 TDB).
The direct functions, such as `mars(jd_tdb)`, evaluate unchecked.

On success, the position is a geometric barycentric `[X, Y, Z]` in AU in the native
VSOP87 dynamical ecliptic and equinox of J2000.0. Subtract the Earth vector at
the same epoch for an Earth-relative vector. The compiled coefficients require
no runtime I/O or allocation.

The [validation report](reports/compact-vsop87.md) includes accuracy, linked
size, performance, source checksums, and reproduction commands.

## Compact embedded lunar position

```toml
siderust-archive = { version = "0.1.8", default-features = false, features = ["elp-compact"] }
```

`elp_compact::try_moon_geocentric(jd_tdb)` returns a geometric Cartesian
Earth-to-Moon vector in kilometers in the native ELP2000-82B mean dynamical
ecliptic and inertial equinox of J2000.0. The checked API accepts finite TDB
Julian dates from 2020-01-01 through 2050-12-31. Subtract the observer's vector
in the same frame to include geometric lunar parallax. Enable
`vsop-compact` alongside it for planetary positions.

The [lunar validation report](reports/compact-lunar-ephemeris.md) documents
distance and pointing errors, the native frame, and firmware measurements.

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

[Update Time Data](https://github.com/Siderust/archive/actions/workflows/update-time-data.yml)
runs Mondays at 05:23 UTC and can also be dispatched from `main`. It refreshes
`UTC-TAI.history`, `deltat.data`, `deltat.preds`, and `finals2000A.all` under
`src/time/eop/raw/`. Other dataset families use their reviewed, version-pinned
maintenance procedures.

The updater stages all four downloads, checks parsing and coverage against the
committed data, and compares the scientific files byte for byte. A changed
snapshot carries the fetch time and SHA-256 hashes in
`time_data.provenance.toml`. It also updates the checksum and byte-count
entries in `src/time/eop/manifest.toml` and regenerates
`src/time/bundled/snapshot.rs`. Identical payloads retain the committed
provenance and generated files. The workflow validates the manifest and runs
the all-features tests before it can commit a refresh.

Transient download failures get up to three attempts per source, each limited
to 60 seconds, with 2- and 4-second backoff. Transport failures, invalid payloads,
and regressed coverage fail the refresh and preserve the committed snapshot.
The job summary reports the changed datasets, validation, and publication
status. Runtime fetch remains available with the `fetch` feature.

## Releases

The package version in `Cargo.toml` selects the corresponding entry in
`Cargo.lock` and the exact annotated `vMAJOR.MINOR.PATCH` tag. The tag identifies
the release commit. The crates.io index and package checksum establish whether
that version has been published. The GitHub Release uses the matching
`CHANGELOG.md` section as its notes.

Both workflows call the same release workflow and share a concurrency group.
They use GitHub Actions' `GITHUB_TOKEN` with `contents: write` for authorized Git
pushes and GitHub Releases. Configure `CARGO_REGISTRY_TOKEN` as a repository
Actions secret with permission to publish `siderust-archive` on crates.io.
The workflow carries publication through in the same run because GitHub
suppresses follow-on workflows for pushes made with its workflow token.

### Automatic patch releases

A fresh scientific payload change can produce a patch when the exact tag
matching the current Cargo version is on `main`, its crate and GitHub Release
exist, and every intervening commit changes only the approved time-data paths.
The refreshed scientific files must also differ from those at that tag. The
workflow checks the published package's Git commit before selecting a new
patch. It blocks on missing or divergent tags, version mismatches, and
non-data commits, including changes later reverted. A blocked baseline can
still allow a validated data-only commit on `main`.

After manifest validation, tests, Cargo packaging, and the packaged docs.rs
check, the workflow pushes the snapshot commit, release commit, and annotated
tag together. The push is a normal fast-forward and honors branch and tag
rules. If repository rules require review for those commits, prepare the
snapshot and release metadata in a PR and use the manual procedure after
merging it. A normal refresh with unchanged payloads produces no new patch.

### Manual releases and retries

1. Review and merge a PR that sets the desired version in `Cargo.toml` and
   `Cargo.lock` and adds its notes under `## [VERSION]` in `CHANGELOG.md`.
   Confirm CI is green for the intended commit on `main`.
2. Run [Publish](https://github.com/Siderust/archive/actions/workflows/publish.yml)
   on `main` with the bare version (for example, `0.1.6`). With an
   authenticated GitHub CLI, use
   `gh workflow run publish.yml --repo Siderust/archive --ref main -f version=0.1.6`.
   The workflow validates the candidate, then creates the annotated tag,
   publishes to crates.io, verifies the registry package, and creates the
   GitHub Release.
3. Inspect the job summary and compare the tagged commit and GitHub Release
   notes. Verify the version on crates.io and check the corresponding docs.rs
   build.

If a release fails after the tag is reserved, dispatch **Publish** with the
same version. An existing tag on `main` is reused, even if `main` has advanced.
When crates.io already has that version, the workflow verifies the archive
checksum and embedded Git commit before creating any missing GitHub Release.
A yanked version or a package whose source differs from the tag requires
maintainer investigation. If a patch fails before its atomic push, rerun
**Update Time Data** to re-fetch and validate the snapshot. For a manual
release with no reserved tag, rerun **Publish** after fixing the reported
problem. Preserve all existing release tags and versions.

### Recovering from the divergent v0.1.5 tag

The published `0.1.5` crate identifies commit `56954e4`, the commit tagged
`v0.1.5`. Its history and `main` forked at `ae8951c`. The tag branch contains
the release and lockfile commits, while `main` took the squash-merged
`no_std` PR (`921f7f3`) and later snapshot refreshes. The old guard selected
the reachable `v0.1.4` and skipped publication because the newer history
included code changes. See the
[blocked run](https://github.com/Siderust/archive/actions/runs/42631146720).

Preserve `v0.1.5` and its crates.io package. Use the manual procedure above with
`0.1.6` once the recovery PR is merged and CI passes. Verify the
[crate](https://crates.io/crates/siderust-archive/0.1.6),
[GitHub Release](https://github.com/Siderust/archive/releases/tag/v0.1.6), and
[docs.rs builds](https://docs.rs/crate/siderust-archive/0.1.6/builds).
The next eligible refresh uses the reachable `v0.1.6` as its baseline.

### Investigating docs.rs builds

Open the failed build log on the crate's docs.rs Builds page and identify its
nightly revision. CI runs `bash scripts/check-packaged-docs.sh` from a clean
checkout with stable and nightly Rust installed. The script unpacks the
Cargo package and runs offline rustdoc with `DOCS_RS=1`, `--cfg docsrs`, and
all features enabled, so missing inputs to the scientific table generators
fail before release. Use the matching nightly when investigating a
toolchain-specific failure. The
[0.1.5 log](https://docs.rs/crate/siderust-archive/0.1.5/builds/4646880) shows
that Rust removed `doc_auto_cfg` and folded it into `doc_cfg`.

## License

BSD-3-Clause — see [LICENSE](LICENSE).

Each upstream dataset keeps the license and attribution declared by its
original source. Check the relevant family manifest before redistributing data
payloads outside this repository.

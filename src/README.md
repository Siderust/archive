# `archive/src`

This directory is the source tree for the `siderust-archive` crate and the
repo-local home of the archive datasets it exposes. It contains three kinds of
material:

- Rust APIs for manifest parsing, checksum verification, provenance, download
  helpers, and typed access to selected datasets.
- Per-family `manifest.toml` files that describe dataset provenance, validity,
  units, references, payload files, byte counts, and SHA-256 checksums.
- Raw or generated scientific payloads under each dataset family, usually in a
  `raw/` subdirectory.

The top-level registry is [`../MANIFEST.toml`](../MANIFEST.toml). It points to
the family manifests in this directory.

## What Is Here

- `lib.rs` declares the public crate modules and feature-gated dataset APIs.
- `manifest.rs`, `checksum.rs`, `provenance.rs`, and `error.rs` provide the
  always-available archive metadata layer.
- `vsop/`, `elp/`, `nutation/`, `gravity/`, `atmosphere/`, and `pluto/`
  contain coefficient tables plus build-time pipelines that generate Rust
  constants into Cargo `OUT_DIR` when their features are enabled.
- `time/` contains IERS time-scale support, the bundled offline snapshot, and
  subdirectories for leap seconds and Earth-orientation data.
- `jpl/` contains runtime download and cache support for JPL DE ephemeris
  kernels. Large `.bsp` files are not committed here.
- `lagrange/vsop87/` contains Sun-Earth Lagrange Chebyshev kernels in SCK
  format plus a specialized manifest.
- `frames/` and `constants/` are skeleton dataset families for future
  SPICE-style frame and body-constant data.
- `bin/update_time_data.rs` is the maintenance binary used to refresh the
  committed IERS time-data snapshot.

Each module directory has its own `README.md` with local inspection, update,
and usage notes.

## Inspecting Data

Start from the registry:

```bash
sed -n '1,220p' archive/MANIFEST.toml
```

Then inspect the relevant family manifest:

```bash
sed -n '1,220p' archive/src/nutation/manifest.toml
```

For payload integrity, compare the manifest's `[[files]]` entries with the
actual payloads:

```bash
sha256sum archive/src/nutation/raw/nut00a_ls.csv
wc -c archive/src/nutation/raw/nut00a_ls.csv
```

For Rust API shape, read the corresponding module and any `refs.rs`,
`tables.rs`, or `pipeline.rs` files in the same family directory.

## Validating Manifests

Run the structural validator from the archive crate root:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
```

The validator checks the top-level registry, referenced family manifests,
schema versions, basic validity intervals, and declared file metadata shape.
It does not replace domain validation of the scientific data.

## Updating Data

When updating a committed payload:

1. Put upstream files under the family's `raw/` directory, or replace the
   existing payload in place.
2. Update the family `manifest.toml` with source, generator, generation time,
   validity, units, references, `sha256`, and `bytes`.
3. Run the manifest validator.
4. Build or test with the relevant feature enabled so the build-time pipeline
   parses the payload.

Examples:

```bash
cd archive
cargo check --features nutation
cargo check --features "vsop elp gravity atmosphere pluto"
```

The build script regenerates feature-gated Rust tables into Cargo `OUT_DIR`.
Those generated files are build artifacts; do not edit or commit them.

### Refreshing IERS Time Data

The IERS UTC-TAI, Delta T, and EOP bundle has a dedicated updater:

```bash
cd archive
cargo run --features fetch --bin siderust-archive-update-time-data -- --archive-root .
```

The updater downloads the upstream bundle into a staging directory, verifies
that it parses, copies changed raw files under `src/time/eop/raw/`, updates the
time-data provenance file, and regenerates `src/time/bundled/snapshot.rs`.

## Using From Rust

Depend on `siderust-archive` with only the features needed by the caller:

```toml
siderust-archive = "0.1"
siderust-archive = { version = "0.1", features = ["time"] }
siderust-archive = { version = "0.1", features = ["fetch"] }
siderust-archive = { version = "0.1", features = ["vsop", "nutation"] }
```

Feature flags control which dataset modules are compiled. The default feature
set exposes only manifest, checksum, provenance, and shared error APIs.

## Rules Of Thumb

- Keep metadata in TOML.
- Keep large generated build outputs out of source control.
- Record provenance and checksums whenever a payload changes.
- Do not fetch network data from downstream build scripts.
- Use the lowest-level dataset family that matches the data, and keep runtime
  download/cache behavior separate from committed payloads.

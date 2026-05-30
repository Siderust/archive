# `archive/src/gravity`

This module contains the committed low-degree EGM2008 geopotential coefficient
subset used by Siderust gravity models.

## Contents

- `manifest.toml` records provenance, units, references, and checksums.
- `raw/egm2008_n2_4.csv` contains normalized Stokes coefficients for degree
  and order 2 through 4.
- `pipeline.rs` converts the CSV payload into Rust constants during builds.
- `tables.rs` exposes `EGM2008_COEFFS`.
- `refs.rs` keeps source/reference metadata.

## Inspect

```bash
sed -n '1,220p' archive/src/gravity/manifest.toml
sha256sum archive/src/gravity/raw/egm2008_n2_4.csv
wc -c archive/src/gravity/raw/egm2008_n2_4.csv
```

Build the generated table:

```bash
cd archive
cargo check --features gravity
```

## Update

Replace the CSV payload, update `manifest.toml` with the new checksum and byte
count, then validate and build:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features gravity
```

## Use

Enable the `gravity` feature and read coefficients from
`siderust_archive::gravity::tables::EGM2008_COEFFS`.

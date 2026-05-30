# `archive/src/pluto`

This module contains the low-precision Pluto abbreviated series from Meeus
(1998), chapter 37.

## Contents

- `manifest.toml` records provenance, validity, units, references, and
  checksums.
- `raw/pluto_tables.csv` is the committed source table.
- `pipeline.rs` converts the CSV payload into Rust constants during builds.
- `refs.rs` keeps source/reference metadata.
- `mod.rs` exposes the generated `pluto_data` module.

## Inspect

```bash
sed -n '1,220p' archive/src/pluto/manifest.toml
sha256sum archive/src/pluto/raw/pluto_tables.csv
wc -c archive/src/pluto/raw/pluto_tables.csv
```

Build the generated table:

```bash
cd archive
cargo check --features pluto
```

## Update

Replace `raw/pluto_tables.csv`, update `manifest.toml`, then validate and
build:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features pluto
```

## Use

Enable the `pluto` feature and read the generated series from
`siderust_archive::pluto::pluto_data`.

# `archive/src/atmosphere`

This module contains the NRLMSISE-00 lite atmosphere-density lookup table used
by Siderust low-fidelity drag models.

## Contents

- `manifest.toml` records provenance, units, references, validity metadata,
  and checksums for the committed table.
- `raw/nrlmsise_table.csv` is the source payload.
- `pipeline.rs` converts the CSV payload into Rust constants during the crate
  build when the `atmosphere` feature is enabled.
- `tables.rs` exposes the generated table as `NRLMSISE_TABLE`.
- `refs.rs` keeps reference metadata close to the module.

## Inspect

```bash
sed -n '1,220p' archive/src/atmosphere/manifest.toml
sha256sum archive/src/atmosphere/raw/nrlmsise_table.csv
wc -c archive/src/atmosphere/raw/nrlmsise_table.csv
```

Build the parser and generated table:

```bash
cd archive
cargo check --features atmosphere
```

## Update

Replace `raw/nrlmsise_table.csv`, update `manifest.toml` with the new source,
reference, `sha256`, and `bytes`, then run the validator and feature build:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features atmosphere
```

## Use

Enable the crate feature and read the typed table from
`siderust_archive::atmosphere::tables`.

# `archive/src/pluto/raw`

This directory contains the committed source payload for the Pluto abbreviated
series module.

## Contents

- `pluto_tables.csv` is parsed by `../pipeline.rs` into generated Rust
  constants.

## Inspect

```bash
sha256sum archive/src/pluto/raw/pluto_tables.csv
wc -c archive/src/pluto/raw/pluto_tables.csv
```

Compare those values with `../manifest.toml`.

## Update

Replace the CSV only together with an update to `../manifest.toml`, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features pluto
```

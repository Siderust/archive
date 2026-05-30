# `archive/src/atmosphere/raw`

This directory contains the committed source payload for the atmosphere module.

## Contents

- `nrlmsise_table.csv` is the NRLMSISE-00 lite density lookup table consumed by
  `../pipeline.rs`.

## Inspect

```bash
sha256sum archive/src/atmosphere/raw/nrlmsise_table.csv
wc -c archive/src/atmosphere/raw/nrlmsise_table.csv
```

Compare those values with `../manifest.toml`.

## Update

Replace the CSV only together with an update to `../manifest.toml`, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features atmosphere
```

# `archive/src/nutation/raw`

This directory contains the committed CSV payloads for IAU 2000A/2000B
nutation.

## Contents

- `nut00a_ls.csv` contains IAU 2000A luni-solar terms.
- `nut00a_pl.csv` contains IAU 2000A planetary terms.
- `nut00b_ls.csv` contains IAU 2000B luni-solar terms.

## Inspect

```bash
sha256sum archive/src/nutation/raw/*.csv
wc -c archive/src/nutation/raw/*.csv
```

Compare those values with `../manifest.toml`.

## Update

Update CSV files and `../manifest.toml` together, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features nutation
```

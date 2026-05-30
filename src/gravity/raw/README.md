# `archive/src/gravity/raw`

This directory contains the committed source payload for the gravity module.

## Contents

- `egm2008_n2_4.csv` contains normalized EGM2008 Stokes coefficients for
  degree/order 2 through 4.

## Inspect

```bash
sha256sum archive/src/gravity/raw/egm2008_n2_4.csv
wc -c archive/src/gravity/raw/egm2008_n2_4.csv
```

Compare those values with `../manifest.toml`.

## Update

Replace the CSV only together with an update to `../manifest.toml`, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features gravity
```

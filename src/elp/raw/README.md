# `archive/src/elp/raw`

This directory contains the upstream ELP2000-82B sub-series files consumed by
the ELP build pipeline.

## Contents

- `ELP1` through `ELP36` are parsed by `../pipeline.rs`.
- `.gitkeep` keeps the directory present in sparse or transitional checkouts.

## Inspect

```bash
find archive/src/elp/raw -maxdepth 1 -type f | sort
sha256sum archive/src/elp/raw/ELP1
wc -c archive/src/elp/raw/ELP1
```

## Update

Replace ELP raw files as a consistent set, update `../manifest.toml` with
payload metadata, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features elp
```

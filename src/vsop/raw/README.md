# `archive/src/vsop/raw`

This directory contains the upstream IMCCE VSOP87 files consumed by the VSOP
build pipeline.

## Contents

- `VSOP87A.*` files provide heliocentric rectangular series.
- `VSOP87E.*` files provide barycentric rectangular series.
- `.gitkeep` keeps the directory present in sparse or transitional checkouts.

## Inspect

```bash
find archive/src/vsop/raw -maxdepth 1 -type f | sort
sha256sum archive/src/vsop/raw/VSOP87A.ear
wc -c archive/src/vsop/raw/VSOP87A.ear
```

## Update

Replace raw files as a consistent source set, update `../manifest.toml` with
payload metadata, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features vsop
```

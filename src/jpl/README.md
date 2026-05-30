# `archive/src/jpl`

This module contains metadata and runtime cache/download support for JPL
DE-series planetary ephemeris BSP kernels.

## Contents

- `manifest.toml` records the DE440/DE441 dataset metadata and expected
  download checksums.
- `refs.rs` defines dataset identifiers and authoritative download metadata.
- `constants.rs` holds shared JPL constants.
- `cache.rs` and `download.rs` are compiled only with the `fetch` feature.

Large `.bsp` files are not committed. They are downloaded and cached at
runtime.

## Inspect

```bash
sed -n '1,260p' archive/src/jpl/manifest.toml
sed -n '1,220p' archive/src/jpl/refs.rs
```

Build the module:

```bash
cd archive
cargo check --features jpl
cargo check --features fetch
```

## Update

When JPL kernel metadata changes, update both `manifest.toml` and `refs.rs`
so manifest provenance and runtime download metadata stay aligned. Then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features fetch
```

## Use

Enable `fetch` and call `siderust_archive::jpl::DatasetManager`. The default
cache directory is `~/.siderust/data/`; set `SIDERUST_DATA_DIR` to override it.

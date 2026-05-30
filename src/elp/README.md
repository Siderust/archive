# `archive/src/elp`

This module contains ELP2000-82B lunar-theory data and build-time generation
logic for the Rust coefficient tables.

## Contents

- `manifest.toml` records the ELP2000-82B dataset metadata.
- `raw/ELP1` through `raw/ELP36` are the upstream sub-series files.
- `pipeline.rs` parses the raw files and writes generated Rust tables into
  Cargo `OUT_DIR`.
- `constants.rs` contains ELP constants used by the generated representation.
- `refs.rs` records bibliographic/source references.

## Inspect

```bash
sed -n '1,220p' archive/src/elp/manifest.toml
find archive/src/elp/raw -maxdepth 1 -type f | sort
```

Build the generated tables:

```bash
cd archive
cargo check --features elp
```

## Update

Replace or add raw ELP files under `raw/`, update `manifest.toml` with file
metadata and provenance, then validate and rebuild:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features elp
```

Generated `elp_data.rs` output lives in Cargo `OUT_DIR`; do not commit it.

## Use

Enable the `elp` feature and use `siderust_archive::elp::elp_data` plus the
typed term structs from `siderust_archive::elp`.

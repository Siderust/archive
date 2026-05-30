# `archive/src/vsop`

This module contains VSOP87 planetary-theory data and build-time generation
logic for Rust coefficient tables.

## Contents

- `manifest.toml` records VSOP87 metadata and references.
- `raw/` contains the VSOP87A and VSOP87E upstream files for supported bodies.
- `pipeline.rs`, `collect.rs`, `io.rs`, and `codegen.rs` parse and generate
  Rust tables into Cargo `OUT_DIR`.
- `refs.rs` records source/reference metadata.
- `mod.rs` exposes typed VSOP87 terms and generated `vsop87a_data` /
  `vsop87e_data` modules.

## Inspect

```bash
sed -n '1,220p' archive/src/vsop/manifest.toml
find archive/src/vsop/raw -maxdepth 1 -type f | sort
```

Build the generated tables:

```bash
cd archive
cargo check --features vsop
```

## Update

Replace or add raw IMCCE VSOP files under `raw/`, update `manifest.toml` with
payload metadata and provenance, then validate and build:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features vsop
```

Generated `vsop87a.rs` and `vsop87e.rs` output lives in Cargo `OUT_DIR`; do
not commit it.

## Use

Enable the `vsop` feature and read generated terms from
`siderust_archive::vsop::vsop87a_data` or
`siderust_archive::vsop::vsop87e_data`.

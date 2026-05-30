# `archive/src/constants`

This module is the placeholder dataset family for SPICE PCK-style body
constants: gravitational parameters, body radii, and rotation-model data.

## Contents

- `manifest.toml` is a skeleton manifest with no payload entries yet.
- `mod.rs` and `refs.rs` reserve the public Rust module shape.

## Inspect

```bash
sed -n '1,220p' archive/src/constants/manifest.toml
```

## Update

When body-constant payloads are added, place them under this directory, add
`[[files]]` entries to `manifest.toml`, and include source, references,
`sha256`, and `bytes` metadata.

Validate the archive after updating:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features constants
```

## Use

Enable the `constants` feature. The current module exposes references only
until real PCK-style payloads are migrated.

# `archive/src/frames`

This module is the placeholder dataset family for SPICE FK-style reference
frame definitions.

## Contents

- `manifest.toml` is a skeleton manifest with no payload entries yet.
- `mod.rs` and `refs.rs` reserve the public Rust module shape.

## Inspect

```bash
sed -n '1,220p' archive/src/frames/manifest.toml
```

## Update

When frame kernels are added, place the FK-style payloads under this directory,
declare them in `manifest.toml`, and record provenance, `sha256`, `bytes`,
validity, and references.

Validate after changes:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features frames
```

## Use

Enable the `frames` feature. The current module exposes references only until
real frame-definition payloads are migrated.

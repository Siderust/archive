# `archive/src/time/leap-seconds`

This directory is reserved for committed leap-second kernel payloads referenced
by `archive/src/time/manifest.toml`.

## Contents

- `.gitkeep` keeps the directory present until a leap-second kernel is
  committed.

## Inspect

```bash
sed -n '1,220p' archive/src/time/manifest.toml
find archive/src/time/leap-seconds -maxdepth 1 -type f | sort
```

## Update

When adding a NAIF LSK or equivalent leap-second payload, place it here,
update `../manifest.toml` with source URL, `sha256`, `bytes`, and references,
then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features time
```

## Use

Consumers should go through `siderust_archive::time`; this directory is a
payload location, not a public Rust API.

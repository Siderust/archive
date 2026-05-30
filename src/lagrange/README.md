# `archive/src/lagrange`

This module is the dataset family for Lagrange-point Chebyshev kernels.

## Contents

- `mod.rs` and `refs.rs` reserve the public module shape.
- `vsop87/` contains the current Sun-Earth L1-L5 Chebyshev kernel payloads
  generated from VSOP87-based ephemerides.

## Inspect

```bash
find archive/src/lagrange -maxdepth 2 -type f | sort
sed -n '1,220p' archive/src/lagrange/vsop87/manifest.toml
```

## Update

Add new generated kernels under a source-specific subdirectory, keep a
manifest next to the payloads, and record source ephemeris, generator version,
validity interval, target IDs, checksums, and validation errors.

Then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features lagrange
```

## Use

Enable the `lagrange` feature. The current top-level Rust module exposes
references; consumers that need the `.sck` payloads should inspect the
source-specific manifest and load the listed files.

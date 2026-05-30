# `archive/src/lagrange/vsop87`

This directory contains Sun-Earth L1-L5 Chebyshev kernels generated from a
VSOP87-based source ephemeris.

## Contents

- `manifest.toml` records target IDs, frame/time identifiers, record layout,
  checksum, record count, and validation errors for each point.
- `l1.sck` through `l5.sck` are Siderust Chebyshev Kernel payloads.

## Inspect

```bash
sed -n '1,260p' archive/src/lagrange/vsop87/manifest.toml
sha256sum archive/src/lagrange/vsop87/*.sck
wc -c archive/src/lagrange/vsop87/*.sck
```

The SCK record layout is documented by the manifest and by
`archive/schema/sck-v1.md`.

## Update

Regenerate all affected `.sck` files together so the manifest remains a
consistent set. Update checksums, record counts, generator metadata, validity
intervals, and error metrics in `manifest.toml`.

Then validate from the archive root:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
```

## Use

Read `manifest.toml` first, choose the target point by `target_id` or `point`,
then load the corresponding `.sck` file using the documented SCK layout.

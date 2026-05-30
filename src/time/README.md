# `archive/src/time`

This module contains IERS time-scale data support: UTC-TAI history, Delta T,
and Earth Orientation Parameters.

## Contents

- `manifest.toml` describes leap-second kernel metadata.
- `refs.rs` records upstream IERS, USNO, and NAIF URLs.
- `mod.rs` contains parsers, bundle types, provenance types, and the optional
  runtime fetch manager.
- `bundled/` contains the compiled offline UTC-TAI and Delta T snapshot.
- `eop/` contains the EOP/Delta T dataset manifest for runtime-fetched files.
- `leap-seconds/` is reserved for committed leap-second kernel payloads.

## Inspect

```bash
sed -n '1,220p' archive/src/time/manifest.toml
sed -n '1,220p' archive/src/time/eop/manifest.toml
sed -n '1,120p' archive/src/time/bundled/time_data.provenance.toml
```

Build the module:

```bash
cd archive
cargo check --features time
cargo check --features fetch
```

## Update

For the maintained IERS bundle, use the dedicated updater:

```bash
cd archive
cargo run --bin siderust-archive-update-time-data --features fetch
```

The updater refreshes runtime-fetched raw files, provenance, and
`bundled/snapshot.rs`. For leap-second kernel changes, update
`manifest.toml` and the payload under `leap-seconds/`, then validate.

## Use

Enable `time` for parsers and the bundled offline snapshot. Enable `fetch` for
`TimeDataManager`, which downloads and verifies current upstream data.

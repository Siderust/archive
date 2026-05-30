# `archive/src/time/eop`

This directory describes the IERS Earth Orientation Parameters and Delta T
dataset used by `siderust_archive::time::TimeDataManager`.

## Contents

- `manifest.toml` records the upstream IERS/USNO sources, time scales, units,
  references, and expected dataset shape.

IERS/USNO files are operational data and may be refreshed regularly. The repo
may carry a committed frozen snapshot under `raw/` for reproducible builds and
releases, while runtime fetch remains available for users who want current
upstream data without waiting for a committed snapshot.

## Inspect

```bash
sed -n '1,260p' archive/src/time/eop/manifest.toml
```

After a runtime fetch, inspect the cache directory chosen by
`siderust_archive::time::TimeDataManager`; by default it is under
`.tempoch/data` unless the environment override is set.

## Update

Use the updater when refreshing the committed repo snapshot:

```bash
cd archive
cargo run --features fetch --bin siderust-archive-update-time-data -- --archive-root .
```

The updater writes `time_data.provenance.toml` with per-file SHA-256 values
and regenerates the `[[files]]` block in `manifest.toml`. Run the archive
validator after refreshing the snapshot.

## Use

Enable `fetch` and load current EOP data with
`siderust_archive::time::TimeDataManager`.

# `archive/src/time/eop`

This directory describes the IERS Earth Orientation Parameters and Delta T
dataset used by `TimeDataManager`.

## Contents

- `manifest.toml` records the upstream IERS/USNO sources, time scales, units,
  references, and expected dataset shape.

The large and frequently updated raw files are normally downloaded at runtime,
not committed here.

## Inspect

```bash
sed -n '1,260p' archive/src/time/eop/manifest.toml
```

After a fetch, inspect the cache directory chosen by `TimeDataManager`; by
default it is under `.tempoch/data` unless the environment override is set.

## Update

Use the updater when a frozen repo snapshot is desired:

```bash
cd archive
cargo run --bin siderust-archive-update-time-data --features fetch
```

If raw files are committed in the future, populate `[[files]]` entries in
`manifest.toml` from the generated provenance record and run the archive
validator.

## Use

Enable `fetch` and load current EOP data with
`siderust_archive::time::TimeDataManager`.

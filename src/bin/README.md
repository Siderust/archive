# `archive/src/bin`

This directory contains maintenance binaries for the `siderust-archive` crate.

## Contents

- `update_time_data.rs` refreshes the canonical IERS time-data files and the
  compiled bundled snapshot.

## Inspect

```bash
cargo run --manifest-path archive/Cargo.toml --bin siderust-archive-update-time-data --features fetch -- --help
```

## Update

Keep maintenance binaries focused on archive upkeep. If a binary modifies
committed data, it should verify inputs, write provenance, and fail non-zero on
partial or invalid updates.

## Use

Refresh IERS time data from the archive crate root:

```bash
cd archive
cargo run --features fetch --bin siderust-archive-update-time-data -- --archive-root .
```

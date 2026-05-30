# `archive/src/nutation`

This module contains the IAU 2000A and IAU 2000B nutation coefficient tables
derived from MHB2000, SOFA, and IERS Conventions sources.

## Contents

- `manifest.toml` records provenance, units, references, checksums, and
  validity metadata.
- `raw/nut00a_ls.csv` contains IAU 2000A luni-solar terms.
- `raw/nut00a_pl.csv` contains IAU 2000A planetary terms.
- `raw/nut00b_ls.csv` contains IAU 2000B luni-solar terms.
- `pipeline.rs` converts the CSV payloads into Rust constants during builds.
- `tables.rs` exposes `NUT00A_LS`, `NUT00A_PL`, and `NUT00B_LS`.

## Inspect

```bash
sed -n '1,220p' archive/src/nutation/manifest.toml
sha256sum archive/src/nutation/raw/*.csv
wc -c archive/src/nutation/raw/*.csv
```

Build the generated tables:

```bash
cd archive
cargo check --features nutation
```

## Update

Update the raw CSV files and `manifest.toml` together. Keep the manifest's
checksums and byte counts synchronized with the payloads, then run:

```bash
cd archive
cargo run -p archive-validate -- MANIFEST.toml
cargo check --features nutation
```

## Use

Enable the `nutation` feature and read the generated tables from
`siderust_archive::nutation::tables`.

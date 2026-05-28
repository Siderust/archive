# Siderust Archive

Canonical storage for scientific coefficient tables, LUTs, and Chebyshev fits used by the
[Siderust](https://github.com/Siderust/siderust) family of libraries.

All data is stored in language-agnostic formats so that any Siderust language implementation
(Rust, Python, …) can consume it directly or through its own build pipeline.

---

## Repository layout

```
archive/
├── schema/                     ← JSON Schema files for each dataset
├── nutation/
│   └── iau2000a.json           ← IAU 2000A luni-solar + planetary nutation (MHB2000)
├── pluto/
│   └── meeus1998.json          ← Pluto abbreviated series (Meeus 1998 ch.36)
├── vsop87/
│   ├── vsop87a.json            ← VSOP87 version A — heliocentric rectangular
│   └── vsop87e.json            ← VSOP87 version E — barycentric rectangular
├── elp2000/
│   └── elp2000.json            ← ELP2000-82B lunar series
└── lagrange/
    └── vsop87/
        ├── manifest.json       ← NCOEFF, JD span, fit-error metadata
        ├── l1.f64le            ← L1 Chebyshev records (flat f64 little-endian binary)
        ├── l2.f64le
        ├── l3.f64le
        ├── l4.f64le
        └── l5.f64le
```

---

## Formats

### JSON (nutation, pluto, vsop87, elp2000)

Human-readable, auditable, diff-friendly. Each file has a `source` field documenting the
original publication, and a `note` field with accuracy information.

Schema files in `schema/` describe every field.

### Flat binary f64 LE (lagrange)

The Lagrange–Chebyshev records are stored as raw IEEE 754 little-endian `f64` values
(no header, no framing). A companion `manifest.json` documents the block geometry.

This format was chosen over JSON for the Lagrange data because it is ~3.5× smaller
(raw bytes vs. decimal text representation of doubles).

Reading in Rust:
```rust
let bytes = std::fs::read("l1.f64le")?;
let floats: Vec<f64> = bytes
    .chunks_exact(8)
    .map(|b| f64::from_le_bytes(b.try_into().unwrap()))
    .collect();
```

---

## Consuming from siderust

The `siderust` crate's `build.rs` reads `src/data/archive/` (a committed copy of this
repository's data) and generates Rust source into `$OUT_DIR` at compile time. No runtime
I/O is required; all coefficients are embedded in the final binary.

To update siderust's copy after making changes here, run from the siderust root:

```sh
cp -r archive/nutation archive/pluto archive/vsop87 archive/elp2000 archive/lagrange \
      src/data/archive/
```

Or use the provided sync script:

```sh
scripts/sync-from-archive.sh
```

---

## Regenerating data from upstream sources

Each dataset directory contains a `README.md` with instructions for regenerating its JSON
from the canonical upstream source (IMCCE, SOFA, etc.).

For VSOP87 and ELP2000, regeneration is driven by the `regen-data` feature in `siderust`'s
build script. Run from the siderust root:

```sh
SIDERUST_REGEN=1 cargo build --features regen-data
```

This downloads the latest coefficients, parses them, and writes updated JSON to
`src/data/archive/` (which should then be synced to this repository).

---

## License

All data in this archive is derived from publicly available scientific publications and
released under the same [AGPL-3.0-or-later](LICENSE) license as the Siderust project.

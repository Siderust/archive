# Siderust Archive

Canonical storage for scientific coefficient tables, kernels, Chebyshev fits,
and dataset-generation tools used by the
[Siderust](https://github.com/Siderust/siderust) family of libraries.

All metadata is stored in **TOML** so that any tooling — Rust, Python, Julia,
or shell — can consume it directly. Binary payloads (SPICE `.bsp` kernels,
Siderust Chebyshev Kernel `.sck` files, and raw upstream `.dat` files) are
kept as-is in their authoritative formats.

---

## Repository layout

```
archive/
├── README.md
├── LICENSE
├── MANIFEST.toml                ← top-level registry of dataset families
├── schema/                      ← machine-readable manifest specifications
├── crates/                      ← reusable Rust bindings (siderust-archive)
├── generators/                  ← standalone Rust binary crates that produce data
├── tools/                       ← validate / convert utilities
├── vsop87/                      ← VSOP87 planetary theory (raw + manifest)
├── nutation/                    ← IAU 2000A nutation (raw + manifest)
├── elp2000/                     ← ELP2000-82B lunar theory (raw + manifest)
├── pluto/                       ← Meeus 1998 Pluto series (raw + manifest)
├── time/leap-seconds/           ← SPICE LSK text kernel
├── time/eop/                    ← IERS Earth Orientation Parameters + ΔT (fetched at runtime)
├── frames/                      ← SPICE FK-style frame definitions
├── constants/                   ← SPICE PCK-style body constants
├── lagrange/                    ← generated Sun-Earth Lagrange Chebyshev kernels
└── reports/validation/          ← validator output
```

## Rust bindings

The archive ships a reusable Rust crate,
[`crates/siderust-archive`](crates/siderust-archive), that provides
the shared data-access layer: TOML manifest parsing, SHA-256 verification,
provenance, and runtime download of IERS time data. **The crate is published
to [crates.io](https://crates.io/crates/siderust-archive)**, so downstream
projects no longer need to vendor the archive as a git submodule:

```toml
# Manifest + checksum only:
siderust-archive = "0.1"

# IERS time-data types + parsers (e.g. tempoch):
siderust-archive = { version = "0.1", features = ["time"] }

# Full runtime download manager:
siderust-archive = { version = "0.1", features = ["fetch"] }
```

The crate declares an empty `[workspace]` table so it is never absorbed into a
consuming repository's cargo workspace when consumed via a `path` dependency.


## Conventions

- **Format**: every manifest is TOML. JSON is forbidden.
- **Provenance**: every dataset records source, generator, generator version,
  git commit if available, validity interval, frame, time scale, units, and
  SHA-256 checksums.
- **Reproducibility**: dataset generators are explicit. They are *never*
  invoked from a downstream consumer's build script.
- **Determinism**: file SHA-256 digests are committed alongside the data so
  validators can detect silent corruption.
- **No giant Rust arrays**: large embedded `.rs` tables in downstream crates
  are removed and replaced with `include_bytes!`-based loaders or external
  paths.

## Manifest schema (v1)

See [`schema/archive-manifest-v1.md`](schema/archive-manifest-v1.md) for the
authoritative specification. Quick reference:

```toml
schema_version       = 1
dataset_id           = "vsop87"
dataset_kind         = "planetary-theory"
source               = "Bretagnon & Francou 1988 (IMCCE)"
generator            = "siderust/import-vsop87"
generator_version    = "0.8.0"
git_commit           = "deadbeef"
generated_at         = "2026-05-28T20:00:00Z"
time_scale           = "TDB-compatible JD"
frame                = "EclipticMeanJ2000"
center               = "Sun"
units                = "AU, day"
valid_from_jd        = 625296.5
valid_to_jd          = 2816787.5
dynamical_model      = "VSOP87 A/E"

[[files]]
path   = "raw/vsop87a.dat"
format = "vsop87-text"
sha256 = "…"
bytes  = 0

[[references]]
citation = "Bretagnon, P., Francou, G. (1988). Planetary theories in rectangular and spherical variables. A&A 202, 309-315."
```

## Adding a dataset

1. Place upstream raw data in `<family>/raw/`.
2. Create or extend `<family>/manifest.toml`.
3. Run the validator: `cargo run -p archive-validate -- <family>/manifest.toml`.
4. Commit and bump the consuming `siderust` submodule pointer.

## Regenerating derived datasets

Each generator documents its own recipe under
`generators/<name>/README.md`. Generators must:

- write outputs only under `archive/<family>/`,
- compute SHA-256 for each output,
- update the family `manifest.toml`,
- record the git commit and generator version.

## Consuming the archive from Rust

`siderust` accesses the archive through `archive_registry.rs`, an artefact
emitted by `build.rs` into `OUT_DIR`. Three feature flags control behaviour:

- `archive-data`: enable the typed archive registry and runtime loaders.
- `embedded-data`: also embed selected datasets with `include_bytes!`.
- `external-data`: enable runtime loading from caller-provided paths.

Without any of these features the runtime builds with no large data
dependencies.

## License

The archive code and tooling are released under the BSD 3-Clause License (see
[`LICENSE`](LICENSE)). Each upstream dataset retains the license declared by
its original author; see the per-family `manifest.toml` and
`references` arrays for attribution.

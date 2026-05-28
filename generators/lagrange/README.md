# lagrange generator

This directory is reserved for the standalone Cargo binary crate that produces
the `archive/lagrange/<source>/` kernels (`l*.sck` + `manifest.toml`).

Today the generator lives in the `siderust` repository as
`scripts/generate-lagrange-cheby.rs`, gated by the `lagrange-centers` Cargo
feature, because it shares the Newton solver implementation in
`siderust::ephemeris::lagrange::solver`. The plan is to move the generator
here once the public solver API is stable enough to consume as a Cargo
dependency.

## Current invocation

```sh
cd siderust
cargo run --release --features lagrange-centers \
    --bin generate-lagrange-cheby -- \
    --source vsop87 \
    --out archive/lagrange/vsop87 \
    --from 2415020.5 \
    --to 2488070.5
```

The generator writes one `.sck` file per Lagrange point that fits within the
declared 100 m tolerance, plus a `manifest.toml` with checksums and statistics.

See [`../../schema/sck-v1.md`](../../schema/sck-v1.md) for the binary format.

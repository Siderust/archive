# Lagrange generator

Standalone tool crate at [`../../tools/generate-lagrange-cheby/`](../../tools/generate-lagrange-cheby/)
that produces the `src/lagrange/<source>/` kernels (`l*.sck` + `manifest.toml`).

The generator depends on `siderust` for the Lagrange fit/solver APIs and on
`siderust-archive` for checksum helpers. Build it with its manifest directly;
it is not part of the archive workspace test surface.

## Invocation

```sh
cd archive
cargo run --manifest-path tools/generate-lagrange-cheby/Cargo.toml -- \
    --source vsop87 \
    --out src/lagrange/vsop87 \
    --from 2415020.5 \
    --to 2488070.5
```

The generator writes one `.sck` file per Lagrange point that fits within the
declared tolerance, plus a `manifest.toml` with checksums and statistics.

See [`../../schema/sck-v1.md`](../../schema/sck-v1.md) for the binary format.

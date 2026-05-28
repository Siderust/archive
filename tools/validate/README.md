# archive-validate

Validator for Siderust Archive manifests. It enforces the contract described
in [`../../schema/archive-manifest-v1.md`](../../schema/archive-manifest-v1.md):

- `schema_version == 1`
- All required keys present and non-empty
- `valid_to_jd > valid_from_jd`
- Each `[[files]]` entry exists at the declared path
- Each `[[files]]` entry has the declared `sha256` and `bytes`

## Usage

The validator is a small standalone Cargo binary crate. Once implemented:

```sh
cd archive/tools/validate
cargo run --release -- ../../vsop87/manifest.toml
cargo run --release -- ../../MANIFEST.toml    # validates all referenced family manifests
```

Exit codes:

- `0` — every manifest validated successfully
- `1` — at least one manifest failed validation
- `2` — I/O or TOML parse error

Output is a human-readable report on stderr; a machine-readable summary is
written to `archive/reports/validation/<dataset_id>.toml`.

## Implementation notes

This tool intentionally has zero dependency on the `siderust` runtime crate so
that it can validate the archive independently of the consuming codebase. The
only dependencies are `serde`, `toml`, `sha2`, and `hex`.

A skeleton crate will be added under this directory once the first set of
manifests is committed.

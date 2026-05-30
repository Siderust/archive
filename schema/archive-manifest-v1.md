# Siderust Archive Manifest v1

This document is the authoritative specification of the per-family
`manifest.toml` files used in the Siderust Archive. All metadata is TOML;
JSON is not used anywhere in the archive.

## Required top-level keys

| key | type | description |
|-----|------|-------------|
| `schema_version` | integer | Must equal `1`. |
| `dataset_id` | string | Stable identifier (`vsop87`, `lagrange-sun-earth-vsop87`, …). |
| `dataset_kind` | string | One of: `planetary-theory`, `lunar-theory`, `nutation`, `planetary-series`, `lagrange-chebyshev`, `time-scale`, `reference-frame`, `body-constants`, `spice-kernel`, `planetary-ephemeris`, `atmosphere-model`, `geopotential`, `atmosphere`. |
| `source` | string | Human-readable upstream citation. |
| `generator` | string | Identifier of the producer (e.g. `siderust-archive/tools/generate-lagrange-cheby`). |
| `generator_version` | string | Producer version (typically a Cargo package version). |
| `generated_at` | string | RFC 3339 timestamp in UTC. |
| `time_scale` | string | E.g. `TDB-compatible JD`, `TT`, `UTC`. |
| `frame` | string | E.g. `EclipticMeanJ2000`, `ICRF`. |
| `center` | string | E.g. `Solar-System-Barycenter`, `Sun`, `Earth`. |
| `units` | string | Comma-separated list of natural units, e.g. `km, s`. |
| `valid_from_jd` | float | First covered Julian Date in the declared time scale. |
| `valid_to_jd` | float | Last covered Julian Date in the declared time scale. |
| `dynamical_model` | string | Short description of the model used. |

## Optional top-level keys

| key | type | description |
|-----|------|-------------|
| `git_commit` | string | Short SHA of the generator commit if available. |
| `notes` | string | Free-form notes for maintainers. |

## `[[files]]` table

Every committed file shipped with the dataset must appear in a `[[files]]` table.

| key | type | description |
|-----|------|-------------|
| `path` | string | Path relative to the manifest's directory. |
| `format` | string | Format identifier (e.g. `vsop87-text`, `sck-v1`, `iers-eopc04`). |
| `sha256` | string | Lowercase hexadecimal SHA-256 of the file. |
| `bytes` | integer | File size in bytes. |

## `[[remote_files]]` table

Remote-only files that are not committed may appear in `[[remote_files]]`.

| key | type | description |
|-----|------|-------------|
| `path` | string | Relative cache path used after download. |
| `url` | string | Download URL. |
| `sha256` | string | Lowercase hexadecimal SHA-256 of the remote file. |
| `bytes` | integer | Expected file size in bytes, if known. |
| `min_size` | integer | Minimum acceptable byte count for partial sanity checks. |
| `format` | string | Format identifier (e.g. `spk-bsp`). |
| `size_hint` | string | Human-readable size hint for UI or logs. |
| `notes` | string | Maintainer-facing notes about the remote payload. |

## `[[references]]` table (optional but encouraged)

| key | type | description |
|-----|------|-------------|
| `citation` | string | Full bibliographic citation. |
| `doi` | string | DOI without the `https://doi.org/` prefix. |
| `url` | string | Canonical URL. |

## `[error_metrics]` table (optional, required for derived datasets)

| key | type | description |
|-----|------|-------------|
| `max_abs_error_m` | float | Worst-case absolute error in metres on the validation grid. |
| `rms_error_m` | float | RMS error in metres on the validation grid. |
| `validation_step_seconds` | float | Spacing of the validation grid in seconds. |

## Worked example

```toml
schema_version       = 1
dataset_id           = "lagrange-sun-earth-vsop87"
dataset_kind         = "lagrange-chebyshev"
source               = "Generated from Siderust VSOP87E Sun-Earth ephemeris."
generator            = "siderust-archive/tools/generate-lagrange-cheby"
generator_version    = "0.8.0"
git_commit           = "e235139"
generated_at         = "2026-05-28T19:34:00Z"
time_scale           = "TDB-compatible JD"
frame                = "EclipticMeanJ2000"
center               = "Solar-System-Barycenter"
units                = "km, s"
valid_from_jd        = 2415020.5
valid_to_jd          = 2488070.5
dynamical_model      = "Restricted-three-body (Sun-Earth) with lunar perturbation"

[[files]]
path   = "l1.sck"
format = "sck-v1"
sha256 = "…"
bytes  = 1392704

[error_metrics]
max_abs_error_m       = 87.0
rms_error_m           = 27.0
validation_step_seconds = 21600.0

[[references]]
citation = "Szebehely, V. (1967). Theory of Orbits. Academic Press."
```

## Validation

The validator at `tools/validate/` enforces:

1. `schema_version == 1`.
2. All required keys present.
3. Each committed `[[files]]` entry exists and matches its declared SHA-256 and byte count.
4. `[[remote_files]]` metadata is structurally valid.
5. `valid_to_jd > valid_from_jd`.
6. Time scale, frame, center, units strings are non-empty.

A failed check produces a non-zero exit code and a human-readable report.

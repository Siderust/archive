# Siderust Chebyshev Kernel (SCK) v1

The SCK binary format stores Chebyshev coefficients for one body (a Lagrange
point, a small body, a generated spacecraft trajectory) with SPK-compatible
semantics. It is a transitional format used until full SPK/BSP writing is
available in Siderust.

## File layout

All multi-byte integers and floats are little-endian.

```
Header (64 bytes):
  offset  size  field           description
  ------  ----  --------------  ------------------------------------------
   0       8    magic           ASCII "SCKERN01"
   8       4    ncoeff          u32   — Chebyshev coefficients per coordinate
  12       4    record_count    u32   — number of segments
  16       4    center_id       u32   — see [target_ids] in companion manifest
  20       4    target_id       u32   — see [target_ids] in companion manifest
  24       4    frame_id        u32   — see [frame_ids] in companion manifest
  28       4    time_scale_id   u32   — see [time_scale_ids] in companion manifest
  32       8    valid_from_jd   f64   — first covered Julian Date
  40       8    valid_to_jd     f64   — last covered Julian Date
  48      16    reserved        zeros

Data (record_count * (2 + 3 * ncoeff) * 8 bytes):
  Per record (all f64):
    mid_seconds      — segment midpoint, seconds since J2000 in declared time scale
    radius_seconds   — half the segment length, in seconds
    x_c0 .. x_c(n-1) — Chebyshev coefficients for X (kilometres)
    y_c0 .. y_c(n-1) — Chebyshev coefficients for Y (kilometres)
    z_c0 .. z_c(n-1) — Chebyshev coefficients for Z (kilometres)
```

## Evaluation

For an epoch `t` (seconds since J2000 in the declared time scale), locate the
record whose `[mid - radius, mid + radius]` interval contains `t`. Then:

```
tau = (t - mid) / radius                    # tau in [-1, 1]
x   = sum_{k=0..n-1} x_ck * T_k(tau)        # similarly for y, z
```

where `T_k(tau)` are Chebyshev polynomials of the first kind.

## ID assignments

Numeric IDs are defined in the companion `manifest.toml` under
`[frame_ids]`, `[time_scale_ids]`, and `[target_ids]` tables, allowing new
identifiers to be introduced without breaking older readers.

Reserved IDs in v1:

- `frame_id = 0` → `EclipticMeanJ2000`
- `time_scale_id = 0` → TDB-compatible seconds since J2000
- `center_id = 0` → Solar System Barycenter

## Endianness and alignment

All fields are little-endian. Header size is fixed at 64 bytes. Records are
packed and naturally aligned (8-byte boundary) immediately after the header.

## Forward compatibility

Readers MUST refuse files whose magic is not exactly `SCKERN01`. The reserved
header bytes [48..64] MUST be zero. A future SCK v2 will use a different magic
(`SCKERN02`) and may extend the header.

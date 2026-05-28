# archive-convert

Conversion utilities for raw upstream files. Placeholder. Future contents:

- `vsop87-to-cache`: parse an IMCCE VSOP87 ASCII file and emit a compact
  little-endian binary cache (`.bin`) alongside the raw `.dat`.
- `nutation-to-cache`: parse IAU 2000A ASCII tables into a compact binary
  cache.
- `bsp-inspect`: dump segment metadata from a SPICE SPK kernel.

Each converter is expected to be a small standalone Cargo binary with no
runtime dependency on the `siderust` crate.

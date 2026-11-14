#!/usr/bin/env bash
set -euo pipefail

# Shared by CI and both release entry points.
export RUSTFLAGS="-D warnings"

cargo fmt --all -- --check
cargo clippy --locked --all-features --all-targets -- -D warnings
cargo check --locked --no-default-features
cargo check --locked --no-default-features --features vsop,elp,nutation,gravity,atmosphere,frames,constants,lagrange,pluto,time,jpl,bundled-time
cargo check --locked --target thumbv7em-none-eabihf --no-default-features
cargo check --locked --target thumbv7em-none-eabihf --no-default-features --features vsop,elp,nutation,gravity,atmosphere,frames,constants,lagrange,pluto,time,jpl,bundled-time
cargo check --locked --target thumbv7em-none-eabihf --no-default-features --features vsop-compact
cargo check --locked --target thumbv7em-none-eabihf --no-default-features --features elp-compact
cargo test --locked --no-default-features --features elp-compact
cargo test --locked --no-default-features --features vsop-compact,elp-compact
cargo test --locked --no-default-features --features vsop-compact
cargo test --locked --no-default-features
cargo test --locked --features time
cargo test --locked --features time,fetch
cargo test --locked
cargo test --locked --all-features
cargo run --locked -p archive-validate -- MANIFEST.toml
cargo publish --dry-run --locked --all-features -p siderust-archive

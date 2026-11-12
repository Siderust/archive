#!/usr/bin/env bash
set -euo pipefail

# Document the archive Cargo would upload, with docs.rs's feature set and cfg.
# cargo rustdoc passes the cfg only to the archive, as docs.rs does.
cargo package --locked --all-features --no-verify -p siderust-archive "$@"
version=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c '
import json, sys
metadata = json.load(sys.stdin)
print(next(pkg["version"] for pkg in metadata["packages"] if pkg["name"] == "siderust-archive"))
')
target_dir=$(cargo metadata --locked --no-deps --format-version 1 | python3 -c '
import json, sys
print(json.load(sys.stdin)["target_directory"])
')
package="siderust-archive-$version"
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
tar -xzf "$target_dir/package/$package.crate" -C "$scratch"
cargo fetch --locked --manifest-path "$scratch/$package/Cargo.toml"
DOCS_RS=1 cargo +nightly rustdoc --offline --locked --all-features --lib \
    --manifest-path "$scratch/$package/Cargo.toml" -- --cfg docsrs -D warnings

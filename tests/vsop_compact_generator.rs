// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![cfg(feature = "vsop-compact")]

#[allow(dead_code)]
#[path = "../src/vsop/pipeline.rs"]
mod pipeline;

#[test]
fn full_tables_are_generated_only_when_requested() {
    let out = std::path::Path::new(env!("OUT_DIR"));
    for table in ["vsop87a.rs", "vsop87e.rs"] {
        assert_eq!(out.join(table).exists(), cfg!(feature = "vsop"));
    }
}

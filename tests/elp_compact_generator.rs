// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![cfg(feature = "elp-compact")]

#[allow(dead_code)]
#[path = "../src/elp/pipeline.rs"]
mod pipeline;

#[test]
fn full_tables_are_generated_only_when_requested() {
    let out = std::path::Path::new(env!("OUT_DIR"));
    assert_eq!(out.join("elp_data.rs").exists(), cfg!(feature = "elp"));
}

// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#[cfg(any(feature = "elp", feature = "elp-compact"))]
#[path = "support/lunar_models.rs"]
mod models;

#[cfg(any(feature = "elp", feature = "elp-compact"))]
fn main() -> std::io::Result<()> {
    use std::{
        hint::black_box,
        io::{self, BufRead, BufWriter, Write},
        time::Instant,
    };

    if std::env::args().nth(1).as_deref() == Some("dump") {
        let mut out = BufWriter::new(io::stdout().lock());
        for line in io::stdin().lock().lines() {
            let jd: f64 = line?.parse().expect("one TDB Julian date per line");
            let [x, y, z] = models::moon(jd);
            writeln!(out, "{jd:.17},{x:.17e},{y:.17e},{z:.17e}")?;
        }
        return out.flush();
    }

    const EPOCHS: u32 = 2048;
    let epoch = |i| 2_458_849.5 + (2_470_172.5 - 2_458_849.5) * f64::from(i) / f64::from(EPOCHS);
    let mut trials = [0.0; 7];
    for elapsed in &mut trials {
        for i in (0..EPOCHS).step_by(256) {
            black_box(models::moon(black_box(epoch(i))));
        }
        let start = Instant::now();
        for i in 0..EPOCHS {
            black_box(models::moon(black_box(epoch(i))));
        }
        *elapsed = start.elapsed().as_secs_f64();
    }
    trials.sort_by(f64::total_cmp);
    let median = trials[trials.len() / 2];
    println!(
        "{:.3} us/position, {:.0} positions/s",
        median * 1e6 / f64::from(EPOCHS),
        f64::from(EPOCHS) / median
    );
    Ok(())
}

#[cfg(not(any(feature = "elp", feature = "elp-compact")))]
fn main() {}

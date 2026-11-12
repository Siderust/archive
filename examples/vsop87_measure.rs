// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Desktop linked-size, scalar throughput, and validation data harness.

#[cfg(any(feature = "vsop", feature = "vsop-compact"))]
#[path = "support/vsop87_models.rs"]
mod models;

#[cfg(any(feature = "vsop", feature = "vsop-compact"))]
mod measure {
    use std::hint::black_box;
    use std::io::{self, BufRead, BufWriter, Write};
    use std::time::Instant;

    const START: f64 = 2_458_849.5;
    const END: f64 = 2_470_172.5;

    use super::models::positions;

    pub fn run() -> io::Result<()> {
        if std::env::args().nth(1).as_deref() == Some("dump") {
            let mut out = BufWriter::new(io::stdout().lock());
            for line in io::stdin().lock().lines() {
                let jd = line?.parse::<f64>().expect("one TDB Julian date per line");
                write!(out, "{jd:.17}")?;
                for coordinate in positions(jd).into_iter().flatten() {
                    write!(out, ",{coordinate:.17e}")?;
                }
                writeln!(out)?;
            }
            return out.flush();
        }

        const EPOCHS: u32 = 4096;
        // Warm up the code and coefficient data before timing.
        black_box(positions(black_box(START)));
        let start = Instant::now();
        for index in 0..EPOCHS {
            let jd = START + (END - START) * f64::from(index) / f64::from(EPOCHS);
            black_box(positions(black_box(jd)));
        }
        let elapsed = start.elapsed().as_secs_f64();
        println!(
            "{:.3} us/epoch, {:.0} positions/s",
            elapsed * 1e6 / f64::from(EPOCHS),
            f64::from(9 * EPOCHS) / elapsed
        );
        Ok(())
    }
}

fn main() -> std::io::Result<()> {
    #[cfg(any(feature = "vsop", feature = "vsop-compact"))]
    return measure::run();

    #[cfg(not(any(feature = "vsop", feature = "vsop-compact")))]
    Ok(())
}

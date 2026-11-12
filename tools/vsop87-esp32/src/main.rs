// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::hint::black_box;
use core::panic::PanicInfo;

use esp_hal::{clock::CpuClock, time::Instant};
use esp_println::println;

#[path = "../../../examples/support/vsop87_models.rs"]
mod models;

esp_bootloader_esp_idf::esp_app_desc!();

// The archive's general-purpose APIs link against alloc. This harness
// supplies the required allocator symbol but reserves no heap memory.
struct NoHeap;

// SAFETY: Returning null for every request is a valid allocation failure.
// No pointer can be returned from this allocator for later deallocation.
unsafe impl GlobalAlloc for NoHeap {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

#[global_allocator]
static ALLOCATOR: NoHeap = NoHeap;

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    println!("{info}");
    loop {
        core::hint::spin_loop();
    }
}

const EPOCHS: u32 = 128;
const BODIES: u64 = 9;
const TRIALS: usize = 7;

fn evaluate_epoch(index: u32) {
    const START: f64 = 2_458_849.5;
    const END: f64 = 2_470_172.5;
    let jd = START + (END - START) * f64::from(index) / f64::from(EPOCHS);
    black_box(models::positions(black_box(jd)));
}

#[esp_hal::main]
fn main() -> ! {
    let cpu_clock = CpuClock::_80MHz;
    let _peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(cpu_clock));

    let mut elapsed_us = [0; TRIALS];
    for elapsed in &mut elapsed_us {
        // Warm up at eight dates distributed across the timed grid.
        for index in (0..EPOCHS).step_by(16) {
            evaluate_epoch(index);
        }
        let start = Instant::now();
        for index in 0..EPOCHS {
            evaluate_epoch(index);
        }
        *elapsed = start.elapsed().as_micros();
    }

    const POSITIONS: u64 = EPOCHS as u64 * BODIES;
    let total_us: u64 = elapsed_us.iter().sum();
    elapsed_us.sort_unstable();
    let median_us = elapsed_us[TRIALS / 2];
    println!(
        "CPU {} MHz, {TRIALS} trials of {EPOCHS} epochs ({POSITIONS} positions each)",
        cpu_clock as u32
    );
    println!("Trial totals in us (sorted): {elapsed_us:?}");
    println!("Timed positions: {}", POSITIONS * TRIALS as u64);
    // Scale to thousandths of a microsecond without pulling in float formatting.
    let mean = total_us * 1_000 / (POSITIONS * TRIALS as u64);
    let median = median_us * 1_000 / POSITIONS;
    println!("Mean us/position: {}.{:03}", mean / 1_000, mean % 1_000);
    println!(
        "Median trial us/position: {}.{:03}",
        median / 1_000,
        median % 1_000
    );
    loop {
        core::hint::spin_loop();
    }
}

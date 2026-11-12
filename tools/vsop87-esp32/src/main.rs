// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![no_std]
#![no_main]

use core::alloc::{GlobalAlloc, Layout};
use core::hint::black_box;
use core::panic::PanicInfo;

use esp_hal::time::Instant;
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

#[esp_hal::main]
fn main() -> ! {
    let _peripherals = esp_hal::init(esp_hal::Config::default());
    const EPOCHS: u32 = 128;
    const START: f64 = 2_458_849.5;
    const END: f64 = 2_470_172.5;
    println!("Evaluating nine positions per epoch");
    black_box(models::positions(black_box(START)));
    let start = Instant::now();
    for index in 0..EPOCHS {
        let jd = START + (END - START) * f64::from(index) / f64::from(EPOCHS);
        black_box(models::positions(black_box(jd)));
    }
    println!("{EPOCHS} epochs in {} us", start.elapsed().as_micros());
    loop {
        core::hint::spin_loop();
    }
}

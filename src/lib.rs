mod apu;
mod cartridge;
mod cpu;
pub mod emulator;
mod gpu;
mod joypad;
mod memory;
mod timer;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// use crate::cpu::cpu::Cpu;
// use wasm_bindgen::prelude::*;

// const MAX_CYCLES: usize = 69905;

// #[wasm_bindgen]
// pub struct Emulator {
//     cpu: Cpu,
// }

// #[wasm_bindgen]
// impl Emulator {
//     pub fn new(&mut self) -> Self {
//         Emulator { cpu: Cpu::new() }
//     }

//     pub fn update(&mut self) {
//         for _ in 0..MAX_CYCLES {
//             self.cpu.tick();
//         }
//     }

//     pub fn screen(&self) -> *const u8 {
//         self.cpu.mmu.gpu.screen.as_ptr()
//     }
// }

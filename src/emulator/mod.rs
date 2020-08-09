use crate::apu::queue::BUFFER_SIZE;
use crate::cpu::Cpu;
use crate::events::Event;
use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

// 4194300
const AUDIO_SAMPLE_RATE: f32 = 44100.0;
const NUM_AUDIO_CHANNELS: u32 = 2;
const SAMPLE_DURATION: f64 = BUFFER_SIZE as f64 / AUDIO_SAMPLE_RATE as f64;
const LATENCY: f64 = 0.000;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    ctx: AudioContext,
    next_start_time: Option<f64>,
    left_audio: Vec<f32>,
    right_audio: Vec<f32>,
}

#[wasm_bindgen]
impl Emulator {
    pub fn new(data: Vec<u8>) -> Self {
        let mut cpu = Cpu::new(data);

        let ctx = AudioContext::new().unwrap();

        cpu.simulate_bootrom();
        // cpu.emulate_bootrom();

        Emulator {
            cpu,
            ctx,
            next_start_time: None,
            left_audio: vec![0.0; BUFFER_SIZE],
            right_audio: vec![0.0; BUFFER_SIZE],
        }
    }

    pub fn run_till_event(&mut self, max_cycles: usize) -> f64 {
        match self.cpu.run_till_event(max_cycles) {
            Event::VBlank => 0.0,
            Event::AudioBufferFull(left, right) => {
                for i in 0..BUFFER_SIZE {
                    self.left_audio[i] = left[i];
                    self.right_audio[i] = right[i];
                }

                1.0
            }
            Event::MaxCycles => 2.0,
        }
    }

    pub fn audio_buffer_left(&self) -> *const f32 {
        self.left_audio.as_ptr()
    }

    pub fn audio_buffer_right(&self) -> *const f32 {
        self.right_audio.as_ptr()
    }

    pub fn screen(&self) -> *const u8 {
        self.cpu.screen()
    }

    pub fn keyup(&mut self, key: usize) {
        self.cpu.keyup(key);
    }

    pub fn keydown(&mut self, key: usize) {
        self.cpu.keydown(key);
    }
}

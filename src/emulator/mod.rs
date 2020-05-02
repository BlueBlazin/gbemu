use crate::apu::queue::BUFFER_SIZE;
use crate::cpu::Cpu;
use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

const MAX_CYCLES: usize = 69905;
const AUDIO_SAMPLE_RATE: f32 = 44100.0;
const NUM_AUDIO_CHANNELS: u32 = 1;
const TIME_DELTA: f64 = BUFFER_SIZE as f64 / AUDIO_SAMPLE_RATE as f64;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    ctx: AudioContext,
    start_time: Option<f64>,
}

#[wasm_bindgen]
impl Emulator {
    pub fn new(data: Vec<u8>) -> Self {
        let mut cpu = Cpu::new(data);

        let ctx = AudioContext::new().unwrap();

        cpu.simulate_bootrom();

        Emulator {
            cpu,
            ctx,
            start_time: None,
        }
    }

    pub fn update(&mut self) {
        let mut cycles = 0;
        while cycles < MAX_CYCLES {
            cycles += self.cpu.tick();
        }
        self.play_audio();
    }

    #[allow(unused_variables)]
    fn play_audio(&mut self) {
        let start = match self.start_time {
            None => self.ctx.current_time() + 0.001,
            Some(t) => t,
        };

        if let Some(mut q) = self.cpu.mmu.apu.get_next_buffer() {
            let buffer = self
                .ctx
                .create_buffer(NUM_AUDIO_CHANNELS, BUFFER_SIZE as u32, AUDIO_SAMPLE_RATE)
                .unwrap();
            buffer
                .copy_to_channel_with_start_in_channel(&mut q, 0, 0)
                .unwrap();

            let source = self.ctx.create_buffer_source().unwrap();
            source.set_buffer(Some(&buffer));
            source.start_with_when(start).unwrap();
            source
                .connect_with_audio_node(&self.ctx.destination())
                .unwrap();
        }

        self.start_time = Some(start + TIME_DELTA);
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

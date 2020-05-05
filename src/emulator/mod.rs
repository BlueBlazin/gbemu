use crate::apu::queue::BUFFER_SIZE;
use crate::cpu::Cpu;
use wasm_bindgen::prelude::*;
use web_sys::AudioContext;

// 4194300
const MAX_CYCLES: usize = 69905;
const AUDIO_SAMPLE_RATE: f32 = 44100.0;
const NUM_AUDIO_CHANNELS: u32 = 1;
const SAMPLE_DURATION: f64 = BUFFER_SIZE as f64 / AUDIO_SAMPLE_RATE as f64;
const LATENCY: f64 = 0.001;

#[wasm_bindgen]
pub struct Emulator {
    cpu: Cpu,
    ctx: AudioContext,
    next_start_time: Option<f64>,
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
            next_start_time: None,
        }
    }

    pub fn update(&mut self) {
        // let mut cycles = 0;
        // while cycles < MAX_CYCLES {
        //     cycles += self.cpu.tick();
        // }
        // self.play_audio();
        self.update_with_audio();
    }

    pub fn update_with_audio(&mut self) {
        loop {
            // self.update_frame();
            self.cpu.frame();
            if let (Some(left), Some(right)) = self.cpu.mmu.apu.get_next_buffer() {
                self.play_audio_sample(left, right);
                break;
            }
        }
    }

    fn update_frame(&mut self) {
        let mut cycles = 0;
        while cycles < MAX_CYCLES {
            cycles += self.cpu.tick();
        }
    }

    fn play_audio_sample(&mut self, mut left: Vec<f32>, mut right: Vec<f32>) {
        let start_time = match self.next_start_time {
            None => self.ctx.current_time() + LATENCY,
            Some(t) => t,
        };

        let buffer = self
            .ctx
            .create_buffer(NUM_AUDIO_CHANNELS, BUFFER_SIZE as u32, AUDIO_SAMPLE_RATE)
            .unwrap();
        buffer
            .copy_to_channel_with_start_in_channel(&mut left, 0, 0)
            .unwrap();

        let source = self.ctx.create_buffer_source().unwrap();
        source.set_buffer(Some(&buffer));
        source.start_with_when(start_time).unwrap();
        source
            .connect_with_audio_node(&self.ctx.destination())
            .unwrap();
        self.next_start_time = Some(start_time + SAMPLE_DURATION);
    }

    #[allow(unused_variables)]
    fn play_audio(&mut self) {
        let start_time = match self.next_start_time {
            None => self.ctx.current_time() + LATENCY,
            Some(t) => t,
        };

        if let (Some(mut ql), Some(mut qr)) = self.cpu.mmu.apu.get_next_buffer() {
            let buffer = self
                .ctx
                .create_buffer(NUM_AUDIO_CHANNELS, BUFFER_SIZE as u32, AUDIO_SAMPLE_RATE)
                .unwrap();
            buffer
                .copy_to_channel_with_start_in_channel(&mut ql, 0, 0)
                .unwrap();

            let source = self.ctx.create_buffer_source().unwrap();
            source.set_buffer(Some(&buffer));
            source.start_with_when(start_time).unwrap();
            source
                .connect_with_audio_node(&self.ctx.destination())
                .unwrap();
            self.next_start_time = Some(start_time + SAMPLE_DURATION);
        }
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

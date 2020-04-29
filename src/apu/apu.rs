use crate::apu::queue::AudioQueue;
use crate::apu::square::SquareWave;
use crate::apu::wave::WaveChannel;

pub const SAMPLE_RATE: usize = 95; // in M-cycles
                                   // pub const BUFFER_SIZE: usize = 735;
const SEQUENCER_PERIOD: usize = 8192;

pub struct Apu {
    clocks: usize,
    sample_clocks: usize,
    channel1: SquareWave,
    channel2: SquareWave,
    channel3: WaveChannel,
    // pub channel1_samples: Vec<f32>,
    // pub channel2_samples: Vec<f32>,
    // pub channel3_samples: Vec<f32>,
    pub channel1_samples: AudioQueue,
    pub channel2_samples: AudioQueue,
    pub channel3_samples: AudioQueue,
    i: usize,
    seq_ptr: usize,
    master_on: bool,
    master_vol_left: f32,
    master_vol_right: f32,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            clocks: 0,
            sample_clocks: 0,
            channel1: SquareWave::new(),
            channel2: SquareWave::new(),
            channel3: WaveChannel::new(),
            // channel1_samples: vec![0.0; BUFFER_SIZE],
            // channel2_samples: vec![0.0; BUFFER_SIZE],
            // channel3_samples: vec![0.0; BUFFER_SIZE],
            channel1_samples: AudioQueue::new(),
            channel2_samples: AudioQueue::new(),
            channel3_samples: AudioQueue::new(),
            i: 0,
            seq_ptr: 0,
            master_on: false,
            master_vol_left: 1.0,
            master_vol_right: 1.0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.clocks += 1;
            self.sample_clocks += 1;

            if self.clocks >= SEQUENCER_PERIOD {
                self.clocks = self.clocks - SEQUENCER_PERIOD;
                match self.seq_ptr {
                    0 => {
                        self.channel1.length_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                    }
                    2 => {
                        self.channel1.length_tick();
                        self.channel1.sweep_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                    }
                    4 => {
                        self.channel1.length_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                    }
                    6 => {
                        self.channel1.length_tick();
                        self.channel1.sweep_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                    }
                    7 => {
                        self.channel1.volume_tick();
                        self.channel2.volume_tick();
                    }
                    _ => (),
                }
                self.seq_ptr = (self.seq_ptr + 1) % 8;
            }

            self.channel1.tick(1);
            self.channel2.tick(1);
            self.channel3.tick(1);

            if self.sample_clocks >= SAMPLE_RATE {
                self.sample_clocks = self.sample_clocks - SAMPLE_RATE;
                // if self.master_on {
                //     self.channel1_samples[self.i] = self.channel1.dac() * self.master_vol_left;
                //     self.channel2_samples[self.i] = self.channel2.dac() * self.master_vol_left;
                //     self.channel3_samples[self.i] = self.channel3.dac() * self.master_vol_left;
                // }
                let on = self.master_on as u8 as f32;
                self.channel1_samples
                    .push(self.channel1.dac() * self.master_vol_left * on);
                self.channel2_samples
                    .push(self.channel2.dac() * self.master_vol_left * on);
                self.channel3_samples
                    .push(self.channel3.dac() * self.master_vol_left * on);
                self.i += 1;
            }
        }
    }

    // pub fn tick(&mut self, cycles: usize) {
    //     self.clocks += cycles;
    //     self.sample_clocks += cycles;

    //     if self.clocks >= SEQUENCER_PERIOD {
    //         self.clocks = self.clocks - SEQUENCER_PERIOD;
    //         match self.seq_ptr {
    //             0 => {
    //                 self.channel1.length_tick();
    //                 self.channel2.length_tick();
    //             }
    //             2 => {
    //                 self.channel1.length_tick();
    //                 self.channel1.sweep_tick();
    //                 self.channel2.length_tick();
    //             }
    //             4 => {
    //                 self.channel1.length_tick();
    //                 self.channel2.length_tick();
    //             }
    //             6 => {
    //                 self.channel1.length_tick();
    //                 self.channel1.sweep_tick();
    //                 self.channel2.length_tick();
    //             }
    //             7 => {
    //                 self.channel1.volume_tick();
    //                 self.channel2.volume_tick();
    //             }
    //             _ => (),
    //         }
    //         self.seq_ptr = (self.seq_ptr + 1) % 8;
    //     }

    //     self.channel1.tick(cycles);
    //     self.channel2.tick(cycles);

    //     if (self.i < BUFFER_SIZE) && (self.sample_clocks >= SAMPLE_RATE) {
    //         self.sample_clocks = self.sample_clocks - SAMPLE_RATE;
    //         if self.master_on {
    //             self.channel1_samples[self.i] = self.channel1.dac() * self.master_vol_left;
    //             self.channel2_samples[self.i] = self.dac() * self.master_vol_left;
    //         }
    //         self.i += 1;
    //     }
    // }

    fn dac(&self) -> f32 {
        let amp = self.channel2.output_volume as u32 as f32 / 4.0;
        (amp - 8.0) / 8.0
    }

    // pub fn reset(&mut self) {
    //     for i in 0..BUFFER_SIZE {
    //         self.channel1_samples[i] = 0.0;
    //         self.channel2_samples[i] = 0.0;
    //     }
    //     self.i = 0;
    // }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel1.get_byte(addr),
            0xFF16..=0xFF19 => self.channel2.get_byte(addr),
            0xFF1A..=0xFF1E => self.channel3.get_byte(addr),
            0xFF26 => 0xFF,
            0xFF30..=0xFF3F => self.channel3.get_byte(addr),
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10..=0xFF14 => self.channel1.set_byte(addr, value),
            0xFF16..=0xFF19 => self.channel2.set_byte(addr, value),
            0xFF1A..=0xFF1E => self.channel3.set_byte(addr, value),
            0xFF24 => {
                self.master_vol_left = (((value & 0x70) >> 4) as f32) / 7.0;
                self.master_vol_right = ((value & 0x7) as f32) / 7.0;
            }
            0xFF26 => {
                self.master_on = (value & 0x80) != 0;
                if !self.master_on {
                    // self.reset();
                    self.channel1.restart();
                    self.channel2.restart();
                }
            }
            0xFF30..=0xFF3F => self.channel3.set_byte(addr, value),
            _ => (),
        }
    }

    pub fn get_next_buffer(&mut self) -> Option<Vec<f32>> {
        self.channel2_samples.dequeue()
    }
}

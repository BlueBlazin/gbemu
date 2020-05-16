pub mod envelope;
pub mod noise;
pub mod queue;
pub mod square;
pub mod wave;

use crate::apu::noise::Noise;
use crate::apu::queue::AudioQueue;
use crate::apu::square::SquareWave;
use crate::apu::wave::WaveChannel;

const SAMPLE_RATE: usize = 95;
const SEQUENCER_PERIOD: usize = 8192;

pub struct AudioRegisters {
    nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,
}

impl Default for AudioRegisters {
    fn default() -> Self {
        AudioRegisters {
            nrx0: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
        }
    }
}

pub struct Apu {
    clocks: usize,
    sample_clocks: usize,
    channel1: SquareWave,
    channel2: SquareWave,
    channel3: WaveChannel,
    channel4: Noise,
    samples: AudioQueue,
    i: usize,
    seq_ptr: usize,
    master_on: bool,
    master_vol_left: f32,
    master_vol_right: f32,
    nr50: u8,
    nr51: u8,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            clocks: 0,
            sample_clocks: 0,
            channel1: SquareWave::new(),
            channel2: SquareWave::new(),
            channel3: WaveChannel::new(),
            channel4: Noise::new(),
            samples: AudioQueue::new(),
            i: 0,
            seq_ptr: 0,
            master_on: false,
            master_vol_left: 1.0,
            master_vol_right: 1.0,
            nr50: 0,
            nr51: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.clocks += 1;
            self.sample_clocks += 1;

            if self.clocks >= SEQUENCER_PERIOD {
                self.clocks -= SEQUENCER_PERIOD;
                match self.seq_ptr {
                    0 => {
                        self.channel1.length_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                        self.channel4.length_tick();
                    }
                    2 => {
                        self.channel1.length_tick();
                        self.channel1.sweep_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                        self.channel4.length_tick();
                    }
                    4 => {
                        self.channel1.length_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                        self.channel4.length_tick();
                    }
                    6 => {
                        self.channel1.length_tick();
                        self.channel1.sweep_tick();
                        self.channel2.length_tick();
                        self.channel3.length_tick();
                        self.channel4.length_tick();
                    }
                    7 => {
                        self.channel1.volume_tick();
                        self.channel2.volume_tick();
                        self.channel4.volume_tick();
                    }
                    _ => (),
                }
                self.seq_ptr = (self.seq_ptr + 1) % 8;
            }

            self.channel1.tick(1);
            self.channel2.tick(1);
            self.channel3.tick(1);
            self.channel4.tick(1);

            if self.sample_clocks >= SAMPLE_RATE {
                self.sample_clocks -= SAMPLE_RATE;
                let left = self.audio_out_left();
                let right = self.audio_out_right();
                self.samples.push(left, right);
                self.i += 1;
            }
        }
    }

    fn audio_out_left(&mut self) -> f32 {
        let gain = (self.master_on as u8 as f32) * self.master_vol_left;
        let tot_amp = self.total_amp(self.nr51 >> 4) * gain;
        tot_amp / 4.0
    }

    fn audio_out_right(&mut self) -> f32 {
        let gain = (self.master_on as u8 as f32) * self.master_vol_right;
        let tot_amp = self.total_amp(self.nr51) * gain;
        tot_amp / 4.0
    }

    fn total_amp(&self, nr51: u8) -> f32 {
        let mut tot_amp = 0.0;
        if self.channel1.enabled && (nr51 & 0x1) != 0 {
            tot_amp += self.channel1.dac();
        }
        if self.channel2.enabled && (nr51 & 0x2) != 0 {
            tot_amp += self.channel2.dac();
        }
        if self.channel3.enabled && (nr51 & 0x4) != 0 {
            tot_amp += self.channel3.dac();
        }
        if self.channel4.enabled && (nr51 & 0x8) != 0 {
            tot_amp += self.channel4.dac();
        }
        tot_amp
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF10..=0xFF14 => self.channel1.get_byte(addr),
            0xFF15..=0xFF19 => self.channel2.get_byte(addr),
            0xFF1A..=0xFF1E => self.channel3.get_byte(addr),
            0xFF1F..=0xFF23 => self.channel4.get_byte(addr),
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            0xFF26 => {
                (self.master_on as u8) << 7
                    | (self.channel4.enabled as u8) << 3
                    | (self.channel3.enabled as u8) << 2
                    | (self.channel2.enabled as u8) << 1
                    | (self.channel1.enabled as u8)
            }
            0xFF30..=0xFF3F => self.channel3.get_byte(addr),
            _ => panic!("Unhandled APU register get {:#X}", addr),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10..=0xFF14 => self.channel1.set_byte(addr, value),
            0xFF15..=0xFF19 => self.channel2.set_byte(addr, value),
            0xFF1A..=0xFF1E => self.channel3.set_byte(addr, value),
            0xFF1F..=0xFF23 => self.channel4.set_byte(addr, value),
            0xFF24 => {
                self.master_vol_left = (((value & 0x70) >> 4) as f32) / 7.0;
                self.master_vol_right = ((value & 0x07) as f32) / 7.0;
                self.nr50 = value;
            }
            0xFF25 => self.nr51 = value,
            0xFF26 => {
                self.master_on = (value & 0x80) != 0;
                if !self.master_on {
                    self.channel1.enabled = false;
                    self.channel2.enabled = false;
                    self.channel3.enabled = false;
                    self.channel4.enabled = false;
                }
            }
            0xFF30..=0xFF3F => self.channel3.set_byte(addr, value),
            _ => panic!("Unhandled APU register set {:#X}", addr),
        }
    }

    pub fn get_next_buffer(&mut self) -> (Option<Vec<f32>>, Option<Vec<f32>>) {
        self.samples.dequeue()
    }
}

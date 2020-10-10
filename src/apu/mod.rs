// References: https://github.com/torch2424/wasmboy/tree/master/lib/audio

pub mod noise;
pub mod queue;
pub mod square;
pub mod wave;

use crate::apu::noise::Noise;
use crate::apu::queue::AudioQueue;
use crate::apu::square::SquareWave;
use crate::apu::wave::WaveChannel;
use crate::cpu::EmulationMode;

const SAMPLE_RATE: usize = 95;
const SEQUENCER_PERIOD: usize = 8192;

const NR50: u16 = 0xFF24;
const NR51: u16 = 0xFF25;
const NR52: u16 = 0xFF26;

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
    mode: EmulationMode,
}

impl Apu {
    pub fn new(mode: EmulationMode) -> Self {
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
            mode,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.clocks += 1;
            self.sample_clocks += 1;

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
                        self.channel1.sweep_tick();
                        self.channel1.length_tick();
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
                        self.channel1.sweep_tick();
                        self.channel1.length_tick();
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
            0xFF15 => 0xFF,
            0xFF16..=0xFF19 => self.channel2.get_byte(addr),
            0xFF1A..=0xFF1E => self.channel3.get_byte(addr),
            0xFF1F..=0xFF23 => self.channel4.get_byte(addr),
            NR50 => self.nr50,
            NR51 => self.nr51,
            NR52 => {
                0x70 | (self.master_on as u8) << 7
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
            0xFF10..=0xFF13 if self.master_on => self.channel1.set_byte(addr, value),
            0xFF14 if self.master_on => self.channel1.set_nrx4(value, self.seq_ptr),
            0xFF15..=0xFF18 if self.master_on => self.channel2.set_byte(addr, value),
            0xFF19 if self.master_on => self.channel2.set_nrx4(value, self.seq_ptr),
            0xFF1A..=0xFF1D if self.master_on => self.channel3.set_byte(addr, value),
            0xFF1E if self.master_on => self.channel3.set_nrx4(value, self.seq_ptr),
            0xFF1F..=0xFF22 if self.master_on => self.channel4.set_byte(addr, value),
            0xFF23 if self.master_on => self.channel4.set_nrx4(value, self.seq_ptr),
            NR50 if self.master_on => {
                self.master_vol_left = (((value & 0x70) >> 4) as f32) / 7.0;
                self.master_vol_right = ((value & 0x07) as f32) / 7.0;
                self.nr50 = value;
            }
            NR51 if self.master_on => self.nr51 = value,
            NR52 => {
                let old_master_on = self.master_on;

                self.master_on = (value & 0x80) != 0;

                // Power off
                if old_master_on && !self.master_on {
                    self.clear_registers();
                }

                // Power on
                if !old_master_on && self.master_on {
                    self.seq_ptr = 0;
                    self.channel1.step = 0;
                    self.channel2.step = 0;
                    self.channel3.sample = 0;
                }
            }
            0xFF30..=0xFF3F => self.channel3.set_byte(addr, value),
            _ => (),
        }
    }

    fn clear_registers(&mut self) {
        self.master_on = true;

        for addr in 0xFF10..=0xFF25 {
            self.set_byte(addr, 0);
        }

        self.master_on = false;
    }

    pub fn get_next_buffer(&mut self) -> (Option<Vec<f32>>, Option<Vec<f32>>) {
        self.samples.dequeue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registers_with(d: u8) {
        let mut apu = Apu::new(EmulationMode::Cgb);
        apu.set_byte(NR50, 0x77);

        let targets = [
            0x80, 0x3F, 0x00, 0xFF, 0xBF, 0xFF, 0x3F, 0x00, 0xFF, 0xBF, 0x7F, 0xFF, 0x9F, 0xFF,
            0xBF, 0xFF, 0xFF, 0x00, 0x00, 0xBF, 0x00, 0x00, 0x70,
        ];

        let mut addr = 0xFF10;

        for target in targets.iter() {
            if addr == NR52 {
                continue;
            }
            apu.set_byte(addr, d);
            assert_eq!(
                apu.get_byte(addr) | *target,
                *target,
                "addr: {:#X}, d: {:#X}",
                addr,
                d
            );
            addr += 1;
        }

        apu.set_byte(0xFF1A, 0);

        for addr in 0xFF30..=0xFF3F {
            apu.set_byte(addr, d);
            assert_eq!(apu.get_byte(addr), d, "addr: {:#X}", addr);
        }
    }

    #[test]
    fn test_registers() {
        for d in 0..0xFF {
            test_registers_with(d);
        }
    }
}

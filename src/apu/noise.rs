use crate::apu::envelope::{EnvelopeDirection, VolumeEnvelope};
use crate::apu::AudioRegisters;

const DIVISORS: [usize; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct Noise {
    pub clock: usize,
    registers: AudioRegisters,
    length_load: usize,
    volume: VolumeEnvelope,
    dac_enabled: bool,
    length_enabled: bool,
    period: usize,
    width_mode: u8,
    lfsr: u16,
    pub output_volume: u8,
    pub enabled: bool,
    length_counter: usize,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            clock: 0,
            registers: AudioRegisters::default(),
            length_load: 0,
            volume: VolumeEnvelope::default(),
            dac_enabled: false,
            length_enabled: false,
            period: 0,
            width_mode: 0,
            lfsr: 1,
            output_volume: 0,
            enabled: false,
            length_counter: 0,
        }
    }

    pub fn dac(&self) -> f32 {
        if self.enabled && self.dac_enabled {
            self.output_volume as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.clock += cycles;
        if self.clock >= self.period {
            self.clock -= self.period;

            let value = (self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);
            self.lfsr >>= 1;
            self.lfsr |= value << 14;

            if self.width_mode != 0 {
                self.lfsr &= !0x40;
                self.lfsr |= value << 6;
            }

            self.output_volume = if (self.lfsr & 0x01) == 0 {
                self.volume.volume
            } else {
                0
            };
        }
    }

    pub fn volume_tick(&mut self) {
        if self.volume.period == 0 {
            return;
        }

        self.volume.clock += 1;

        if self.volume.clock >= self.volume.period {
            self.volume.clock -= self.volume.period;

            self.volume.volume = match self.volume.direction {
                EnvelopeDirection::Increase if self.volume.volume < 15 => self.volume.volume + 1,
                EnvelopeDirection::Decrease if self.volume.volume > 0 => self.volume.volume - 1,
                _ => self.volume.volume,
            };
        }
    }

    pub fn length_tick(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
                self.length_enabled = false;
                self.registers.nrx4 &= !0x40;
            }
        }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF1F => self.registers.nrx0,
            0xFF20 => self.registers.nrx1,
            0xFF21 => self.registers.nrx2,
            0xFF22 => self.registers.nrx3,
            0xFF23 => self.registers.nrx4,
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1F => self.registers.nrx0 = value,
            0xFF20 => {
                self.registers.nrx1 = value;
                self.length_load = (value & 0x7F) as usize;
            }
            0xFF21 => {
                self.registers.nrx2 = value;
                self.dac_enabled = (value & 0xF8) != 0;
                self.volume.volume = (value & 0xF0) >> 4;
                self.volume.set_direction((value & 0x8) != 0);
                self.volume.period = (value & 0x7) as usize;
            }
            0xFF22 => {
                self.registers.nrx3 = value;
                let shift = (value >> 4) & 0x0F;
                let divisor_code = value & 0x07;
                self.width_mode = ((value & 0x08) != 0) as u8;
                self.period = DIVISORS[divisor_code as usize] << shift;
            }
            0xFF23 => {
                self.registers.nrx4 = value;
                self.length_enabled = (value & 0x40) != 0;
                if (value & 0x80) != 0 {
                    self.restart();
                }
            }
            _ => (),
        }
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 64;
        }
        self.clock = 0;
        self.volume.clock = 0;
        self.volume.volume = (self.registers.nrx2 & 0xF0) >> 4;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }
}

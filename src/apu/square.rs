use crate::apu::envelope::{EnvelopeDirection, VolumeEnvelope};
use crate::apu::AudioRegisters;

const DUTY_TABLE: [[bool; 8]; 4] = [
    [false, false, false, false, false, false, false, true],
    [true, false, false, false, false, false, false, true],
    [true, false, false, false, false, true, true, true],
    [false, true, true, true, true, true, true, false],
];

pub struct Timer {
    pub clock: usize,
    pub period: usize,
    pub duty: usize,
    pub step: usize,
}

impl Default for Timer {
    fn default() -> Self {
        Timer {
            clock: 0,
            period: 0,
            duty: 0,
            step: 0,
        }
    }
}

impl Timer {
    pub fn set_period(&mut self, freq: u16) {
        self.period = ((2048 - freq) * 4) as usize;
        self.clock = 0;
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.period == 0 {
            return;
        }
        self.clock += cycles;
        if self.clock >= self.period {
            self.clock -= self.period;
            self.step = (self.step + 1) % 8;
        }
    }
}

// ----------------------------------------------------------------------------------------------------

pub struct Sweep {
    pub shadow_freq: u16,
    pub shift: u16,
    pub clock: usize,
    pub period: usize,
    pub negate: bool,
    pub active: bool,
}

impl Default for Sweep {
    fn default() -> Self {
        Sweep {
            shadow_freq: 0,
            shift: 0,
            clock: 0,
            period: 0,
            negate: true,
            active: false,
        }
    }
}

// ----------------------------------------------------------------------------------------------------

// pub enum EnvelopeDirection {
//     Increase,
//     Decrease,
// }

// pub struct VolumeEnvelope {
//     pub volume: u8,
//     pub direction: EnvelopeDirection,
//     pub clock: usize,
//     pub period: usize,
// }

// impl Default for VolumeEnvelope {
//     fn default() -> Self {
//         Self {
//             volume: 0,
//             direction: EnvelopeDirection::Decrease,
//             clock: 0,
//             period: 0,
//         }
//     }
// }

// impl VolumeEnvelope {
//     pub fn set_direction(&mut self, add: bool) {
//         self.direction = if add {
//             EnvelopeDirection::Increase
//         } else {
//             EnvelopeDirection::Decrease
//         };
//     }
// }

// ----------------------------------------------------------------------------------------------------

pub struct LengthCounter {
    pub counter: usize,
    pub enabled: bool,
}

impl Default for LengthCounter {
    fn default() -> Self {
        Self {
            counter: 0,
            enabled: false,
        }
    }
}

// ----------------------------------------------------------------------------------------------------

pub struct SquareWave {
    pub output_volume: u8,
    registers: AudioRegisters,
    // timing
    timer: Timer,
    // modulation units
    sweep: Sweep,
    length: LengthCounter,
    volume: VolumeEnvelope,
    enabled: bool,
    dac_enabled: bool,
}

impl SquareWave {
    pub fn new() -> Self {
        SquareWave {
            output_volume: 0,
            registers: AudioRegisters::default(),
            timer: Timer::default(),
            sweep: Sweep::default(),
            length: LengthCounter::default(),
            volume: VolumeEnvelope::default(),
            enabled: false,
            dac_enabled: false,
        }
    }

    pub fn dac(&self) -> f32 {
        if self.dac_enabled && self.enabled {
            self.output_volume as f32 / 15.0
        } else {
            0.0
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.timer.tick(cycles);

        self.output_volume = if self.enabled && DUTY_TABLE[self.timer.duty][self.timer.step] {
            self.volume.volume
        } else {
            0
        };
    }

    pub fn sweep_tick(&mut self) {}

    pub fn length_tick(&mut self) {
        if self.length.enabled && self.length.counter > 0 {
            self.length.counter -= 1;
            if self.length.counter == 0 {
                self.enabled = false;
            }
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

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 | 0xFF15 => self.registers.nrx0,
            0xFF11 | 0xFF16 => self.registers.nrx1,
            0xFF12 | 0xFF17 => self.registers.nrx2,
            0xFF13 | 0xFF18 => self.registers.nrx3,
            0xFF14 | 0xFF19 => self.registers.nrx4,
            _ => unreachable!(),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF15 => self.registers.nrx0 = value,
            0xFF10 => {
                self.registers.nrx0 = value;
                self.sweep.period = ((value & 0x70) >> 4) as usize;
                self.sweep.negate = (value & 0x08) != 0;
                self.sweep.shift = (value & 0x07) as u16;
            }
            0xFF11 | 0xFF16 => {
                self.registers.nrx1 = value;
                self.timer.duty = ((value & 0xC0) >> 6) as usize;
                // self.length.counter = 64 - (value & 0x3F) as usize;
                self.length.counter = (value & 0x3F) as usize;
            }
            0xFF12 | 0xFF17 => {
                self.registers.nrx2 = value;
                self.dac_enabled = (value & 0xF8) != 0;
                self.volume.volume = (value & 0xF0) >> 4;
                self.volume.set_direction((value & 0x8) != 0);
                self.volume.period = (value & 0x7) as usize;
            }
            0xFF13 | 0xFF18 => {
                self.registers.nrx3 = value;
                self.sweep.shadow_freq = (self.sweep.shadow_freq & 0x700) | value as u16;
            }
            0xFF14 | 0xFF19 => {
                self.registers.nrx4 = value;
                self.sweep.shadow_freq =
                    (self.sweep.shadow_freq & 0xFF) | (((value & 0x07) as u16) << 8);
                self.timer.set_period(self.sweep.shadow_freq);
                self.length.enabled = (value & 0x40) != 0;
                if (value & 0x80) != 0 {
                    self.restart();
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        if self.length.counter == 0 {
            self.length.counter = 64;
        }
        self.timer.step = 0;
        self.timer.set_period(self.sweep.shadow_freq);
        self.volume.clock = 0;
        self.sweep.active = false;
        self.volume.volume = (self.registers.nrx2 & 0xF0) >> 4;

        if !self.dac_enabled {
            self.enabled = false;
        }
    }
}

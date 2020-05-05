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
    pub enabled: bool,
}

impl Default for Sweep {
    fn default() -> Self {
        Sweep {
            shadow_freq: 0,
            shift: 0,
            clock: 0,
            period: 0,
            negate: true,
            enabled: false,
        }
    }
}

impl Sweep {
    fn next_freq(&mut self) -> u16 {
        if self.negate {
            self.shadow_freq
                .wrapping_sub(self.shadow_freq >> self.shift)
        } else {
            self.shadow_freq + self.shadow_freq >> self.shift
        }
    }
}

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

    pub fn sweep_tick(&mut self) {
        self.sweep.clock += 1;
        if self.sweep.clock >= self.sweep.period {
            self.sweep.clock -= self.sweep.period;
            // The sweep timer is clocked at 128 Hz by the frame sequencer.
            // When it generates a clock and the sweep's internal enabled flag is set and
            // the sweep period is not zero, a new frequency is calculated and the overflow
            // check is performed. If the new frequency is 2047 or less and the sweep shift
            // is not zero, this new frequency is written back to the shadow frequency
            // and square 1's frequency in NR13 and NR14, then frequency calculation and overflow
            // check are run AGAIN immediately using this new value, but this second new frequency is not written back.

            // Square 1's frequency can be modified via NR13 and NR14 while sweep
            // is active, but the shadow frequency won't be affected so the next time the
            // sweep updates the channel's frequency this modification will be lost.

            if self.sweep.enabled && self.sweep.period != 0 {
                let next_freq = self.sweep_and_overflow();

                if next_freq <= 2047 && self.sweep.shift != 0 {
                    self.sweep.shadow_freq = next_freq;
                    self.registers.nrx3 = (next_freq & 0xFF) as u8;
                    self.registers.nrx4 = ((next_freq >> 8) & 0x07) as u8;
                    self.timer.set_period(next_freq);
                }

                self.sweep_and_overflow();
            }
        }
    }

    fn sweep_and_overflow(&mut self) -> u16 {
        let next_freq = self.sweep.next_freq();
        if next_freq > 2047 {
            self.enabled = false;
        }
        next_freq
    }

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
                // self.sweep.shadow_freq = (self.sweep.shadow_freq & 0x700) | value as u16;
            }
            0xFF14 | 0xFF19 => {
                self.registers.nrx4 = value;
                // self.sweep.shadow_freq =
                //     (self.sweep.shadow_freq & 0xFF) | (((value & 0x07) as u16) << 8);
                // self.timer.set_period(self.sweep.shadow_freq);
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

        // During a trigger event, several things occur:
        //     Square 1's frequency is copied to the shadow register.
        //     The sweep timer is reloaded.
        //     The internal enabled flag is set if either the sweep period or shift are non-zero, cleared otherwise.
        //     If the sweep shift is non-zero, frequency calculation and the overflow check are performed immediately.
        self.sweep.shadow_freq = (self.sweep.shadow_freq & 0x700) | self.registers.nrx3 as u16;
        self.sweep.shadow_freq =
            (self.sweep.shadow_freq & 0xFF) | (((self.registers.nrx4 & 0x07) as u16) << 8);

        let freq = self.sweep.shadow_freq;

        self.sweep.clock = 0;
        self.sweep.enabled = (self.sweep.period != 0) || (self.sweep.shift != 0);
        if self.sweep.shift != 0 {
            self.sweep_and_overflow();
        }

        self.timer.step = 0;
        self.timer.set_period(freq);
        self.volume.clock = 0;
        self.volume.volume = (self.registers.nrx2 & 0xF0) >> 4;

        if !self.dac_enabled {
            self.enabled = false;
        }
    }
}

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
        self.clock = 0;
        self.period = ((2048 - freq) * 4) as usize;
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
    pub timer: Timer,
    length: LengthCounter,
    // volume: VolumeEnvelope,
    pub enabled: bool,
    dac_enabled: bool,

    shadow_freq: u16,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_enabled: bool,
    sweep_counter: usize,
    sweep_negate_used: bool,

    volume: u8,
    starting_volume: u8,
    volume_add: bool,
    volume_period: usize,
    volume_counter: usize,
    volume_auto_update: bool,
}

impl SquareWave {
    pub fn new() -> Self {
        SquareWave {
            output_volume: 0,
            registers: AudioRegisters::default(),
            timer: Timer::default(),
            length: LengthCounter::default(),
            // volume: VolumeEnvelope::default(),
            enabled: false,
            dac_enabled: false,

            shadow_freq: 0,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_enabled: false,
            sweep_counter: 0,
            sweep_negate_used: false,

            volume: 0,
            starting_volume: 0,
            volume_add: false,
            volume_period: 0,
            volume_counter: 0,
            volume_auto_update: false,
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
            self.volume
        } else {
            0
        };
    }

    pub fn sweep_tick(&mut self) {
        if !self.enabled || !self.sweep_enabled {
            return;
        }

        self.sweep_counter -= 1;

        if self.sweep_counter == 0 {
            if self.sweep_period == 0 {
                self.sweep_counter = 8;
            } else {
                self.sweep_counter = self.sweep_period as usize;

                let freq = self.freq_calc_and_overflow_check();

                if self.enabled && self.sweep_shift > 0 {
                    if freq <= 2047 {
                        self.shadow_freq = freq;
                        self.registers.nrx3 = freq as u8;
                        self.registers.nrx4 |= (freq >> 8) as u8;
                        self.timer.set_period(self.shadow_freq);
                    }

                    self.freq_calc_and_overflow_check();
                }
            }
        }
    }

    fn freq_calc_and_overflow_check(&mut self) -> u16 {
        let mut freq = self.shadow_freq >> self.sweep_shift;

        freq = if self.sweep_negate {
            self.sweep_negate_used = true;
            self.shadow_freq - freq
        } else {
            self.shadow_freq + freq
        };

        if freq > 2047 {
            self.enabled = false;
        }

        freq
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
        if !self.enabled || !self.volume_auto_update {
            return;
        }

        self.volume_counter -= 1;

        if self.volume_counter == 0 {
            if self.volume_period == 0 {
                self.volume_counter = 8;
            } else {
                self.volume_counter = self.volume_period;

                if self.volume_add {
                    if self.volume < 0xF {
                        self.volume += 1;
                    } else {
                        self.volume_auto_update = false;
                    }
                } else {
                    if self.volume > 0 {
                        self.volume -= 1;
                    } else {
                        self.volume_auto_update = false;
                    }
                }
            }
        }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => 0x80 | self.registers.nrx0,
            0xFF15 => 0xFF,
            0xFF11 | 0xFF16 => 0x3F | self.registers.nrx1,
            0xFF12 | 0xFF17 => self.registers.nrx2,
            0xFF13 | 0xFF18 => 0xFF,
            0xFF14 | 0xFF19 => 0xBF | self.registers.nrx4,
            _ => unreachable!(),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF15 => self.registers.nrx0 = value,
            0xFF10 => {
                self.registers.nrx0 = value;

                self.sweep_period = (value & 0x70) >> 4;

                let old_sweep_negate = self.sweep_negate;
                self.sweep_negate = (value & 0x08) != 0;

                if old_sweep_negate && !self.sweep_negate && self.sweep_negate_used {
                    self.enabled = false;
                }

                self.sweep_shift = value & 0x07;
            }
            0xFF11 | 0xFF16 => {
                self.registers.nrx1 = value;

                self.timer.duty = ((value & 0xC0) >> 6) as usize;
                self.length.counter = 64 - (value & 0x3F) as usize;
            }
            0xFF12 | 0xFF17 => {
                self.registers.nrx2 = value;

                self.starting_volume = (value & 0xF0) >> 4;
                self.volume_add = (value & 0x8) != 0;
                self.volume_period = (value & 0x7) as usize;

                let old_dac_enabled = self.dac_enabled;
                self.dac_enabled = (value & 0xF8) != 0;
                if old_dac_enabled && !self.dac_enabled {
                    self.enabled = false;
                }
            }
            0xFF13 | 0xFF18 => {
                self.registers.nrx3 = value;
            }
            _ => unreachable!(),
        }
    }

    pub fn set_nrx4(&mut self, value: u8, counter_wont_clock: bool) {
        self.registers.nrx4 = value;

        let trigger = (value & 0x80) != 0;

        if trigger {
            self.trigger();
        }

        if counter_wont_clock
            && !self.length.enabled
            && (value & 0x40) != 0
            && self.length.counter > 0
        {
            self.length.counter -= 1;

            if self.length.counter == 0 {
                if trigger {
                    self.length.counter = 63;
                } else {
                    self.enabled = false;
                }
            }
        }

        self.length.enabled = (value & 0x40) != 0;
    }

    pub fn trigger(&mut self) {
        self.enabled = true;

        if self.length.counter == 0 {
            self.length.counter = 64;
            self.length.enabled = false;
        }

        self.shadow_freq = (self.registers.nrx4 as u16 & 0x7) << 8 | self.registers.nrx3 as u16;

        self.timer.set_period(self.shadow_freq);

        self.sweep_counter = self.sweep_period as usize;

        if self.sweep_counter == 0 {
            self.sweep_counter = 8;
        }

        self.sweep_enabled = self.sweep_period > 0 || self.sweep_shift > 0;

        self.sweep_negate_used = false;

        if self.sweep_shift > 0 {
            self.freq_calc_and_overflow_check();
        }

        self.volume = self.starting_volume;

        self.volume_counter = self.volume_period;

        if self.volume_counter == 0 {
            self.volume_counter = 8;
        }

        self.volume_auto_update = true;

        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn clear_registers(&mut self) {
        // self.registers = AudioRegisters::default();
        self.registers.nrx0 = 0;
        self.registers.nrx1 = 0;
        self.registers.nrx2 = 0;
        self.registers.nrx3 = 0;
        self.registers.nrx4 = 0;
    }
}

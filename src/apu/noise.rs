use crate::apu::AudioRegisters;

const DIVISORS: [usize; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

pub struct Noise {
    counter: usize,
    clock_shift: u8,

    registers: AudioRegisters,
    dac_enabled: bool,
    length_enabled: bool,
    period: usize,
    width_mode: u8,
    lfsr: u16,
    pub output_volume: u8,
    pub enabled: bool,
    length_counter: usize,

    volume: u8,
    starting_volume: u8,
    volume_add: bool,
    volume_period: usize,
    volume_counter: usize,
    volume_auto_update: bool,
}

impl Noise {
    pub fn new() -> Self {
        Self {
            counter: 0,
            clock_shift: 0,

            registers: AudioRegisters::default(),
            dac_enabled: false,
            length_enabled: false,
            period: 0,
            width_mode: 0,
            lfsr: 1,
            output_volume: 0,
            enabled: false,
            length_counter: 0,

            volume: 0,
            starting_volume: 0,
            volume_add: false,
            volume_period: 0,
            volume_counter: 0,
            volume_auto_update: false,
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
        if !self.enabled {
            return;
        }

        if self.counter <= cycles {
            let delta = cycles - self.counter;

            self.counter = self.period - delta;

            let xor = (self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);

            if self.clock_shift < 14 {
                self.lfsr >>= 1;
                self.lfsr &= 0x3FFF;
                self.lfsr |= xor << 14;

                if self.width_mode != 0 {
                    self.lfsr &= 0x3F;
                    self.lfsr |= xor << 6;
                }
            }

            self.output_volume = if self.lfsr & 0x1 == 0 { self.volume } else { 0 };
        } else {
            self.counter -= cycles;
        }
    }

    pub fn volume_tick(&mut self) {
        if !self.enabled {
            return;
        }

        self.volume_counter -= 1;

        if self.volume_counter == 0 {
            if self.volume_period == 0 {
                self.volume_counter = 8;
            } else {
                self.volume_counter = self.volume_period;

                if self.volume_auto_update {
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
    }

    pub fn length_tick(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF1F => 0xFF,
            0xFF20 => 0xFF,
            0xFF21 => self.registers.nrx2,
            0xFF22 => self.registers.nrx3,
            0xFF23 => 0xBF | self.registers.nrx4,
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1F => self.registers.nrx0 = value,
            0xFF20 => {
                self.registers.nrx1 = value;
                self.length_counter = 64 - (value & 0x3F) as usize;
            }
            0xFF21 => {
                self.registers.nrx2 = value;

                if self.volume_period == 0 && self.volume_auto_update {
                    self.volume = (self.volume + 1) & 0xF;
                }

                if self.volume_add != ((value & 0x8) != 0) {
                    self.volume = (16 - self.volume) & 0xF;
                }

                self.starting_volume = (value & 0xF0) >> 4;
                self.volume_add = (value & 0x8) != 0;
                self.volume_period = (value & 0x7) as usize;

                let old_dac_enabled = self.dac_enabled;
                self.dac_enabled = (value & 0xF8) != 0;
                if old_dac_enabled && !self.dac_enabled {
                    self.enabled = false;
                }
            }
            0xFF22 => {
                self.registers.nrx3 = value;

                self.clock_shift = value >> 4;

                let divisor_code = value & 0x07;

                self.width_mode = ((value & 0x08) != 0) as u8;

                self.period = DIVISORS[divisor_code as usize] << self.clock_shift;
            }
            _ => (),
        }
    }

    pub fn set_nrx4(&mut self, value: u8, seq_ptr: usize) {
        self.registers.nrx4 = value;

        let counter_wont_clock = seq_ptr % 2 == 1;

        let trigger = (value & 0x80) != 0;

        if trigger {
            self.trigger();
        }

        if counter_wont_clock
            && !self.length_enabled
            && (value & 0x40) != 0
            && self.length_counter > 0
        {
            self.length_counter -= 1;

            if self.length_counter == 0 {
                if trigger {
                    self.length_counter = 63;
                } else {
                    self.enabled = false;
                }
            }
        }

        if seq_ptr == 7 && trigger {
            self.volume_counter += 1;
        }

        self.length_enabled = (value & 0x40) != 0;
    }

    pub fn trigger(&mut self) {
        self.enabled = self.dac_enabled;

        if self.length_counter == 0 {
            self.length_counter = 64;
            self.length_enabled = false;
        }

        self.counter = self.period;

        self.volume = self.starting_volume;

        self.volume_counter = self.volume_period;

        if self.volume_counter == 0 {
            self.volume_counter = 8;
        }

        self.volume_auto_update = true;

        self.lfsr = 0x7FFF;
    }

    pub fn clear_registers(&mut self) {
        self.registers = AudioRegisters::default();
    }
}

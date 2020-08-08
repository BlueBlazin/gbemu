use crate::apu::AudioRegisters;

pub struct WaveChannel {
    pub table: [u8; 32],
    wave_ram: [u8; 16],
    pub freq: u16,
    // pub period: usize,
    // pub clock: usize,
    pub i: usize,
    pub enabled: bool,
    pub sample: u8,
    registers: AudioRegisters,
    dac_enabled: bool,
    length_counter: usize,
    volume_code: u8,
    length_enabled: bool,
    pub counter: usize,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            table: [
                0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
                0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
            ],
            wave_ram: [
                0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF,
                0x00, 0xFF,
            ],
            freq: 0,
            // period: 0,
            // clock: 0,
            i: 0,
            enabled: false,
            sample: 0,
            registers: AudioRegisters::default(),
            dac_enabled: false,
            length_counter: 0,
            volume_code: 0,
            length_enabled: false,
            counter: 0,
        }
    }

    pub fn dac(&self) -> f32 {
        let enabled = (self.enabled && self.dac_enabled) as u8;

        let out = match self.volume_code {
            0 => (enabled * (self.sample >> 4)) as f32,
            1 => (enabled * (self.sample >> 0)) as f32,
            2 => (enabled * (self.sample >> 1)) as f32,
            3 => (enabled * (self.sample >> 2)) as f32,
            _ => panic!("Invalid volume code."),
        };

        out / 15.0 * 2.0 - 1.0
    }

    pub fn tick(&mut self, cycles: usize) {
        if !self.enabled {
            return;
        }

        if self.counter <= cycles {
            let delta = cycles - self.counter;

            self.counter = ((2048 - self.freq) * 2) as usize;
            self.counter -= delta;

            self.i = (self.i + 1) % 32;

            self.sample = self.table[self.i];
        } else {
            self.counter -= cycles;
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

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF1A => 0x7F | self.registers.nrx0,
            0xFF1B => 0xFF,
            0xFF1C => 0x9F | self.registers.nrx2,
            0xFF1D => 0xFF,
            0xFF1E => 0xBF | self.registers.nrx4,
            0xFF30..=0xFF3F => {
                if self.enabled {
                    self.wave_ram[self.i / 2]
                } else {
                    let offset = (addr - 0xFF30) as usize;
                    self.wave_ram[offset]
                }
            }
            _ => 0xFF,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => {
                self.registers.nrx0 = value;

                let old_dac_enabled = self.dac_enabled;
                self.dac_enabled = (value & 0x80) != 0;
                if old_dac_enabled && !self.dac_enabled {
                    self.enabled = false;
                }
            }
            0xFF1B => {
                self.registers.nrx1 = value;
                self.length_counter = 256 - value as usize;
            }
            0xFF1C => {
                self.registers.nrx2 = value;
                self.volume_code = (value & 0x60) >> 5;
            }
            0xFF1D => {
                self.registers.nrx3 = value;
                self.freq = (self.freq & 0x700) | value as u16;
            }
            0xFF30..=0xFF3F => {
                if self.enabled {
                    self.wave_ram[self.i / 2] = value;
                } else {
                    let offset = (addr - 0xFF30) as usize;

                    self.table[offset * 2] = value >> 4;
                    self.table[offset * 2 + 1] = value & 0x0F;

                    self.wave_ram[offset] = value;
                }
            }
            _ => (),
        }
    }

    pub fn set_nrx4(&mut self, value: u8, counter_wont_clock: bool) {
        self.registers.nrx4 = value;

        self.freq = (self.freq & 0xFF) | ((value as u16 & 0x7) << 8);

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
                    self.length_counter = 255;
                } else {
                    self.enabled = false;
                }
            }
        }

        self.length_enabled = (value & 0x40) != 0;

        if self.enabled && !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn trigger(&mut self) {
        self.enabled = true;

        // self.period = ((2048 - self.freq) * 2) as usize;

        if self.length_counter == 0 {
            self.length_counter = 256;
            self.length_enabled = false;
        }

        // let high_nibble = self.i - (self.i % 2);
        // self.sample = self.table[high_nibble];

        self.i = 0;
        // self.clock = 0;
        self.counter = ((2048 - self.freq) * 2) as usize;
        self.counter += 6;

        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    pub fn clear_registers(&mut self) {
        self.registers = AudioRegisters::default();
    }
}

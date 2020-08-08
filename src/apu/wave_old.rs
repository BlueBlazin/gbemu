use crate::apu::AudioRegisters;

pub struct WaveChannel {
    pub table: [u8; 32],
    pub freq: u16,
    pub period: usize,
    pub clock: usize,
    pub i: usize,
    pub enabled: bool,
    registers: AudioRegisters,
    dac_enabled: bool,
    length_counter: usize,
    volume_code: u8,
    length_enabled: bool,
    pub start_clocking: bool,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            table: [
                0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
                0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF, 0x0, 0x0, 0xF, 0xF,
            ],
            freq: 0,
            period: 0,
            clock: 0,
            i: 0,
            enabled: false,
            registers: AudioRegisters::default(),
            dac_enabled: false,
            length_counter: 0,
            volume_code: 0,
            length_enabled: false,
            start_clocking: false,
        }
    }

    pub fn dac(&self) -> f32 {
        let enabled = self.enabled as u8;

        let out = match self.volume_code {
            0 => (enabled * (self.table[self.i] >> 4)) as f32,
            1 => (enabled * self.table[self.i]) as f32,
            2 => (enabled * (self.table[self.i] >> 1)) as f32,
            3 => (enabled * (self.table[self.i] >> 2)) as f32,
            _ => panic!("Invalid volume code."),
        };

        out / 15.0 * 2.0 - 1.0
    }

    pub fn tick(&mut self, cycles: usize) {
        if !self.enabled || !self.start_clocking {
            return;
        }

        self.clock += cycles;

        if self.clock >= self.period {
            self.clock -= self.period;
            self.i = (self.i + 1) % 32;
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
                    (self.table[self.i] << 4) | self.table[self.i + 1]
                } else {
                    let offset = (addr - 0xFF30) as usize * 2;
                    (self.table[offset] << 4) | self.table[offset + 1]
                }
            }
            _ => 0x00,
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
                let offset = (addr - 0xFF30) as usize * 2;
                self.table[offset] = value >> 4;
                self.table[offset + 1] = value & 0x0F;
            }
            _ => (),
        }
    }

    pub fn set_nrx4(&mut self, value: u8, counter_wont_clock: bool) {
        self.registers.nrx4 = value;

        self.freq = (value as u16 & 0x7) << 8 | self.freq;

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
        self.start_clocking = true;
        self.enabled = self.dac_enabled;

        self.period = ((2048 - self.freq) * 2) as usize;

        if self.length_counter == 0 {
            self.length_counter = 256;
            self.length_enabled = false;
        }

        self.i = 0;
        self.clock = 0;
    }

    pub fn clear_registers(&mut self) {
        // self.registers = AudioRegisters {
        //     nrx1: 0xFF,
        //     ..AudioRegisters::default()
        // };
        self.registers = AudioRegisters::default();
    }
}

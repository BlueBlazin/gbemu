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
    length_load: usize,
    length_counter: usize,
    volume_code: u8,
    length_enabled: bool,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            table: [0; 32],
            freq: 0,
            period: 0,
            clock: 0,
            i: 0,
            enabled: false,
            registers: AudioRegisters::default(),
            dac_enabled: false,
            length_load: 0,
            length_counter: 0,
            volume_code: 0,
            length_enabled: false,
        }
    }

    pub fn dac(&self) -> f32 {
        let out = match self.volume_code {
            0 => (self.enabled as u8 * (self.table[self.i] >> 4)) as f32,
            1 => (self.enabled as u8 * self.table[self.i]) as f32,
            2 => (self.enabled as u8 * (self.table[self.i] >> 1)) as f32,
            3 => (self.enabled as u8 * (self.table[self.i] >> 2)) as f32,
            _ => panic!("Invalid volume code."),
        };

        out / 15.0 * 2.0 - 1.0
    }

    pub fn tick(&mut self, cycles: usize) {
        if !self.enabled {
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
                self.length_enabled = false;
                self.registers.nrx4 &= !0x40;
            }
        }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF1A => self.registers.nrx0,
            0xFF1B => self.registers.nrx1,
            0xFF1C => self.registers.nrx2,
            0xFF1D => self.registers.nrx3,
            0xFF1E => self.registers.nrx4,
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF1A => {
                self.registers.nrx0 = value;
                self.dac_enabled = (value & 0x80) != 0;
            }
            0xFF1B => {
                self.registers.nrx1 = value;
                self.length_load = 256 - value as usize;
            }
            0xFF1C => {
                self.registers.nrx2 = value;
                self.volume_code = (value & 0x60) >> 5;
            }
            0xFF1D => {
                self.registers.nrx3 = value;
                self.freq = (self.freq & 0x700) | value as u16;
            }
            0xFF1E => {
                self.registers.nrx4 = value;
                self.length_enabled = (value & 0x40) != 0;
                self.freq = (self.freq & 0xFF) | (((value & 0x07) as u16) << 8);
                self.period = (2048 - self.freq as usize) * 2;
                if (value & 0x80) != 0 {
                    self.restart();
                }
            }
            0xFF30..=0xFF3F => {
                let offset = (addr - 0xFF30) as usize * 2;
                self.table[offset] = value >> 4;
                self.table[offset + 1] = value & 0x0F;
            }
            _ => (),
        }
    }

    pub fn restart(&mut self) {
        self.enabled = true;
        if self.length_counter == 0 {
            self.length_counter = 256;
        }
        self.i = 0;
        self.clock = 0;
        self.length_counter = self.length_load;
    }
}

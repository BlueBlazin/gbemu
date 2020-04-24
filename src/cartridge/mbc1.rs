use crate::cartridge::Mbc;

const RAM_OFFSET: usize = 0xA000;
const ROM_OFFSET: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

enum Mode {
    RomBanking,
    RamBanking,
}

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_enabled: bool,
    mode: Mode,
}

impl Mbc1 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_size = match data[0x0149] {
            1 => 0x800,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            _ => 0,
        };

        Mbc1 {
            rom: data,
            ram: vec![0; ram_size],
            rom_bank: 0,
            ram_bank: 0,
            ram_enabled: false,
            mode: Mode::RomBanking,
        }
    }
}

impl Mbc for Mbc1 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                println!("rom bank: {}, addr: {:#X}", self.rom_bank, addr);
                let addr = self.rom_bank as usize * ROM_BANK_SIZE + (addr as usize - ROM_OFFSET);
                self.rom[addr]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                let addr = RAM_OFFSET + self.ram_bank as usize * RAM_BANK_SIZE;
                self.ram[addr]
            }
            _ => panic!("Address out of bounds."),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0A) != 0;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = match value & 0x1F {
                    0x00 => (self.rom_bank & 0x60) | 0x01,
                    low => (self.rom_bank & 0x60) | low,
                };
            }
            0x4000..=0x5FFF => match self.mode {
                Mode::RomBanking => {
                    self.rom_bank = match value & 0x03 {
                        0x00 => (self.rom_bank & 0x1F) | 0x01 << 5,
                        hi => (self.rom_bank & 0x1F) | hi << 5,
                    };
                }
                Mode::RamBanking => {
                    self.ram_bank = value & 0x03;
                }
            },
            0x6000..=0x7FFF => {
                self.mode = match value & 0x3 {
                    0x00 => Mode::RomBanking,
                    _ => Mode::RamBanking,
                };
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let addr = RAM_OFFSET as usize + self.ram_bank as usize * RAM_BANK_SIZE;
                    self.ram[addr] = value;
                }
            }
            _ => (),
        }
    }
}

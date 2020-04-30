use crate::cartridge::Mbc;

const RAM_OFFSET: usize = 0xA000;
const ROM_OFFSET: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

pub struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    ram_enabled: bool,
}

impl Mbc5 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_size = match data[0x0149] {
            1 => 0x800,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            5 => 0x10000,
            _ => 0,
        };

        Mbc5 {
            rom: data,
            ram: vec![0; ram_size],
            rom_bank: 0,
            ram_bank: 0,
            ram_enabled: false,
        }
    }
}

impl Mbc for Mbc5 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                let addr = self.rom_bank as usize * ROM_BANK_SIZE + (addr as usize - ROM_OFFSET);
                self.rom[addr]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0xFF;
                }
                let addr = self.ram_bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
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
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | value as u16;
            }
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0xFF) | (value as u16 & 0x1);
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
            }
            _ => panic!("Address out of bounds."),
        }
    }
}

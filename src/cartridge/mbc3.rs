use crate::cartridge::Mbc;

const RAM_OFFSET: usize = 0xA000;
const ROM_OFFSET: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_or_rtc_enabled: bool,
}

impl Mbc3 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_size = match data[0x0149] {
            1 => 0x800,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            5 => 0x10000,
            _ => 0,
        };

        Mbc3 {
            rom: data,
            ram: vec![0; ram_size],
            rom_bank: 0,
            ram_bank: 0,
            ram_or_rtc_enabled: false,
        }
    }
}

impl Mbc for Mbc3 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => {
                let addr = self.rom_bank as usize * ROM_BANK_SIZE + (addr as usize - ROM_OFFSET);
                self.rom[addr]
            }
            0xA000..=0xBFFF => {
                if !self.ram_or_rtc_enabled {
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
                self.ram_or_rtc_enabled = (value & 0x0A) != 0;
            }
            0x2000..=0x3FFF => {
                self.rom_bank = match value & 0x1F {
                    0x01..=0x07 => value & 0x1F,
                    _ => 0x00,
                };
            }
            0x4000..=0x5FFF => match value {
                0x00..=0x03 => self.ram_bank = value,
                0x08..=0x0C => (),
                _ => (),
            },
            0x6000..=0x7FFF => (),
            0xA000..=0xBFFF => {
                if self.ram_or_rtc_enabled {
                    let addr =
                        self.ram_bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
                    self.ram[addr] = value;
                }
            }
            _ => panic!("Address out of bounds {:#X}.", addr),
        }
    }
}

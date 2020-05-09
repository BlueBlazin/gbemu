use crate::cartridge::Mbc;

const RAM_SIZE: usize = 0x2000;
const RAM_OFFSET: usize = 0xA000;

pub struct Mbc0 {
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl Mbc0 {
    pub fn new(rom: Vec<u8>) -> Self {
        Mbc0 {
            rom,
            ram: vec![0; RAM_SIZE],
        }
    }
}

impl Mbc for Mbc0 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            0xA000..=0xBFFF => self.ram[addr as usize - RAM_OFFSET],
            _ => panic!(format!("Address {:#X} out of bounds.", addr)),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => (),
            0xA000..=0xBFFF => self.ram[addr as usize - RAM_OFFSET] = value,
            _ => panic!(format!("Address {:#X} out of bounds.", addr)),
        }
    }
}

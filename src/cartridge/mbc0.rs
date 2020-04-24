use crate::cartridge::Mbc;

pub struct Mbc0 {
    rom: Vec<u8>,
}

impl Mbc0 {
    pub fn new(rom: Vec<u8>) -> Self {
        Mbc0 { rom }
    }
}

impl Mbc for Mbc0 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[addr as usize],
            _ => panic!(format!("Address {:#X} out of bounds.", addr)),
        }
    }

    fn set_byte(&mut self, addr: u16, _: u8) {
        match addr {
            0x0000..=0x7FFF => (),
            _ => panic!(format!("Address {:#X} out of bounds.", addr)),
        }
    }
}

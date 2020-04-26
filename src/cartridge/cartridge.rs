use crate::cartridge::mbc0::Mbc0;
use crate::cartridge::mbc1::Mbc1;
use crate::cartridge::Mbc;

pub struct Cartridge {
    mbc: Box<dyn Mbc>,
}

impl Cartridge {
    pub fn new(data: Vec<u8>) -> Self {
        let mbc: Box<dyn Mbc> = match data[0x0147] {
            0x00 => Box::from(Mbc0::new(data)),
            0x01..=0x03 => Box::from(Mbc1::new(data)),
            n => panic!("Unsupported MBC type. Code {:#X}", n),
        };

        Self { mbc }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        self.mbc.get_byte(addr)
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        self.mbc.set_byte(addr, value);
    }
}

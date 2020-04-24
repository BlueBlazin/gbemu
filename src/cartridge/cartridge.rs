use crate::cartridge::mbc0::Mbc0;
use crate::cartridge::mbc1::Mbc1;
use crate::cartridge::Mbc;

pub struct Cartridge {
    mbc: Option<Box<dyn Mbc>>,
}

impl Cartridge {
    pub fn new() -> Self {
        Self { mbc: None }
    }

    pub fn load(&mut self, data: Vec<u8>) {
        self.mbc = match data[0x0147] {
            0x00 => Some(Box::from(Mbc0::new(data))),
            0x01..=0x02 => Some(Box::from(Mbc1::new(data))),
            _ => panic!("Unsupported MBC type"),
        };
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match &mut self.mbc {
            Some(mbc) => mbc.get_byte(addr),
            None => panic!("Uninitialized Catridge"),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match &mut self.mbc {
            Some(mbc) => mbc.set_byte(addr, value),
            None => panic!("Uninitialized Catridge"),
        }
    }
}

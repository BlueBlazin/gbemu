pub mod cartridge;
pub mod mbc0;
pub mod mbc1;

pub trait Mbc {
    fn get_byte(&mut self, addr: u16) -> u8;
    fn set_byte(&mut self, addr: u16, value: u8);
}

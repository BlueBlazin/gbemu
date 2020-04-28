const WRAM_BANK_SIZE: usize = 0x1000;
const WRAM_OFFSET: usize = 0xC000;
// const WRAM_SIZE: usize = 0x8000;

pub struct Wram {
    wram: Vec<u8>,
    bank: usize,
}

impl Wram {
    pub fn new() -> Self {
        Self {
            wram: vec![0; WRAM_BANK_SIZE * 8],
            bank: 1,
        }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xC000..=0xCFFF => self.wram[addr as usize - WRAM_OFFSET],
            0xD000..=0xDFFF => {
                let addr = self.bank * WRAM_BANK_SIZE + (addr as usize - WRAM_OFFSET);
                self.wram[addr]
            }
            0xFF70 => self.bank as u8,
            _ => panic!("Invalid WRAM address {:#X}", addr),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xC000..=0xCFFF => self.wram[addr as usize - WRAM_OFFSET] = value,
            0xD000..=0xDFFF => {
                let addr = self.bank * WRAM_BANK_SIZE + (addr as usize - WRAM_OFFSET);
                self.wram[addr] = value;
            }
            0xFF70 => {
                self.bank = match value {
                    0x00..=0x07 => (value & 0x03) as usize,
                    _ => 0x01,
                };
            }
            _ => panic!("Invalid WRAM address {:#X}", addr),
        }
    }
}

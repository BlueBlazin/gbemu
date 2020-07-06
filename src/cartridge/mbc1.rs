use crate::cartridge::Mbc;

const RAM_OFFSET: usize = 0xA000;
const ROM_OFFSET: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

#[derive(PartialEq)]
enum Mode {
    // RomBanking,
    // RamBanking,
    Mode0,
    Mode1,
}

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    // rom_bank: u8,
    // ram_bank: u8,
    ram_enabled: bool,
    mode: Mode,

    bank1: u8,
    bank2: u8,
}

impl Mbc1 {
    pub fn new(data: Vec<u8>) -> Self {
        let ram_size = match data[0x0149] {
            1 => 0x800,
            2 => 0x2000,
            3 => 0x8000,
            4 => 0x20000,
            _ => 0x800,
        };

        Mbc1 {
            rom: data,
            ram: vec![0; ram_size],
            // rom_bank: 1,
            // ram_bank: 0,
            ram_enabled: false,
            mode: Mode::Mode0,

            bank1: 1,
            bank2: 0,
        }
    }
}

impl Mbc for Mbc1 {
    fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => match self.mode {
                Mode::Mode0 => self.rom[addr as usize],
                Mode::Mode1 => {
                    let addr =
                        (self.bank2 << 5) as usize * ROM_BANK_SIZE + (addr as usize - ROM_OFFSET);
                    self.rom[addr]
                }
            },
            0x4000..=0x7FFF => {
                // let bank = match self.mode {
                //     Mode::RomBanking => self.rom_bank & 0x7F,
                //     Mode::RamBanking => self.rom_bank & 0x1F,
                // };
                let bank = self.bank2 << 5 | self.bank1;
                let addr = bank as usize * ROM_BANK_SIZE + (addr as usize - ROM_OFFSET);
                self.rom[addr]
            }
            0xA000..=0xBFFF => {
                if !self.ram_enabled {
                    return 0x00;
                }

                // let bank = match self.mode {
                //     Mode::RomBanking => 0x0,
                //     Mode::RamBanking => self.ram_bank,
                // };
                let bank = match self.mode {
                    Mode::Mode0 => 0x0,
                    Mode::Mode1 => self.bank2,
                };
                let addr = bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
                self.ram[addr]
            }
            _ => panic!("Address out of bounds."),
        }
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                // self.rom_bank = match value & 0x1F {
                //     0x0 => (self.rom_bank & 0x60) | 0x1,
                //     low => (self.rom_bank & 0x60) | low,
                // };

                self.bank1 = match value & 0x1F {
                    0 => 1,
                    n => n,
                };
            }
            0x4000..=0x5FFF => {
                // match self.mode {
                //     Mode::RomBanking => {
                //         self.rom_bank = (self.rom_bank & 0x9F) | ((value & 0x03) << 5);
                //     }
                //     Mode::RamBanking => {
                //         self.ram_bank = value & 0x03;
                //     }
                // }
                self.bank2 = value & 0x3;
            }
            0x6000..=0x7FFF => {
                // self.mode = match value & 0x3 {
                //     0x00 => Mode::RomBanking,
                //     _ => Mode::RamBanking,
                // };
                self.mode = match value & 0x1 {
                    0 => Mode::Mode0,
                    _ => Mode::Mode1,
                };
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank = match self.mode {
                        Mode::Mode0 => 0x0,
                        Mode::Mode1 => self.bank2,
                    };
                    let addr = bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
                    // let addr = self.ram_bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
                    self.ram[addr] = value;
                }
            }
            _ => panic!("Address out of bounds."),
        }
    }
}

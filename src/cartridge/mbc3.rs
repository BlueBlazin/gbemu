use crate::cartridge::Mbc;
use std::time;

const RAM_OFFSET: usize = 0xA000;
const ROM_OFFSET: usize = 0x4000;
const RAM_BANK_SIZE: usize = 0x2000;
const ROM_BANK_SIZE: usize = 0x4000;

enum Mode {
    Ram,
    Rtc,
}

// 08h  RTC S   Seconds   0-59 (0-3Bh)
// 09h  RTC M   Minutes   0-59 (0-3Bh)
// 0Ah  RTC H   Hours     0-23 (0-17h)
// 0Bh  RTC DL  Lower 8 bits of Day Counter (0-FFh)
// 0Ch  RTC DH  Upper 1 bit of Day Counter, Carry Bit, Halt Flag
//       Bit 0  Most significant bit of Day Counter (Bit 8)
//       Bit 6  Halt (0=Active, 1=Stop Timer)
//       Bit 7  Day Counter Carry Bit (1=Counter Overflow)
struct Rtc {
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_counter_lo: u8,
    day_counter_hi: u8,
    t0: u64,
}

impl Rtc {
    pub fn new() -> Self {
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            day_counter_hi: 0,
            day_counter_lo: 0,
            t0: 0,
        }
    }

    pub fn get_byte(&self, addr: u8) -> u8 {
        match addr {
            0x08 => self.seconds,
            0x09 => self.minutes,
            0x0A => self.hours,
            0x0B => self.day_counter_hi,
            0x0C => self.day_counter_lo,
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u8, value: u8) {
        match addr {
            0x08 => self.seconds = value,
            0x09 => self.minutes = value,
            0x0A => self.hours = value,
            0x0B => self.day_counter_hi = value,
            0x0C => self.day_counter_lo = value,
            _ => (),
        }
    }

    pub fn latch_clock_data(&mut self) {
        // Adapted from https://github.com/mvdnes/rboy/blob/master/src/mbc/mbc3.rs
        if (self.day_counter_lo & 0x40) == 0 {
            return;
        }
        let t0 = time::UNIX_EPOCH + time::Duration::from_secs(self.t0);
        let difftime = match time::SystemTime::now().duration_since(t0) {
            Ok(n) => n.as_secs(),
            _ => 0,
        };
        self.seconds = (difftime % 60) as u8;
        self.minutes = ((difftime / 60) % 60) as u8;
        self.hours = ((difftime / 3600) % 24) as u8;
        let days = difftime / (3600 * 24);
        self.day_counter_hi = days as u8;
    }
}

pub struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u8,
    ram_bank: u8,
    ram_or_rtc_enabled: bool,
    latch_state0: bool,
    rtc_register: u8,
    mode: Mode,
    rtc: Rtc,
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
            rom_bank: 1,
            ram_bank: 0,
            ram_or_rtc_enabled: false,
            latch_state0: false,
            rtc_register: 0,
            mode: Mode::Ram,
            rtc: Rtc::new(),
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
                    return 0x00;
                }
                match self.mode {
                    Mode::Ram => {
                        let addr =
                            self.ram_bank as usize * RAM_BANK_SIZE + (addr as usize - RAM_OFFSET);
                        self.ram[addr]
                    }
                    Mode::Rtc => self.rtc.get_byte(self.rtc_register),
                }
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
                self.rom_bank = match value & 0x7F {
                    0x01..=0x7F => value & 0x7F,
                    _ => 0x01,
                };
            }
            0x4000..=0x5FFF => match value {
                0x00..=0x03 => {
                    self.mode = Mode::Ram;
                    self.ram_bank = value;
                }
                0x08..=0x0C => {
                    self.mode = Mode::Rtc;
                    self.rtc_register = value;
                }
                _ => (),
            },
            0x6000..=0x7FFF => match value & 0x01 {
                0x01 if self.latch_state0 => {
                    self.latch_state0 = false;
                    self.rtc.latch_clock_data();
                }
                0x00 if !self.latch_state0 => {
                    self.latch_state0 = true;
                }
                _ => self.latch_state0 = false,
            },
            0xA000..=0xBFFF => {
                if self.ram_or_rtc_enabled {
                    match self.mode {
                        Mode::Ram => {
                            let addr = self.ram_bank as usize * RAM_BANK_SIZE
                                + (addr as usize - RAM_OFFSET);
                            self.ram[addr] = value;
                        }
                        Mode::Rtc => {
                            self.rtc.set_byte(self.rtc_register, value);
                        }
                    }
                }
            }
            _ => panic!("Address out of bounds {:#X}.", addr),
        }
    }
}

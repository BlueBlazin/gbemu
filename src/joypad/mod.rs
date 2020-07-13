pub enum Key {
    Up,
    Down,
    Left,
    Right,
    BtnA,
    BtnB,
    Start,
    Select,
}

pub struct Joypad {
    pub request_joypad_int: bool,
    joyp: u8,
    keys: [u8; 8],
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            request_joypad_int: false,
            joyp: 0xCF,
            keys: [1; 8],
        }
    }

    pub fn get_byte(&mut self, addr: u16) -> u8 {
        match addr {
            0xFF00 => 0xC0 | self.joyp,
            _ => unreachable!(),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF00 => {
                if self.joyp & 0x30 != value & 0x30 {
                    self.joyp = value & 0xF0 | self.joyp & 0x0F;
                    self.update_joyp();
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn press_key(&mut self, key: Key) {
        // Bit 5 - P15 Select Button Keys (0=Select)
        // Bit 4 - P14 Select Direction Keys (0=Select)
        // Bit 3 - P13 Input Down or Start (0=Pressed) (Read Only)
        // Bit 2 - P12 Input Up or Select (0=Pressed) (Read Only)
        // Bit 1 - P11 Input Left or Button B (0=Pressed) (Read Only)
        // Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)

        match key {
            Key::BtnA => self.keys[0] = 0,
            Key::BtnB => self.keys[1] = 0,
            Key::Select => self.keys[2] = 0,
            Key::Start => self.keys[3] = 0,

            Key::Right => self.keys[4] = 0,
            Key::Left => self.keys[5] = 0,
            Key::Up => self.keys[6] = 0,
            Key::Down => self.keys[7] = 0,
        };

        self.update_joyp();
    }

    pub fn release_key(&mut self, key: Key) {
        // Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
        // Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
        // Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
        // Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
        match key {
            Key::BtnA => self.keys[0] = 1,
            Key::BtnB => self.keys[1] = 1,
            Key::Select => self.keys[2] = 1,
            Key::Start => self.keys[3] = 1,

            Key::Right => self.keys[4] = 1,
            Key::Left => self.keys[5] = 1,
            Key::Up => self.keys[6] = 1,
            Key::Down => self.keys[7] = 1,
        };

        self.update_joyp();
    }

    fn update_joyp(&mut self) {
        let old_signal = self.joyp & 0xF;
        let select = (self.joyp >> 0x4) & 0x3;
        self.joyp &= 0xF0;

        match select {
            // Both
            0 => {
                for i in 0..4 {
                    self.joyp |= (self.keys[i] | self.keys[i + 4]) << i;
                }
            }
            // Button
            1 => {
                for i in 0..4 {
                    self.joyp |= self.keys[i] << i;
                }
            }
            // Direction
            2 => {
                for i in 0..4 {
                    self.joyp |= self.keys[i + 4] << i;
                }
            }
            _ => (),
        }

        if old_signal != self.joyp & 0xF {
            self.request_joypad_int = true;
        }
    }
}

// pub struct Joypad {
//     pub request_joypad_int: bool,
//     joyp: u8,
//     btn_keys: u8,
//     dir_keys: u8,
// }

// impl Joypad {
//     pub fn new() -> Self {
//         Joypad {
//             request_joypad_int: false,
//             joyp: 0xF0,
//             btn_keys: 0x0F,
//             dir_keys: 0x0F,
//         }
//     }

//     pub fn get_byte(&mut self, addr: u16) -> u8 {
//         match addr {
//             0xFF00 => self.joyp(),
//             _ => unreachable!(),
//         }
//     }

//     pub fn set_byte(&mut self, addr: u16, value: u8) {
//         match addr {
//             0xFF00 => {
//                 let old_signal = self.joyp();

//                 self.joyp = value & 0x30;

//                 // if old_signal & !self.joyp() != 0 {
//                 //     self.request_joypad_int = true;
//                 // }
//             }
//             _ => unreachable!(),
//         }
//     }

//     pub fn press_key(&mut self, key: Key) {
//         let old_signal = self.joyp();
//         // Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
//         // Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
//         // Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
//         // Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
//         match key {
//             Key::BtnA => self.btn_keys = self.btn_keys & 0x0E,
//             Key::BtnB => self.btn_keys = self.btn_keys & 0x0D,
//             Key::Select => self.btn_keys = self.btn_keys & 0x0B,
//             Key::Start => self.btn_keys = self.btn_keys & 0x07,
//             Key::Right => self.dir_keys = self.dir_keys & 0x0E,
//             Key::Left => self.dir_keys = self.dir_keys & 0x0D,
//             Key::Up => self.dir_keys = self.dir_keys & 0x0B,
//             Key::Down => self.dir_keys = self.dir_keys & 0x07,
//         }

//         // if old_signal & !self.joyp() != 0 {
//         //     self.request_joypad_int = true;
//         // }
//     }

//     pub fn release_key(&mut self, key: Key) {
//         let old_signal = self.joyp();
//         // Bit 3 - P13 Input Down  or Start    (0=Pressed) (Read Only)
//         // Bit 2 - P12 Input Up    or Select   (0=Pressed) (Read Only)
//         // Bit 1 - P11 Input Left  or Button B (0=Pressed) (Read Only)
//         // Bit 0 - P10 Input Right or Button A (0=Pressed) (Read Only)
//         match key {
//             Key::BtnA => self.btn_keys = self.btn_keys | 0x01,
//             Key::BtnB => self.btn_keys = self.btn_keys | 0x02,
//             Key::Select => self.btn_keys = self.btn_keys | 0x04,
//             Key::Start => self.btn_keys = self.btn_keys | 0x08,
//             Key::Right => self.dir_keys = self.dir_keys | 0x01,
//             Key::Left => self.dir_keys = self.dir_keys | 0x02,
//             Key::Up => self.dir_keys = self.dir_keys | 0x04,
//             Key::Down => self.dir_keys = self.dir_keys | 0x08,
//         }

//         // if old_signal & !self.joyp() != 0 {
//         //     self.request_joypad_int = true;
//         // }
//     }

//     fn joyp(&self) -> u8 {
//         if (self.joyp & 0x10) == 0 {
//             self.dir_keys
//         } else if (self.joyp & 0x20) == 0 {
//             self.btn_keys
//         } else {
//             0xCF
//         }
//     }
// }

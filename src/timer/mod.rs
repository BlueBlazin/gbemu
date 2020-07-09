use crate::cpu::EmulationMode;

const COUNTER_SHIFT: [u16; 4] = [9, 3, 5, 7];

pub struct Divider {
    pub counter: u16,
}

impl Divider {
    pub fn new(mode: EmulationMode) -> Self {
        Self {
            counter: match mode {
                EmulationMode::Dmg => 0x267C,
                EmulationMode::Cgb => 0x1EA0,
            },
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.counter = self.counter.wrapping_add(cycles as u16);
    }

    pub fn get_byte(&self) -> u8 {
        (self.counter >> 8) as u8
    }

    pub fn set_byte(&mut self) {
        self.counter = 0;
    }
}

#[derive(Debug, PartialEq)]
enum TimerState {
    Reloading,
    Reloaded,
    Running,
}

pub struct Timer {
    pub acc: u8,          // TIMA
    pub tma: u8,          // TMA
    pub timer_enable: u8, // TMC
    pub freq: u8,         // TMC
    pub divider: Divider,
    pub request_timer_int: bool,
    tima_bit: u16,
    state: TimerState,
    clock: usize,
}

impl Timer {
    pub fn new(mode: EmulationMode) -> Self {
        Self {
            acc: 0,
            tma: 0,
            timer_enable: 0,
            freq: 0,
            divider: Divider::new(mode),
            request_timer_int: false,
            tima_bit: 9,
            state: TimerState::Running,
            clock: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        for _ in 0..cycles {
            self.clock += 1;
            let old_signal = self.signal();
            self.divider.tick(1);
            // self.advance_state(old_signal);
            if self.clock >= 4 {
                self.clock -= 4;
                self.advance_state();
            }
            self.detect_falling_edge(old_signal)
        }
    }

    fn advance_state(&mut self) {
        match self.state {
            TimerState::Reloading => {
                self.acc = self.tma;
                self.request_timer_int = true;
                self.state = TimerState::Reloaded;
            }
            TimerState::Reloaded => {
                self.state = TimerState::Running;
            }
            TimerState::Running => (),
        }
    }

    fn detect_falling_edge(&mut self, old_signal: u8) {
        let new_signal = self.signal();

        if old_signal != 0 && new_signal == 0 {
            self.acc = self.acc.wrapping_add(1);

            if self.acc == 0 {
                self.state = TimerState::Reloading;
            }
        }
    }

    #[inline]
    fn signal(&self) -> u8 {
        ((self.timer_enable & 4) >> 2) & (self.divider.counter >> self.tima_bit) as u8
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.divider.get_byte(),
            0xFF05 => {
                if self.state == TimerState::Reloading {
                    0
                } else {
                    self.acc
                }
            }
            0xFF06 => self.tma,
            0xFF07 => self.timer_enable | self.freq,
            _ => 0x00,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => {
                let old_signal = self.signal();
                self.divider.set_byte();
                self.detect_falling_edge(old_signal);
            }
            0xFF05 if self.state != TimerState::Reloaded => {
                self.acc = value;
            }
            0xFF06 => {
                self.tma = value;
                // if self.timer_enable != 0 && self.state == TimerState::Reloaded {
                //     self.acc = value;
                //     self.state = TimerState::Running;
                // }
                if self.state != TimerState::Running {
                    self.acc = value;
                }
            }
            0xFF07 => {
                self.timer_enable = value & 0x04;
                self.freq = value & 0x03;
                self.tima_bit = COUNTER_SHIFT[self.freq as usize];
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIV: u16 = 0xFF04;
    const TIMA: u16 = 0xFF05;
    const TMA: u16 = 0xFF06;
    const TAC: u16 = 0xFF07;

    #[test]
    fn test_div_trigger() {
        let mut timer = Timer::new(EmulationMode::Dmg);

        let mut a = 0;
        let b = 4;
        timer.set_byte(DIV, a);
        a = b;
        timer.set_byte(TIMA, a);
        timer.set_byte(TMA, a);
        a = 0b00000100;
        timer.set_byte(TAC, a);
        a ^= a;
        timer.set_byte(DIV, a);

        timer.tick(512);
        println!("{}", timer.get_byte(TIMA));

        timer.set_byte(DIV, 0);

        println!("{}", timer.get_byte(TIMA));
    }

    #[test]
    fn test_timer() {
        let mut timer = Timer::new(EmulationMode::Dmg);

        let mut a = 0;
        let b = 4;
        timer.set_byte(DIV, a);
        a = b;
        timer.set_byte(TIMA, a);
        timer.set_byte(TMA, a);
        a = 0b00000100;
        timer.set_byte(TAC, a);
        a ^= a;
        timer.set_byte(DIV, a);
        a = b;
        timer.set_byte(TIMA, a);
        a ^= a;
        timer.set_byte(DIV, a);
        timer.tick(252 * 4);
        a = timer.get_byte(TIMA);
        let d = a;
        println!("D: {}", d);

        a = b;
        timer.set_byte(TIMA, a);
        a ^= a;
        timer.set_byte(DIV, a);
        a = b;
        timer.set_byte(TIMA, a);
        a ^= a;
        timer.set_byte(DIV, a);
        timer.tick(253 * 4);
        a = timer.get_byte(TIMA);
        let e = a;
        println!("E: {}", e);
    }
}

// use crate::cpu::EmulationMode;

// const COUNTER_SHIFT: [u16; 4] = [9, 3, 5, 7];

// pub struct Divider {
//     pub counter: u16,
// }

// impl Divider {
//     pub fn new(mode: EmulationMode) -> Self {
//         Self {
//             counter: match mode {
//                 EmulationMode::Dmg => 0x267C,
//                 EmulationMode::Cgb => 0x1EA0,
//             },
//         }
//     }

//     pub fn tick(&mut self, cycles: usize) {
//         self.counter = self.counter.wrapping_add(cycles as u16);
//     }

//     pub fn get_byte(&self) -> u8 {
//         (self.counter >> 8) as u8
//     }

//     pub fn set_byte(&mut self) {
//         self.counter = 0;
//     }
// }

// #[derive(Debug, PartialEq)]
// enum TimerState {
//     Reloading,
//     Reloaded,
//     Running,
// }

// pub struct Timer {
//     pub counter: u8,      // TIMA
//     pub tma: u8,          // TMA
//     pub timer_enable: u8, // TMC
//     pub freq: u8,         // TMC
//     pub divider: Divider,
//     pub request_timer_int: bool,
//     tima_bit: u16,
//     state: TimerState,
//     // state_counter: usize,
// }

// impl Timer {
//     pub fn new(mode: EmulationMode) -> Self {
//         Self {
//             counter: 0,
//             tma: 0,
//             timer_enable: 0,
//             freq: 0,
//             divider: Divider::new(mode),
//             request_timer_int: false,
//             tima_bit: 9,
//             state: TimerState::Running,
//             // state_counter: 0,
//         }
//     }

//     pub fn tick(&mut self, cycles: usize) {
//         for _ in 0..cycles {
//             let old_signal = self.signal();
//             self.divider.tick(1);
//             self.advance_state(old_signal);
//         }
//     }

//     fn advance_state(&mut self, old_signal: u8) {
//         match self.state {
//             TimerState::Reloading => {
//                 self.counter = self.tma;
//                 self.request_timer_int = true;
//                 self.state = TimerState::Reloaded;
//             }
//             TimerState::Reloaded => {
//                 self.state = TimerState::Running;
//             }
//             TimerState::Running => self.detect_falling_edge(old_signal),
//         }
//     }

//     fn detect_falling_edge(&mut self, old_signal: u8) {
//         let new_signal = self.signal();
//         if old_signal != 0 && new_signal == 0 {
//             self.counter = self.counter.wrapping_add(1);
//             if self.counter == 0 {
//                 self.state = TimerState::Reloading;
//                 // self.state_counter = 4;
//             }
//         }
//     }

//     #[inline]
//     fn signal(&self) -> u8 {
//         (self.timer_enable >> 2) & (self.divider.counter >> self.tima_bit) as u8
//     }

//     pub fn get_byte(&self, addr: u16) -> u8 {
//         match addr {
//             0xFF04 => self.divider.get_byte(),
//             0xFF05 => self.counter,
//             0xFF06 => self.tma,
//             0xFF07 => self.timer_enable | self.freq,
//             _ => 0x00,
//         }
//     }

//     pub fn set_byte(&mut self, addr: u16, value: u8) {
//         match addr {
//             0xFF04 => {
//                 let old_signal = self.signal();
//                 self.divider.set_byte();
//                 self.detect_falling_edge(old_signal);
//             }
//             0xFF05 if self.state == TimerState::Running => {
//                 self.counter = value;
//             }
//             0xFF06 => {
//                 self.tma = value;
//                 if self.timer_enable != 0 && self.state == TimerState::Reloaded {
//                     self.counter = value;
//                     self.state = TimerState::Running;
//                 }
//             }
//             0xFF07 => {
//                 self.timer_enable = value & 0x04;
//                 self.freq = value & 0x03;
//                 self.tima_bit = COUNTER_SHIFT[self.freq as usize];
//             }
//             _ => (),
//         }
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const DIV: u16 = 0xFF04;
//     const TIMA: u16 = 0xFF05;
//     const TMA: u16 = 0xFF06;
//     const TAC: u16 = 0xFF07;

//     #[test]
//     fn test_div_trigger() {
//         let mut timer = Timer::new(EmulationMode::Dmg);

//         let mut a = 0;
//         let b = 4;
//         timer.set_byte(DIV, a);
//         a = b;
//         timer.set_byte(TIMA, a);
//         timer.set_byte(TMA, a);
//         a = 0b00000100;
//         timer.set_byte(TAC, a);
//         a ^= a;
//         timer.set_byte(DIV, a);

//         timer.tick(512);
//         println!("{}", timer.get_byte(TIMA));

//         timer.set_byte(DIV, 0);

//         println!("{}", timer.get_byte(TIMA));
//     }

//     #[test]
//     fn test_timer() {
//         let mut timer = Timer::new(EmulationMode::Dmg);

//         let mut a = 0;
//         let b = 4;
//         timer.set_byte(DIV, a);
//         a = b;
//         timer.set_byte(TIMA, a);
//         timer.set_byte(TMA, a);
//         a = 0b00000100;
//         timer.set_byte(TAC, a);
//         a ^= a;
//         timer.set_byte(DIV, a);
//         a = b;
//         timer.set_byte(TIMA, a);
//         a ^= a;
//         timer.set_byte(DIV, a);
//         timer.tick(252 * 4);
//         a = timer.get_byte(TIMA);
//         let d = a;
//         println!("D: {}", d);

//         a = b;
//         timer.set_byte(TIMA, a);
//         a ^= a;
//         timer.set_byte(DIV, a);
//         a = b;
//         timer.set_byte(TIMA, a);
//         a ^= a;
//         timer.set_byte(DIV, a);
//         timer.tick(253 * 4);
//         a = timer.get_byte(TIMA);
//         let e = a;
//         println!("E: {}", e);
//     }
// }

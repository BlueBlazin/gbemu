pub struct Timer {
    pub request_timer_int: bool,
    pub tima: u8,
    pub tma: u8,
    pub timer_enable: u8,
    clock: usize,
    timer_threshold: usize,
    div_clock: usize,
    divider: u8,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            request_timer_int: false,
            tima: 0,
            tma: 0,
            timer_enable: 0,
            clock: 0,
            timer_threshold: 1024,
            div_clock: 0,
            divider: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        self.update_divider(cycles);
        self.update_timer(cycles);
    }

    pub fn update_timer(&mut self, cycles: usize) {
        if self.timer_enable != 0 {
            self.clock += cycles;

            while self.clock >= self.timer_threshold {
                self.clock = self.clock - self.timer_threshold;

                if self.tima == 0xFF {
                    self.tima = self.tma;
                    self.request_timer_interrupt();
                } else {
                    self.tima += 1;
                }
            }
        }
    }

    #[inline]
    fn request_timer_interrupt(&mut self) {
        self.request_timer_int = true;
    }

    fn update_divider(&mut self, cycles: usize) {
        self.div_clock += cycles;
        if self.div_clock >= 0xFF {
            self.div_clock = self.div_clock - 0xFF;
            self.divider = self.divider.wrapping_add(1);
        }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.divider,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => {
                let input_clock_sel = match self.timer_threshold {
                    1024 => 0x00,
                    16 => 0x01,
                    64 => 0x02,
                    256 => 0x03,
                    _ => unreachable!(),
                };
                self.timer_enable | input_clock_sel
            }
            _ => panic!("Unexpected address in timer"),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF04 => self.divider = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => {
                self.timer_enable = value & 0x04;
                self.timer_threshold = match value & 0x03 {
                    0x00 => 1024,
                    0x01 => 16,
                    0x02 => 64,
                    0x03 => 256,
                    _ => unreachable!(),
                };
            }
            _ => panic!("Unexpected address in timer"),
        }
    }
}

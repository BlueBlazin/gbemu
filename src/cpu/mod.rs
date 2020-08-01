pub mod opcodes;

use crate::events::Event;
use crate::joypad::Key;
use crate::memory::mmu::{HdmaType, Mmu};

const MAX_CYCLES: usize = 69905;

#[derive(Debug, PartialEq, Clone)]
pub enum EmulationMode {
    Dmg,
    Cgb,
}

#[derive(Debug, Clone)]
pub enum CgbSpeed {
    Normal,
    Double,
}

#[derive(Debug)]
pub struct CgbMode {
    pub speed: CgbSpeed,
    pub prepare_speed_switch: u8,
}

impl CgbMode {
    pub fn new() -> Self {
        Self {
            speed: CgbSpeed::Normal,
            prepare_speed_switch: 0,
        }
    }
}

impl From<&CgbMode> for u8 {
    fn from(value: &CgbMode) -> Self {
        match value.speed {
            CgbSpeed::Normal => value.prepare_speed_switch & 0x7F | 0x7E,
            CgbSpeed::Double => value.prepare_speed_switch & 0x7F | 0xFE,
        }
    }
}

/// The 8 bit registers.
pub enum R8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

/// The combined 16 bit registers.
pub enum R16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[allow(dead_code)]
pub enum Addr {
    AF,
    BC,
    DE,
    HL,
    SP,
}

#[derive(PartialEq)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

pub struct Cpu {
    r: [u8; 8],
    pub pc: u16,
    sp: u16,
    pub mmu: Mmu,
    pub cycles: usize,
    ime: bool,
    halted: bool,
    emu_mode: EmulationMode,
    stopped: bool,
    halt_bug: bool,
    ime_set_pending: bool,
    just_halted: bool,

    event_cycles: usize,
}

impl Cpu {
    pub fn new(data: Vec<u8>) -> Self {
        let emu_mode = if (data[0x0143] & 0x80) != 0 {
            EmulationMode::Cgb
        } else {
            EmulationMode::Dmg
        };

        Cpu {
            r: [0; 8],
            pc: 0,
            sp: 0,
            mmu: Mmu::new(data, emu_mode.clone()),
            cycles: 0,
            ime: true,
            halted: false,
            emu_mode,
            stopped: false,
            halt_bug: false,
            ime_set_pending: false,
            just_halted: false,
            event_cycles: 0,
        }
    }

    pub fn keydown(&mut self, key: usize) {
        match key {
            0 => self.mmu.joypad.press_key(Key::Right),
            1 => self.mmu.joypad.press_key(Key::Left),
            2 => self.mmu.joypad.press_key(Key::Up),
            3 => self.mmu.joypad.press_key(Key::Down),
            4 => self.mmu.joypad.press_key(Key::BtnA),
            5 => self.mmu.joypad.press_key(Key::BtnB),
            6 => self.mmu.joypad.press_key(Key::Select),
            7 => self.mmu.joypad.press_key(Key::Start),
            _ => panic!("Unknown key."),
        }
    }

    pub fn keyup(&mut self, key: usize) {
        match key {
            0 => self.mmu.joypad.release_key(Key::Right),
            1 => self.mmu.joypad.release_key(Key::Left),
            2 => self.mmu.joypad.release_key(Key::Up),
            3 => self.mmu.joypad.release_key(Key::Down),
            4 => self.mmu.joypad.release_key(Key::BtnA),
            5 => self.mmu.joypad.release_key(Key::BtnB),
            6 => self.mmu.joypad.release_key(Key::Select),
            7 => self.mmu.joypad.release_key(Key::Start),
            _ => panic!("Unknown key."),
        }
    }

    pub fn screen(&self) -> *const u8 {
        self.mmu.screen()
    }

    pub fn run_till_event(&mut self, max_cycles: usize) -> Event {
        let max_cycles = match self.mmu.cgb_mode.speed {
            CgbSpeed::Normal => max_cycles,
            CgbSpeed::Double => max_cycles * 2,
        };

        while self.event_cycles < max_cycles {
            if self.mmu.gpu.vblank_event {
                self.mmu.gpu.vblank_event = false;
                return Event::VBlank;
            }

            if let (Some(left), Some(right)) = self.mmu.apu.get_next_buffer() {
                return Event::AudioBufferFull(left, right);
            }

            self.event_cycles += self.tick();
        }

        self.event_cycles -= max_cycles;
        Event::MaxCycles
    }

    pub fn frame(&mut self) {
        let mut cycles = 0;
        while cycles < MAX_CYCLES {
            cycles += self.tick();
        }
    }

    pub fn tick(&mut self) -> usize {
        self.cycles = 0;

        if self.stopped {
            return self.stop_tick();
        }

        if self.halted {
            return self.halt_tick();
        }

        // match self.mmu.hdma.hdma_type {
        //     HdmaType::GPDma => self.gdma_tick(),
        //     HdmaType::HBlankDma if self.mmu.in_hblank() => self.hdma_tick(),
        //     _ => self.cpu_tick(),
        // }

        match self.mmu.hdma.hdma_type {
            HdmaType::GPDma => self.gdma_tick(),
            HdmaType::HBlankDma if self.mmu.gpu.hdma_flag => self.hdma_tick(),
            _ => self.cpu_tick(),
        }

        self.cycles
    }

    fn cpu_tick(&mut self) {
        self.just_halted = false;

        let ie = self.mmu.ie;
        let irr = self.mmu.get_byte(0xFF0F);

        let ints_pending = ie & irr & 0x1F;

        let ime = self.ime;
        if self.ime_set_pending {
            self.ime = !self.ime;
            self.ime_set_pending = false;
        }

        if ime && ints_pending != 0 {
            self.service_pending_interrupts();
            return;
        }

        let opcode = self.fetch();

        if self.halt_bug {
            self.halt_bug = false;
            self.pc -= 1;
        }

        self.decode_exec(opcode);
    }

    fn halt_tick(&mut self) -> usize {
        if self.emu_mode != EmulationMode::Cgb && !self.just_halted {
            self.add_cycles(2);
        }

        let irr = self.mmu.get_byte(0xFF0F);

        let ints_pending = self.mmu.ie & irr & 0x1F;

        self.add_cycles(if self.emu_mode == EmulationMode::Cgb || self.just_halted {
            4
        } else {
            2
        });

        self.just_halted = false;

        let ime = self.ime;
        if self.ime_set_pending {
            self.ime = !self.ime;
            self.ime_set_pending = false;
        }

        if ints_pending != 0 {
            if !ime {
                self.halted = false;
            } else {
                self.service_pending_interrupts();
            }
        }

        self.cycles
    }

    fn stop_tick(&mut self) -> usize {
        self.add_cycles(4);
        if self.mmu.get_byte(0xFF00) & 0xF != 0xF {
            self.leave_stop_mode();
            self.add_cycles(8);
        }
        self.cycles
    }

    // fn gdma_tick(&mut self) {
    //     // self.cycles += 4;
    //     let cycles = self.mmu.gdma_tick();
    //     self.add_cycles(cycles);
    // }

    // fn hdma_tick(&mut self) {
    //     if self.mmu.hdma.new_hdma {
    //         self.mmu.hdma.new_hdma = false;
    //         // self.cycles += 4;
    //         self.add_cycles(4);
    //     }
    //     let cycles = self.mmu.hdma_tick();
    //     self.add_cycles(cycles);
    // }

    fn gdma_tick(&mut self) {
        self.mmu.gdma_tick();

        self.add_cycles(match self.mmu.cgb_mode.speed {
            CgbSpeed::Normal => 32,
            CgbSpeed::Double => 64,
        });
    }

    fn hdma_tick(&mut self) {
        self.mmu.hdma_tick();

        self.add_cycles(match self.mmu.cgb_mode.speed {
            CgbSpeed::Normal => 32,
            CgbSpeed::Double => 64,
        });
    }

    fn service_pending_interrupts(&mut self) {
        self.halted = false;

        self.sp = self.sp.wrapping_sub(1);

        self.add_cycles(12);

        self.mmu.set_byte(self.sp, (self.pc >> 8) as u8);

        let mut ints = self.mmu.ie;

        if self.sp == 0xFF0F + 1 {
            self.sp = self.sp.wrapping_sub(1);
            self.add_cycles(4);
            let old_irr = self.mmu.get_byte(0xFF0F);
            self.mmu.set_byte(0xFF0F, (self.pc & 0xFF) as u8);

            ints &= old_irr & 0x1F;
        } else {
            self.sp = self.sp.wrapping_sub(1);
            self.add_cycles(4);
            self.mmu.set_byte(self.sp, (self.pc & 0xFF) as u8);

            ints &= self.mmu.get_byte(0xFF0F) & 0x1F;
        }

        self.add_cycles(4);

        if ints == 0 {
            self.pc = 0;
            self.ime = false;
            return;
        }

        for i in 0..5 {
            if ints & (1u8 << i) != 0 {
                return self.handle_interrupt(i);
            }
        }
    }

    fn handle_interrupt(&mut self, i: u16) {
        let mask = 1u8 << i;
        self.ime = false;
        let irr = self.mmu.get_byte(0xFF0F);
        self.mmu.set_byte(0xFF0F, irr & !mask);
        self.pc = 0x40 + 8 * i;
    }

    fn leave_stop_mode(&mut self) {
        for _ in 0..0x200 {
            self.add_cycles(0x10);
        }

        self.stopped = false;
    }

    #[allow(dead_code)]
    pub fn emulate_bootrom(&mut self) {
        self.mmu.bootrom.activate();
    }

    pub fn simulate_bootrom(&mut self) {
        // match self.emu_mode {
        //     EmulationMode::Dmg => self.set_r16(R16::AF, 0x01B0),
        //     EmulationMode::Cgb => self.set_r16(R16::AF, 0x11B0),
        // }
        // self.set_r16(R16::BC, 0x0013);
        // self.set_r16(R16::DE, 0x00D8);
        // self.set_r16(R16::HL, 0x014D);
        // self.set_r16(R16::SP, 0xFFFE);
        match self.emu_mode {
            EmulationMode::Dmg => {
                // AF = 0x01B0
                self.r[0] = 0x01;
                self.r[1] = 0xB0;
                // BC = 0x0013
                self.r[2] = 0x00;
                self.r[3] = 0x13;
                // DE = 0x00D8
                self.r[4] = 0x00;
                self.r[5] = 0xD8;
                // HL = 0x014D
                self.r[6] = 0x01;
                self.r[7] = 0x4D;
            }
            EmulationMode::Cgb => {
                // AF = 0x1180;
                self.r[0] = 0x11;
                self.r[1] = 0x80;
                // BC = 0x0000;
                self.r[2] = 0x00;
                self.r[3] = 0x00;
                // DE = 0xFF56;
                self.r[4] = 0xFF;
                self.r[5] = 0x56;
                // HL = 0x000D;
                self.r[6] = 0x00;
                self.r[7] = 0x0D;
            }
        }

        self.sp = 0xFFFE;

        self.mmu.set_byte(0xFF05, 0x00);
        self.mmu.set_byte(0xFF06, 0x00);
        self.mmu.set_byte(0xFF07, 0x00);
        self.mmu.set_byte(0xFF10, 0x80);
        self.mmu.set_byte(0xFF11, 0xBF);
        self.mmu.set_byte(0xFF12, 0xF3);
        self.mmu.set_byte(0xFF14, 0xBF);
        self.mmu.set_byte(0xFF16, 0x3F);
        self.mmu.set_byte(0xFF17, 0x00);
        self.mmu.set_byte(0xFF19, 0xBF);
        self.mmu.set_byte(0xFF1A, 0x7F);
        self.mmu.set_byte(0xFF1B, 0xFF);
        self.mmu.set_byte(0xFF1C, 0x9F);
        self.mmu.set_byte(0xFF1E, 0xBF);
        self.mmu.set_byte(0xFF20, 0xFF);
        self.mmu.set_byte(0xFF21, 0x00);
        self.mmu.set_byte(0xFF22, 0x00);
        self.mmu.set_byte(0xFF23, 0xBF);
        self.mmu.set_byte(0xFF24, 0x77);
        self.mmu.set_byte(0xFF25, 0xF3);
        self.mmu.set_byte(0xFF26, 0xF1);

        self.mmu.set_byte(0xFF40, 0x91);
        self.mmu.set_byte(0xFF41, 0x81);
        self.mmu.set_byte(0xFF42, 0x00);
        self.mmu.set_byte(0xFF43, 0x00);
        self.mmu.set_byte(0xFF45, 0x00);
        self.mmu.set_byte(0xFF47, 0xFC);
        self.mmu.set_byte(0xFF48, 0xFF);
        self.mmu.set_byte(0xFF49, 0xFF);
        self.mmu.set_byte(0xFF4A, 0x00);
        self.mmu.set_byte(0xFF4B, 0x00);
        self.mmu.set_byte(0xFFFF, 0x00);

        self.pc = 0x100;
    }

    // -------------------------------------------------------------
    //  Restarts & Returns
    // -------------------------------------------------------------

    // pub fn rst(&mut self, value: u8) {
    //     let ms = ((self.pc & 0xFF00) >> 8) as u8;
    //     let ls = (self.pc & 0x00FF) as u8;
    //     self.push(ms);
    //     self.push(ls);
    //     self.jp_addr(value as u16);
    // }
    pub fn rst(&mut self, value: u8) {
        self.add_cycles(4);
        self.call_addr(value as u16);
    }

    // pub fn ret(&mut self) {
    //     let ls = self.pop() as u16;
    //     let ms = self.pop() as u16;
    //     self.jp_addr((ms << 8) | ls);
    // }
    pub fn ret(&mut self) {
        self.pc = self.pop() as u16;
        self.pc |= (self.pop() as u16) << 8;
        self.add_cycles(4);
    }

    pub fn ret_cc(&mut self, flag: Flag, set: bool) {
        self.add_cycles(4);
        if self.get_flag(flag) == set as u8 {
            self.ret();
        }
    }

    // pub fn reti(&mut self) {
    //     self.ime = true;
    //     self.ret();
    // }
    pub fn reti(&mut self) {
        self.ime = true;
        self.ret();
    }

    // -------------------------------------------------------------
    //  Calls
    // -------------------------------------------------------------

    // pub fn call_addr(&mut self, addr: u16) {
    //     let ms = ((self.pc & 0xFF00) >> 8) as u8;
    //     let ls = (self.pc & 0x00FF) as u8;
    //     self.push(ms);
    //     self.push(ls);
    //     self.jp_addr(addr);
    // }
    pub fn call_addr(&mut self, addr: u16) {
        self.push((self.pc >> 8) as u8);
        self.push((self.pc & 0xFF) as u8);
        self.pc = addr as u16;
    }

    // pub fn call(&mut self) {
    //     let addr = self.get_imm16();
    //     self.call_addr(addr);
    // }
    pub fn call(&mut self) {
        let addr = self.get_imm16();
        self.add_cycles(4);
        self.call_addr(addr);
    }

    // pub fn call_cc_nn(&mut self, flag: Flag, set: bool) {
    //     let addr = self.get_imm16();
    //     if self.get_flag(flag) == set as u8 {
    //         self.call_addr(addr);
    //     }
    // }
    pub fn call_cc_nn(&mut self, flag: Flag, set: bool) {
        let addr = self.get_imm16();
        if self.get_flag(flag) == set as u8 {
            self.add_cycles(4);
            self.call_addr(addr);
        }
    }

    // -------------------------------------------------------------
    //  Jumps
    // -------------------------------------------------------------

    // #[inline]
    // pub fn jp_addr(&mut self, addr: u16) {
    //     self.add_cycles(4);
    //     self.pc = addr;
    // }

    // pub fn jp_nn(&mut self) {
    //     let addr = self.get_imm16();
    //     self.jp_addr(addr);
    // }
    pub fn jp_nn(&mut self) {
        // self.pc = self.get_imm8() as u16;
        // self.pc |= (self.get_imm8() as u16) << 8;
        self.pc = self.get_imm16();
        self.add_cycles(4);
    }

    // pub fn jp_cc_nn(&mut self, flag: Flag, set: bool) {
    //     let addr = self.get_imm16();
    //     if self.get_flag(flag) == set as u8 {
    //         self.jp_addr(addr);
    //     }
    // }
    pub fn jp_cc_nn(&mut self, flag: Flag, set: bool) {
        let addr = self.get_imm16();
        if self.get_flag(flag) == set as u8 {
            self.add_cycles(4);
            self.pc = addr;
        }
    }

    // pub fn jr_n(&mut self) {
    //     let n = self.get_imm8();
    //     let addr = self.pc.wrapping_add(n as i8 as i16 as u16);
    //     self.jp_addr(addr);
    // }
    pub fn jr_n(&mut self) {
        let n = self.get_imm8();
        self.add_cycles(4);
        self.pc = self.pc.wrapping_add(n as i8 as i16 as u16);
    }

    // pub fn jr_cc_n(&mut self, flag: Flag, set: bool) {
    //     let n = self.get_imm8();
    //     if self.get_flag(flag) == set as u8 {
    //         let addr = self.pc.wrapping_add(n as i8 as i16 as u16);
    //         self.jp_addr(addr);
    //     }
    // }
    pub fn jr_cc_n(&mut self, flag: Flag, set: bool) {
        let n = self.get_imm8();
        if self.get_flag(flag) == set as u8 {
            self.add_cycles(4);
            self.pc = self.pc.wrapping_add(n as i8 as i16 as u16);
        }
    }

    // -------------------------------------------------------------
    //  Bit Operations
    // -------------------------------------------------------------

    /// Test bit `b` in `value`.
    pub fn bit_value(&mut self, b: u8, value: u8) {
        self.setc_flag(Flag::Z, (value & (0x1 << b)) == 0);
        self.reset_flag(Flag::N);
        self.set_flag(Flag::H);
    }

    pub fn bit_r8(&mut self, b: u8, r: R8) {
        let value = self.get_r8(&r);
        self.bit_value(b, value);
    }

    pub fn bit_addr(&mut self, b: u8, addr: Addr) {
        let value = self.get_addr(&addr);
        self.bit_value(b, value);
    }

    pub fn setb_r8(&mut self, b: u8, r: R8) {
        let value = self.get_r8(&r);
        self.set_r8(r, value | (0x01 << b));
    }

    pub fn setb_addr(&mut self, b: u8, addr: Addr) {
        let value = self.get_addr(&addr);
        self.set_addr(addr, value | (0x01 << b));
    }

    pub fn res_r8(&mut self, b: u8, r: R8) {
        let value = self.get_r8(&r);
        self.set_r8(r, value & !(0x01 << b));
    }

    pub fn res_addr(&mut self, b: u8, addr: Addr) {
        let value = self.get_addr(&addr);
        self.set_addr(addr, value & !(0x01 << b));
    }

    // -------------------------------------------------------------
    //  Rotates & Shifts
    // -------------------------------------------------------------

    pub fn rlca(&mut self) {
        let value = self.get_r8(&R8::A);
        let res = self.rlc_value(value, false);
        self.set_r8(R8::A, res);
    }

    pub fn rlc_value(&mut self, value: u8, prefixed: bool) -> u8 {
        let res = value.rotate_left(1);
        if prefixed {
            self.setc_flag(Flag::Z, res == 0);
        } else {
            self.reset_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (value & 0x80) != 0);
        res
    }

    pub fn rla(&mut self) {
        let value = self.get_r8(&R8::A);
        let res = self.rl_value(value, false);
        self.set_r8(R8::A, res);
    }

    pub fn rl_value(&mut self, value: u8, prefixed: bool) -> u8 {
        let c = self.get_flag(Flag::C);
        let res = (value << 1) | c;
        if prefixed {
            self.setc_flag(Flag::Z, res == 0);
        } else {
            self.reset_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (value & 0x80) != 0);
        res
    }

    pub fn rrca(&mut self) {
        let value = self.get_r8(&R8::A);
        let res = self.rrc_value(value, false);
        self.set_r8(R8::A, res);
    }

    pub fn rrc_value(&mut self, value: u8, prefixed: bool) -> u8 {
        let res = value.rotate_right(1);
        if prefixed {
            self.setc_flag(Flag::Z, res == 0);
        } else {
            self.reset_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (value & 0x01) != 0);
        res
    }

    pub fn rra(&mut self) {
        let value = self.get_r8(&R8::A);
        let res = self.rr_value(value, false);
        self.set_r8(R8::A, res);
    }

    pub fn rr_value(&mut self, value: u8, prefixed: bool) -> u8 {
        let c = self.get_flag(Flag::C);
        let res = (value >> 1) | (c << 7);
        if prefixed {
            self.setc_flag(Flag::Z, res == 0);
        } else {
            self.reset_flag(Flag::Z);
        }
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (value & 0x01) != 0);
        res
    }

    pub fn rlc_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.rlc_value(value, true);
        self.set_r8(r, res);
    }

    pub fn rlc_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.rlc_value(value, true);
        self.set_addr(addr, res);
    }

    pub fn rl_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.rl_value(value, true);
        self.set_r8(r, res);
    }

    pub fn rl_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.rl_value(value, true);
        self.set_addr(addr, res);
    }

    pub fn rrc_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.rrc_value(value, true);
        self.set_r8(r, res);
    }

    pub fn rrc_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.rrc_value(value, true);
        self.set_addr(addr, res);
    }

    pub fn rr_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.rr_value(value, true);
        self.set_r8(r, res);
    }

    pub fn rr_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.rr_value(value, true);
        self.set_addr(addr, res);
    }

    pub fn sla_value(&mut self, value: u8) -> u8 {
        let res = value << 1;
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (0x80 & value) != 0);
        res
    }

    pub fn sla_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.sla_value(value);
        self.set_r8(r, res);
    }

    pub fn sla_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.sla_value(value);
        self.set_addr(addr, res);
    }

    pub fn sra_value(&mut self, value: u8) -> u8 {
        let res = (value >> 1) | (0x80 & value);
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (value & 0x01) != 0);
        res
    }

    pub fn sra_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.sra_value(value);
        self.set_r8(r, res);
    }

    pub fn sra_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.sra_value(value);
        self.set_addr(addr, res);
    }

    pub fn srl_value(&mut self, value: u8) -> u8 {
        let res = value >> 1;
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.setc_flag(Flag::C, (0x01 & value) != 0);
        res
    }

    pub fn srl_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.srl_value(value);
        self.set_r8(r, res);
    }

    pub fn srl_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.srl_value(value);
        self.set_addr(addr, res);
    }

    // -------------------------------------------------------------
    //  Misc.
    // -------------------------------------------------------------

    pub fn ei(&mut self) {
        if !self.ime && !self.ime_set_pending {
            self.ime_set_pending = true;
        }
    }

    pub fn di(&mut self) {
        self.ime = false;
    }

    pub fn stop(&mut self) {
        self.stopped = true;

        if self.mmu.cgb_mode.prepare_speed_switch != 0x0 {
            self.mmu.cgb_mode.speed = match self.mmu.cgb_mode.speed {
                CgbSpeed::Normal => CgbSpeed::Double,
                CgbSpeed::Double => CgbSpeed::Normal,
            };
            self.mmu.cgb_mode.prepare_speed_switch = 0x0;
            self.leave_stop_mode();
        }
    }

    pub fn halt(&mut self) {
        self.halted = true;

        let ie = self.mmu.ie;
        let irr = self.mmu.get_byte(0xFF0F);

        if ie & irr & 0x1F != 0 {
            if self.ime {
                self.halted = false;
                self.pc -= 1;
            } else {
                self.halted = false;
                self.halt_bug = true;
            }
        }

        self.just_halted = true;
    }

    pub fn nop(&mut self) {}

    /// Set carry flag.
    pub fn scf(&mut self) {
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.set_flag(Flag::C);
    }

    /// Complement carry flag.
    pub fn ccf(&mut self) {
        let c = self.get_flag(Flag::C);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        if c == 1 {
            self.reset_flag(Flag::C);
        } else {
            self.set_flag(Flag::C);
        }
    }

    /// Complement register A.
    pub fn cpl(&mut self) {
        let value = self.get_r8(&R8::A);
        self.set_flag(Flag::N);
        self.set_flag(Flag::H);
        self.set_r8(R8::A, !value);
    }

    pub fn daa(&mut self) {
        let mut a = self.get_r8(&R8::A) as u16;

        if self.get_flag(Flag::N) == 0 {
            // previous op was addition
            if self.get_flag(Flag::H) == 1 || (a & 0xF > 0x9) {
                a = a.wrapping_add(0x6);
            }
            if self.get_flag(Flag::C) == 1 || (a > 0x9F) {
                a = a.wrapping_add(0x60);
                self.set_flag(Flag::C);
            }
        } else {
            // previous op was subtraction
            if self.get_flag(Flag::H) == 1 {
                a = a.wrapping_sub(0x6);
            }
            if self.get_flag(Flag::C) == 1 {
                a = a.wrapping_sub(0x60);
            }
        }
        self.setc_flag(Flag::Z, a & 0xFF == 0);
        self.reset_flag(Flag::H);
        self.set_r8(R8::A, a as u8);
    }

    pub fn swap_value(&mut self, value: u8) -> u8 {
        let upper = (value & 0xF0) >> 4;
        let lower = value & 0x0F;
        self.setc_flag(Flag::Z, value == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.reset_flag(Flag::C);
        lower << 4 | upper
    }

    pub fn swap_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.swap_value(value);
        self.set_r8(r, res);
    }

    pub fn swap_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.swap_value(value);
        self.set_addr(addr, res);
    }

    // -------------------------------------------------------------
    //  ALU
    // -------------------------------------------------------------

    pub fn add_sp_imm(&mut self) {
        let value = self.get_imm8() as i8 as u16;
        self.add_cycles(4);
        let n = self.get_r16(&R16::SP);
        let res = n.wrapping_add(value);
        self.reset_flag(Flag::Z);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) + (n & 0xF) > 0xF);
        self.setc_flag(Flag::C, (value & 0xFF) + (n & 0xFF) > 0xFF);
        self.set_r16(R16::SP, res);
    }

    pub fn add_sp_imm_hl(&mut self) {
        let value = self.get_imm8() as i8 as u16;
        let n = self.get_r16(&R16::SP);
        let res = n.wrapping_add(value);
        self.reset_flag(Flag::Z);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) + (n & 0xF) > 0xF);
        self.setc_flag(Flag::C, (value & 0xFF) + (n & 0xFF) > 0xFF);
        self.set_r16(R16::HL, res);
    }

    pub fn inc_r16(&mut self, r: R16) {
        let value = self.get_r16(&r).wrapping_add(0x1);
        self.set_r16(r, value);
    }

    pub fn dec_r16(&mut self, r: R16) {
        let value = self.get_r16(&r).wrapping_sub(0x1);
        self.set_r16(r, value);
    }

    pub fn add_r16(&mut self, r1: R16, r2: R16) {
        let value = self.get_r16(&r2);
        self.add_r16_imm(r1, value);
    }

    pub fn add_r16_imm(&mut self, r1: R16, value: u16) {
        let n = self.get_r16(&r1);
        let res = n.wrapping_add(value);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0x7FF) + (n & 0x7FF) > 0x7FF);
        self.setc_flag(Flag::C, (value as u32) + (n as u32) > 0xFFFF);
        self.set_r16(r1, res);
    }

    pub fn add_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.add_r8_imm(r1, value);
    }

    pub fn add_r8_imm(&mut self, r1: R8, value: u8) {
        let n = self.get_r8(&r1);
        let (sum, overflow) = n.overflowing_add(value);
        self.setc_flag(Flag::Z, sum == 0);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) + (n & 0xF) > 0xF);
        self.setc_flag(Flag::C, overflow);
        self.set_r8(r1, sum);
    }

    pub fn adc_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.adc_r8_imm(r1, value);
    }

    pub fn adc_r8_imm(&mut self, r1: R8, value: u8) {
        let c = self.get_flag(Flag::C);
        let n = self.get_r8(&r1);
        let sum = n.wrapping_add(value).wrapping_add(c);
        self.setc_flag(Flag::Z, sum == 0);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) + (n & 0xF) + c > 0xF);
        self.setc_flag(Flag::C, (value as u16) + (n as u16) + (c as u16) > 0xFF);
        self.set_r8(r1, sum);
    }

    pub fn sub_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.sub_r8_imm(r1, value);
    }

    pub fn sub_r8_imm(&mut self, r1: R8, value: u8) {
        let n = self.get_r8(&r1);
        let diff = n.wrapping_sub(value);
        self.setc_flag(Flag::Z, diff == 0);
        self.set_flag(Flag::N);
        self.setc_flag(Flag::H, (n & 0xF) < (value & 0xF));
        self.setc_flag(Flag::C, (n as u16) < (value as u16));
        self.set_r8(r1, diff);
    }

    pub fn sbc_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.sbc_r8_imm(r1, value);
    }

    pub fn sbc_r8_imm(&mut self, r1: R8, value: u8) {
        let c = self.get_flag(Flag::C);
        let n = self.get_r8(&r1);
        let diff = n.wrapping_sub(value).wrapping_sub(c);
        self.setc_flag(Flag::Z, diff == 0);
        self.set_flag(Flag::N);
        self.setc_flag(Flag::H, (n & 0xF) < (value & 0xF) + c);
        self.setc_flag(Flag::C, (n as u16) < (value as u16) + (c as u16));
        self.set_r8(r1, diff);
    }

    pub fn and_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.and_r8_imm(r1, value);
    }

    pub fn and_r8_imm(&mut self, r1: R8, value: u8) {
        let res = self.get_r8(&r1) & value;
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.set_flag(Flag::H);
        self.reset_flag(Flag::C);
        self.set_r8(r1, res);
    }

    pub fn or_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.or_r8_imm(r1, value);
    }

    pub fn or_r8_imm(&mut self, r1: R8, value: u8) {
        let res = self.get_r8(&r1) | value;
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.reset_flag(Flag::C);
        self.set_r8(r1, res);
    }

    pub fn xor_r8(&mut self, r1: R8, r2: R8) {
        let value = self.get_r8(&r2);
        self.xor_r8_imm(r1, value);
    }

    pub fn xor_r8_imm(&mut self, r1: R8, value: u8) {
        let res = self.get_r8(&r1) ^ value;
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.reset_flag(Flag::H);
        self.reset_flag(Flag::C);
        self.set_r8(r1, res);
    }

    pub fn cp_r8(&mut self, r1: R8, r2: R8) {
        let val2 = self.get_r8(&r2);
        self.cp_r8_imm(r1, val2);
    }

    pub fn cp_r8_imm(&mut self, r1: R8, value: u8) {
        let n = self.get_r8(&r1);
        // let diff = n.wrapping_sub(value);
        self.setc_flag(Flag::Z, n == value);
        self.set_flag(Flag::N);
        self.setc_flag(Flag::H, (n & 0xF) < (value & 0xF));
        self.setc_flag(Flag::C, n < value);
    }

    pub fn inc_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.inc_value(value);
        self.set_r8(r, res);
    }

    pub fn inc_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.inc_value(value);
        self.set_addr(addr, res);
    }

    pub fn inc_value(&mut self, value: u8) -> u8 {
        let res = value.wrapping_add(1);
        self.setc_flag(Flag::Z, res == 0);
        self.reset_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) + 1 > 0xF);
        res
    }

    pub fn dec_r8(&mut self, r: R8) {
        let value = self.get_r8(&r);
        let res = self.dec_value(value);
        self.set_r8(r, res);
    }

    pub fn dec_addr(&mut self, addr: Addr) {
        let value = self.get_addr(&addr);
        let res = self.dec_value(value);
        self.set_addr(addr, res);
    }

    pub fn dec_value(&mut self, value: u8) -> u8 {
        let res = value.wrapping_sub(1);
        self.setc_flag(Flag::Z, res == 0);
        self.set_flag(Flag::N);
        self.setc_flag(Flag::H, (value & 0xF) == 0);
        res
    }

    // -------------------------------------------------------------
    //  Load, Store, Push, Pop
    // -------------------------------------------------------------

    pub fn pop(&mut self) -> u8 {
        let value = self.get_addr(&Addr::SP);
        self.sp = self.sp.wrapping_add(1);
        value
    }

    pub fn push(&mut self, value: u8) {
        self.sp = self.sp.wrapping_sub(1);
        self.set_addr(Addr::SP, value);
    }

    pub fn pop_r16(&mut self, r: R16) {
        match r {
            R16::AF => {
                let ls = self.pop();
                let ms = self.pop();
                self.r[0] = ms;
                self.r[1] = ls;
            }
            R16::BC => {
                let ls = self.pop();
                let ms = self.pop();
                self.r[2] = ms;
                self.r[3] = ls;
            }
            R16::DE => {
                let ls = self.pop();
                let ms = self.pop();
                self.r[4] = ms;
                self.r[5] = ls;
            }
            R16::HL => {
                let ls = self.pop();
                let ms = self.pop();
                self.r[6] = ms;
                self.r[7] = ls;
            }
            _ => (),
        }
    }

    pub fn push_r16(&mut self, r: R16) {
        self.add_cycles(4);
        match r {
            R16::AF => {
                let ms = self.get_r8(&R8::A) as u8;
                self.push(ms);
                let ls = self.get_r8(&R8::F) as u8;
                self.push(ls & 0xF0);
            }
            R16::BC => {
                let ms = self.get_r8(&R8::B);
                self.push(ms);
                let ls = self.get_r8(&R8::C);
                self.push(ls);
            }
            R16::DE => {
                let ms = self.get_r8(&R8::D);
                self.push(ms);
                let ls = self.get_r8(&R8::E);
                self.push(ls);
            }
            R16::HL => {
                let ms = self.get_r8(&R8::H);
                self.push(ms);
                let ls = self.get_r8(&R8::L);
                self.push(ls);
            }
            R16::SP => (),
        }
    }

    pub fn get_flag(&self, flag: Flag) -> u8 {
        let f = self.get_r8(&R8::F);

        match flag {
            Flag::Z => (0x80 & f) >> 7,
            Flag::N => (0x40 & f) >> 6,
            Flag::H => (0x20 & f) >> 5,
            Flag::C => (0x10 & f) >> 4,
        }
    }

    #[inline]
    pub fn setc_flag(&mut self, flag: Flag, cond: bool) {
        if cond {
            self.set_flag(flag);
        } else {
            self.reset_flag(flag);
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        let f = self.r[1];

        match flag {
            Flag::Z => self.r[1] = 0x80 | f,
            Flag::N => self.r[1] = 0x40 | f,
            Flag::H => self.r[1] = 0x20 | f,
            Flag::C => self.r[1] = 0x10 | f,
        }
    }

    pub fn reset_flag(&mut self, flag: Flag) {
        let f = self.r[1];

        match flag {
            Flag::Z => self.r[1] = !0x80 & f,
            Flag::N => self.r[1] = !0x40 & f,
            Flag::H => self.r[1] = !0x20 & f,
            Flag::C => self.r[1] = !0x10 & f,
        }
    }

    pub fn get_addr_dec(&mut self) -> u8 {
        let addr = self.get_r16(&R16::HL);
        let value = self.memory_get(addr);
        let addr = addr.wrapping_sub(1);
        self.r[6] = (addr >> 8) as u8;
        self.r[7] = (addr & 0xFF) as u8;
        value
    }

    pub fn set_addr_dec(&mut self, value: u8) {
        let addr = self.get_r16(&R16::HL);
        self.memory_set(addr, value);
        let addr = addr.wrapping_sub(1);
        self.r[6] = (addr >> 8) as u8;
        self.r[7] = (addr & 0xFF) as u8;
    }

    pub fn get_addr_inc(&mut self) -> u8 {
        let addr = self.get_r16(&R16::HL);
        let value = self.memory_get(addr);
        let addr = addr.wrapping_add(1);
        self.r[6] = (addr >> 8) as u8;
        self.r[7] = (addr & 0xFF) as u8;
        value
    }

    pub fn set_addr_inc(&mut self, value: u8) {
        let addr = self.get_r16(&R16::HL);
        self.memory_set(addr, value);
        let addr = addr.wrapping_add(1);
        self.r[6] = (addr >> 8) as u8;
        self.r[7] = (addr & 0xFF) as u8;
    }

    pub fn get_addr(&mut self, addr: &Addr) -> u8 {
        match addr {
            Addr::AF => self.memory_get(self.get_r16(&R16::AF)),
            Addr::BC => self.memory_get(self.get_r16(&R16::BC)),
            Addr::DE => self.memory_get(self.get_r16(&R16::DE)),
            Addr::HL => self.memory_get(self.get_r16(&R16::HL)),
            Addr::SP => self.memory_get(self.get_r16(&R16::SP)),
        }
    }

    pub fn set_addr(&mut self, addr: Addr, value: u8) {
        match addr {
            Addr::AF => self.memory_set(self.get_r16(&R16::AF), value),
            Addr::BC => self.memory_set(self.get_r16(&R16::BC), value),
            Addr::DE => self.memory_set(self.get_r16(&R16::DE), value),
            Addr::HL => self.memory_set(self.get_r16(&R16::HL), value),
            Addr::SP => self.memory_set(self.get_r16(&R16::SP), value),
        }
    }

    pub fn get_imm16(&mut self) -> u16 {
        let ls = self.get_imm8() as u16;
        let ms = self.get_imm8() as u16;
        ms << 8 | ls
    }

    pub fn set_addr_imm(&mut self, addr: u16, value: u8) {
        self.memory_set(addr, value);
    }

    pub fn get_addr_imm(&mut self, addr: u16) -> u8 {
        self.memory_get(addr)
    }

    #[inline]
    pub fn get_imm8(&mut self) -> u8 {
        self.fetch()
    }

    // -------------------------------------------------------------
    //  Atomic operations
    // -------------------------------------------------------------

    fn add_cycles(&mut self, cycles: usize) {
        self.cycles += cycles;

        self.mmu.timer_tick(cycles);

        if !self.stopped && !self.halted && self.mmu.oam_dma.active {
            self.mmu.oam_dma_tick(cycles);
        }

        match self.mmu.cgb_mode.speed {
            CgbSpeed::Normal => {
                self.mmu.gpu_tick(cycles);
                self.mmu.apu_tick(cycles);
            }
            CgbSpeed::Double => {
                self.mmu.gpu_tick(cycles >> 1);
                self.mmu.apu_tick(cycles >> 1);
            }
        }
    }

    /// Fetch next byte at pc from memory and increment pc.
    pub fn fetch(&mut self) -> u8 {
        let byte = self.mmu.get_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        self.add_cycles(4);
        byte
    }

    pub fn get_r8(&self, r: &R8) -> u8 {
        match r {
            R8::A => self.r[0],
            R8::F => self.r[1],
            R8::B => self.r[2],
            R8::C => self.r[3],
            R8::D => self.r[4],
            R8::E => self.r[5],
            R8::H => self.r[6],
            R8::L => self.r[7],
        }
    }

    /// Sets the value of the given register.
    pub fn set_r8(&mut self, r: R8, value: u8) {
        match r {
            R8::A => self.r[0] = value,
            R8::F => self.r[1] = value,
            R8::B => self.r[2] = value,
            R8::C => self.r[3] = value,
            R8::D => self.r[4] = value,
            R8::E => self.r[5] = value,
            R8::H => self.r[6] = value,
            R8::L => self.r[7] = value,
        }
    }

    pub fn get_r16(&self, r: &R16) -> u16 {
        match r {
            R16::AF => (self.r[0] as u16) << 8 | (self.r[1] as u16),
            R16::BC => (self.r[2] as u16) << 8 | (self.r[3] as u16),
            R16::DE => (self.r[4] as u16) << 8 | (self.r[5] as u16),
            R16::HL => (self.r[6] as u16) << 8 | (self.r[7] as u16),
            R16::SP => self.sp,
        }
    }

    /// Sets values of combined 16bit register.
    pub fn set_r16(&mut self, r: R16, value: u16) {
        self.add_cycles(4);
        let ms = ((0xFF00 & value) >> 8) as u8;
        let ls = (0x00FF & value) as u8;
        match r {
            R16::AF => {
                self.r[0] = ms;
                self.r[1] = ls;
            }
            R16::BC => {
                self.r[2] = ms;
                self.r[3] = ls;
            }
            R16::DE => {
                self.r[4] = ms;
                self.r[5] = ls;
            }
            R16::HL => {
                self.r[6] = ms;
                self.r[7] = ls;
            }
            R16::SP => {
                self.sp = value;
            }
        }
    }

    pub fn set_r16_imm(&mut self, r: R16) {
        let value = self.get_imm16();
        let ms = ((0xFF00 & value) >> 8) as u8;
        let ls = (0x00FF & value) as u8;
        match r {
            R16::AF => {
                self.r[0] = ms;
                self.r[1] = ls;
            }
            R16::BC => {
                self.r[2] = ms;
                self.r[3] = ls;
            }
            R16::DE => {
                self.r[4] = ms;
                self.r[5] = ls;
            }
            R16::HL => {
                self.r[6] = ms;
                self.r[7] = ls;
            }
            R16::SP => {
                self.sp = value;
            }
        }
    }

    pub fn memory_set(&mut self, addr: u16, value: u8) {
        self.mmu.set_byte(addr, value);
        self.add_cycles(4);
    }

    pub fn memory_get(&mut self, addr: u16) -> u8 {
        let value = self.mmu.get_byte(addr);
        self.add_cycles(4);
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_blargg() {
        // let rom = fs::read("roms/instr_timing.gb").unwrap();
        // let rom = fs::read("roms/acceptance/ei_timing.gb").unwrap();
        // let rom = fs::read("roms/interrupt_time.gb").unwrap();
        let rom = fs::read("roms/Shantae (USA).gbc").unwrap();
        // let rom = fs::read("roms/Aladdin (USA).gbc").unwrap();
        // let rom = fs::read("roms/dmg-acid2.gb").unwrap();
        // let rom = fs::read("roms/cgb-acid2.gbc").unwrap();
        // let rom = fs::read("roms/Pokemon Pinball (U) (C).gbc").unwrap();
        // let rom = fs::read("roms/Tetris.gb").unwrap();
        // let rom = fs::read("roms/Dr. Mario (World).gb").unwrap();
        // let rom = fs::read("roms/intr_2_mode3_timing.gb").unwrap();
        // let rom = fs::read("roms/Pinball Deluxe (U).gb").unwrap();
        // let rom = fs::read("roms/Super Mario Land 2 - 6 Golden Coins (UE) (V1.2) [!].gb").unwrap();
        // let rom = fs::read("roms/bits_mode.gb").unwrap();
        // let rom = fs::read("roms/unused_hwio-GS.gb").unwrap();
        // let rom = fs::read("roms/sources-GS.gb").unwrap();
        // let rom = fs::read("roms/rst_timing.gb").unwrap();
        // let rom = fs::read("roms/bits_mode.gb").unwrap();
        // let rom = fs::read("roms/Aladdin (U) [S][!].gb").unwrap();
        // let rom = fs::read("roms/Prehistorik Man (USA, Europe).gb").unwrap();
        println!("{:#X}", rom[0x147]);
        let mut cpu = Cpu::new(rom);
        cpu.simulate_bootrom();
        println!("Starting");

        // let mut i = 1;

        loop {
            cpu.tick();

            // if i % 10 == 0 {
            //     cpu.mmu.joypad.press_key(Key::Start);
            // } else if i % 10 == 1 {
            //     cpu.mmu.joypad.release_key(Key::Start);
            // }
        }
    }

    fn print_and_step(cpu: &mut Cpu) {
        println!(
            "pc: {:#X}, opcode: {:#X}, halted: {}",
            cpu.pc,
            cpu.mmu.get_byte(cpu.pc),
            cpu.halted,
        );

        cpu.tick();
    }

    #[test]
    fn test_steps() {
        let rom = fs::read("roms/Pinball Deluxe (U).gb").unwrap();
        let mut cpu = Cpu::new(rom);
        cpu.simulate_bootrom();
        println!("Starting");

        // while cpu.pc != 0x17A {
        //     cpu.tick();
        // }
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
        print_and_step(&mut cpu);
    }
}

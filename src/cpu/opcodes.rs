/// This is a generated file. Do not modify.
/// See `opcodes.py` to understand how this file was generated.
use crate::cpu::{Addr, Cpu, Flag, R16, R8};

impl Cpu {
	pub fn decode_exec(&mut self, opcode: u8) {
		match opcode {
			0x06 => {
				let value = self.get_imm8();
				self.set_r8(R8::B, value);
			}
			0x0E => {
				let value = self.get_imm8();
				self.set_r8(R8::C, value);
			}
			0x16 => {
				let value = self.get_imm8();
				self.set_r8(R8::D, value);
			}
			0x1E => {
				let value = self.get_imm8();
				self.set_r8(R8::E, value);
			}
			0x26 => {
				let value = self.get_imm8();
				self.set_r8(R8::H, value);
			}
			0x2E => {
				let value = self.get_imm8();
				self.set_r8(R8::L, value);
			}
			0x7F => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::A, value);
			}
			0x78 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::A, value);
			}
			0x79 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::A, value);
			}
			0x7A => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::A, value);
			}
			0x7B => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::A, value);
			}
			0x7C => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::A, value);
			}
			0x7D => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::A, value);
			}
			0x7E => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::A, value);
			}
			0x40 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::B, value);
			}
			0x41 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::B, value);
			}
			0x42 => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::B, value);
			}
			0x43 => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::B, value);
			}
			0x44 => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::B, value);
			}
			0x45 => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::B, value);
			}
			0x46 => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::B, value);
			}
			0x48 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::C, value);
			}
			0x49 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::C, value);
			}
			0x4A => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::C, value);
			}
			0x4B => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::C, value);
			}
			0x4C => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::C, value);
			}
			0x4D => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::C, value);
			}
			0x4E => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::C, value);
			}
			0x50 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::D, value);
			}
			0x51 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::D, value);
			}
			0x52 => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::D, value);
			}
			0x53 => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::D, value);
			}
			0x54 => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::D, value);
			}
			0x55 => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::D, value);
			}
			0x56 => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::D, value);
			}
			0x58 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::E, value);
			}
			0x59 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::E, value);
			}
			0x5A => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::E, value);
			}
			0x5B => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::E, value);
			}
			0x5C => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::E, value);
			}
			0x5D => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::E, value);
			}
			0x5E => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::E, value);
			}
			0x60 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::H, value);
			}
			0x61 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::H, value);
			}
			0x62 => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::H, value);
			}
			0x63 => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::H, value);
			}
			0x64 => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::H, value);
			}
			0x65 => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::H, value);
			}
			0x66 => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::H, value);
			}
			0x68 => {
				let value = self.get_r8(&R8::B);
				self.set_r8(R8::L, value);
			}
			0x69 => {
				let value = self.get_r8(&R8::C);
				self.set_r8(R8::L, value);
			}
			0x6A => {
				let value = self.get_r8(&R8::D);
				self.set_r8(R8::L, value);
			}
			0x6B => {
				let value = self.get_r8(&R8::E);
				self.set_r8(R8::L, value);
			}
			0x6C => {
				let value = self.get_r8(&R8::H);
				self.set_r8(R8::L, value);
			}
			0x6D => {
				let value = self.get_r8(&R8::L);
				self.set_r8(R8::L, value);
			}
			0x6E => {
				let value = self.get_addr(&Addr::HL);
				self.set_r8(R8::L, value);
			}
			0x70 => {
				let value = self.get_r8(&R8::B);
				self.set_addr(Addr::HL, value);
			}
			0x71 => {
				let value = self.get_r8(&R8::C);
				self.set_addr(Addr::HL, value);
			}
			0x72 => {
				let value = self.get_r8(&R8::D);
				self.set_addr(Addr::HL, value);
			}
			0x73 => {
				let value = self.get_r8(&R8::E);
				self.set_addr(Addr::HL, value);
			}
			0x74 => {
				let value = self.get_r8(&R8::H);
				self.set_addr(Addr::HL, value);
			}
			0x75 => {
				let value = self.get_r8(&R8::L);
				self.set_addr(Addr::HL, value);
			}
			0x36 => {
				let value = self.get_imm8();
				self.set_addr(Addr::HL, value);
			}
			0x0A => {
				let value = self.get_addr(&Addr::BC);
				self.set_r8(R8::A, value);
			}
			0x1A => {
				let value = self.get_addr(&Addr::DE);
				self.set_r8(R8::A, value);
			}
			0xFA => {
				let addr = self.get_imm16();
				let value = self.get_addr_imm(addr);
				self.set_r8(R8::A, value);
			}
			0x3E => {
				let value = self.get_imm8();
				self.set_r8(R8::A, value);
			}
			0x47 => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::B, value);
			}
			0x4F => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::C, value);
			}
			0x57 => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::D, value);
			}
			0x5F => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::E, value);
			}
			0x67 => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::H, value);
			}
			0x6F => {
				let value = self.get_r8(&R8::A);
				self.set_r8(R8::L, value);
			}
			0x02 => {
				let value = self.get_r8(&R8::A);
				self.set_addr(Addr::BC, value);
			}
			0x12 => {
				let value = self.get_r8(&R8::A);
				self.set_addr(Addr::DE, value);
			}
			0x77 => {
				let value = self.get_r8(&R8::A);
				self.set_addr(Addr::HL, value);
			}
			0xEA => {
				let addr = self.get_imm16();
				let value = self.get_r8(&R8::A);
				self.set_addr_imm(addr, value);
			}
			0xF2 => {
				let addr = 0xFF00 + (self.get_r8(&R8::C) as u16);
				let value = self.get_addr_imm(addr);
				self.set_r8(R8::A, value);
			}
			0xE2 => {
				let addr = 0xFF00 + (self.get_r8(&R8::C) as u16);
				self.set_addr_imm(addr, self.get_r8(&R8::A));
			}
			0x3A => {
				let value = self.get_addr_dec();
				self.set_r8(R8::A, value);
			}
			0x32 => {
				let value = self.get_r8(&R8::A);
				self.set_addr_dec(value);
			}
			0x2A => {
				let value = self.get_addr_inc();
				self.set_r8(R8::A, value);
			}
			0x22 => {
				let value = self.get_r8(&R8::A);
				self.set_addr_inc(value);
			}
			0xE0 => {
				let addr = 0xFF00 + (self.fetch() as u16);
				let value = self.get_r8(&R8::A);
				self.set_addr_imm(addr, value);
			}
			0xF0 => {
				let addr = 0xFF00 + (self.fetch() as u16);
				let value = self.get_addr_imm(addr);
				self.set_r8(R8::A, value);
			}
			0x01 => {
				self.set_r16_imm(R16::BC);
			}
			0x11 => {
				self.set_r16_imm(R16::DE);
			}
			0x21 => {
				self.set_r16_imm(R16::HL);
			}
			0x31 => {
				self.set_r16_imm(R16::SP);
			}
			0xF9 => {
				let value = self.get_r16(&R16::HL);
				self.set_r16(R16::SP, value);
			}
			0xF8 => {
				self.add_sp_imm_hl();
			}
			0x08 => {
				let value = self.get_r16(&R16::SP);
				let addr = self.get_imm16();
				self.set_addr_imm(addr, (value & 0x00FF) as u8);
				self.set_addr_imm(addr.wrapping_add(0x1), ((value & 0xFF00) >> 8) as u8);
			}
			0xC5 => {
				self.push_r16(R16::BC);
			}
			0xD5 => {
				self.push_r16(R16::DE);
			}
			0xE5 => {
				self.push_r16(R16::HL);
			}
			0xF5 => {
				self.push_r16(R16::AF);
			}
			0xC1 => {
				self.pop_r16(R16::BC);
			}
			0xD1 => {
				self.pop_r16(R16::DE);
			}
			0xE1 => {
				self.pop_r16(R16::HL);
			}
			0xF1 => {
				self.pop_r16(R16::AF);
			}
			0x80 => {
				self.add_r8(R8::A, R8::B);
			}
			0x81 => {
				self.add_r8(R8::A, R8::C);
			}
			0x82 => {
				self.add_r8(R8::A, R8::D);
			}
			0x83 => {
				self.add_r8(R8::A, R8::E);
			}
			0x84 => {
				self.add_r8(R8::A, R8::H);
			}
			0x85 => {
				self.add_r8(R8::A, R8::L);
			}
			0x86 => {
				let value = self.get_addr(&Addr::HL);
				self.add_r8_imm(R8::A, value);
			}
			0x87 => {
				self.add_r8(R8::A, R8::A);
			}
			0xC6 => {
				let value = self.get_imm8();
				self.add_r8_imm(R8::A, value);
			}
			0x88 => {
				self.adc_r8(R8::A, R8::B);
			}
			0x89 => {
				self.adc_r8(R8::A, R8::C);
			}
			0x8A => {
				self.adc_r8(R8::A, R8::D);
			}
			0x8B => {
				self.adc_r8(R8::A, R8::E);
			}
			0x8C => {
				self.adc_r8(R8::A, R8::H);
			}
			0x8D => {
				self.adc_r8(R8::A, R8::L);
			}
			0x8E => {
				let value = self.get_addr(&Addr::HL);
				self.adc_r8_imm(R8::A, value);
			}
			0x8F => {
				self.adc_r8(R8::A, R8::A);
			}
			0xCE => {
				let value = self.get_imm8();
				self.adc_r8_imm(R8::A, value);
			}
			0x90 => {
				self.sub_r8(R8::A, R8::B);
			}
			0x91 => {
				self.sub_r8(R8::A, R8::C);
			}
			0x92 => {
				self.sub_r8(R8::A, R8::D);
			}
			0x93 => {
				self.sub_r8(R8::A, R8::E);
			}
			0x94 => {
				self.sub_r8(R8::A, R8::H);
			}
			0x95 => {
				self.sub_r8(R8::A, R8::L);
			}
			0x96 => {
				let value = self.get_addr(&Addr::HL);
				self.sub_r8_imm(R8::A, value);
			}
			0x97 => {
				self.sub_r8(R8::A, R8::A);
			}
			0xD6 => {
				let value = self.get_imm8();
				self.sub_r8_imm(R8::A, value)
			}
			0x98 => {
				self.sbc_r8(R8::A, R8::B);
			}
			0x99 => {
				self.sbc_r8(R8::A, R8::C);
			}
			0x9A => {
				self.sbc_r8(R8::A, R8::D);
			}
			0x9B => {
				self.sbc_r8(R8::A, R8::E);
			}
			0x9C => {
				self.sbc_r8(R8::A, R8::H);
			}
			0x9D => {
				self.sbc_r8(R8::A, R8::L);
			}
			0x9E => {
				let value = self.get_addr(&Addr::HL);
				self.sbc_r8_imm(R8::A, value);
			}
			0x9F => {
				self.sbc_r8(R8::A, R8::A);
			}
			0xDE => {
				let value = self.get_imm8();
				self.sbc_r8_imm(R8::A, value)
			}
			0xA0 => {
				self.and_r8(R8::A, R8::B);
			}
			0xA1 => {
				self.and_r8(R8::A, R8::C);
			}
			0xA2 => {
				self.and_r8(R8::A, R8::D);
			}
			0xA3 => {
				self.and_r8(R8::A, R8::E);
			}
			0xA4 => {
				self.and_r8(R8::A, R8::H);
			}
			0xA5 => {
				self.and_r8(R8::A, R8::L);
			}
			0xA6 => {
				let value = self.get_addr(&Addr::HL);
				self.and_r8_imm(R8::A, value);
			}
			0xA7 => {
				self.and_r8(R8::A, R8::A);
			}
			0xE6 => {
				let value = self.get_imm8();
				self.and_r8_imm(R8::A, value)
			}
			0xB0 => {
				self.or_r8(R8::A, R8::B);
			}
			0xB1 => {
				self.or_r8(R8::A, R8::C);
			}
			0xB2 => {
				self.or_r8(R8::A, R8::D);
			}
			0xB3 => {
				self.or_r8(R8::A, R8::E);
			}
			0xB4 => {
				self.or_r8(R8::A, R8::H);
			}
			0xB5 => {
				self.or_r8(R8::A, R8::L);
			}
			0xB6 => {
				let value = self.get_addr(&Addr::HL);
				self.or_r8_imm(R8::A, value);
			}
			0xB7 => {
				self.or_r8(R8::A, R8::A);
			}
			0xF6 => {
				let value = self.get_imm8();
				self.or_r8_imm(R8::A, value)
			}
			0xA8 => {
				self.xor_r8(R8::A, R8::B);
			}
			0xA9 => {
				self.xor_r8(R8::A, R8::C);
			}
			0xAA => {
				self.xor_r8(R8::A, R8::D);
			}
			0xAB => {
				self.xor_r8(R8::A, R8::E);
			}
			0xAC => {
				self.xor_r8(R8::A, R8::H);
			}
			0xAD => {
				self.xor_r8(R8::A, R8::L);
			}
			0xAE => {
				let value = self.get_addr(&Addr::HL);
				self.xor_r8_imm(R8::A, value);
			}
			0xAF => {
				self.xor_r8(R8::A, R8::A);
			}
			0xEE => {
				let value = self.get_imm8();
				self.xor_r8_imm(R8::A, value)
			}
			0xB8 => {
				self.cp_r8(R8::A, R8::B);
			}
			0xB9 => {
				self.cp_r8(R8::A, R8::C);
			}
			0xBA => {
				self.cp_r8(R8::A, R8::D);
			}
			0xBB => {
				self.cp_r8(R8::A, R8::E);
			}
			0xBC => {
				self.cp_r8(R8::A, R8::H);
			}
			0xBD => {
				self.cp_r8(R8::A, R8::L);
			}
			0xBE => {
				let value = self.get_addr(&Addr::HL);
				self.cp_r8_imm(R8::A, value);
			}
			0xBF => {
				self.cp_r8(R8::A, R8::A);
			}
			0xFE => {
				let value = self.get_imm8();
				self.cp_r8_imm(R8::A, value)
			}
			0x04 => {
				self.inc_r8(R8::B);
			}
			0x0C => {
				self.inc_r8(R8::C);
			}
			0x14 => {
				self.inc_r8(R8::D);
			}
			0x1C => {
				self.inc_r8(R8::E);
			}
			0x24 => {
				self.inc_r8(R8::H);
			}
			0x2C => {
				self.inc_r8(R8::L);
			}
			0x34 => {
				self.inc_addr(Addr::HL);
			}
			0x3C => {
				self.inc_r8(R8::A);
			}
			0x05 => {
				self.dec_r8(R8::B);
			}
			0x0D => {
				self.dec_r8(R8::C);
			}
			0x15 => {
				self.dec_r8(R8::D);
			}
			0x1D => {
				self.dec_r8(R8::E);
			}
			0x25 => {
				self.dec_r8(R8::H);
			}
			0x2D => {
				self.dec_r8(R8::L);
			}
			0x35 => {
				self.dec_addr(Addr::HL);
			}
			0x3D => {
				self.dec_r8(R8::A);
			}
			0x09 => {
				self.add_r16(R16::HL, R16::BC);
			}
			0x19 => {
				self.add_r16(R16::HL, R16::DE);
			}
			0x29 => {
				self.add_r16(R16::HL, R16::HL);
			}
			0x39 => {
				self.add_r16(R16::HL, R16::SP);
			}
			0xE8 => {
				self.add_sp_imm();
			}
			0x03 => {
				self.inc_r16(R16::BC);
			}
			0x13 => {
				self.inc_r16(R16::DE);
			}
			0x23 => {
				self.inc_r16(R16::HL);
			}
			0x33 => {
				self.inc_r16(R16::SP);
			}
			0x0B => {
				self.dec_r16(R16::BC);
			}
			0x1B => {
				self.dec_r16(R16::DE);
			}
			0x2B => {
				self.dec_r16(R16::HL);
			}
			0x3B => {
				self.dec_r16(R16::SP);
			}
			0x27 => {
				self.daa();
			}
			0x2F => {
				self.cpl();
			}
			0x3F => {
				self.ccf();
			}
			0x37 => {
				self.scf();
			}
			0x00 => {
				self.nop();
			}
			0x76 => {
				self.halt();
			}
			0x10 => {
				self.stop();
				self.fetch();
			}
			0xF3 => {
				self.di();
			}
			0xFB => {
				self.ei();
			}
			0x07 => {
				self.rlca();
			}
			0x17 => {
				self.rla();
			}
			0x0F => {
				self.rrca();
			}
			0x1F => {
				self.rra();
			}
			0xC3 => {
				self.jp_nn();
			}
			0xC2 => {
				self.jp_cc_nn(Flag::Z, false);
			}
			0xCA => {
				self.jp_cc_nn(Flag::Z, true);
			}
			0xD2 => {
				self.jp_cc_nn(Flag::C, false);
			}
			0xDA => {
				self.jp_cc_nn(Flag::C, true);
			}
			0xE9 => {
				self.pc = self.get_r16(&R16::HL);
			}
			0x18 => {
				self.jr_n();
			}
			0x20 => {
				self.jr_cc_n(Flag::Z, false);
			}
			0x28 => {
				self.jr_cc_n(Flag::Z, true);
			}
			0x30 => {
				self.jr_cc_n(Flag::C, false);
			}
			0x38 => {
				self.jr_cc_n(Flag::C, true);
			}
			0xCD => {
				self.call();
			}
			0xC4 => {
				self.call_cc_nn(Flag::Z, false);
			}
			0xCC => {
				self.call_cc_nn(Flag::Z, true);
			}
			0xD4 => {
				self.call_cc_nn(Flag::C, false);
			}
			0xDC => {
				self.call_cc_nn(Flag::C, true);
			}
			0xC7 => {
				self.rst(0x00);
			}
			0xCF => {
				self.rst(0x08);
			}
			0xD7 => {
				self.rst(0x10);
			}
			0xDF => {
				self.rst(0x18);
			}
			0xE7 => {
				self.rst(0x20);
			}
			0xEF => {
				self.rst(0x28);
			}
			0xF7 => {
				self.rst(0x30);
			}
			0xFF => {
				self.rst(0x38);
			}
			0xC9 => {
				self.ret();
			}
			0xC0 => {
				self.ret_cc(Flag::Z, false);
			}
			0xC8 => {
				self.ret_cc(Flag::Z, true);
			}
			0xD0 => {
				self.ret_cc(Flag::C, false);
			}
			0xD8 => {
				self.ret_cc(Flag::C, true);
			}
			0xD9 => {
				self.reti();
			}
			0xCB => match self.fetch() {
				0x30 => {
					self.swap_r8(R8::B);
				}
				0x31 => {
					self.swap_r8(R8::C);
				}
				0x32 => {
					self.swap_r8(R8::D);
				}
				0x33 => {
					self.swap_r8(R8::E);
				}
				0x34 => {
					self.swap_r8(R8::H);
				}
				0x35 => {
					self.swap_r8(R8::L);
				}
				0x36 => {
					self.swap_addr(Addr::HL);
				}
				0x37 => {
					self.swap_r8(R8::A);
				}
				0x00 => {
					self.rlc_r8(R8::B);
				}
				0x01 => {
					self.rlc_r8(R8::C);
				}
				0x02 => {
					self.rlc_r8(R8::D);
				}
				0x03 => {
					self.rlc_r8(R8::E);
				}
				0x04 => {
					self.rlc_r8(R8::H);
				}
				0x05 => {
					self.rlc_r8(R8::L);
				}
				0x06 => {
					self.rlc_addr(Addr::HL);
				}
				0x07 => {
					self.rlc_r8(R8::A);
				}
				0x10 => {
					self.rl_r8(R8::B);
				}
				0x11 => {
					self.rl_r8(R8::C);
				}
				0x12 => {
					self.rl_r8(R8::D);
				}
				0x13 => {
					self.rl_r8(R8::E);
				}
				0x14 => {
					self.rl_r8(R8::H);
				}
				0x15 => {
					self.rl_r8(R8::L);
				}
				0x16 => {
					self.rl_addr(Addr::HL);
				}
				0x17 => {
					self.rl_r8(R8::A);
				}
				0x08 => {
					self.rrc_r8(R8::B);
				}
				0x09 => {
					self.rrc_r8(R8::C);
				}
				0x0A => {
					self.rrc_r8(R8::D);
				}
				0x0B => {
					self.rrc_r8(R8::E);
				}
				0x0C => {
					self.rrc_r8(R8::H);
				}
				0x0D => {
					self.rrc_r8(R8::L);
				}
				0x0E => {
					self.rrc_addr(Addr::HL);
				}
				0x0F => {
					self.rrc_r8(R8::A);
				}
				0x18 => {
					self.rr_r8(R8::B);
				}
				0x19 => {
					self.rr_r8(R8::C);
				}
				0x1A => {
					self.rr_r8(R8::D);
				}
				0x1B => {
					self.rr_r8(R8::E);
				}
				0x1C => {
					self.rr_r8(R8::H);
				}
				0x1D => {
					self.rr_r8(R8::L);
				}
				0x1E => {
					self.rr_addr(Addr::HL);
				}
				0x1F => {
					self.rr_r8(R8::A);
				}
				0x20 => {
					self.sla_r8(R8::B);
				}
				0x21 => {
					self.sla_r8(R8::C);
				}
				0x22 => {
					self.sla_r8(R8::D);
				}
				0x23 => {
					self.sla_r8(R8::E);
				}
				0x24 => {
					self.sla_r8(R8::H);
				}
				0x25 => {
					self.sla_r8(R8::L);
				}
				0x26 => {
					self.sla_addr(Addr::HL);
				}
				0x27 => {
					self.sla_r8(R8::A);
				}
				0x28 => {
					self.sra_r8(R8::B);
				}
				0x29 => {
					self.sra_r8(R8::C);
				}
				0x2A => {
					self.sra_r8(R8::D);
				}
				0x2B => {
					self.sra_r8(R8::E);
				}
				0x2C => {
					self.sra_r8(R8::H);
				}
				0x2D => {
					self.sra_r8(R8::L);
				}
				0x2E => {
					self.sra_addr(Addr::HL);
				}
				0x2F => {
					self.sra_r8(R8::A);
				}
				0x38 => {
					self.srl_r8(R8::B);
				}
				0x39 => {
					self.srl_r8(R8::C);
				}
				0x3A => {
					self.srl_r8(R8::D);
				}
				0x3B => {
					self.srl_r8(R8::E);
				}
				0x3C => {
					self.srl_r8(R8::H);
				}
				0x3D => {
					self.srl_r8(R8::L);
				}
				0x3E => {
					self.srl_addr(Addr::HL);
				}
				0x3F => {
					self.srl_r8(R8::A);
				}
				0x40 => {
					self.bit_r8(0, R8::B);
				}
				0x41 => {
					self.bit_r8(0, R8::C);
				}
				0x42 => {
					self.bit_r8(0, R8::D);
				}
				0x43 => {
					self.bit_r8(0, R8::E);
				}
				0x44 => {
					self.bit_r8(0, R8::H);
				}
				0x45 => {
					self.bit_r8(0, R8::L);
				}
				0x46 => {
					self.bit_addr(0, Addr::HL);
				}
				0x47 => {
					self.bit_r8(0, R8::A);
				}
				0x48 => {
					self.bit_r8(1, R8::B);
				}
				0x49 => {
					self.bit_r8(1, R8::C);
				}
				0x4A => {
					self.bit_r8(1, R8::D);
				}
				0x4B => {
					self.bit_r8(1, R8::E);
				}
				0x4C => {
					self.bit_r8(1, R8::H);
				}
				0x4D => {
					self.bit_r8(1, R8::L);
				}
				0x4E => {
					self.bit_addr(1, Addr::HL);
				}
				0x4F => {
					self.bit_r8(1, R8::A);
				}
				0x50 => {
					self.bit_r8(2, R8::B);
				}
				0x51 => {
					self.bit_r8(2, R8::C);
				}
				0x52 => {
					self.bit_r8(2, R8::D);
				}
				0x53 => {
					self.bit_r8(2, R8::E);
				}
				0x54 => {
					self.bit_r8(2, R8::H);
				}
				0x55 => {
					self.bit_r8(2, R8::L);
				}
				0x56 => {
					self.bit_addr(2, Addr::HL);
				}
				0x57 => {
					self.bit_r8(2, R8::A);
				}
				0x58 => {
					self.bit_r8(3, R8::B);
				}
				0x59 => {
					self.bit_r8(3, R8::C);
				}
				0x5A => {
					self.bit_r8(3, R8::D);
				}
				0x5B => {
					self.bit_r8(3, R8::E);
				}
				0x5C => {
					self.bit_r8(3, R8::H);
				}
				0x5D => {
					self.bit_r8(3, R8::L);
				}
				0x5E => {
					self.bit_addr(3, Addr::HL);
				}
				0x5F => {
					self.bit_r8(3, R8::A);
				}
				0x60 => {
					self.bit_r8(4, R8::B);
				}
				0x61 => {
					self.bit_r8(4, R8::C);
				}
				0x62 => {
					self.bit_r8(4, R8::D);
				}
				0x63 => {
					self.bit_r8(4, R8::E);
				}
				0x64 => {
					self.bit_r8(4, R8::H);
				}
				0x65 => {
					self.bit_r8(4, R8::L);
				}
				0x66 => {
					self.bit_addr(4, Addr::HL);
				}
				0x67 => {
					self.bit_r8(4, R8::A);
				}
				0x68 => {
					self.bit_r8(5, R8::B);
				}
				0x69 => {
					self.bit_r8(5, R8::C);
				}
				0x6A => {
					self.bit_r8(5, R8::D);
				}
				0x6B => {
					self.bit_r8(5, R8::E);
				}
				0x6C => {
					self.bit_r8(5, R8::H);
				}
				0x6D => {
					self.bit_r8(5, R8::L);
				}
				0x6E => {
					self.bit_addr(5, Addr::HL);
				}
				0x6F => {
					self.bit_r8(5, R8::A);
				}
				0x70 => {
					self.bit_r8(6, R8::B);
				}
				0x71 => {
					self.bit_r8(6, R8::C);
				}
				0x72 => {
					self.bit_r8(6, R8::D);
				}
				0x73 => {
					self.bit_r8(6, R8::E);
				}
				0x74 => {
					self.bit_r8(6, R8::H);
				}
				0x75 => {
					self.bit_r8(6, R8::L);
				}
				0x76 => {
					self.bit_addr(6, Addr::HL);
				}
				0x77 => {
					self.bit_r8(6, R8::A);
				}
				0x78 => {
					self.bit_r8(7, R8::B);
				}
				0x79 => {
					self.bit_r8(7, R8::C);
				}
				0x7A => {
					self.bit_r8(7, R8::D);
				}
				0x7B => {
					self.bit_r8(7, R8::E);
				}
				0x7C => {
					self.bit_r8(7, R8::H);
				}
				0x7D => {
					self.bit_r8(7, R8::L);
				}
				0x7E => {
					self.bit_addr(7, Addr::HL);
				}
				0x7F => {
					self.bit_r8(7, R8::A);
				}
				0xC0 => {
					self.setb_r8(0, R8::B);
				}
				0xC1 => {
					self.setb_r8(0, R8::C);
				}
				0xC2 => {
					self.setb_r8(0, R8::D);
				}
				0xC3 => {
					self.setb_r8(0, R8::E);
				}
				0xC4 => {
					self.setb_r8(0, R8::H);
				}
				0xC5 => {
					self.setb_r8(0, R8::L);
				}
				0xC6 => {
					self.setb_addr(0, Addr::HL);
				}
				0xC7 => {
					self.setb_r8(0, R8::A);
				}
				0xC8 => {
					self.setb_r8(1, R8::B);
				}
				0xC9 => {
					self.setb_r8(1, R8::C);
				}
				0xCA => {
					self.setb_r8(1, R8::D);
				}
				0xCB => {
					self.setb_r8(1, R8::E);
				}
				0xCC => {
					self.setb_r8(1, R8::H);
				}
				0xCD => {
					self.setb_r8(1, R8::L);
				}
				0xCE => {
					self.setb_addr(1, Addr::HL);
				}
				0xCF => {
					self.setb_r8(1, R8::A);
				}
				0xD0 => {
					self.setb_r8(2, R8::B);
				}
				0xD1 => {
					self.setb_r8(2, R8::C);
				}
				0xD2 => {
					self.setb_r8(2, R8::D);
				}
				0xD3 => {
					self.setb_r8(2, R8::E);
				}
				0xD4 => {
					self.setb_r8(2, R8::H);
				}
				0xD5 => {
					self.setb_r8(2, R8::L);
				}
				0xD6 => {
					self.setb_addr(2, Addr::HL);
				}
				0xD7 => {
					self.setb_r8(2, R8::A);
				}
				0xD8 => {
					self.setb_r8(3, R8::B);
				}
				0xD9 => {
					self.setb_r8(3, R8::C);
				}
				0xDA => {
					self.setb_r8(3, R8::D);
				}
				0xDB => {
					self.setb_r8(3, R8::E);
				}
				0xDC => {
					self.setb_r8(3, R8::H);
				}
				0xDD => {
					self.setb_r8(3, R8::L);
				}
				0xDE => {
					self.setb_addr(3, Addr::HL);
				}
				0xDF => {
					self.setb_r8(3, R8::A);
				}
				0xE0 => {
					self.setb_r8(4, R8::B);
				}
				0xE1 => {
					self.setb_r8(4, R8::C);
				}
				0xE2 => {
					self.setb_r8(4, R8::D);
				}
				0xE3 => {
					self.setb_r8(4, R8::E);
				}
				0xE4 => {
					self.setb_r8(4, R8::H);
				}
				0xE5 => {
					self.setb_r8(4, R8::L);
				}
				0xE6 => {
					self.setb_addr(4, Addr::HL);
				}
				0xE7 => {
					self.setb_r8(4, R8::A);
				}
				0xE8 => {
					self.setb_r8(5, R8::B);
				}
				0xE9 => {
					self.setb_r8(5, R8::C);
				}
				0xEA => {
					self.setb_r8(5, R8::D);
				}
				0xEB => {
					self.setb_r8(5, R8::E);
				}
				0xEC => {
					self.setb_r8(5, R8::H);
				}
				0xED => {
					self.setb_r8(5, R8::L);
				}
				0xEE => {
					self.setb_addr(5, Addr::HL);
				}
				0xEF => {
					self.setb_r8(5, R8::A);
				}
				0xF0 => {
					self.setb_r8(6, R8::B);
				}
				0xF1 => {
					self.setb_r8(6, R8::C);
				}
				0xF2 => {
					self.setb_r8(6, R8::D);
				}
				0xF3 => {
					self.setb_r8(6, R8::E);
				}
				0xF4 => {
					self.setb_r8(6, R8::H);
				}
				0xF5 => {
					self.setb_r8(6, R8::L);
				}
				0xF6 => {
					self.setb_addr(6, Addr::HL);
				}
				0xF7 => {
					self.setb_r8(6, R8::A);
				}
				0xF8 => {
					self.setb_r8(7, R8::B);
				}
				0xF9 => {
					self.setb_r8(7, R8::C);
				}
				0xFA => {
					self.setb_r8(7, R8::D);
				}
				0xFB => {
					self.setb_r8(7, R8::E);
				}
				0xFC => {
					self.setb_r8(7, R8::H);
				}
				0xFD => {
					self.setb_r8(7, R8::L);
				}
				0xFE => {
					self.setb_addr(7, Addr::HL);
				}
				0xFF => {
					self.setb_r8(7, R8::A);
				}
				0x80 => {
					self.res_r8(0, R8::B);
				}
				0x81 => {
					self.res_r8(0, R8::C);
				}
				0x82 => {
					self.res_r8(0, R8::D);
				}
				0x83 => {
					self.res_r8(0, R8::E);
				}
				0x84 => {
					self.res_r8(0, R8::H);
				}
				0x85 => {
					self.res_r8(0, R8::L);
				}
				0x86 => {
					self.res_addr(0, Addr::HL);
				}
				0x87 => {
					self.res_r8(0, R8::A);
				}
				0x88 => {
					self.res_r8(1, R8::B);
				}
				0x89 => {
					self.res_r8(1, R8::C);
				}
				0x8A => {
					self.res_r8(1, R8::D);
				}
				0x8B => {
					self.res_r8(1, R8::E);
				}
				0x8C => {
					self.res_r8(1, R8::H);
				}
				0x8D => {
					self.res_r8(1, R8::L);
				}
				0x8E => {
					self.res_addr(1, Addr::HL);
				}
				0x8F => {
					self.res_r8(1, R8::A);
				}
				0x90 => {
					self.res_r8(2, R8::B);
				}
				0x91 => {
					self.res_r8(2, R8::C);
				}
				0x92 => {
					self.res_r8(2, R8::D);
				}
				0x93 => {
					self.res_r8(2, R8::E);
				}
				0x94 => {
					self.res_r8(2, R8::H);
				}
				0x95 => {
					self.res_r8(2, R8::L);
				}
				0x96 => {
					self.res_addr(2, Addr::HL);
				}
				0x97 => {
					self.res_r8(2, R8::A);
				}
				0x98 => {
					self.res_r8(3, R8::B);
				}
				0x99 => {
					self.res_r8(3, R8::C);
				}
				0x9A => {
					self.res_r8(3, R8::D);
				}
				0x9B => {
					self.res_r8(3, R8::E);
				}
				0x9C => {
					self.res_r8(3, R8::H);
				}
				0x9D => {
					self.res_r8(3, R8::L);
				}
				0x9E => {
					self.res_addr(3, Addr::HL);
				}
				0x9F => {
					self.res_r8(3, R8::A);
				}
				0xA0 => {
					self.res_r8(4, R8::B);
				}
				0xA1 => {
					self.res_r8(4, R8::C);
				}
				0xA2 => {
					self.res_r8(4, R8::D);
				}
				0xA3 => {
					self.res_r8(4, R8::E);
				}
				0xA4 => {
					self.res_r8(4, R8::H);
				}
				0xA5 => {
					self.res_r8(4, R8::L);
				}
				0xA6 => {
					self.res_addr(4, Addr::HL);
				}
				0xA7 => {
					self.res_r8(4, R8::A);
				}
				0xA8 => {
					self.res_r8(5, R8::B);
				}
				0xA9 => {
					self.res_r8(5, R8::C);
				}
				0xAA => {
					self.res_r8(5, R8::D);
				}
				0xAB => {
					self.res_r8(5, R8::E);
				}
				0xAC => {
					self.res_r8(5, R8::H);
				}
				0xAD => {
					self.res_r8(5, R8::L);
				}
				0xAE => {
					self.res_addr(5, Addr::HL);
				}
				0xAF => {
					self.res_r8(5, R8::A);
				}
				0xB0 => {
					self.res_r8(6, R8::B);
				}
				0xB1 => {
					self.res_r8(6, R8::C);
				}
				0xB2 => {
					self.res_r8(6, R8::D);
				}
				0xB3 => {
					self.res_r8(6, R8::E);
				}
				0xB4 => {
					self.res_r8(6, R8::H);
				}
				0xB5 => {
					self.res_r8(6, R8::L);
				}
				0xB6 => {
					self.res_addr(6, Addr::HL);
				}
				0xB7 => {
					self.res_r8(6, R8::A);
				}
				0xB8 => {
					self.res_r8(7, R8::B);
				}
				0xB9 => {
					self.res_r8(7, R8::C);
				}
				0xBA => {
					self.res_r8(7, R8::D);
				}
				0xBB => {
					self.res_r8(7, R8::E);
				}
				0xBC => {
					self.res_r8(7, R8::H);
				}
				0xBD => {
					self.res_r8(7, R8::L);
				}
				0xBE => {
					self.res_addr(7, Addr::HL);
				}
				0xBF => {
					self.res_r8(7, R8::A);
				}
			},
			0xD3 => {
				panic!("Invalid opcode {}", opcode);
			}
			0xE3 => {
				panic!("Invalid opcode {}", opcode);
			}
			0xE4 => {
				panic!("Invalid opcode {}", opcode);
			}
			0xF4 => {
				panic!("Invalid opcode {}", opcode);
			}
			0xDB => {
				panic!("Invalid opcode {}", opcode);
			}
			0xEB => {
				panic!("Invalid opcode {}", opcode);
			}
			0xEC => {
				panic!("Invalid opcode {}", opcode);
			}
			0xFC => {
				panic!("Invalid opcode {}", opcode);
			}
			0xDD => {
				panic!("Invalid opcode {}", opcode);
			}
			0xED => {
				panic!("Invalid opcode {}", opcode);
			}
			0xFD => {
				panic!("Invalid opcode {}", opcode);
			}
		}
	}
}

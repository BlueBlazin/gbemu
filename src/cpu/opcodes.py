from pprint import pprint
import re


template_header = """\
/// This is a generated file. Do not modify.
/// See `opcodes.py` to understand how this file was generated.
use crate::cpu::{Cpu, R8, R16, Addr, Flag};

impl Cpu {
"""
template_footer = "}\n"

template_match_start = "match opcode {\n"
template_match_end = "}\n"

template_decode_exec_start = "pub fn decode_exec(&mut self, opcode: u8) {\n"
template_decode_exec_end = "}\n"

template_match_arm_pat = "{} => {{\n"
template_match_arm_line = "\t{}\n"
template_match_arm_end = "}\n"

template_match_arm_prefix_start = "0xCB => match self.fetch() {\n"
template_match_arm_prefix_end = "},\n"


INDENT = 0


def indent_block(inner):
    def wrapper(*args, **kwargs):
        global INDENT
        INDENT += 1
        inner(*args, **kwargs)
        INDENT -= 1
    return wrapper


def gen_opcodes(start, step, choices):
    for i, A in enumerate(choices):
        if isinstance(A, tuple):
            yield (f"0x{start + step * i:02X}", *A)
        else:
            yield (f"0x{start + step * i:02X}", A)


@indent_block
def ld_A_n(opcode: str, register: str, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_imm8();"))
    f.write("\t" * INDENT + template_match_arm_line.format(f"self.set_r8(R8::{register}, value);"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def ld_r1_r2(opcode: str, r1: str, r2: str, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))

    if res := re.search(r"\(([A-Z][A-Z])\)", r2):
        f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_addr(&Addr::{res.group(1)});"))
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.set_r8(R8::{r1}, value);"))
    elif res := re.search(r"\(([A-Z][A-Z])\)", r1):
        f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_r8(&R8::{r2});"))
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.set_addr(Addr::{res.group(1)}, value);"))
    else:
        f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_r8(&R8::{r2});"))
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.set_r8(R8::{r1}, value);"))

    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def alu_r1_r2(opcode: str, r1: str, r2: str, method: str, f, r16: bool = False):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    enum = "R16" if r16 else "R8"

    if res := re.search(r"\(([A-Z][A-Z])\)", r2):
        f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_addr(&Addr::{res.group(1)});"))
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_imm({enum}::{r1}, value);"))
    else:
        # f.write("\t" * INDENT + template_match_arm_line.format(f"let value = self.get_r8(&R8::{r2});"))
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}({enum}::{r1}, {enum}::{r2});"))

    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def alu_inc_dec(opcode: str, r: str, method: str, f, r16: bool = False):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    enum = "R16" if r16 else "R8"

    if res := re.search(r"\(([A-Z][A-Z])\)", r):
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_addr(Addr::{res.group(1)});"))
    else:
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_{enum.lower()}({enum}::{r});"))

    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def ld_AA_nn(opcode: str, register: str, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    f.write("\t" * INDENT + template_match_arm_line.format(f"self.set_r16_imm(R16::{register});"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def custom_match_arm(opcode, body, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    if type(body) == list:
        for line in body:
            f.write("\t" * INDENT + template_match_arm_line.format(line))
    else:
        f.write("\t" * INDENT + template_match_arm_line.format(body))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def push_AA(opcode: str, register: str):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    f.write("\t" * INDENT + template_match_arm_line.format(f"self.push_r16(R16::{register});"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def pop_AA(opcode: str, register: str):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    f.write("\t" * INDENT + template_match_arm_line.format(f"self.pop_r16(R16::{register});"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def prefixed(opcode: str, r: str, method: str, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    if res := re.search(r"\(([A-Z][A-Z])\)", r):
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_addr(Addr::{res.group(1)});"))
    else:
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_r8(R8::{r});"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def bitops(opcode: str, r: str, method: str, b: int, f):
    f.write("\t" * INDENT + template_match_arm_pat.format(opcode))
    # f.write("\t" * INDENT + template_match_arm_line.format(f"let b = self.get_imm8();"))
    if res := re.search(r"\(([A-Z][A-Z])\)", r):
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_addr({b}, Addr::{res.group(1)});"))
    else:
        f.write("\t" * INDENT + template_match_arm_line.format(f"self.{method}_r8({b}, R8::{r});"))
    f.write("\t" * INDENT + template_match_arm_end)


@indent_block
def prefix_opcodes(f):
    f.write("\t" * INDENT + template_match_arm_prefix_start)

    ########################################################################
    #  Instructions
    ########################################################################

    # 3.3.5. Miscellaneous

    # 1.
    for op, r in gen_opcodes(0x30, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "swap", f)

    # 3.3.6. Rotates & Shifts

    # 5.
    for op, r in gen_opcodes(0x00, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "rlc", f)

    # 6.
    for op, r in gen_opcodes(0x10, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "rl", f)

    # 7.
    for op, r in gen_opcodes(0x08, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "rrc", f)

    # 8.
    for op, r in gen_opcodes(0x18, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "rr", f)

    # 9.
    for op, r in gen_opcodes(0x20, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "sla", f)

    # 10.
    for op, r in gen_opcodes(0x28, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "sra", f)

    # 11.
    for op, r in gen_opcodes(0x38, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        prefixed(op, r, "srl", f)

    # 3.3.7. Bit Opcodes

    # 1.
    for b in range(8):
        for op, r in gen_opcodes(0x40 + b * 8, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
            bitops(op, r, "bit", b, f)

    # 2.
    for b in range(8):
        for op, r in gen_opcodes(0xC0 + b * 8, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
            bitops(op, r, "setb", b, f)

    # 3.
    for b in range(8):
        for op, r in gen_opcodes(0x80 + b * 8, 0x1, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
            bitops(op, r, "res", b, f)

    # default
    # custom_match_arm("_", "self.nop();", f)


    ########################################################################


    f.write("\t" * INDENT + template_match_arm_prefix_end)


def main(f):
    global INDENT
    f.write(template_header)
    INDENT += 1
    f.write("\t" * INDENT + template_decode_exec_start)
    INDENT += 1
    f.write("\t" * INDENT + template_match_start)

    ########################################################################
    #  Instructions
    ########################################################################

    # 8 bit immediate loads
    for op, r in gen_opcodes(0x06, 0x08, ["B", "C", "D", "E", "H", "L"]):
        ld_A_n(op, r, f)

    # load r2 into A
    custom_match_arm("0x7F",
                     ["let value = self.get_r8(&R8::A);",
                      "self.set_r8(R8::A, value);"],
                     f)

    for op, r2 in gen_opcodes(0x78, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "A", r2, f)
    
    for op, r2 in gen_opcodes(0x40, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "B", r2, f)

    for op, r2 in gen_opcodes(0x48, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "C", r2, f)

    for op, r2 in gen_opcodes(0x50, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "D", r2, f)

    for op, r2 in gen_opcodes(0x58, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "E", r2, f)

    for op, r2 in gen_opcodes(0x60, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "H", r2, f)

    for op, r2 in gen_opcodes(0x68, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)"]):
        ld_r1_r2(op, "L", r2, f)

    for op, r2 in gen_opcodes(0x70, 0x01, ["B", "C", "D", "E", "H", "L"]):
        ld_r1_r2(op, "(HL)", r2, f)

    custom_match_arm("0x36",
                     ["let value = self.get_imm8();",
                      "self.set_addr(Addr::HL, value);"],
                      f)

    ld_r1_r2("0x0A", "A", "(BC)", f)
    ld_r1_r2("0x1A", "A", "(DE)", f)

    custom_match_arm("0xFA",
                     ["let addr = self.get_imm16();",
                     "let value = self.get_addr_imm(addr);",
                     "self.set_r8(R8::A, value);"],
                     f)
    
    ld_A_n("0x3E", "A", f)

    for op, r2 in gen_opcodes(0x47, 0x08, ["B", "C", "D", "E", "H", "L"]):
        ld_r1_r2(op, r2, "A", f)
    
    ld_r1_r2("0x02", "(BC)", "A", f)
    ld_r1_r2("0x12", "(DE)", "A", f)
    ld_r1_r2("0x77", "(HL)", "A", f)

    custom_match_arm("0xEA",
                     ["let addr = self.get_imm16();",
                      "let value = self.get_r8(&R8::A);",
                      "self.set_addr_imm(addr, value);"],
                     f)
    
    # 5.
    custom_match_arm("0xF2", 
                     ["let addr = 0xFF00 | (self.get_r8(&R8::C) as u16);",
                      "let value = self.get_addr_imm(addr);",
                      "self.set_r8(R8::A, value);"],
                     f)
    
    # 6.
    custom_match_arm("0xE2", 
                     ["let addr = 0xFF00 | (self.get_r8(&R8::C) as u16);",
                      "self.set_addr_imm(addr, self.get_r8(&R8::A));"],
                     f)

    # 7. 8. 9.
    custom_match_arm("0x3A",
                     ["let value = self.get_addr_dec();",
                      "self.set_r8(R8::A, value);"],
                     f)
    
    # 10. 11. 12.
    custom_match_arm("0x32",
                     ["let value = self.get_r8(&R8::A);",
                      "self.set_addr_dec(value);"],
                     f)
    
    # 13. 14. 15.
    custom_match_arm("0x2A",
                     ["let value = self.get_addr_inc();",
                      "self.set_r8(R8::A, value);"],
                     f)

    # 16. 17. 18.
    custom_match_arm("0x22",
                     ["let value = self.get_r8(&R8::A);",
                      "self.set_addr_inc(value);"],
                     f)

    # 19.
    custom_match_arm("0xE0",
                     ["let addr = 0xFF00 | (self.fetch() as u16);",
                      "let value = self.get_r8(&R8::A);",
                      "self.set_addr_imm(addr, value);"],
                     f)

    # 20.
    custom_match_arm("0xF0",
                     ["let addr = 0xFF00 | (self.fetch() as u16);",
                      "let value = self.get_addr_imm(addr);",
                      "self.set_r8(R8::A, value);"],
                     f)

    # 3.3.2. 16-Bit Loads

    # 1.
    for op, r in gen_opcodes(0x01, 0x10, ["BC", "DE", "HL", "SP"]):
        ld_AA_nn(op, r, f)

    # 2.
    custom_match_arm("0xF9",
                     ["let value = self.get_r16(&R16::HL);",
                      "self.set_r16(R16::SP, value);"],
                     f)

    # 3. 4.
    custom_match_arm("0xF8",
                     "self.add_sp_imm_hl();",
                     f)
    
    # 5.
    custom_match_arm("0x08",
                     ["let value = self.get_r16(&R16::SP);",
                      "let addr = self.get_imm16();",
                      "self.set_addr_imm(addr, (value & 0x00FF) as u8);",
                      "self.set_addr_imm(addr.wrapping_add(0x1), ((value & 0xFF00) >> 8) as u8);"],
                     f)

    # 6.
    for op, r in gen_opcodes(0xC5, 0x10, ["BC", "DE", "HL", "AF"]):
        push_AA(op, r)

    # 7.
    for op, r in gen_opcodes(0xC1, 0x10, ["BC", "DE", "HL", "AF"]):
        pop_AA(op, r)

    # 3.3.3. 8-Bit ALU

    # 1.
    for op, r2 in gen_opcodes(0x80, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "add_r8", f)

    custom_match_arm("0xC6",
                     ["let value = self.get_imm8();",
                      "self.add_r8_imm(R8::A, value);"],
                     f)

    # 2.
    for op, r2 in gen_opcodes(0x88, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "adc_r8", f)

    custom_match_arm("0xCE",
                     ["let value = self.get_imm8();",
                      "self.adc_r8_imm(R8::A, value);"],
                     f)

    # 3.
    for op, r2 in gen_opcodes(0x90, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "sub_r8", f)

    custom_match_arm("0xD6",
                     ["let value = self.get_imm8();",
                      "self.sub_r8_imm(R8::A, value)"],
                     f)

    # 4.
    for op, r2 in gen_opcodes(0x98, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "sbc_r8", f)

    custom_match_arm("0xDE",
                     ["let value = self.get_imm8();",
                      "self.sbc_r8_imm(R8::A, value)"],
                     f)

    # 5.
    for op, r2 in gen_opcodes(0xA0, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "and_r8", f)

    custom_match_arm("0xE6",
                     ["let value = self.get_imm8();",
                      "self.and_r8_imm(R8::A, value)"],
                     f)

    # 6.
    for op, r2 in gen_opcodes(0xB0, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "or_r8", f)

    custom_match_arm("0xF6",
                     ["let value = self.get_imm8();",
                      "self.or_r8_imm(R8::A, value)"],
                     f)

    # 7.
    for op, r2 in gen_opcodes(0xA8, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "xor_r8", f)

    custom_match_arm("0xEE",
                     ["let value = self.get_imm8();",
                      "self.xor_r8_imm(R8::A, value)"],
                     f)

    # 8.
    for op, r2 in gen_opcodes(0xB8, 0x01, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_r1_r2(op, "A", r2, "cp_r8", f)

    custom_match_arm("0xFE",
                     ["let value = self.get_imm8();",
                      "self.cp_r8_imm(R8::A, value)"],
                     f)

    # 9.
    for op, r in gen_opcodes(0x04, 0x08, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_inc_dec(op, r, "inc", f)


    # 10.
    for op, r in gen_opcodes(0x05, 0x08, ["B", "C", "D", "E", "H", "L", "(HL)", "A"]):
        alu_inc_dec(op, r, "dec", f)

    # 3.3.4. 16-Bit Arithmetic

    # 1.
    for op, r2 in gen_opcodes(0x09, 0x10, ["BC", "DE", "HL", "SP"]):
        alu_r1_r2(op, "HL", r2, "add_r16", f, r16=True)

    # 2.
    custom_match_arm("0xE8",
                     "self.add_sp_imm();",
                     f)

    # 3.
    for op, r in gen_opcodes(0x03, 0x10, ["BC", "DE", "HL", "SP"]):
        alu_inc_dec(op, r, "inc", f, r16=True)

    # 4.
    for op, r in gen_opcodes(0x0B, 0x10, ["BC", "DE", "HL", "SP"]):
        alu_inc_dec(op, r, "dec", f, r16=True)

    # 3.3.5. Miscellaneous

    # 1.
    # Deferred

    # 2.
    custom_match_arm("0x27", "self.daa();", f)

    # 3.
    custom_match_arm("0x2F", "self.cpl();", f)

    # 4.
    custom_match_arm("0x3F", "self.ccf();", f)

    # 5.
    custom_match_arm("0x37", "self.scf();", f)

    # 6.
    custom_match_arm("0x00", "self.nop();", f)

    # 7.
    custom_match_arm("0x76", "self.halt();", f)

    # 8.
    custom_match_arm("0x10", ["self.stop();", "self.fetch();"], f)

    # 9.
    custom_match_arm("0xF3", "self.di();", f)

    # 10.
    custom_match_arm("0xFB", "self.ei();", f)


    # 3.3.6. Rotates & Shifts

    # 1.
    custom_match_arm("0x07", "self.rlca();", f)

    # 2.
    custom_match_arm("0x17", "self.rla();", f)

    # 3.
    custom_match_arm("0x0F", "self.rrca();", f)

    # 4.
    custom_match_arm("0x1F", "self.rra();", f)

    # 3.3.8. Jumps

    # 1.
    custom_match_arm("0xC3", "self.jp_nn();", f)

    # 2.
    for op, flag, _set in gen_opcodes(0xC2, 0x08, [("Z", "false"), ("Z", "true"), ("C", "false"), ("C", "true")]):
        custom_match_arm(op, f"self.jp_cc_nn(Flag::{flag}, {_set});", f)

    # 3.
    custom_match_arm("0xE9",
                     ["self.pc = self.get_r16(&R16::HL);"],
                     f)

    # 4.
    custom_match_arm("0x18", "self.jr_n();", f)

    # 5.
    for op, flag, _set in gen_opcodes(0x20, 0x08, [("Z", "false"), ("Z", "true"), ("C", "false"), ("C", "true")]):
        custom_match_arm(op, f"self.jr_cc_n(Flag::{flag}, {_set});", f)

    # 3.3.9. Calls

    # 1.
    custom_match_arm("0xCD", "self.call();", f)

    # 2.
    for op, flag, _set in gen_opcodes(0xC4, 0x08, [("Z", "false"), ("Z", "true"), ("C", "false"), ("C", "true")]):
        custom_match_arm(op, f"self.call_cc_nn(Flag::{flag}, {_set});", f)

    # 3.3.10. Restarts

    # 1.
    for op, value in gen_opcodes(0xC7, 0x08, ["0x00", "0x08", "0x10", "0x18", "0x20", "0x28", "0x30", "0x38"]):
        custom_match_arm(op, f"self.rst({value});", f)

    # 3.3.11. Returns

    # 1.
    custom_match_arm("0xC9", "self.ret();", f)

    # 2.
    for op, flag, _set in gen_opcodes(0xC0, 0x08, [("Z", "false"), ("Z", "true"), ("C", "false"), ("C", "true")]):
        custom_match_arm(op, f"self.ret_cc(Flag::{flag}, {_set});", f)

    # 3.
    custom_match_arm("0xD9", "self.reti();", f)


    # Prefixed Instructions
    prefix_opcodes(f)

    # invalid opcodes
    for op in ["0xD3", "0xE3", "0xE4", "0xF4", "0xDB", "0xEB", "0xEC", "0xFC", "0xDD", "0xED", "0xFD"]:
        custom_match_arm(op,
                        ['panic!("Invalid opcode {}", opcode);'],
                        f)

    # default
    # custom_match_arm("_", "self.nop();", f)


    ########################################################################

    f.write("\t" * INDENT + template_match_end)
    INDENT -= 1
    f.write("\t" * INDENT + template_decode_exec_end)
    INDENT -= 1
    f.write(template_footer)


if __name__ == "__main__":
    with open('opcodes.rs', 'w') as f:
        main(f)

use gbemu::cpu::Cpu;
use glob::glob;
use std::fs;

const MAX_FRAMES: usize = 60 * 60;

fn main() {
    for entry in glob("tmproms/*.gbc").unwrap() {
        if let Ok(path) = entry {
            let rom = fs::read(path).unwrap();
            let mut cpu = Cpu::new(rom);
            let mut cycles = 0;
            for _ in 0..MAX_FRAMES {
                cpu.frame();
            }
            let screen = cpu.mmu.gpu.lcd;
            println!("{:?}", screen);
        }
        break;
    }
}

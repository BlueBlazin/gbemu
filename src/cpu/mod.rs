pub mod cpu;
pub mod opcodes;

#[derive(Debug, PartialEq)]
pub enum EmulationMode {
    Dmg,
    Cgb,
}

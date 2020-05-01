pub mod cpu;
pub mod opcodes;

#[derive(Debug, PartialEq, Clone)]
pub enum EmulationMode {
    Dmg,
    Cgb,
}

pub mod apu;
pub mod pulse;
pub mod queue;
pub mod square;
pub mod wave;

pub struct AudioRegisters {
    nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,
}

impl Default for AudioRegisters {
    fn default() -> Self {
        AudioRegisters {
            nrx0: 0,
            nrx1: 0,
            nrx2: 0,
            nrx3: 0,
            nrx4: 0,
        }
    }
}

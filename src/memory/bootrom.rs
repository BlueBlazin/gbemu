const BOOTROM: [u8; 256] = [0; 256]; // Placeholder for actual boot rom data

pub struct Bootrom {
    pub bootrom: [u8; 256],
    pub is_active: bool,
}

impl Bootrom {
    pub fn new() -> Self {
        Bootrom {
            bootrom: BOOTROM,
            is_active: false,
        }
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn get_byte(&self, addr: usize) -> u8 {
        self.bootrom[addr]
    }
}

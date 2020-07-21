pub struct Bootrom {
    pub bootrom: Vec<u8>,
    pub is_active: bool,
}

impl Bootrom {
    pub fn new() -> Self {
        Bootrom {
            bootrom: vec![], // Placeholder for Boot ROM data
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

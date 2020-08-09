pub struct Sprite {
    pub y: u8,
    pub x: u8,
    pub number: u16,
    pub obj_to_bg_prio: u8,
    pub mirror_vertical: bool,
    pub mirror_horizontal: bool,
    pub obp1: bool,
    pub vram_bank: usize,
    pub obp_num: u8,
}

impl From<&[u8]> for Sprite {
    fn from(bytes: &[u8]) -> Sprite {
        Sprite {
            y: bytes[0],
            x: bytes[1],
            number: bytes[2] as u16,
            obj_to_bg_prio: bytes[3] & 0x80,
            mirror_vertical: (bytes[3] & 0x40) != 0,
            mirror_horizontal: (bytes[3] & 0x20) != 0,
            obp1: (bytes[3] & 0x10) != 0,
            vram_bank: ((bytes[3] & 0x08) >> 3) as usize,
            obp_num: bytes[3] & 0x07,
        }
    }
}

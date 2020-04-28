use crate::cpu::EmulationMode;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

const VRAM_BANK_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;
const PALETTE_RAM_SIZE: usize = 0x40;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const SCREEN_DEPTH: usize = 4;
const VRAM_OFFSET: u16 = 0x8000;
const OAM_OFFSET: u16 = 0xFE00;

macro_rules! bit {
    ( $upper:expr , $lower:expr , $mask:expr ) => {
        ((((($upper & $mask) != 0) as u8) << 1) | ((($lower & $mask) != 0) as u8))
    };
}

#[derive(Debug, PartialEq)]
pub enum GpuMode {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
}

impl From<&GpuMode> for u8 {
    fn from(mode: &GpuMode) -> u8 {
        match mode {
            GpuMode::HBlank => 0,
            GpuMode::VBlank => 1,
            GpuMode::OamSearch => 2,
            GpuMode::PixelTransfer => 3,
        }
    }
}

#[derive(Eq)]
struct Sprite {
    pub y: i32,
    pub x: i32,
    pub number: u16,
    pub has_priority: bool,
    pub mirror_vertical: bool,
    pub mirror_horizontal: bool,
    pub obp1: bool,
    pub vram_bank: usize,
    pub obp_num: usize,
}

impl Ord for Sprite {
    fn cmp(&self, other: &Self) -> Ordering {
        self.x.cmp(&other.x)
    }
}

impl PartialOrd for Sprite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Sprite {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
    }
}

impl From<&[u8]> for Sprite {
    fn from(bytes: &[u8]) -> Sprite {
        Sprite {
            y: bytes[0] as u16 as i32 - 16,
            x: bytes[1] as u16 as i32 - 8,
            number: bytes[2] as u16,
            has_priority: (bytes[3] & 0x80) == 0,
            mirror_vertical: (bytes[3] & 0x40) != 0,
            mirror_horizontal: (bytes[3] & 0x20) != 0,
            obp1: (bytes[3] & 0x10) != 0,
            vram_bank: ((bytes[3] & 0x08) >> 3) as usize,
            obp_num: (bytes[3] & 0x07) as usize,
        }
    }
}

struct BgAttr {
    bgp_num: usize,
    vram_bank: usize,
    mirror_horizontal: bool,
    mirror_vertical: bool,
    has_priority: bool,
}

impl From<u8> for BgAttr {
    fn from(value: u8) -> Self {
        Self {
            bgp_num: (value & 0x07) as usize,
            vram_bank: if (value & 0x08) == 0 { 0 } else { 1 },
            mirror_horizontal: (value & 0x20) != 0,
            mirror_vertical: (value & 0x40) != 0,
            has_priority: (value & 0x80) != 0,
        }
    }
}

#[derive(PartialEq)]
enum PixelType {
    BgColor0,
    BgColorOpaque,
}

pub struct Gpu {
    pub screen: Vec<u8>,
    pub vram0: Vec<u8>,
    pub vram1: Vec<u8>,
    pub bgp_ram: Vec<u8>,
    pub obp_ram: Vec<u8>,
    bgp_idx: u8,
    bgp_auto_incr: bool,
    obp_idx: u8,
    obp_auto_incr: bool,
    emu_mode: EmulationMode,
    oam: Vec<u8>,
    pixel_types: Vec<PixelType>,
    scroll_x: u8,
    scroll_y: u8,
    window_x: u8,
    window_y: u8,
    lcd_enable: u8,
    win_tilemap_sel: u8,
    window_enable: u8,
    tiledata_sel: u8,
    bg_tilemap_sel: u8,
    obj_size: u8,
    obj_enable: u8,
    bg_display: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    pub mode: GpuMode,
    ly: u8,
    lyc: u8,
    lyc_int: u8,
    oam_int: u8,
    vblank_int: u8,
    hblank_int: u8,
    coincident: u8,
    clock: usize,
    pub request_vblank_int: bool,
    pub request_lcd_int: bool,
    pub gdma_active: bool,
    vram_bank: usize,
}

impl Gpu {
    pub fn new(emu_mode: EmulationMode) -> Self {
        let mut pixel_types = vec![];
        for _ in 0..SCREEN_WIDTH {
            pixel_types.push(PixelType::BgColor0);
        }

        Gpu {
            screen: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH * SCREEN_DEPTH],
            vram0: vec![0; VRAM_BANK_SIZE],
            vram1: vec![0; VRAM_BANK_SIZE],
            bgp_ram: vec![0; PALETTE_RAM_SIZE],
            obp_ram: vec![0; PALETTE_RAM_SIZE],
            oam: vec![0; OAM_SIZE],
            bgp_idx: 0,
            bgp_auto_incr: false,
            obp_idx: 0,
            obp_auto_incr: false,
            emu_mode,
            pixel_types,
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            lcd_enable: 0,
            win_tilemap_sel: 1,
            window_enable: 0,
            tiledata_sel: 0,
            bg_tilemap_sel: 1,
            obj_size: 0,
            obj_enable: 0,
            bg_display: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            mode: GpuMode::OamSearch,
            ly: 0,
            lyc: 0,
            lyc_int: 0,
            oam_int: 0,
            vblank_int: 0,
            hblank_int: 0,
            coincident: 0,
            clock: 0,
            request_vblank_int: false,
            request_lcd_int: false,
            gdma_active: false,
            vram_bank: 0,
        }
    }

    pub fn screen(&self) -> *const u8 {
        self.screen.as_ptr()
    }

    fn draw_line(&mut self) {
        for i in 0..SCREEN_WIDTH {
            self.pixel_types[i] = PixelType::BgColor0;
            let ly = self.ly as usize;
            self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 0] = 255;
            self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 1] = 255;
            self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 2] = 255;
            self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 3] = 255;
        }

        self.draw_line_bg();
        self.draw_line_sprites();
    }

    fn draw_line_bg(&mut self) {
        for i in 0..SCREEN_WIDTH {
            if self.is_win_enabled() && self.is_win_pixel(i) {
                self.put_win_pixel(i);
            } else if self.bg_display != 0 {
                self.put_bg_pixel(i);
            }
        }
    }

    #[inline]
    fn is_win_enabled(&self) -> bool {
        (self.window_enable != 0) && (self.window_x < 167) && (self.window_y < 144)
    }

    #[inline]
    fn is_win_pixel(&self, i: usize) -> bool {
        self.window_x <= (i + 7) as u8 && self.window_y <= self.ly
    }

    fn put_win_pixel(&mut self, i: usize) {
        let (lx, ly) = (i as u8, self.ly);
        let (wx, wy) = (self.window_x, self.window_y);

        // get idx of the coincident window tile
        let base_addr = self.base_tilemap_addr(self.win_tilemap_sel);
        let tilemap_offset = ((ly - wy) as usize / 8) * 32 + (i + 7 - wx as usize) / 8;
        let tilemap_addr = base_addr + tilemap_offset as u16;
        let tile_idx = self.get_byte(tilemap_addr);

        // get addr of tile
        let addr = self.tiledata_addr(self.tiledata_sel, tile_idx);

        // set pixel value
        let row = ((ly - wy) % 8) as u16;
        let col = (lx + 7 - wx) % 8;
        match self.emu_mode {
            EmulationMode::Dmg => self.set_bg_pixel(addr, row, col, i),
            EmulationMode::Cgb => self.set_cgb_bg_pixel(addr, row, col, i, tilemap_addr),
        }
    }

    fn put_bg_pixel(&mut self, i: usize) {
        // For normal background, the origin is transformed to (sx, sy).
        // A consequence of this is that values can wrap around.
        let (cx, ly) = (i as u8, self.ly);
        let (sx, sy) = (self.scroll_x, self.scroll_y);

        // get index of coincident bg tile
        let base_addr = self.base_tilemap_addr(self.bg_tilemap_sel);
        let tilemap_offset = (sy.wrapping_add(ly) as u16 / 8) * 32 + sx.wrapping_add(cx) as u16 / 8;
        let tilemap_addr = base_addr + tilemap_offset;
        let tile_idx = self.get_vram_byte(tilemap_addr, 0);

        // get addr of tile
        let addr = self.tiledata_addr(self.tiledata_sel, tile_idx);

        // set pixel value
        let row = (sy.wrapping_add(ly) % 8) as u16;
        let col = sx.wrapping_add(cx) % 8;
        match self.emu_mode {
            EmulationMode::Dmg => self.set_bg_pixel(addr, row, col, i),
            EmulationMode::Cgb => self.set_cgb_bg_pixel(addr, row, col, i, tilemap_addr),
        }
    }

    fn set_cgb_bg_pixel(&mut self, addr: u16, row: u16, col: u8, i: usize, tilemap_addr: u16) {
        let tile_attr = BgAttr::from(self.get_vram_byte(tilemap_addr, 1));

        let (lower, upper) = if tile_attr.mirror_vertical {
            (
                self.get_vram_byte(addr + (7 - row) * 2 + 0, tile_attr.vram_bank),
                self.get_vram_byte(addr + (7 - row) * 2 + 1, tile_attr.vram_bank),
            )
        } else {
            (
                self.get_vram_byte(addr + row * 2 + 0, tile_attr.vram_bank),
                self.get_vram_byte(addr + row * 2 + 1, tile_attr.vram_bank),
            )
        };

        let value = if tile_attr.mirror_horizontal {
            bit!(upper, lower, 0x80 >> (7 - col))
        } else {
            bit!(upper, lower, 0x80 >> col)
        };

        self.pixel_types[i] = match value {
            0 => PixelType::BgColor0,
            _ => PixelType::BgColorOpaque,
        };
        let (r, g, b) = self.get_rgb_cgb(value, tile_attr.bgp_num, false);
        self.update_screen_row(i, r, g, b);
    }

    fn set_bg_pixel(&mut self, addr: u16, row: u16, col: u8, i: usize) {
        let lower = self.get_byte(addr + row * 2 + 0);
        let upper = self.get_byte(addr + row * 2 + 1);

        // set screen[i] with appropriate color value
        let value = bit!(upper, lower, 0x80 >> col);
        self.pixel_types[i] = match value {
            0 => PixelType::BgColor0,
            _ => PixelType::BgColorOpaque,
        };
        let (r, g, b) = self.get_rgb(value, self.bgp);
        self.update_screen_row(i, r, g, b);
    }

    fn tiledata_addr(&self, sel: u8, idx: u8) -> u16 {
        if sel == 0 {
            0x9000u16.wrapping_add((idx as i8 as i16 * 16) as u16)
        } else {
            0x8000u16 + (idx as u16 * 16)
        }
    }

    #[inline]
    fn base_tilemap_addr(&self, sel: u8) -> u16 {
        if sel == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    fn draw_line_sprites(&mut self) {
        let ly = self.ly as i32;
        let height = if self.obj_size == 0 { 8i32 } else { 16i32 };

        let sprites: Vec<_> = (0..40)
            .map(|i| (Sprite::from(&self.oam[i * 4..(i + 1) * 4]), 40 - i))
            .filter(|(s, _)| (ly >= s.y) && (ly < s.y + height))
            .collect::<BinaryHeap<_>>()
            .into_sorted_vec()
            .into_iter()
            .take(10)
            .filter(|(s, _)| (s.x >= -7) && (s.x < SCREEN_WIDTH as i32))
            .collect();

        for (sprite, _) in sprites.into_iter() {
            let row = if sprite.mirror_vertical {
                (height - 1 - (ly - sprite.y)) as u16
            } else {
                (ly - sprite.y) as u16
            };

            let tile_idx = sprite.number & if self.obj_size != 0 { 0x00FE } else { 0x00FF };

            let tile_addr = 0x8000u16 + tile_idx * 16 + row * 2;

            let (lower, upper) = match self.emu_mode {
                EmulationMode::Dmg => (self.get_byte(tile_addr + 0), self.get_byte(tile_addr + 1)),
                EmulationMode::Cgb => (
                    self.get_vram_byte(tile_addr + 0, sprite.vram_bank),
                    self.get_vram_byte(tile_addr + 1, sprite.vram_bank),
                ),
            };

            for j in 0..8 {
                let col = sprite.x + j;
                if (col < 0) || (col >= SCREEN_WIDTH as i32) {
                    continue;
                }
                let value = if !sprite.mirror_horizontal {
                    (upper & (0x80 >> j)).min(1) << 1 | (lower & (0x80 >> j)).min(1)
                } else {
                    (upper & (0x01 << j)).min(1) << 1 | (lower & (0x01 << j)).min(1)
                };
                let pixel_type = self.pixel_types[col as usize] != PixelType::BgColor0;
                let below_bg = !sprite.has_priority && pixel_type;
                if value != 0 && !below_bg {
                    match self.emu_mode {
                        EmulationMode::Dmg => {
                            let palette = if sprite.obp1 { self.obp1 } else { self.obp0 };
                            let (r, g, b) = self.get_rgb(value, palette);
                            self.update_screen_row(col as usize, r, g, b);
                        }
                        EmulationMode::Cgb => {
                            let (r, g, b) = self.get_rgb_cgb(value, sprite.obp_num, true);
                            self.update_screen_row(col as usize, r, g, b);
                        }
                    }
                }
            }
        }
    }

    fn update_screen_row(&mut self, x: usize, r: u8, g: u8, b: u8) {
        let ly = self.ly as usize;
        self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 0] = r;
        self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 1] = g;
        self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 2] = b;
        self.screen[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 3] = 255;
    }

    fn get_rgb(&self, value: u8, palette: u8) -> (u8, u8, u8) {
        match (palette >> (2 * value)) & 0x03 {
            0 => (224, 247, 208),
            1 => (136, 192, 112),
            2 => (52, 104, 86),
            _ => (8, 23, 33),
        }
    }

    fn get_rgb_cgb(&self, color_num: u8, palette_num: usize, obp: bool) -> (u8, u8, u8) {
        let palette_idx = palette_num * 8;
        let color_idx = palette_idx + color_num as usize * 2;

        let palette = if obp {
            (self.obp_ram[color_idx + 0] as u16) << 8 | self.obp_ram[color_idx + 1] as u16
        } else {
            (self.bgp_ram[color_idx + 1] as u16) << 8 | self.bgp_ram[color_idx + 0] as u16
        };

        self.color_correct(
            (palette & 0x001F) >> 0,
            (palette & 0x03E0) >> 5,
            (palette & 0x7C00) >> 10,
        )
    }

    fn color_correct(&self, r: u16, g: u16, b: u16) -> (u8, u8, u8) {
        let r = r as u32;
        let g = g as u32;
        let b = b as u32;

        (
            ((r << 3) | (r >> 2)) as u8,
            ((g << 3) | (g >> 2)) as u8,
            ((b << 3) | (b >> 2)) as u8,
        )
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.lcd_enable == 0 {
            return;
        }
        // Increment clock by cycles elapsed in cpu.
        self.clock += cycles;

        match self.mode {
            GpuMode::OamSearch => {
                // 80 clocks
                if self.clock >= 80 {
                    self.change_mode(GpuMode::PixelTransfer);
                    self.clock = self.clock - 80;
                }
            }
            GpuMode::PixelTransfer => {
                // 172 clocks
                if self.clock >= 172 {
                    self.change_mode(GpuMode::HBlank);
                    self.clock = self.clock - 172;
                    self.draw_line();
                }
            }
            GpuMode::HBlank => {
                // 204 clocks
                if self.clock >= 204 {
                    self.clock = self.clock - 204;
                    self.ly += 1;
                    self.check_coincidence();

                    if self.ly > 143 {
                        self.change_mode(GpuMode::VBlank);
                        self.request_vblank_interrupt();
                    } else {
                        self.change_mode(GpuMode::OamSearch);
                    }
                }
            }
            GpuMode::VBlank => {
                // 4560 clocks, 10 lines
                if self.clock >= 456 {
                    self.clock = self.clock - 456;
                    self.ly += 1;
                    self.check_coincidence();

                    if self.ly > 153 {
                        self.ly = 0;
                        self.change_mode(GpuMode::OamSearch);
                    }
                }
            }
        }
    }

    fn check_coincidence(&mut self) {
        if self.ly == self.lyc {
            self.coincident = 0x04;
            if self.lyc_int != 0 {
                self.request_lcd_interrupt();
            }
        }
    }

    fn change_mode(&mut self, mode: GpuMode) {
        self.mode = mode;
        match self.mode {
            GpuMode::OamSearch if self.oam_int != 0 => self.request_lcd_interrupt(),
            GpuMode::HBlank if self.hblank_int != 0 => self.request_lcd_interrupt(),
            GpuMode::VBlank if self.vblank_int != 0 => self.request_lcd_interrupt(),
            _ => (),
        }
    }

    #[inline]
    fn request_lcd_interrupt(&mut self) {
        self.request_lcd_int = true;
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => match self.mode {
                GpuMode::PixelTransfer if !self.gdma_active => (),
                _ => self.set_vram_byte(addr, value, self.vram_bank),
            },
            0xFE00..=0xFE9F => match self.mode {
                GpuMode::OamSearch | GpuMode::PixelTransfer => (),
                _ => self.oam[(addr - OAM_OFFSET) as usize] = value,
            },
            0xFF40 => {
                self.lcd_enable = value & 0x80;
                if self.lcd_enable == 0 {
                    self.change_mode(GpuMode::HBlank);
                }
                self.win_tilemap_sel = value & 0x40;
                self.window_enable = value & 0x20;
                self.tiledata_sel = value & 0x10;
                self.bg_tilemap_sel = value & 0x08;
                self.obj_size = value & 0x04;
                self.obj_enable = value & 0x02;
                self.bg_display = value & 0x01;
            }
            0xFF41 => {
                self.lyc_int = value & 0x40;
                self.oam_int = value & 0x20;
                self.vblank_int = value & 0x10;
                self.hblank_int = value & 0x08;
            }
            0xFF42 => self.scroll_y = value,
            0xFF43 => self.scroll_x = value,
            0xFF44 => {
                // Writing to 0xFF44 resets ly.
                self.ly = 0;
            }
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.window_y = value,
            0xFF4B => self.window_x = value,
            0xFF4F => self.vram_bank = (value & 0x01) as usize,
            0xFF68 if self.emu_mode == EmulationMode::Cgb => {
                self.bgp_idx = value & 0x3F;
                self.bgp_auto_incr = (value & 0x80) != 0;
            }
            0xFF69 if self.emu_mode == EmulationMode::Cgb => {
                self.bgp_ram[self.bgp_idx as usize] = value;
                if self.bgp_auto_incr {
                    self.bgp_idx = (self.bgp_idx + 1) % 0x40;
                }
            }
            0xFF6A if self.emu_mode == EmulationMode::Cgb => {
                self.obp_idx = value & 0x3F;
                self.obp_auto_incr = (value & 0x80) != 0;
            }
            0xFF6B if self.emu_mode == EmulationMode::Cgb => {
                self.obp_ram[self.bgp_idx as usize] = value;
                if self.obp_auto_incr {
                    self.obp_idx = (self.obp_idx + 1) % 0x40;
                }
            }
            _ => panic!("Unexpected addr in gpu.set_byte"),
        }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => match self.mode {
                GpuMode::PixelTransfer => 0x00,
                _ => self.get_vram_byte(addr, self.vram_bank),
            },
            0xFE00..=0xFE9F => match self.mode {
                GpuMode::OamSearch | GpuMode::PixelTransfer => 0x00,
                _ => self.oam[(addr - OAM_OFFSET) as usize],
            },
            0xFF40 => {
                0x0 | self.lcd_enable
                    | self.win_tilemap_sel
                    | self.window_enable
                    | self.tiledata_sel
                    | self.bg_tilemap_sel
                    | self.obj_size
                    | self.obj_enable
                    | self.bg_display
            }
            0xFF41 => {
                0x0 | self.lyc_int
                    | self.oam_int
                    | self.vblank_int
                    | self.hblank_int
                    | self.coincident
                    | u8::from(&self.mode)
            }
            0xFF42 => self.scroll_y,
            0xFF43 => self.scroll_x,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            // Write only register FF46
            0xFF46 => 0xFF,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.window_y,
            0xFF4B => self.window_x,
            0xFF4F => 0xFE | self.vram_bank as u8,
            0xFF68 if self.emu_mode == EmulationMode::Cgb => {
                (self.bgp_auto_incr as u8) << 7 | self.bgp_idx
            }
            0xFF69 if self.emu_mode == EmulationMode::Cgb => self.bgp_ram[self.bgp_idx as usize],
            0xFF6A if self.emu_mode == EmulationMode::Cgb => {
                (self.obp_auto_incr as u8) << 7 | self.obp_idx
            }
            0xFF6B if self.emu_mode == EmulationMode::Cgb => self.obp_ram[self.obp_idx as usize],
            _ => panic!("Unexpected addr in gpu.get_byte {:#X}", addr),
        }
    }

    fn set_vram_byte(&mut self, addr: u16, value: u8, bank: usize) {
        match addr {
            0x8000..=0x9FFF => {
                if bank == 0 {
                    self.vram0[(addr - VRAM_OFFSET) as usize] = value;
                } else {
                    self.vram1[(addr - VRAM_OFFSET) as usize] = value;
                }
            }
            _ => panic!("Unexpected addr in get_vram_byte"),
        }
    }

    fn get_vram_byte(&self, addr: u16, bank: usize) -> u8 {
        match addr {
            0x8000..=0x9FFF => {
                if bank == 0 {
                    self.vram0[(addr - VRAM_OFFSET) as usize]
                } else {
                    self.vram1[(addr - VRAM_OFFSET) as usize]
                }
            }
            _ => panic!("Unexpected addr in get_vram_byte"),
        }
    }

    #[inline]
    fn request_vblank_interrupt(&mut self) {
        self.request_vblank_int = true;
    }
}

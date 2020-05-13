pub mod registers;
pub mod tiles;

use crate::cpu::EmulationMode;
use crate::gpu::registers::{ColorPalette, LcdControl, LcdPosition, LcdStatus, MonochromePalette};
use crate::gpu::tiles::{BgAttr, Sprite};
use std::mem;

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

#[derive(PartialEq)]
enum PixelType {
    BgColor0,
    BgColorOpaque,
    BgPriorityOverride,
}

pub struct Gpu {
    pub lcd: Vec<u8>,
    pub vram0: Vec<u8>,
    pub vram1: Vec<u8>,
    pub bgp_ram: Vec<u8>,
    pub obp_ram: Vec<u8>,
    cgbp: ColorPalette,
    emu_mode: EmulationMode,
    oam: Vec<u8>,
    pixel_types: Vec<PixelType>,
    lcdc: LcdControl,
    dmgp: MonochromePalette,
    position: LcdPosition,
    stat: LcdStatus,
    clock: usize,
    pub request_vblank_int: bool,
    pub request_lcd_int: bool,
    vram_bank: usize,
    win_counter: usize,
    pub oam_dma_active: bool,
}

impl Gpu {
    pub fn new(emu_mode: EmulationMode) -> Self {
        let mut pixel_types = vec![];
        for _ in 0..SCREEN_WIDTH {
            pixel_types.push(PixelType::BgColor0);
        }

        Gpu {
            lcd: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH * SCREEN_DEPTH],
            vram0: vec![0; VRAM_BANK_SIZE],
            vram1: vec![0; VRAM_BANK_SIZE],
            bgp_ram: vec![0; PALETTE_RAM_SIZE],
            obp_ram: vec![0; PALETTE_RAM_SIZE],
            oam: vec![0; OAM_SIZE],
            cgbp: ColorPalette::default(),
            emu_mode,
            pixel_types,
            lcdc: LcdControl::default(),
            dmgp: MonochromePalette::default(),
            position: LcdPosition::default(),
            stat: LcdStatus::default(),
            clock: 0,
            request_vblank_int: false,
            request_lcd_int: false,
            vram_bank: 0,
            win_counter: 0,
            oam_dma_active: false,
        }
    }

    pub fn mode(&self) -> &GpuMode {
        &self.stat.mode
    }

    pub fn screen(&self) -> *const u8 {
        self.lcd.as_ptr()
    }

    pub fn simulate_bootrom(&mut self) {
        // self.position.ly = 0x90;
        match self.emu_mode {
            EmulationMode::Dmg => {
                self.set_byte(0xFF41, 0x85);
                self.set_byte(0xFF46, 0xFF);
                self.set_byte(0xFF47, 0xFC);
                self.set_byte(0xFF48, 0xFF);
                self.set_byte(0xFF49, 0xFF);
            }
            EmulationMode::Cgb => {
                self.set_byte(0xFF41, 0x81);
                self.set_byte(0xFF44, 0x90);
                self.set_byte(0xFF47, 0xFC);
            }
        }

        self.set_byte(0xFF40, 0x90);
    }

    fn draw_line(&mut self) {
        for i in 0..SCREEN_WIDTH {
            self.pixel_types[i] = PixelType::BgColor0;
            let ly = self.position.ly as usize;
            self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 0] = 255;
            self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 1] = 255;
            self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 2] = 255;
            self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 3] = 255;
        }

        self.draw_line_bg();
        self.draw_line_sprites();
    }

    fn clear_screen(&mut self) {
        for i in 0..self.lcd.len() {
            self.lcd[i] = 255;
        }
    }

    fn draw_line_bg(&mut self) {
        if (self.emu_mode == EmulationMode::Dmg) && (self.lcdc.lcdc0 == 0) {
            self.draw_blank_line();
        } else {
            for i in 0..SCREEN_WIDTH {
                if self.is_win_enabled() && self.is_win_pixel(i) {
                    self.put_win_pixel(i);
                } else {
                    self.put_bg_pixel(i);
                }
            }
        }
        self.update_window_counter();
    }

    fn draw_blank_line(&mut self) {
        for i in 0..SCREEN_WIDTH {
            let (r, g, b) = self.get_rgb(0, self.dmgp.bgp);
            self.update_screen_row(i, r, g, b);
        }
    }

    fn is_win_enabled(&self) -> bool {
        self.lcdc.window_enabled(&self.emu_mode)
            && (self.position.window_x < 167)
            && (self.position.window_y < 144)
    }

    #[inline]
    fn is_win_pixel(&self, i: usize) -> bool {
        self.position.window_x <= (i + 7) as u8 && self.position.window_y <= self.position.ly
    }

    #[inline]
    fn update_window_counter(&mut self) {
        if self.is_win_enabled() && self.position.window_y <= self.position.ly {
            self.win_counter += 1;
        }
    }

    fn put_win_pixel(&mut self, i: usize) {
        let lx = i as u8;
        let wx = self.position.window_x;

        // get idx of the coincident window tile
        let base_addr = self.lcdc.win_tilemap();
        let tilemap_offset = (self.win_counter / 8) * 32 + (i + 7 - wx as usize) / 8;
        let tilemap_addr = base_addr + tilemap_offset as u16;
        let tile_idx = self.get_vram_byte(tilemap_addr, 0);

        // get addr of tile
        let addr = self.tiledata_addr(self.lcdc.bg_tiledata_sel, tile_idx);

        // set pixel value
        let row = (self.win_counter % 8) as u16;
        let col = (lx + 7 - wx) % 8;

        match self.emu_mode {
            EmulationMode::Dmg => self.set_bg_pixel(addr, row, col, i),
            EmulationMode::Cgb => self.set_cgb_bg_pixel(addr, row, col, i, tilemap_addr),
        }
    }

    fn put_bg_pixel(&mut self, i: usize) {
        // For normal background, the origin is transformed to (sx, sy).
        // A consequence of this is that values can wrap around.
        let (cx, ly) = (i as u8, self.position.ly);
        let (sx, sy) = (self.position.scroll_x, self.position.scroll_y);

        // get index of coincident bg tile
        let base_addr = self.lcdc.bg_tilemap();
        let tilemap_offset = (sy.wrapping_add(ly) as u16 / 8) * 32 + sx.wrapping_add(cx) as u16 / 8;
        let tilemap_addr = base_addr + tilemap_offset;
        let tile_idx = self.get_vram_byte(tilemap_addr, 0);

        // get addr of tile
        let addr = self.tiledata_addr(self.lcdc.bg_tiledata_sel, tile_idx);

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

        self.pixel_types[i] = match (tile_attr.has_priority, value) {
            (_, 0) => PixelType::BgColor0,
            (true, _) => PixelType::BgPriorityOverride,
            (false, _) => PixelType::BgColorOpaque,
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
        let (r, g, b) = self.get_rgb(value, self.dmgp.bgp);
        self.update_screen_row(i, r, g, b);
    }

    fn tiledata_addr(&self, sel: u8, idx: u8) -> u16 {
        if sel == 0 {
            0x8800u16 + (idx as i8 as i16 + 128) as u16 * 16
        } else {
            0x8000u16 + (idx as u16 * 16)
        }
    }

    fn draw_line_sprites(&mut self) {
        if !self.lcdc.obj_enabled() {
            return;
        }

        let ly = self.position.ly as i32;
        let height = if self.lcdc.obj_size == 0 { 8i32 } else { 16i32 };

        let mut sprites: Vec<_> = (0..40)
            .map(|i| (i, Sprite::from(&self.oam[i * 4..(i + 1) * 4])))
            .filter(|(_, s)| (ly >= s.y) && (ly < s.y + height))
            .take(10)
            .collect();

        sprites.sort_by_key(|(i, s)| match self.emu_mode {
            EmulationMode::Dmg => (s.x, *i),
            EmulationMode::Cgb => (*i as u32 as i32, 0),
        });

        for (_, sprite) in sprites.into_iter().rev() {
            let row = if sprite.mirror_vertical {
                (height - 1 - (ly - sprite.y)) as u16
            } else {
                (ly - sprite.y) as u16
            };

            let tile_idx = if self.lcdc.obj_size != 0 {
                sprite.number & 0x00FE
            } else {
                sprite.number & 0x00FF
            };

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

                let value = if sprite.mirror_horizontal {
                    bit!(upper, lower, 0x80 >> (7 - j))
                } else {
                    bit!(upper, lower, 0x80 >> j)
                };

                let below_bg = match &self.pixel_types[col as usize] {
                    _ if self.lcdc.lcdc0 == 0 => false,
                    PixelType::BgColor0 => false,
                    PixelType::BgColorOpaque if !sprite.has_priority => true,
                    PixelType::BgPriorityOverride => true,
                    _ => false,
                };

                if value != 0 && !below_bg {
                    match self.emu_mode {
                        EmulationMode::Dmg => {
                            let palette = if sprite.obp1 {
                                self.dmgp.obp1
                            } else {
                                self.dmgp.obp0
                            };
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
        let ly = self.position.ly as usize;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 0] = r;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 1] = g;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 2] = b;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + x * SCREEN_DEPTH + 3] = 255;
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
            (self.obp_ram[color_idx + 1] as u16) << 8 | self.obp_ram[color_idx + 0] as u16
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

        // MatCurrie
        // (
        //     ((r << 3) | (r >> 2)) as u8,
        //     ((g << 3) | (g >> 2)) as u8,
        //     ((b << 3) | (b >> 2)) as u8,
        // )

        // Gambatte
        (
            ((r * 13 + g * 2 + b) >> 1) as u8,
            ((g * 3 + b) << 1) as u8,
            ((r * 3 + g * 2 + b * 11) >> 1) as u8,
        )
    }

    pub fn tick(&mut self, cycles: usize) {
        if self.lcdc.display_enable == 0 {
            return;
        }
        // Increment clock by cycles elapsed in cpu.
        self.clock += cycles;

        match self.stat.mode {
            GpuMode::OamSearch => {
                // 80 clocks
                if self.clock >= 80 {
                    self.clock -= 80;
                    self.change_mode(GpuMode::PixelTransfer);
                }
            }
            GpuMode::PixelTransfer => {
                // 172 clocks
                if self.clock >= 172 {
                    self.change_mode(GpuMode::HBlank);
                    self.clock -= 172;
                    self.draw_line();
                }
            }
            GpuMode::HBlank => {
                // 204 clocks
                if self.clock >= 204 {
                    self.clock -= 204;
                    self.position.ly += 1;
                    self.check_coincidence();

                    if self.position.ly > 143 {
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
                    self.clock -= 456;
                    self.position.ly += 1;
                    self.check_coincidence();

                    // STRANGE BEHAVIOR: At line 153, V-Blank has already reached
                    // the top of the screen and is to be treated like line 0.
                    if self.position.ly == 153 {
                        self.position.ly = 0;
                        self.check_coincidence();
                    }

                    if self.position.ly == 1 {
                        self.position.ly = 0;
                        self.win_counter = 0;
                        self.change_mode(GpuMode::OamSearch);
                    }
                }
            }
        }
    }

    fn check_coincidence(&mut self) {
        if self.position.ly == self.position.lyc {
            self.stat.coincident = 0x04;
            if self.stat.lyc_int != 0 {
                self.request_lcd_interrupt();
            }
        }
    }

    fn change_mode(&mut self, mode: GpuMode) {
        self.stat.mode = mode;
        match self.stat.mode {
            GpuMode::OamSearch if self.stat.oam_int != 0 => self.request_lcd_interrupt(),
            GpuMode::HBlank if self.stat.hblank_int != 0 => self.request_lcd_interrupt(),
            GpuMode::VBlank if self.stat.vblank_int != 0 => self.request_lcd_interrupt(),
            _ => (),
        }
    }

    #[inline]
    fn request_lcd_interrupt(&mut self) {
        self.request_lcd_int = true;
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x9FFF => match self.stat.mode {
                GpuMode::PixelTransfer if self.lcdc.display_enabled() => (),
                _ => self.set_vram_byte(addr, value, self.vram_bank),
            },
            0xFE00..=0xFE9F => match self.stat.mode {
                GpuMode::OamSearch | GpuMode::PixelTransfer if self.lcdc.display_enabled() => (),
                _ => self.oam[(addr - OAM_OFFSET) as usize] = value,
            },
            0xFF40 => {
                let old_display_enable = self.lcdc.display_enable;
                self.lcdc.display_enable = value & 0x80;
                if old_display_enable != 0 && self.lcdc.display_enable == 0 {
                    self.change_mode(GpuMode::HBlank);
                    // self.stat.mode = GpuMode::HBlank;
                    self.position.ly = 0;
                    self.win_counter = 0;
                    self.clock = 0;
                    self.clear_screen();
                }
                self.lcdc.win_tilemap_sel = value & 0x40;
                self.lcdc.win_display_enable = value & 0x20;
                self.lcdc.bg_tiledata_sel = value & 0x10;
                self.lcdc.bg_tilemap_sel = value & 0x08;
                self.lcdc.obj_size = value & 0x04;
                self.lcdc.obj_display_enable = value & 0x02;
                self.lcdc.lcdc0 = value & 0x01;
            }
            0xFF41 => {
                self.stat.lyc_int = value & 0x40;
                self.stat.oam_int = value & 0x20;
                self.stat.vblank_int = value & 0x10;
                self.stat.hblank_int = value & 0x08;
            }
            0xFF42 => self.position.scroll_y = value,
            0xFF43 => self.position.scroll_x = value,
            0xFF44 => (),
            0xFF45 => self.position.lyc = value,
            0xFF47 => self.dmgp.bgp = value,
            0xFF48 => self.dmgp.obp0 = value,
            0xFF49 => self.dmgp.obp1 = value,
            0xFF4A => self.position.window_y = value,
            0xFF4B => self.position.window_x = value,
            0xFF4F => self.vram_bank = (value & 0x01) as usize,
            0xFF68 => {
                self.cgbp.bgp_idx = value & 0x3F;
                self.cgbp.bgp_auto_incr = (value & 0x80) != 0;
            }
            0xFF69 => {
                if self.stat.mode != GpuMode::PixelTransfer {
                    self.bgp_ram[self.cgbp.bgp_idx as usize] = value;
                }
                if self.cgbp.bgp_auto_incr {
                    self.cgbp.bgp_idx = (self.cgbp.bgp_idx + 1) % 0x40;
                }
            }
            0xFF6A => {
                self.cgbp.obp_idx = value & 0x3F;
                self.cgbp.obp_auto_incr = (value & 0x80) != 0;
            }
            0xFF6B => {
                if self.stat.mode != GpuMode::PixelTransfer {
                    self.obp_ram[self.cgbp.obp_idx as usize] = value;
                }
                if self.cgbp.obp_auto_incr {
                    self.cgbp.obp_idx = (self.cgbp.obp_idx + 1) % 0x40;
                }
            }
            _ => panic!("Unexpected addr in gpu.set_byte {:#X}", addr),
        }
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => match self.stat.mode {
                GpuMode::PixelTransfer if !self.oam_dma_active => 0x00,
                _ => self.get_vram_byte(addr, self.vram_bank),
            },
            0xFE00..=0xFE9F => match self.stat.mode {
                GpuMode::OamSearch | GpuMode::PixelTransfer if !self.oam_dma_active => 0x00,
                _ => self.oam[(addr - OAM_OFFSET) as usize],
            },
            0xFF40 => u8::from(&self.lcdc),
            0xFF41 => u8::from(&self.stat),
            0xFF42 => self.position.scroll_y,
            0xFF43 => self.position.scroll_x,
            0xFF44 => self.position.ly,
            0xFF45 => self.position.lyc,
            // Write only register FF46
            0xFF46 => 0xFF,
            0xFF47 => self.dmgp.bgp,
            0xFF48 => self.dmgp.obp0,
            0xFF49 => self.dmgp.obp1,
            0xFF4A => self.position.window_y,
            0xFF4B => self.position.window_x,
            0xFF4F => 0xFE | self.vram_bank as u8,
            0xFF68 if self.emu_mode == EmulationMode::Cgb => self.cgbp.bgp(),
            0xFF69 if self.emu_mode == EmulationMode::Cgb => {
                self.bgp_ram[self.cgbp.bgp_idx as usize]
            }
            0xFF6A if self.emu_mode == EmulationMode::Cgb => self.cgbp.obp(),
            0xFF6B if self.emu_mode == EmulationMode::Cgb => {
                self.obp_ram[self.cgbp.obp_idx as usize]
            }
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

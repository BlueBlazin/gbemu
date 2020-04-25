use crate::cpu::EmulationMode;

const VRAM_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;
const PALETTE_RAM_SIZE: usize = 0x40;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const SCREEN_DEPTH: usize = 4;
const VRAM_OFFSET: u16 = 0x8000;
const OAM_OFFSET: u16 = 0xFE00;

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

struct Sprite {
    pub y: i32,
    pub x: i32,
    pub number: u16,
    pub has_priority: bool,
    pub flip_vertical: bool,
    pub flip_horizontal: bool,
    pub obp1: bool,
}

impl From<&[u8]> for Sprite {
    fn from(bytes: &[u8]) -> Sprite {
        Sprite {
            y: bytes[0] as u16 as i32 - 16,
            x: bytes[1] as u16 as i32 - 8,
            number: bytes[2] as u16,
            has_priority: (bytes[3] & 0x80) == 0,
            flip_vertical: (bytes[3] & 0x40) != 0,
            flip_horizontal: (bytes[3] & 0x20) != 0,
            obp1: (bytes[3] & 0x10) != 0,
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
    pub vram: Vec<u8>,
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
}

impl Gpu {
    pub fn new(emu_mode: EmulationMode) -> Self {
        let mut pixel_types = vec![];
        for _ in 0..SCREEN_WIDTH {
            pixel_types.push(PixelType::BgColor0);
        }

        Gpu {
            screen: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH * SCREEN_DEPTH],
            vram: vec![0; VRAM_SIZE],
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
            win_tilemap_sel: 0,
            window_enable: 0,
            tiledata_sel: 0,
            bg_tilemap_sel: 0,
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
        }
    }

    pub fn screen(&self) -> *const u8 {
        self.screen.as_ptr()
    }

    fn draw_line(&mut self) {
        for i in 0..SCREEN_WIDTH {
            self.pixel_types[i] = PixelType::BgColor0;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 0] =
                255;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 1] =
                255;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 2] =
                255;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 3] =
                255;
        }

        // draw bg and window
        self.draw_line_bg();
        // draw sprites
        self.draw_line_sprites();
    }

    fn draw_line_bg(&mut self) {
        let window_y = if self.window_enable != 0 {
            self.ly as i32 - self.window_y as i32
        } else {
            -1
        };
        let win_row = (window_y / 8) as usize;

        let y = self.ly.wrapping_add(self.scroll_y);
        let row = (y / 8) as usize;

        for i in 0..SCREEN_WIDTH {
            let window_x = self.window_x as i32 - 7 + i as i32;
            let win_col = (window_x / 8) as usize;

            let x = (i as u8).wrapping_add(self.scroll_x);
            let col = (x / 8) as usize;

            let tilenum = if window_y >= 0 && window_x >= 0 {
                if self.win_tilemap_sel != 0 {
                    0x9C00 + win_row * 32 + win_col
                } else {
                    0x9800 + win_row * 32 + win_col
                }
            } else {
                if self.bg_tilemap_sel != 0 {
                    0x9C00 + row * 32 + col
                } else {
                    0x9800 + row * 32 + col
                }
            };

            let tile_offset = self.get_byte(tilenum as u16);
            let tile_addr = if self.tiledata_sel != 0 {
                0x8000u16 + tile_offset as u16 * 16
            } else {
                let n = tile_offset as i8 as i16 * 16;
                0x9000u16.wrapping_add(n as u16)
            };

            let tile_row = (y % 8) as u16;
            let lower = self.get_byte(tile_addr + tile_row * 2 + 0);
            let upper = self.get_byte(tile_addr + tile_row * 2 + 1);
            let mask = 0x80 >> (x % 8);
            let value = (((upper & mask) != 0) as u8) << 1 | ((lower & mask) != 0) as u8;

            self.pixel_types[i] = if value == 0 {
                PixelType::BgColor0
            } else {
                PixelType::BgColorOpaque
            };

            let (r, g, b) = self.get_rgb(value, self.bgp);
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 0] = r;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 1] = g;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 2] = b;
            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH + i * SCREEN_DEPTH + 3] =
                255;
        }
    }

    fn draw_line_sprites(&mut self) {
        for i in (0..40).rev() {
            let sprite = Sprite::from(&self.oam[i * 4..(i + 1) * 4]);
            let height = if self.obj_size == 0 { 8i32 } else { 16i32 };
            let ly = self.ly as i32;

            if ((ly >= sprite.y) && (ly < sprite.y + height))
                && ((sprite.x >= -7) && (sprite.x < SCREEN_WIDTH as i32))
            {
                let row = if sprite.flip_vertical {
                    (height - 1 - (ly - sprite.y)) as u16
                } else {
                    (ly - sprite.y) as u16
                };

                let tilenum = if self.obj_size != 0 {
                    sprite.number & 0xFE
                } else {
                    sprite.number
                };

                let tile_addr = 0x8000u16 + tilenum * 16 + row * 2;
                let lower = self.get_byte(tile_addr + 0);
                let upper = self.get_byte(tile_addr + 1);

                for j in 0..8 {
                    if (sprite.x + j >= 0) && (sprite.x + j < (SCREEN_WIDTH as i32)) {
                        let value = if sprite.flip_horizontal {
                            let mask = 0x01 << j;
                            (((upper & mask) != 0) as u8) << 1 | (((lower) & mask) != 0) as u8
                        } else {
                            let mask = 0x80 >> j;
                            (((upper & mask) != 0) as u8) << 1 | ((lower & mask) != 0) as u8
                        };

                        let below_bg = !sprite.has_priority
                            && (self.pixel_types[(sprite.x + j) as usize] != PixelType::BgColor0);

                        if (value != 0) && !below_bg {
                            let x = (sprite.x + j) as usize;
                            let palette = if sprite.obp1 { self.obp1 } else { self.obp0 };
                            let (r, g, b) = self.get_rgb(value, palette);
                            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH
                                + x * SCREEN_DEPTH
                                + 0] = r;
                            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH
                                + x * SCREEN_DEPTH
                                + 1] = g;
                            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH
                                + x * SCREEN_DEPTH
                                + 2] = b;
                            self.screen[self.ly as usize * SCREEN_WIDTH * SCREEN_DEPTH
                                + i * SCREEN_DEPTH
                                + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    fn get_rgb(&self, value: u8, palette: u8) -> (u8, u8, u8) {
        // match palette & (0x3 << (value * 2)) {
        //     0 => (255, 255, 255),
        //     1 => (192, 192, 192),
        //     2 => (96, 96, 96),
        //     _ => (0, 0, 0),
        // }
        // match palette & (0x3 << (value * 2)) {
        //     0 => (155, 188, 15),
        //     1 => (139, 172, 15),
        //     2 => (48, 98, 48),
        //     _ => (15, 56, 15),
        // }
        // match palette & (0b11 << (value * 2)) {
        // match (palette >> (2 * value)) & 0x03 {
        //     0 => (224, 247, 207),
        //     1 => (134, 192, 108),
        //     2 => (47, 104, 80),
        //     _ => (7, 23, 33),
        // }
        // BGB Palette
        match (palette >> (2 * value)) & 0x03 {
            0 => (224, 247, 208),
            1 => (136, 192, 112),
            2 => (52, 104, 86),
            _ => (8, 23, 33),
        }
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

                    if self.ly > 143 {
                        self.change_mode(GpuMode::VBlank);
                        self.request_vblank_interrupt();
                    } else {
                        self.change_mode(GpuMode::OamSearch);
                        self.check_coincidence();
                    }
                }
            }
            GpuMode::VBlank => {
                // 4560 clocks, 10 lines
                if self.clock >= 456 {
                    self.clock = self.clock - 456;
                    self.ly += 1;

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
                self.request_lcd_interrupt()
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
                _ => self.vram[(addr - VRAM_OFFSET) as usize] = value,
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
            0xFF4B => {
                // The value is window_x - 7.
                self.window_x = value + 7;
            }
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
                _ => self.vram[(addr - VRAM_OFFSET) as usize],
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
            0xFF68 if self.emu_mode == EmulationMode::Cgb => {
                (self.bgp_auto_incr as u8) << 7 | self.bgp_idx
            }
            0xFF69 if self.emu_mode == EmulationMode::Cgb => self.bgp_ram[self.bgp_idx as usize],
            0xFF6A if self.emu_mode == EmulationMode::Cgb => {
                (self.obp_auto_incr as u8) << 7 | self.obp_idx
            }
            0xFF6B if self.emu_mode == EmulationMode::Cgb => self.obp_ram[self.obp_idx as usize],
            _ => panic!("Unexpected addr in gpu.get_byte"),
        }
    }

    #[inline]
    fn request_vblank_interrupt(&mut self) {
        self.request_vblank_int = true;
    }
}

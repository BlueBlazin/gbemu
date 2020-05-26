pub mod fifo;
pub mod registers;
pub mod tiles;

use crate::cpu::EmulationMode;
use crate::gpu::registers::{ColorPalette, LcdControl, LcdPosition, LcdStatus, MonochromePalette};
use crate::gpu::tiles::{BgAttr, Sprite};
use std::collections::VecDeque;

const VRAM_BANK_SIZE: usize = 0x2000;
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

#[derive(Debug)]
pub enum FetcherState {
    Sleep(usize),
    ReadTileNumber,
    ReadTileDataLow,
    ReadTileDataHigh,
    Push(usize),
}

pub struct BgFetcher {
    state: FetcherState,
    tile_num: u8,
    low: u8,
    high: u8,
    x: u8,
}

impl BgFetcher {
    pub fn new() -> Self {
        Self {
            state: FetcherState::Sleep(0),
            tile_num: 0,
            low: 0,
            high: 0,
            x: 0,
        }
    }

    pub fn restart(&mut self) {
        self.state = FetcherState::Sleep(0);
        self.tile_num = 0;
        self.low = 0;
        self.high = 0;
        self.x = 0;
    }
}

#[derive(PartialEq)]
pub enum PixelType {
    BgColor0,
    BgColorOpaque,
    Sprite,
}

pub struct PixelFifoItem {
    pub value: u8,
    pub palette: u8,
}

pub struct PixelFifo {
    q: VecDeque<PixelFifoItem>,
    unaligned_scx: u8,
    unaligned_winx: u8,
}

impl PixelFifo {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(16),
            unaligned_scx: 0,
            unaligned_winx: 0,
        }
    }

    pub fn restart(&mut self) {
        self.q.clear();
        self.unaligned_scx = 0;
        self.unaligned_winx = 0;
    }

    pub fn has_space(&self) -> bool {
        self.q.len() <= 8
    }

    pub fn push_row(&mut self, mut low: u8, mut high: u8, palette: u8) {
        for i in 0..8 {
            let value = ((high >> 7) << 1) | (low >> 7);
            self.q.push_back(PixelFifoItem { value, palette });
            low <<= 1;
            high <<= 1;
        }
    }

    pub fn pop(&mut self) -> Option<PixelFifoItem> {
        if self.q.len() <= 8 {
            None
        } else {
            self.q.pop_front()
        }
    }
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
    lcdc: LcdControl,
    dmgp: MonochromePalette,
    position: LcdPosition,
    stat: LcdStatus,
    clock: usize,
    pub request_vblank_int: bool,
    pub request_lcd_int: bool,
    vram_bank: usize,
    win_counter: u8,
    pub oam_dma_active: bool,

    // Pixel Pipeline
    mode3_cycles: usize,
    drawing_window: bool,
    fifo: PixelFifo,

    // Background
    window_was_drawn: bool,
    bg_fetcher: BgFetcher,

    // Sprites
    sprites: Vec<Sprite>,
    sprite_comparators: Vec<u8>,
    cmp_idx: usize,
}

impl Gpu {
    pub fn new(emu_mode: EmulationMode) -> Self {
        Gpu {
            lcd: vec![0; SCREEN_HEIGHT * SCREEN_WIDTH * SCREEN_DEPTH],
            vram0: vec![0; VRAM_BANK_SIZE],
            vram1: vec![0; VRAM_BANK_SIZE],
            bgp_ram: vec![0; PALETTE_RAM_SIZE],
            obp_ram: vec![0; PALETTE_RAM_SIZE],
            oam: vec![0; OAM_SIZE],
            cgbp: ColorPalette::default(),
            emu_mode,
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

            // Pixel Pipeline
            mode3_cycles: 0,
            drawing_window: false,
            fifo: PixelFifo::new(),
            // Background
            window_was_drawn: false,
            bg_fetcher: BgFetcher::new(),

            // Sprites
            sprites: Vec::with_capacity(10),
            sprite_comparators: Vec::with_capacity(10),
            cmp_idx: 0,
        }
    }

    pub fn mode(&self) -> &GpuMode {
        &self.stat.mode
    }

    pub fn screen(&self) -> *const u8 {
        self.lcd.as_ptr()
    }

    fn pixel_pipeline_tick(&mut self) {
        self.bg_fetcher_tick();

        if !self.drawing_window && self.fifo.unaligned_scx > 0 {
            self.fifo.pop();
            self.fifo.unaligned_scx -= 1;
            return;
        }

        if self.drawing_window && self.fifo.unaligned_winx > 0 {
            self.fifo.pop();
            self.fifo.unaligned_winx -= 1;
            return;
        }

        if let Some(pixel) = self.fifo.pop() {
            self.draw_pixel(pixel);
            self.advance_lx();
        }
    }

    fn draw_pixel(&mut self, pixel: PixelFifoItem) {
        let (r, g, b) = if self.lcdc.lcdc0 == 0 {
            self.get_rgb(0, self.dmgp.bgp)
        } else {
            self.get_rgb(pixel.value, pixel.palette)
        };

        self.update_screen_row(self.position.lx as usize, r, g, b);
    }

    fn advance_lx(&mut self) {
        self.position.lx += 1;

        if !self.drawing_window && self.is_win_enabled() && self.is_win_pixel() {
            self.fifo.restart();
            self.bg_fetcher.restart();
            self.drawing_window = true;
            self.fifo.unaligned_winx = (self.position.lx + 7 - self.position.window_x) % 8;
        }
    }

    fn bg_fetcher_tick(&mut self) {
        match self.bg_fetcher.state {
            FetcherState::Sleep(0) => {
                self.bg_fetcher.state = FetcherState::ReadTileNumber;
            }
            // Read tile number from tile map
            FetcherState::ReadTileNumber => {
                let (base, row, col) = if self.drawing_window {
                    let base = self.lcdc.win_tilemap();
                    let row = self.win_counter / 8;
                    let col = self.bg_fetcher.x;
                    (base, row, col)
                } else {
                    let base = self.lcdc.bg_tilemap();
                    let row = self.position.ly.wrapping_add(self.position.scroll_y) / 8;
                    let col = (self.position.scroll_x / 8 + self.bg_fetcher.x) % 32;
                    (base, row, col)
                };

                let addr = base + row as u16 * 32 + col as u16;
                self.bg_fetcher.tile_num = self.get_vram_byte(addr, 0);
                self.bg_fetcher.state = FetcherState::Sleep(1);
            }
            FetcherState::Sleep(1) => {
                self.bg_fetcher.state = FetcherState::ReadTileDataLow;
            }
            // Fetch lower byte of current row from tile at tile number
            FetcherState::ReadTileDataLow => {
                let row = if self.drawing_window {
                    self.win_counter % 8
                } else {
                    self.position.ly.wrapping_add(self.position.scroll_y) % 8
                };

                let tile_n = self.bg_fetcher.tile_num;
                let tile_addr = self.tiledata_addr(self.lcdc.bg_tiledata_sel, tile_n);

                self.bg_fetcher.low = self.get_vram_byte(tile_addr + row as u16 * 2, 0);
                self.bg_fetcher.state = FetcherState::Sleep(2);
            }
            FetcherState::Sleep(2) => {
                self.bg_fetcher.state = FetcherState::ReadTileDataHigh;
            }
            // Fetch upper byte of current row from tile at tile number
            FetcherState::ReadTileDataHigh => {
                let row = if self.drawing_window {
                    self.win_counter % 8
                } else {
                    self.position.ly.wrapping_add(self.position.scroll_y) % 8
                };

                let tile_n = self.bg_fetcher.tile_num;
                let tile_addr = self.tiledata_addr(self.lcdc.bg_tiledata_sel, tile_n);

                self.bg_fetcher.high = self.get_vram_byte(tile_addr + row as u16 * 2 + 1, 0);
                self.bg_fetcher.state = FetcherState::Push(0);
            }
            // Push tile row data to pixel FIFO
            FetcherState::Push(0) => {
                self.bg_fetcher.x = (self.bg_fetcher.x + 1) % 32;
                self.bg_fetcher.state = FetcherState::Push(1);
            }
            // Push tile row data to pixel FIFO
            FetcherState::Push(1) => {
                if self.fifo.has_space() {
                    if self.drawing_window {
                        self.window_was_drawn = true;
                    }
                    let low = self.bg_fetcher.low;
                    let high = self.bg_fetcher.high;
                    self.fifo.push_row(low, high, self.dmgp.bgp);
                    self.bg_fetcher.state = FetcherState::Sleep(0);
                }
            }
            // Idle
            _ => (),
        }
    }

    fn tiledata_addr(&self, sel: u8, idx: u8) -> u16 {
        if sel == 0 {
            0x8800u16 + (idx as i8 as i16 + 128) as u16 * 16
        } else {
            0x8000u16 + (idx as u16 * 16)
        }
    }

    fn is_win_enabled(&self) -> bool {
        self.lcdc.window_enabled(&self.emu_mode)
            && (self.position.window_x < 167)
            && (self.position.window_y < 144)
    }

    #[inline]
    fn is_win_pixel(&self) -> bool {
        self.position.window_x <= (self.position.lx + 7) as u8
            && self.position.window_y <= self.position.ly
    }

    #[inline]
    fn update_window_counter(&mut self) {
        if self.is_win_enabled() && self.position.window_y <= self.position.ly {
            self.win_counter += 1;
        }
    }

    fn load_sprites(&mut self) {
        self.cmp_idx = 0;

        let ly = self.position.ly;
        let height = if self.lcdc.obj_size == 0 { 8 } else { 16 };

        let mut sprites: Vec<_> = (0..40)
            .map(|i| (i, Sprite::from(&self.oam[i * 4..(i + 1) * 4])))
            .filter(|(_, s)| (ly + 16 >= s.y) && (ly + 16 < s.y + height))
            .take(10)
            .collect();

        sprites.sort_by_key(|(i, s)| match self.emu_mode {
            EmulationMode::Dmg => (s.x, *i),
            EmulationMode::Cgb => (*i as u8, 0),
        });

        self.sprite_comparators = sprites.iter().map(|(_, s)| s.x).collect();
        self.sprites = sprites.into_iter().map(|(_, s)| s).collect();
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
        (
            ((r << 3) | (r >> 2)) as u8,
            ((g << 3) | (g >> 2)) as u8,
            ((b << 3) | (b >> 2)) as u8,
        )
    }

    pub fn tick(&mut self, mut cycles: usize) {
        if self.lcdc.display_enable == 0 {
            return;
        }

        while cycles > 0 {
            match self.stat.mode {
                GpuMode::OamSearch => cycles = self.oam_search_tick(cycles),
                GpuMode::PixelTransfer => cycles = self.pixel_transfer_tick(cycles),
                GpuMode::HBlank => cycles = self.hblank_tick(cycles),
                GpuMode::VBlank => cycles = self.vblank_tick(cycles),
            }
        }
    }

    // Mode 2 - OAM Search
    fn oam_search_tick(&mut self, cycles: usize) -> usize {
        if self.clock + cycles >= 80 {
            let cycles_left = self.clock + cycles - 80;
            self.clock = 0;
            self.load_sprites();
            self.change_mode(GpuMode::PixelTransfer);
            cycles_left
        } else {
            self.clock += cycles;
            0
        }
    }

    // Mode 3 - Pixel Transfer
    fn pixel_transfer_tick(&mut self, mut cycles: usize) -> usize {
        while cycles > 0 && (self.position.lx as usize) < SCREEN_WIDTH {
            self.pixel_pipeline_tick();
            self.mode3_cycles += 1;
            cycles -= 1
        }

        if (self.position.lx as usize) >= SCREEN_WIDTH {
            if self.window_was_drawn {
                self.update_window_counter();
                self.window_was_drawn = false;
            }
            self.change_mode(GpuMode::HBlank);
        }

        cycles
    }

    // Mode 0 - H-Blank
    fn hblank_tick(&mut self, cycles: usize) -> usize {
        if self.clock + cycles >= 204 - (self.mode3_cycles - 172) {
            let cycles_left = self.clock + cycles - (204 - (self.mode3_cycles - 172));
            self.clock = 0;
            self.position.ly += 1;
            self.check_coincidence();

            if self.position.ly > 143 {
                self.change_mode(GpuMode::VBlank);
                self.request_vblank_interrupt();
            } else {
                self.change_mode(GpuMode::OamSearch);
            }

            cycles_left
        } else {
            self.clock += cycles;
            0
        }
    }

    // Mode 1 - V-Blank
    fn vblank_tick(&mut self, cycles: usize) -> usize {
        if self.clock + cycles >= 456 {
            let cycles_left = self.clock + cycles - 456;
            self.clock = 0;
            self.position.ly += 1;

            // STRANGE BEHAVIOR
            if self.position.ly == 153 {
                self.position.ly = 0;
                self.check_coincidence();
            }

            if self.position.ly == 1 {
                self.position.ly = 0;
                self.win_counter = 0;
                self.change_mode(GpuMode::OamSearch);
            }

            cycles_left
        } else {
            self.clock += cycles;
            0
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
            GpuMode::PixelTransfer => {
                self.mode3_cycles = 0;
                self.position.lx = 0;
                self.fifo.restart();
                self.bg_fetcher.restart();
                self.drawing_window = false;
                self.fifo.unaligned_scx = self.position.scroll_x % 8;

                if self.is_win_enabled()
                    && self.position.window_x < 7
                    && self.position.window_y <= self.position.ly
                {
                    self.drawing_window = true;
                    self.fifo.unaligned_winx = 7 - self.position.window_x;
                } else {
                    self.fifo.unaligned_winx = 0;
                }
            }
            GpuMode::HBlank => {
                if self.stat.hblank_int != 0 {
                    self.request_lcd_interrupt();
                }
            }
            GpuMode::VBlank if self.stat.vblank_int != 0 => self.request_lcd_interrupt(),
            _ => (),
        }
    }

    #[inline]
    fn request_lcd_interrupt(&mut self) {
        self.request_lcd_int = true;
    }

    fn clear_screen(&mut self) {
        for i in 0..self.lcd.len() {
            self.lcd[i] = 255;
        }
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

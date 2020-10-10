// References:
//  - Ultimate Gameboy Talk: https://www.youtube.com/watch?v=HyzD8pNlpwI
//  - Nitty Gritty Gameboy Cycle Timing:
//      http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.txt
//  - SameBoy: https://github.com/LIJI32/SameBoy/blob/master/Core/display.c
// To implement the Pixel FIFO, I have pretty much copied (a simplified version of) SameBoy's
// pixel pipeline and adapted it here. Some of my (incomplete) notes can be found
// in sameboy_pixel_pipeline.md. I also consulted LIJI and got advice/help from him and
// several others.

pub mod registers;
pub mod tiles;

use crate::cpu::EmulationMode;
use crate::gpu::registers::{ColorPalette, LcdControl, LcdPosition, LcdStatus, MonochromePalette};
use crate::gpu::tiles::Sprite;
use std::collections::VecDeque;

const VRAM_BANK_SIZE: usize = 0x2000;
const OAM_SIZE: usize = 0xA0;
const PALETTE_RAM_SIZE: usize = 0x40;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const SCREEN_DEPTH: usize = 4;
const VRAM_OFFSET: u16 = 0x8000;
pub const OAM_OFFSET: u16 = 0xFE00;
const SCX_TO_WX0_COMPARE: [i16; 8] = [-7, -9, -10, -11, -12, -13, -14, -14];
const CYCLES_IN_LINE: usize = 456;

#[derive(Debug, PartialEq, Clone)]
/// Mode 0 - HBlank
///
/// Mode 1 - VBlank
///
/// Mode 2 - OamSearch
///
/// Mode 3 - PixelTransfer
pub enum GpuMode {
    OamSearch,
    PixelTransfer,
    HBlank,
    VBlank,
    InitPixelTransfer,
}

impl From<&GpuMode> for u8 {
    fn from(mode: &GpuMode) -> u8 {
        match mode {
            GpuMode::HBlank => 0,
            GpuMode::VBlank => 1,
            GpuMode::OamSearch => 2,
            GpuMode::InitPixelTransfer => 3,
            GpuMode::PixelTransfer => 3,
        }
    }
}

#[derive(Debug, Default)]
pub struct PixelFifoItem {
    pub value: u8,
    pub palette_num: u8,
    pub obj_to_bg_prio: u8,
    pub obj_to_obj_prio: u8,
    pub bg_to_oam_prio: u8,
}

pub struct BgFifo {
    pub q: VecDeque<PixelFifoItem>,
}

impl BgFifo {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(8),
        }
    }

    pub fn push_row(
        &mut self,
        mut low: u8,
        mut high: u8,
        palette_num: u8,
        flip_x: bool,
        bg_to_oam_prio: u8,
    ) {
        if flip_x {
            low = low.reverse_bits();
            high = high.reverse_bits();
        }

        for _ in 0..8 {
            let value = ((high >> 7) << 1) | (low >> 7);

            self.q.push_back(PixelFifoItem {
                value,
                palette_num,
                obj_to_bg_prio: 0,
                obj_to_obj_prio: 0,
                bg_to_oam_prio,
            });

            low <<= 1;
            high <<= 1;
        }
    }

    pub fn pop(&mut self) -> Option<PixelFifoItem> {
        self.q.pop_front()
    }

    pub fn is_empty(&self) -> bool {
        self.q.is_empty()
    }

    pub fn clear(&mut self) {
        self.q.clear();
    }
}

pub struct ObjFifo {
    pub q: VecDeque<PixelFifoItem>,
}

impl ObjFifo {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(8),
        }
    }

    pub fn push_row(
        &mut self,
        mut low: u8,
        mut high: u8,
        palette_num: u8,
        obj_to_bg_prio: u8,
        obj_to_obj_prio: u8,
        flip: bool,
    ) {
        if flip {
            low = low.reverse_bits();
            high = high.reverse_bits();
        }

        while self.q.len() < 8 {
            self.q.push_back(PixelFifoItem::default());
        }

        for i in 0..8 {
            let value = ((high >> 7) << 1) | (low >> 7);

            let old_item = &self.q[i];

            if value != 0 && (old_item.value == 0 || obj_to_obj_prio < old_item.obj_to_obj_prio) {
                self.q[i] = PixelFifoItem {
                    value,
                    palette_num,
                    obj_to_bg_prio,
                    obj_to_obj_prio,
                    bg_to_oam_prio: 0,
                };
            }

            low <<= 1;
            high <<= 1;
        }
    }

    pub fn pop(&mut self) -> Option<PixelFifoItem> {
        self.q.pop_front()
    }

    pub fn clear(&mut self) {
        self.q.clear();
    }
}

#[derive(Debug)]
pub enum FetcherState {
    Sleep0,
    ReadTileMap,
    Sleep1,
    ReadTileLow,
    Sleep2,
    ReadTileHigh,
    Push0,
    Push1,
}

impl From<&FetcherState> for usize {
    fn from(state: &FetcherState) -> Self {
        match state {
            FetcherState::Sleep0 => 0,
            FetcherState::ReadTileMap => 1,
            FetcherState::Sleep1 => 2,
            FetcherState::ReadTileLow => 3,
            FetcherState::Sleep2 => 4,
            FetcherState::ReadTileHigh => 5,
            FetcherState::Push0 => 6,
            FetcherState::Push1 => 7,
        }
    }
}

pub struct Fetcher {
    pub state: FetcherState,
    pub x: u8,
    pub y: u8,
    pub win_tile_x: u8,
    pub current_tile: u8,
    pub low: u8,
    pub high: u8,
    pub current_tile_attr: u8,
}

impl Fetcher {
    pub fn new() -> Self {
        Self {
            state: FetcherState::Sleep0,
            x: 0,
            y: 0,
            win_tile_x: 0,
            current_tile: 0,
            low: 0,
            high: 0,
            current_tile_attr: 0,
        }
    }

    pub fn advance_state(&mut self) {
        self.state = match self.state {
            FetcherState::Sleep0 => FetcherState::ReadTileMap,
            FetcherState::ReadTileMap => FetcherState::Sleep1,
            FetcherState::Sleep1 => FetcherState::ReadTileLow,
            FetcherState::ReadTileLow => FetcherState::Sleep2,
            FetcherState::Sleep2 => FetcherState::ReadTileHigh,
            FetcherState::ReadTileHigh => FetcherState::Push0,
            FetcherState::Push0 => FetcherState::Push1,
            FetcherState::Push1 => FetcherState::Sleep0,
        };
    }
}

#[derive(Debug)]
enum SpriteFetchState {
    AdvanceFetcher0,
    AdvanceFetcher1,
    Idle0,
    Idle1,
    LineAddrLow,
    SpriteOverlay,
}

pub struct Gpu {
    pub lcd: Vec<u8>,
    pub vram0: Vec<u8>,
    pub vram1: Vec<u8>,
    pub bgp_ram: Vec<u8>,
    pub obp_ram: Vec<u8>,
    pub oam: Vec<u8>,
    cgbp: ColorPalette,
    emu_mode: EmulationMode,
    lcdc: LcdControl,
    dmgp: MonochromePalette,
    position: LcdPosition,
    stat: LcdStatus,
    clock: usize,
    pub request_vblank_int: bool,
    pub request_lcd_int: bool,
    vram_bank: usize,
    win_counter: i16,
    pub oam_dma_active: bool,

    stat_int_signal: bool,
    lyc_int_signal: bool,

    // new stuff
    pub mode3_clocks: usize,
    lx: i16,
    bg_fifo: BgFifo,
    fetcher: Fetcher,
    wy_triggered: bool,
    wx_triggered: bool,
    comparators: Vec<i16>,
    locations: Vec<usize>,
    // sprites: Vec<Sprite>,
    search_idx: usize,

    sprite_i: usize,
    in_sprite_fetch: bool,
    sprite_fetch_state: SpriteFetchState,
    obj_fifo: ObjFifo,
    cancel_sprite_fetch: bool,
    sprite0_penalty: u8,

    stat_int_update_pending: bool,

    pub mode2_clocks: usize,

    next_mode: GpuMode,

    first_line0: bool,
    line0_clocks: usize,

    pub vblank_event: bool,
    pub hdma_flag: bool,
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
            win_counter: -1,
            oam_dma_active: false,

            stat_int_signal: false,
            lyc_int_signal: false,

            // new stuff
            mode3_clocks: 172,
            lx: 0,
            bg_fifo: BgFifo::new(),
            fetcher: Fetcher::new(),
            wy_triggered: false,
            wx_triggered: false,

            comparators: Vec::with_capacity(10),
            locations: Vec::with_capacity(10),
            // sprites: Vec::with_capacity(40),
            search_idx: 0,

            sprite_i: 0,
            in_sprite_fetch: false,
            sprite_fetch_state: SpriteFetchState::AdvanceFetcher0,
            obj_fifo: ObjFifo::new(),
            cancel_sprite_fetch: false,
            sprite0_penalty: 0,

            stat_int_update_pending: true,

            mode2_clocks: 0,

            next_mode: GpuMode::OamSearch,

            first_line0: true,
            line0_clocks: 0,

            vblank_event: false,
            hdma_flag: false,
        }
    }

    pub fn mode(&self) -> &GpuMode {
        &self.stat.mode
    }

    pub fn screen(&self) -> *const u8 {
        self.lcd.as_ptr()
    }

    pub fn tick(&mut self, mut cycles: usize) {
        if !self.lcdc.display_enabled() {
            return;
        }

        if self.first_line0 {
            cycles = self.line0_tick(cycles);
        }

        while cycles > 0 {
            if self.stat.mode != self.next_mode {
                self.change_mode(self.next_mode.clone());
            }

            match self.stat.mode {
                GpuMode::OamSearch => cycles = self.run_oam_search(cycles),
                GpuMode::InitPixelTransfer => cycles = self.run_init_pixel_transfer(cycles),
                GpuMode::PixelTransfer => cycles = self.run_pixel_transfer(cycles),
                GpuMode::HBlank => cycles = self.run_hblank(cycles),
                GpuMode::VBlank => cycles = self.run_vblank(cycles),
            }
        }
    }

    fn line0_tick(&mut self, mut cycles: usize) -> usize {
        if self.line0_clocks == 0 {
            if self.position.wy == 0 {
                self.wy_triggered = true;
            } else {
                self.wy_triggered = false;
            }
        }

        if self.line0_clocks + cycles >= 80 - 2 {
            cycles = self.line0_clocks + cycles - (80 - 2);
            self.line0_clocks = 80 - 2;

            self.mode2_clocks = 80 - 2 + 8;
            self.next_mode = GpuMode::InitPixelTransfer;

            self.first_line0 = false;

            cycles
        } else {
            self.line0_clocks += cycles;

            0
        }
    }

    fn run_oam_search(&mut self, mut cycles: usize) -> usize {
        if self.stat_int_update_pending {
            self.update_stat_int_signal();
            self.stat_int_update_pending = false;
        }

        while cycles > 0 {
            self.mode2_clocks += 1;
            cycles -= 1;
            self.clock += 1;

            if self.clock % 2 != 0 {
                continue;
            }

            // let sprite = Sprite::from(&self.oam[self.search_idx * 4..(self.search_idx + 1) * 4]);

            // self.sprites.push(sprite);

            if self.comparators.len() < 10 {
                let sprite =
                    Sprite::from(&self.oam[self.search_idx * 4..(self.search_idx + 1) * 4]);
                // let sprite = &self.sprites[self.search_idx];
                let y = self.position.ly + 16;
                let height = if self.lcdc.obj_size == 0 { 8 } else { 16 };

                if (y >= sprite.y) && (y < (sprite.y + height)) {
                    self.insert_sprite(sprite);
                }
            }

            self.search_idx += 1;

            if self.clock == 80 {
                self.next_mode = GpuMode::InitPixelTransfer;
                return cycles;
            }
        }

        0
    }

    fn insert_sprite(&mut self, sprite: Sprite) {
        // let sprite = &self.sprites[self.search_idx];

        let x = sprite.x as i16 - 8;
        let mut i = 0;

        while i < self.comparators.len() {
            if self.comparators[i] <= x {
                i += 1;
            } else {
                break;
            }
        }

        self.comparators.insert(i, x);
        self.locations.insert(i, self.search_idx);
    }

    fn run_init_pixel_transfer(&mut self, cycles: usize) -> usize {
        if self.stat_int_update_pending {
            self.update_stat_int_signal();
            self.stat_int_update_pending = false;
        }

        if self.clock + cycles >= 5 {
            let cycles_left = self.clock + cycles - 5;
            self.mode3_clocks = 5;

            if self.position.ly == self.position.wy {
                self.wy_triggered = true;
            }

            self.next_mode = GpuMode::PixelTransfer;

            cycles_left
        } else {
            self.clock += cycles;

            0
        }
    }

    fn run_pixel_transfer(&mut self, mut cycles: usize) -> usize {
        while cycles > 0 {
            cycles -= 1;
            self.mode3_clocks += 1;

            self.pixel_transfer_tick();

            if self.lx == 160 {
                self.next_mode = GpuMode::HBlank;
                return cycles;
            }
        }

        0
    }

    fn pixel_transfer_tick(&mut self) {
        // Window
        if !self.in_sprite_fetch
            && !self.wx_triggered
            && self.wy_triggered
            && self.lcdc.win_display_enable != 0
        {
            if self.position.wx == 0 {
                if self.lx == SCX_TO_WX0_COMPARE[(self.position.scx % 8) as usize] {
                    self.trigger_window();
                }
            } else if self.lx + 7 == self.position.wx as i16 {
                self.trigger_window();
            }
        }

        // Sprites
        while !self.cancel_sprite_fetch
            && !self.in_sprite_fetch
            && (self.sprite_i < self.comparators.len())
            && (self.lx >= -8 && self.lx < 160)
            && (self.comparators[self.sprite_i] < self.lx)
        {
            self.sprite_i += 1;
        }

        if !self.cancel_sprite_fetch
            && (self.in_sprite_fetch
                || (self.sprite_i < self.comparators.len())
                    && (self.lcdc.obj_enabled() || self.emu_mode == EmulationMode::Cgb)
                    && (self.comparators[self.sprite_i] == self.lx))
        {
            self.sprite_fetch_tick();
            return;
        }

        self.fifo_tick();
        self.fetcher_tick();
        self.cancel_sprite_fetch = false;
    }

    fn sprite_fetch_tick(&mut self) {
        if usize::from(&self.fetcher.state) < 5 || self.bg_fifo.is_empty() {
            self.fetcher_tick();

            if self.cancel_sprite_fetch {
                self.in_sprite_fetch = false;
            }

            return;
        }

        if self.sprite0_penalty > 0 && self.comparators[self.sprite_i] == -8 {
            self.sprite0_penalty -= 1;

            if self.cancel_sprite_fetch {
                self.in_sprite_fetch = false;
            }

            return;
        }

        match self.sprite_fetch_state {
            SpriteFetchState::AdvanceFetcher0 => {
                self.fetcher_tick();

                self.advance_sprite_fetch_state();
            }
            SpriteFetchState::AdvanceFetcher1 => {
                self.fetcher_tick();

                self.advance_sprite_fetch_state();
            }
            SpriteFetchState::Idle0 => {
                self.advance_sprite_fetch_state();
            }
            SpriteFetchState::Idle1 => {
                self.advance_sprite_fetch_state();
            }
            SpriteFetchState::LineAddrLow => {
                self.advance_sprite_fetch_state();
            }
            SpriteFetchState::SpriteOverlay => {
                let i = self.locations[self.sprite_i];
                let sprite = Sprite::from(&self.oam[i * 4..(i + 1) * 4]);
                // let sprite = &self.sprites[i];

                let height16 = self.lcdc.obj_size != 0;

                let tile_num = sprite.number & if height16 { 0xFE } else { 0xFF };

                let height = if height16 { 16 } else { 8 };

                let y = if sprite.mirror_vertical {
                    height - 1 - (self.position.ly + 16 - sprite.y)
                } else {
                    self.position.ly + 16 - sprite.y
                };

                let addr = 0x8000u16 + tile_num * 16 + y as u16 * 2;

                let bank = match self.emu_mode {
                    EmulationMode::Dmg => 0,
                    EmulationMode::Cgb => sprite.vram_bank,
                };

                let low = self.get_vram_byte(addr, bank);
                let high = self.get_vram_byte(addr + 1, bank);

                let palette_num = match self.emu_mode {
                    EmulationMode::Dmg => sprite.obp1 as u8,
                    EmulationMode::Cgb => sprite.obp_num,
                };

                let obj_to_obj_prio = match self.emu_mode {
                    EmulationMode::Dmg => 0,
                    EmulationMode::Cgb => i as u8,
                };

                self.obj_fifo.push_row(
                    low,
                    high,
                    palette_num,
                    sprite.obj_to_bg_prio,
                    obj_to_obj_prio,
                    sprite.mirror_horizontal,
                );

                self.sprite_i += 1;
                self.in_sprite_fetch = false;
                self.advance_sprite_fetch_state();
            }
        }
    }

    fn advance_sprite_fetch_state(&mut self) {
        if self.cancel_sprite_fetch {
            self.in_sprite_fetch = false;
            self.sprite_fetch_state = SpriteFetchState::AdvanceFetcher1;
            return;
        }

        self.sprite_fetch_state = match self.sprite_fetch_state {
            SpriteFetchState::AdvanceFetcher0 => SpriteFetchState::AdvanceFetcher1,
            SpriteFetchState::AdvanceFetcher1 => SpriteFetchState::Idle0,
            SpriteFetchState::Idle0 => SpriteFetchState::Idle1,
            SpriteFetchState::Idle1 => SpriteFetchState::LineAddrLow,
            SpriteFetchState::LineAddrLow => SpriteFetchState::SpriteOverlay,
            SpriteFetchState::SpriteOverlay => SpriteFetchState::AdvanceFetcher0,
        }
    }

    #[inline]
    fn trigger_window(&mut self) {
        self.wx_triggered = true;
        self.fetcher.win_tile_x = 0;
        self.win_counter += 1;

        self.bg_fifo.clear();
        self.fetcher.state = FetcherState::Sleep0;
    }

    fn fifo_tick(&mut self) {
        if let Some(px) = self.bg_fifo.pop() {
            let mut draw_sprite = false;
            let mut bg_over_sprite = px.bg_to_oam_prio;

            let spx = match self.obj_fifo.pop() {
                Some(spx) => {
                    if spx.value > 0 && self.lcdc.obj_display_enable != 0 {
                        draw_sprite = true;
                        bg_over_sprite |= spx.obj_to_bg_prio;
                    }

                    spx
                }
                None => PixelFifoItem::default(),
            };

            if self.lx < 0 {
                self.lx += 1;
                return;
            }

            let mut value = px.value;

            if self.lcdc.lcdc0 == 0 {
                match self.emu_mode {
                    EmulationMode::Dmg => value = 0,
                    EmulationMode::Cgb => bg_over_sprite = 0,
                }
            }

            if value != 0 && bg_over_sprite != 0 {
                draw_sprite = false;
            }

            let mut palette = match self.emu_mode {
                EmulationMode::Dmg => self.dmgp.bgp as u16,
                EmulationMode::Cgb => self.cgb_bg_palette(px, value),
            };

            if draw_sprite {
                value = spx.value;

                palette = match self.emu_mode {
                    EmulationMode::Dmg => {
                        if spx.palette_num == 0 {
                            self.dmgp.obp0 as u16
                        } else {
                            self.dmgp.obp1 as u16
                        }
                    }
                    EmulationMode::Cgb => self.cgb_obj_palette(spx, value),
                }
            }

            let (r, g, b) = self.get_rgb(value, palette);
            self.write_lcd(r, g, b);

            self.lx += 1;
        }
    }

    #[inline]
    fn cgb_bg_palette(&self, px: PixelFifoItem, value: u8) -> u16 {
        let palette_idx = px.palette_num as usize * 8;
        let color_idx = palette_idx + value as usize * 2;
        (self.bgp_ram[color_idx + 1] as u16) << 8 | self.bgp_ram[color_idx + 0] as u16
    }

    #[inline]
    fn cgb_obj_palette(&self, spx: PixelFifoItem, value: u8) -> u16 {
        let palette_idx = spx.palette_num as usize * 8;
        let color_idx = palette_idx + value as usize * 2;
        (self.obp_ram[color_idx + 1] as u16) << 8 | self.obp_ram[color_idx + 0] as u16
    }

    fn fetcher_tick(&mut self) {
        match self.fetcher.state {
            FetcherState::Sleep0 | FetcherState::Sleep1 | FetcherState::Sleep2 => {
                self.fetcher.advance_state();
            }
            FetcherState::ReadTileMap => {
                if self.lcdc.win_display_enable == 0 {
                    self.wx_triggered = false;
                }

                let map = if self.wx_triggered {
                    self.lcdc.win_tilemap()
                } else {
                    self.lcdc.bg_tilemap()
                };

                self.fetcher.y = self.fetcher_y();

                let x = if self.wx_triggered {
                    self.fetcher.win_tile_x
                } else {
                    (self.position.scx / 8 + self.fetcher.x) % 32
                };

                let addr = map + (self.fetcher.y / 8) as u16 * 32 + x as u16;
                self.fetcher.current_tile = self.get_vram_byte(addr, 0);

                if self.emu_mode == EmulationMode::Cgb {
                    self.fetcher.current_tile_attr = self.get_vram_byte(addr, 1);
                }

                self.fetcher.advance_state();
            }
            FetcherState::ReadTileLow => {
                let row = if self.fetcher.current_tile_attr & 0x40 != 0 {
                    (self.fetcher.y % 8) ^ 0x7
                } else {
                    self.fetcher.y % 8
                };

                let addr = self.tiledata_addr(self.lcdc.tiledata_sel, self.fetcher.current_tile);

                let bank = ((self.fetcher.current_tile_attr >> 3) & 0x1) as usize;
                self.fetcher.low = self.get_vram_byte(addr + row as u16 * 2, bank);

                self.fetcher.advance_state();
            }
            FetcherState::ReadTileHigh => {
                let row = if self.fetcher.current_tile_attr & 0x40 != 0 {
                    (self.fetcher.y % 8) ^ 0x7
                } else {
                    self.fetcher.y % 8
                };

                let addr = self.tiledata_addr(self.lcdc.tiledata_sel, self.fetcher.current_tile);

                let bank = ((self.fetcher.current_tile_attr >> 3) & 0x1) as usize;
                self.fetcher.high = self.get_vram_byte(addr + row as u16 * 2 + 1, bank);

                if self.wx_triggered {
                    self.fetcher.win_tile_x = (self.fetcher.win_tile_x + 1) % 32;
                }

                self.fetcher.advance_state();

                if self.bg_fifo.is_empty() {
                    self.bg_fifo.push_row(
                        self.fetcher.low,
                        self.fetcher.high,
                        self.fetcher.current_tile_attr & 0x7,
                        self.fetcher.current_tile_attr & 0x20 != 0,
                        self.fetcher.current_tile_attr & 0x80,
                    );

                    self.fetcher.state = FetcherState::Sleep0;
                }
            }
            FetcherState::Push0 => {
                self.fetcher.x = (self.fetcher.x + 1) % 32;

                self.fetcher.advance_state();

                if self.bg_fifo.is_empty() {
                    self.bg_fifo.push_row(
                        self.fetcher.low,
                        self.fetcher.high,
                        self.fetcher.current_tile_attr & 0x7,
                        self.fetcher.current_tile_attr & 0x20 != 0,
                        self.fetcher.current_tile_attr & 0x80,
                    );

                    self.fetcher.state = FetcherState::Sleep0;
                }
            }
            FetcherState::Push1 => {
                if self.bg_fifo.is_empty() {
                    self.bg_fifo.push_row(
                        self.fetcher.low,
                        self.fetcher.high,
                        self.fetcher.current_tile_attr & 0x7,
                        self.fetcher.current_tile_attr & 0x20 != 0,
                        self.fetcher.current_tile_attr & 0x80,
                    );

                    self.fetcher.advance_state();
                }
            }
        }
    }

    fn fetcher_y(&self) -> u8 {
        if self.wx_triggered {
            self.win_counter as u8
        } else {
            self.position.ly.wrapping_add(self.position.scy)
        }
    }

    fn tiledata_addr(&self, sel: u8, idx: u8) -> u16 {
        if sel == 0 {
            0x8800u16 + (idx as i8 as i16 + 128) as u16 * 16
        } else {
            0x8000u16 + (idx as u16 * 16)
        }
    }

    fn run_hblank(&mut self, cycles: usize) -> usize {
        if self.stat_int_update_pending {
            self.update_stat_int_signal();
            self.stat_int_update_pending = false;
        }

        let hblank_clocks = CYCLES_IN_LINE - (self.mode2_clocks + self.mode3_clocks);

        if self.clock + cycles >= hblank_clocks {
            let cycles_left = self.clock + cycles - hblank_clocks;
            self.position.ly += 1;
            self.update_stat_int_signal();

            if self.position.ly > 143 {
                if self.stat.oam_int != 0 && self.emu_mode == EmulationMode::Dmg {
                    self.stat.mode = GpuMode::OamSearch;
                    self.update_stat_int_signal();
                    self.stat.mode = GpuMode::HBlank;
                }

                self.next_mode = GpuMode::VBlank;
                // self.request_vblank_interrupt();
                self.vblank_event = true;
            } else {
                self.next_mode = GpuMode::OamSearch;
            }

            cycles_left
        } else {
            self.clock += cycles;
            0
        }
    }

    // Mode 1 - V-Blank
    fn run_vblank(&mut self, cycles: usize) -> usize {
        if self.stat_int_update_pending {
            self.update_stat_int_signal();
            self.stat_int_update_pending = false;
        }

        if self.clock + cycles >= 456 {
            let cycles_left = self.clock + cycles - 456;
            self.clock = 0;
            self.position.ly += 1;

            if self.position.ly > 1 {
                self.update_stat_int_signal();
            }

            // STRANGE BEHAVIOR
            if self.position.ly == 153 {
                self.position.ly = 0;
                self.update_stat_int_signal();
            }

            if self.position.ly == 1 {
                self.position.ly = 0;
                self.win_counter = -1;
                self.wy_triggered = false;
                self.next_mode = GpuMode::OamSearch;
            }

            cycles_left
        } else {
            self.clock += cycles;
            0
        }
    }

    fn change_mode(&mut self, mode: GpuMode) {
        self.clock = 0;
        self.stat.mode = mode;

        if self.stat.mode != GpuMode::PixelTransfer {
            self.stat_int_update_pending = true;
        }

        match self.stat.mode {
            GpuMode::HBlank => {
                self.hdma_flag = true;
            }
            GpuMode::VBlank => {
                self.request_vblank_interrupt();
                // self.update_stat_int_signal();
            }
            GpuMode::OamSearch => {
                // self.sprites.clear();
                self.comparators.clear();
                self.locations.clear();
                self.search_idx = 0;

                self.mode2_clocks = 0;
            }
            GpuMode::PixelTransfer => {
                // clear FIFOs
                self.bg_fifo.clear();
                // initial offset to accomodate 8 'junk pixels'
                self.lx = -8;
                // further offset for scroll x
                self.lx -= (self.position.scx % 8) as i16;
                // push 8 'junk' pixels to fifo
                self.bg_fifo.push_row(0, 0, 0, false, 0);
                // reset fetcher
                self.fetcher.x = 0;
                self.fetcher.state = FetcherState::Sleep0;
                // reset wx_triggered
                self.wx_triggered = false;
                // reset sprite vars
                self.sprite_i = 0;
                self.in_sprite_fetch = false;
                self.sprite_fetch_state = SpriteFetchState::AdvanceFetcher0;
                self.cancel_sprite_fetch = false;
                self.obj_fifo.clear();
                self.sprite0_penalty = self.position.scx % 8;
            }
            _ => (),
        }
    }

    fn update_stat_int_signal(&mut self) {
        let old_signal = self.stat_int_signal;

        if self.position.ly == self.position.lyc {
            self.stat.coincident = 0x4;
            self.lyc_int_signal = true;
        } else {
            self.stat.coincident = 0x0;
            self.lyc_int_signal = false;
        }

        self.stat_int_signal = match self.stat.mode {
            GpuMode::OamSearch => self.stat.oam_int != 0,
            GpuMode::HBlank => self.stat.hblank_int != 0,
            GpuMode::VBlank => self.stat.vblank_int != 0,
            _ => false,
        };

        if self.lyc_int_signal && self.stat.lyc_int != 0 {
            self.stat_int_signal = true;
        }

        self.stat_int_trigger(old_signal);
    }

    #[inline]
    fn stat_int_trigger(&mut self, old_signal: bool) {
        if !old_signal && self.stat_int_signal {
            self.request_lcd_interrupt();
        }
    }

    #[inline]
    fn request_lcd_interrupt(&mut self) {
        self.request_lcd_int = true;
    }

    fn get_rgb(&self, value: u8, palette: u16) -> (u8, u8, u8) {
        match self.emu_mode {
            EmulationMode::Dmg => match (palette >> (2 * value)) & 0x3 {
                0 => (224, 247, 208),
                1 => (136, 192, 112),
                2 => (52, 104, 86),
                _ => (8, 23, 33),
            },
            EmulationMode::Cgb => {
                let r = (palette & 0x001F) >> 0;
                let g = (palette & 0x03E0) >> 5;
                let b = (palette & 0x7C00) >> 10;

                (
                    ((r << 3) | (r >> 2)) as u8,
                    ((g << 3) | (g >> 2)) as u8,
                    ((b << 3) | (b >> 2)) as u8,
                )
            }
        }
    }

    #[inline]
    fn write_lcd(&mut self, r: u8, g: u8, b: u8) {
        let ly = self.position.ly as usize;
        let lx = self.lx as usize;

        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + lx * SCREEN_DEPTH + 0] = r;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + lx * SCREEN_DEPTH + 1] = g;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + lx * SCREEN_DEPTH + 2] = b;
        self.lcd[ly * SCREEN_WIDTH * SCREEN_DEPTH + lx * SCREEN_DEPTH + 3] = 255;
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

                // on -> off
                if old_display_enable != 0 && self.lcdc.display_enable == 0 {
                    self.turn_display_off();
                }
                // off -> on
                if old_display_enable == 0 && self.lcdc.display_enable != 0 {
                    self.turn_display_on();
                }

                self.lcdc.win_tilemap_sel = value & 0x40;
                self.lcdc.win_display_enable = value & 0x20;
                self.lcdc.tiledata_sel = value & 0x10;
                self.lcdc.bg_tilemap_sel = value & 0x08;
                self.lcdc.obj_size = value & 0x04;

                if self.lcdc.obj_display_enable != 0 && (value & 0x02) == 0 {
                    self.cancel_sprite_fetch = true;
                }

                let old_obj_display_enable = self.lcdc.obj_display_enable;
                self.lcdc.obj_display_enable = value & 0x02;
                if old_obj_display_enable == 0 && self.lcdc.obj_display_enable != 0 {
                    println!(
                        "ly: {}, lx: {}, mode: {:?}",
                        self.position.ly, self.lx, self.stat.mode
                    );
                }
                self.lcdc.lcdc0 = value & 0x01;
            }
            0xFF41 => {
                self.stat.lyc_int = value & 0x40;
                self.stat.oam_int = value & 0x20;
                self.stat.vblank_int = value & 0x10;
                self.stat.hblank_int = value & 0x08;
                self.update_stat_int_signal();
            }
            0xFF42 => self.position.scy = value,
            0xFF43 => self.position.scx = value,
            0xFF44 => (),
            0xFF45 => {
                self.position.lyc = value;
                self.update_stat_int_signal();
            }
            0xFF47 => self.dmgp.bgp = value,
            0xFF48 => self.dmgp.obp0 = value,
            0xFF49 => self.dmgp.obp1 = value,
            0xFF4A => {
                self.position.wy = value;
                if self.position.ly == self.position.wy {
                    self.wy_triggered = true;
                }
            }
            0xFF4B => self.position.wx = value,
            0xFF4F => {
                if self.emu_mode == EmulationMode::Cgb {
                    self.vram_bank = (value & 0x01) as usize;
                }
            }
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
                GpuMode::PixelTransfer => 0xFF,
                _ => self.get_vram_byte(addr, self.vram_bank),
            },
            0xFE00..=0xFE9F => match self.stat.mode {
                GpuMode::OamSearch | GpuMode::PixelTransfer => 0xFF,
                _ => self.oam[(addr - OAM_OFFSET) as usize],
            },
            0xFF40 => u8::from(&self.lcdc),
            0xFF41 => u8::from(&self.stat),
            0xFF42 => self.position.scy,
            0xFF43 => self.position.scx,
            0xFF44 => self.position.ly,
            0xFF45 => self.position.lyc,
            // Write only register FF46
            0xFF46 => 0xFF,
            0xFF47 => self.dmgp.bgp,
            0xFF48 => self.dmgp.obp0,
            0xFF49 => self.dmgp.obp1,
            0xFF4A => self.position.wy,
            0xFF4B => self.position.wx,
            0xFF4F => {
                if self.emu_mode == EmulationMode::Cgb {
                    0xFE | self.vram_bank as u8
                } else {
                    0xFF
                }
            }
            0xFF68 if self.emu_mode == EmulationMode::Cgb => self.cgbp.bgp(),
            0xFF69 if self.emu_mode == EmulationMode::Cgb => {
                self.bgp_ram[self.cgbp.bgp_idx as usize]
            }
            0xFF6A if self.emu_mode == EmulationMode::Cgb => self.cgbp.obp(),
            0xFF6B if self.emu_mode == EmulationMode::Cgb => {
                self.obp_ram[self.cgbp.obp_idx as usize]
            }
            _ => {
                println!("Unexpected addr in gpu.get_byte {:#X}", addr);
                0xFF
            }
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
            _ => panic!("Unexpected addr in get_vram_byte {:#X}", addr),
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
            _ => panic!("Unexpected addr in get_vram_byte {:#X}", addr),
        }
    }

    #[inline]
    fn request_vblank_interrupt(&mut self) {
        self.request_vblank_int = true;
    }

    #[inline]
    fn turn_display_off(&mut self) {
        self.stat.mode = GpuMode::HBlank;

        self.position.ly = 0;

        self.wx_triggered = false;
        self.win_counter = -1;

        self.clear_screen();
    }

    #[inline]
    fn turn_display_on(&mut self) {
        self.clock = 0;
        self.line0_clocks = 0;

        self.comparators.clear();
        self.locations.clear();
        self.search_idx = 0;
        self.mode2_clocks = 0;

        self.mode3_clocks = 0;

        let lyc = self.position.lyc;
        self.stat.mode = GpuMode::InitPixelTransfer;
        self.position.lyc = 0;
        self.update_stat_int_signal();
        self.position.lyc = lyc;

        self.stat.mode = GpuMode::HBlank;

        self.first_line0 = true;
    }
}

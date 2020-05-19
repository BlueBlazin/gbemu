use crate::cpu::EmulationMode;
use crate::gpu::GpuMode;

// Bit 7 - LCD Display Enable             (0=Off, 1=On)
// Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
// Bit 5 - Window Display Enable          (0=Off, 1=On)
// Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
// Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
// Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
// Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
// Bit 0 - BG/Window Display/Priority     (0=Off, 1=On)
#[derive(Default)]
pub struct LcdControl {
    pub display_enable: u8,
    pub win_tilemap_sel: u8,
    pub win_display_enable: u8,
    pub bg_tiledata_sel: u8,
    pub bg_tilemap_sel: u8,
    pub obj_size: u8,
    pub obj_display_enable: u8,
    pub lcdc0: u8,
}

impl LcdControl {
    pub fn display_enabled(&self) -> bool {
        self.display_enable != 0
    }

    pub fn win_tilemap(&self) -> u16 {
        if self.win_tilemap_sel == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn bg_tilemap(&self) -> u16 {
        if self.bg_tilemap_sel == 0 {
            0x9800
        } else {
            0x9C00
        }
    }

    pub fn window_enabled(&self, mode: &EmulationMode) -> bool {
        match mode {
            EmulationMode::Dmg => self.lcdc0 != 0 && self.win_display_enable != 0,
            EmulationMode::Cgb => self.win_display_enable != 0,
        }
    }

    pub fn obj_enabled(&self) -> bool {
        self.obj_display_enable != 0
    }
}

impl From<&LcdControl> for u8 {
    fn from(lcdc: &LcdControl) -> u8 {
        0x0 | lcdc.display_enable
            | lcdc.win_tilemap_sel
            | lcdc.win_display_enable
            | lcdc.bg_tiledata_sel
            | lcdc.bg_tilemap_sel
            | lcdc.obj_size
            | lcdc.obj_display_enable
            | lcdc.lcdc0
    }
}

// Bit 6 - LYC=LY Coincidence Interrupt (1=Enable) (Read/Write)
// Bit 5 - Mode 2 OAM Interrupt         (1=Enable) (Read/Write)
// Bit 4 - Mode 1 V-Blank Interrupt     (1=Enable) (Read/Write)
// Bit 3 - Mode 0 H-Blank Interrupt     (1=Enable) (Read/Write)
// Bit 2 - Coincidence Flag  (0:LYC<>LY, 1:LYC=LY) (Read Only)
// Bit 1-0 - Mode Flag       (Mode 0-3, see below) (Read Only)
pub struct LcdStatus {
    pub lyc_int: u8,
    pub oam_int: u8,
    pub vblank_int: u8,
    pub hblank_int: u8,
    pub coincident: u8,
    pub mode: GpuMode,
}

impl Default for LcdStatus {
    fn default() -> Self {
        Self {
            lyc_int: 0,
            oam_int: 0,
            vblank_int: 0,
            hblank_int: 0,
            coincident: 0,
            mode: GpuMode::OamSearch,
        }
    }
}

impl From<&LcdStatus> for u8 {
    fn from(stat: &LcdStatus) -> u8 {
        0x0 | stat.lyc_int
            | stat.oam_int
            | stat.vblank_int
            | stat.hblank_int
            | stat.coincident
            | u8::from(&stat.mode)
    }
}

#[derive(Default)]
pub struct LcdPosition {
    pub scroll_y: u8,
    pub scroll_x: u8,
    pub ly: u8,
    pub lx: u8,
    pub lyc: u8,
    pub window_y: u8,
    pub window_x: u8,
}

#[derive(Default)]
pub struct MonochromePalette {
    pub bgp: u8,
    pub obp0: u8,
    pub obp1: u8,
}

#[derive(Default)]
pub struct ColorPalette {
    pub bgp_idx: u8,
    pub bgp_auto_incr: bool,
    pub obp_idx: u8,
    pub obp_auto_incr: bool,
}

impl ColorPalette {
    pub fn bgp(&self) -> u8 {
        (self.bgp_auto_incr as u8) << 7 | (self.bgp_idx & 0x3F)
    }

    pub fn obp(&self) -> u8 {
        (self.obp_auto_incr as u8) << 7 | (self.obp_idx & 0x3F)
    }
}

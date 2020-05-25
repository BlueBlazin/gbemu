use crate::gpu::Sprite;
use std::collections::VecDeque;

#[derive(PartialEq)]
pub enum PixelType {
    BgColor0,
    BgColorOpaque,
    BgPriorityOverride,
    SpriteColor0,
    SpriteOpaque,
}

#[derive(Debug)]
pub enum FetcherState {
    Sleep(usize),
    ReadTileNumber,
    ReadTileDataLow,
    ReadTileDataHigh,
    Push(usize),
}

#[derive(PartialEq)]
pub enum FetchType {
    Background,
    Window,
}

pub struct BgFetcher {
    pub state: FetcherState,
    pub fetching: FetchType,
    pub x: u8,
    pub win_x: u8,
    pub tile_num: u8,
    pub low: u8,
    pub high: u8,
}

impl BgFetcher {
    pub fn new() -> Self {
        Self {
            state: FetcherState::Sleep(0),
            fetching: FetchType::Background,
            x: 0,
            win_x: 0,
            tile_num: 0,
            low: 0xFF,
            high: 0xFF,
        }
    }

    pub fn reset(&mut self) {
        self.state = FetcherState::Sleep(0);
        self.fetching = FetchType::Background;
        self.x = 0;
        self.win_x = 0;
        self.tile_num = 0;
        self.low = 0xFF;
        self.high = 0xFF;
    }
}

pub struct PixelFifoItem {
    pub value: u8,
    pub palette: u8,
    pub pixel_type: PixelType,
}

pub struct BgFifo {
    pub q: VecDeque<PixelFifoItem>,
    pub scx: u8,
    pub winx: u8,
    pub objx: u8,
    pub lcdc0: u8,
}

impl BgFifo {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(16),
            scx: 0,
            winx: 0,
            objx: 0,
            lcdc0: 0,
        }
    }

    pub fn reset(&mut self, scroll_x: u8) {
        self.q.clear();
        self.scx = scroll_x % 8;
        self.winx = 0;
        self.objx = 0;
    }

    pub fn clear_fifo(&mut self) {
        self.q.clear();
    }

    pub fn size(&mut self) -> usize {
        self.q.len()
    }

    pub fn allow_push(&self) -> bool {
        self.q.len() <= 8
    }

    pub fn push(&mut self, mut low: u8, mut high: u8, palette: u8) {
        for i in 0..8 {
            let value = ((high >> 7) << 1) | (low >> 7);
            low <<= 1;
            high <<= 1;

            let pixel_type = if value == 0 {
                PixelType::BgColor0
            } else {
                PixelType::BgColorOpaque
            };

            self.q.push_back(PixelFifoItem {
                value,
                palette,
                pixel_type,
            });
        }
    }

    pub fn pop(&mut self) -> Option<PixelFifoItem> {
        if self.q.len() > 8 {
            self.q.pop_front()
        } else {
            None
        }
    }
}

pub struct SpriteFetcher {
    pub state: FetcherState,
    pub addr: u16,
    pub low: u8,
    pub high: u8,
}

impl SpriteFetcher {
    pub fn new() -> Self {
        Self {
            state: FetcherState::Sleep(0),
            addr: 0,
            low: 0,
            high: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = FetcherState::Sleep(0);
        self.addr = 0;
        self.low = 0;
        self.high = 0;
    }
}

pub struct SpriteFifo {
    pub q: VecDeque<PixelFifoItem>,
    pub objx: u8,
}

impl SpriteFifo {
    pub fn new() -> Self {
        Self {
            q: VecDeque::with_capacity(8),
            objx: 0,
        }
    }

    pub fn reset(&mut self) {
        self.q.clear();
        self.objx = 0;
    }

    pub fn size(&mut self) -> usize {
        self.q.len()
    }

    pub fn allow_push(&self) -> bool {
        self.q.len() <= 8
    }

    pub fn push(&mut self, mut low: u8, mut high: u8, sprite: &Sprite, palette: u8) {
        if sprite.x < 8 {
            self.objx = 8 - sprite.x;
        }

        for _ in 0..8 {
            let value;

            if sprite.mirror_horizontal {
                value = ((high & 1) << 1) | (low & 1);
                low >>= 1;
                high >>= 1;
            } else {
                value = ((high >> 7) << 1) | (low >> 7);
                low <<= 1;
                high <<= 1;
            }

            let pixel_type = if value == 0 {
                PixelType::SpriteColor0
            } else {
                PixelType::SpriteOpaque
            };

            self.q.push_back(PixelFifoItem {
                value,
                palette,
                pixel_type,
            });
        }
    }

    pub fn pop(&mut self) -> Option<PixelFifoItem> {
        self.q.pop_front()
    }
}

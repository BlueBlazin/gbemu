use crate::apu::Apu;
use crate::cartridge::Cartridge;
use crate::cpu::{CgbMode, EmulationMode};
use crate::gpu::{Gpu, GpuMode, OAM_OFFSET};
use crate::joypad::Joypad;
use crate::memory::bootrom::Bootrom;
use crate::memory::wram::Wram;
use crate::timer::Timer;

const HRAM_SIZE: usize = 0x007F;
const HRAM_OFFSET: u16 = 0xFF80;
const WRAM_OFFSET: u16 = 0xC000;
const ECHO_OFFSET: u16 = 0xE000;

#[derive(PartialEq)]
pub enum AddrBus {
    Main,
    Vram,
    Ram,
    Internal,
}

pub struct OamDma {
    pub active: bool,
    pub src_addr: u16,
    pub i: u16,
    pub just_launched: bool,
    pub pending: bool,
    pub restarting: bool,
    pub dst_addr: u16,
}

impl Default for OamDma {
    fn default() -> Self {
        Self {
            active: false,
            src_addr: 0,
            i: 0,
            just_launched: false,
            pending: false,
            restarting: false,
            dst_addr: 0,
        }
    }
}

pub enum HdmaType {
    NoHdma,
    HBlankDma,
    GPDma,
}

pub struct Hdma {
    pub hdma_type: HdmaType,
    pub new_hdma: bool,
    src: u16,
    dst: u16,
    blocks: u8,
}

impl Default for Hdma {
    fn default() -> Self {
        Self {
            hdma_type: HdmaType::NoHdma,
            new_hdma: false,
            src: 0,
            dst: 0,
            blocks: 0,
        }
    }
}

/// Memory Management Unit (MMU)
/// The MMU is responsible for all basic memory operations
/// as well as memory virtualization.
pub struct Mmu {
    pub bootrom: Bootrom,
    pub cartridge: Cartridge,
    pub gpu: Gpu,
    pub joypad: Joypad,
    pub apu: Apu,
    pub ie: u8,
    pub hdma: Hdma,
    pub oam_dma: OamDma,
    pub timer: Timer,
    wram: Wram,
    hram: [u8; HRAM_SIZE],
    serial_out: u8,
    emu_mode: EmulationMode,
    pub cgb_mode: CgbMode,
    request_serial_int: bool,
    oam_dma_cycles: usize,
}

impl Mmu {
    pub fn new(data: Vec<u8>, emu_mode: EmulationMode) -> Self {
        Mmu {
            bootrom: Bootrom::new(),
            cartridge: Cartridge::new(data),
            gpu: Gpu::new(emu_mode.clone()),
            joypad: Joypad::new(),
            apu: Apu::new(),
            ie: 0,
            hdma: Hdma::default(),
            oam_dma: OamDma::default(),
            timer: Timer::new(emu_mode.clone()),
            wram: Wram::new(),
            hram: [0; HRAM_SIZE],
            serial_out: 0,
            emu_mode,
            cgb_mode: CgbMode::new(),
            request_serial_int: false,
            oam_dma_cycles: 0,
        }
    }

    pub fn simulate_bootrom(&mut self) {
        match self.emu_mode {
            EmulationMode::Dmg => {
                self.set_byte(0xFF70, 0xFF);
                self.set_byte(0xFF4F, 0xFF);
                // self.set_byte(0xFF4D, 0xFF);
            }
            EmulationMode::Cgb => {
                self.set_byte(0xFF70, 0xF8);
                self.set_byte(0xFF4F, 0xFE);
                // self.set_byte(0xFF4D, 0x7E);

                self.set_byte(0xFF6C, 0xFE);
                self.set_byte(0xFF75, 0x8F);
            }
        }
        self.set_byte(0xFF00, 0xCF);
        self.set_byte(0xFF0F, 0xE1);
        self.set_byte(0xFFFF, 0);
    }

    pub fn in_hblank(&self) -> bool {
        self.gpu.mode() == &GpuMode::HBlank
    }

    pub fn gdma_tick(&mut self) -> usize {
        let blocks = self.hdma.blocks as usize;
        while self.hdma.blocks > 0 {
            self.hdma_transfer_block();
        }
        self.hdma.hdma_type = HdmaType::NoHdma;
        blocks * 32
    }

    pub fn hdma_tick(&mut self) -> usize {
        self.hdma_transfer_block();
        if self.hdma.blocks == 0 {
            self.hdma.hdma_type = HdmaType::NoHdma;
        }
        32
    }

    // pub fn oam_dma_tick(&mut self, mut cycles: usize) {
    //     self.oam_dma_cycles += cycles;

    //     if self.oam_dma.pending {
    //         self.gpu.oam_dma_active = true;
    //         self.oam_dma.pending = false;
    //     }

    //     if self.oam_dma.restarting {
    //         self.oam_dma.restarting = false;
    //     }

    //     if self.oam_dma_cycles >= 4 && self.oam_dma.just_launched {
    //         self.oam_dma.just_launched = false;
    //         self.oam_dma_cycles -= 4;
    //     }

    //     while self.oam_dma_cycles >= 4 {
    //         self.oam_dma_cycles -= 4;

    //         let offset = self.oam_dma.i;
    //         let value = self.get_byte(self.oam_dma.src_addr + offset);
    //         self.gpu.oam[(0xFE00 + offset - OAM_OFFSET) as usize] = value;

    //         self.oam_dma.i += 1;
    //         if self.oam_dma.i == 160 {
    //             self.deactivate_oam_dma();
    //             break;
    //         }
    //     }
    // }

    pub fn oam_dma_tick(&mut self, cycles: usize) {
        self.oam_dma_cycles += cycles;

        if self.oam_dma.pending {
            self.gpu.oam_dma_active = true;
            self.oam_dma.pending = false;
        }

        if self.oam_dma.restarting {
            self.oam_dma.restarting = false;
        }

        if self.oam_dma_cycles >= 4 && self.oam_dma.just_launched {
            self.oam_dma.just_launched = false;
            self.oam_dma_cycles -= 4;
        }

        while self.oam_dma_cycles >= 4 {
            self.oam_dma_cycles -= 4;

            if self.oam_dma.src_addr < 0xE000 {
                self.gpu.oam[self.oam_dma.i as usize] = self.get_byte(self.oam_dma.src_addr);
            } else {
                self.gpu.oam[self.oam_dma.i as usize] =
                    self.get_byte(self.oam_dma.src_addr & !0x2000);
            }

            self.oam_dma.src_addr += 1;
            self.oam_dma.dst_addr += 1;
            self.oam_dma.i += 1;

            if self.oam_dma.i == 160 {
                self.deactivate_oam_dma();
                break;
            }
        }
    }

    fn hdma_transfer_block(&mut self) {
        if self.hdma.blocks == 0 {
            return;
        }

        for _ in 0..16 {
            let value = self.get_byte(self.hdma.src);
            self.set_byte(0x8000 | (self.hdma.dst & 0x1FFF), value);
            self.hdma.src += 1;
            self.hdma.dst += 1;
        }

        self.hdma.blocks -= 1;
    }

    pub fn apu_tick(&mut self, cycles: usize) {
        self.apu.tick(cycles);
    }

    pub fn gpu_tick(&mut self, cycles: usize) {
        self.gpu.tick(cycles);
    }

    pub fn timer_tick(&mut self, cycles: usize) {
        self.timer.tick(cycles);
    }

    pub fn screen(&self) -> *const u8 {
        self.gpu.screen()
    }

    fn addr_bus(&self, addr: u16) -> AddrBus {
        if addr < 0x8000 {
            return AddrBus::Main;
        }

        if addr < 0xA000 {
            return AddrBus::Vram;
        }

        if addr < 0xC000 {
            return AddrBus::Main;
        }

        if addr < 0xFE00 {
            return if self.emu_mode == EmulationMode::Cgb {
                AddrBus::Ram
            } else {
                AddrBus::Main
            };
        }

        AddrBus::Internal
    }

    fn shared_dma_addr(&self, addr: u16) -> bool {
        if !self.oam_dma.active || !self.gpu.oam_dma_active {
            return false;
        }

        self.addr_bus(addr) == self.addr_bus(self.oam_dma.src_addr)
    }

    pub fn get_byte(&mut self, mut addr: u16) -> u8 {
        // if self.shared_dma_addr(addr) {
        //     addr = self.oam_dma.src_addr;
        // }

        match addr {
            // 0000-0100   256 byte Boot ROM
            0x0000..=0x00FF => {
                if self.bootrom.is_active {
                    self.bootrom.get_byte(addr as usize)
                } else {
                    self.cartridge.get_byte(addr)
                }
            }
            // 0000-3FFF   16KB ROM Bank 0
            0x0100..=0x7FFF => self.cartridge.get_byte(addr),
            // 8000-9FFF   8KB Video RAM (VRAM)
            0x8000..=0x9FFF => self.gpu.get_byte(addr),
            // A000-BFFF   8KB External RAM
            0xA000..=0xBFFF => self.cartridge.get_byte(addr),
            // C000-CFFF   4KB Work RAM Bank 0
            // D000-DFFF   4KB Work RAM Bank 1
            0xC000..=0xDFFF => self.wram.get_byte(addr),
            // E000-FDFF   Same as C000-DDFF (ECHO)
            0xE000..=0xFDFF => self.wram.get_byte(WRAM_OFFSET + (addr - ECHO_OFFSET)),
            // FE00-FE9F   Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F if self.gpu.oam_dma_active | self.oam_dma.restarting => 0xFF,
            0xFE00..=0xFE9F => self.gpu.get_byte(addr),
            // FEA0-FEFF   Not Usable
            0xFEA0..=0xFEFF => 0x00,
            // FF00-FF7F   I/O Ports
            0xFF00..=0xFF3F => match addr {
                0xFF00 => self.joypad.get_byte(addr),
                0xFF01 => self.serial_out,
                0xFF04..=0xFF07 => self.timer.get_byte(addr),
                0xFF0F => {
                    0xE0 | (self.request_serial_int as u8) << 3
                        | (self.timer.request_timer_int as u8) << 2
                        | (self.gpu.request_lcd_int as u8) << 1
                        | (self.gpu.request_vblank_int as u8)
                }
                0xFF10..=0xFF1E => self.apu.get_byte(addr),
                0xFF20..=0xFF26 => self.apu.get_byte(addr),
                0xFF30..=0xFF3F => self.apu.get_byte(addr),
                _ => {
                    println!("Reading from io ports {:#X}", addr);
                    0x00
                }
            },
            0xFF40..=0xFF45 => self.gpu.get_byte(addr),
            0xFF46 => (self.oam_dma.src_addr >> 8) as u8,
            0xFF47..=0xFF4B => self.gpu.get_byte(addr),
            0xFF4C..=0xFF7F => match addr {
                0xFF4D => match self.emu_mode {
                    EmulationMode::Dmg => 0xFF,
                    EmulationMode::Cgb => u8::from(&self.cgb_mode),
                },
                0xFF4F => self.gpu.get_byte(addr),
                0xFF51..=0xFF54 => 0xFF,
                0xFF55 => match self.hdma.hdma_type {
                    HdmaType::GPDma => self.hdma.blocks,
                    HdmaType::HBlankDma => self.hdma.blocks,
                    HdmaType::NoHdma => 0x80,
                },
                0xFF68..=0xFF6B => self.gpu.get_byte(addr),
                0xFF70 => self.wram.get_byte(addr),
                _ => {
                    println!("Read from io ports {:#X}", addr);
                    0xFF
                }
            },
            // FF80-FFFE   High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[(addr - HRAM_OFFSET) as usize],
            // FFFF        Interrupt Enable Register
            0xFFFF => self.ie,
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            // 0000-3FFF   16KB ROM Bank 0
            0x0000..=0x7FFF => self.cartridge.set_byte(addr, value),
            // 8000-9FFF   8KB Video RAM (VRAM)
            0x8000..=0x9FFF => self.gpu.set_byte(addr, value),
            // A000-BFFF   8KB External RAM
            0xA000..=0xBFFF => self.cartridge.set_byte(addr, value),
            // C000-CFFF   4KB Work RAM Bank 0
            // D000-DFFF   4KB Work RAM Bank 1
            0xC000..=0xDFFF => self.wram.set_byte(addr, value),
            // E000-FDFF   Same as C000-DDFF (ECHO)
            0xE000..=0xFDFF => self
                .wram
                .set_byte(WRAM_OFFSET + (addr - ECHO_OFFSET), value),
            // FE00-FE9F   Sprite Attribute Table (OAM)
            0xFE00..=0xFE9F if self.gpu.oam_dma_active || self.oam_dma.restarting => (),
            0xFE00..=0xFE9F => self.gpu.set_byte(addr, value),
            // FEA0-FEFF   Not Usable
            0xFEA0..=0xFEFF => (),
            // FF00-FF7F   I/O Ports
            0xFF00..=0xFF3F => match addr {
                0xFF00 => self.joypad.set_byte(addr, value),
                0xFF01 => {
                    println!("Serial out: {}", value as char);
                    self.serial_out = value;
                }
                0xFF04..=0xFF07 => self.timer.set_byte(addr, value),
                0xFF0F => {
                    self.gpu.request_vblank_int = (value & 0x01) != 0;
                    self.gpu.request_lcd_int = (value & 0x02) != 0;
                    self.timer.request_timer_int = (value & 0x04) != 0;
                    self.request_serial_int = (value & 0x08) != 0;
                }
                0xFF10..=0xFF1E => self.apu.set_byte(addr, value),
                0xFF20..=0xFF26 => self.apu.set_byte(addr, value),
                0xFF30..=0xFF3F => self.apu.set_byte(addr, value),
                _ => (),
            },
            0xFF40..=0xFF45 => self.gpu.set_byte(addr, value),
            0xFF46 => self.activate_oam_dma(value),
            0xFF47..=0xFF4B => self.gpu.set_byte(addr, value),
            0xFF4C..=0xFF4E => match addr {
                0xFF4D => {
                    println!("Speed switch requested");
                    self.cgb_mode.prepare_speed_switch = value & 0x1;
                }
                _ => println!("Write to io ports {:#X}", addr),
            },
            0xFF4F => self.gpu.set_byte(addr, value),
            0xFF50 => {
                if self.bootrom.is_active && value == 1 {
                    self.bootrom.deactivate();
                } else {
                    println!("Write to io ports {:#X}", addr);
                }
            }
            0xFF51..=0xFF7F => match addr {
                0xFF51 => self.hdma.src = (self.hdma.src & 0xF0) | ((value as u16) << 8),
                0xFF52 => self.hdma.src = (self.hdma.src & 0xFF00) | (value as u16 & 0xF0),
                0xFF53 => self.hdma.dst = (self.hdma.dst & 0xF0) | ((value as u16) << 8),
                0xFF54 => self.hdma.dst = (self.hdma.dst & 0x1F00) | (value as u16 & 0xF0),
                0xFF55 => {
                    self.hdma.hdma_type = match value & 0x80 {
                        0x00 => HdmaType::GPDma,
                        _ => {
                            self.hdma.new_hdma = true;
                            HdmaType::HBlankDma
                        }
                    };
                    self.hdma.blocks = value & 0x7F;
                }
                0xFF68..=0xFF6B => self.gpu.set_byte(addr, value),
                0xFF70 => self.wram.set_byte(addr, value),
                _ => (),
            },
            // FF80-FFFE   High RAM (HRAM)
            0xFF80..=0xFFFE => self.hram[(addr - HRAM_OFFSET) as usize] = value,
            // FFFF        Interrupt Enable Register
            0xFFFF => self.ie = value,
        }
    }

    #[inline]
    fn activate_oam_dma(&mut self, value: u8) {
        if self.oam_dma.active {
            self.oam_dma.restarting = true;
        }
        self.oam_dma_cycles = 0;
        self.oam_dma.active = true;
        self.oam_dma.just_launched = true;
        self.oam_dma.src_addr = (value as u16) << 8;
        self.oam_dma.pending = true;
        self.oam_dma.dst_addr = 0;
    }

    #[inline]
    fn deactivate_oam_dma(&mut self) {
        self.oam_dma.pending = false;
        self.oam_dma.restarting = false;
        self.oam_dma.active = false;
        self.gpu.oam_dma_active = false;
        self.oam_dma.i = 0;
    }
}

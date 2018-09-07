use super::memory::MemoryBus;

use std::cell::RefCell;
use std::rc::Rc;

type VBuffer = [u32; 256 * 240];

/// Represents the PPU
pub struct PPU {
    cycle: i32,
    scanline: i32,
    // Memory
    palettes: [u8; 32],
    nametables: [u8; 0x800],
    oam: [u8; 0xFF],
    front: VBuffer,
    back: VBuffer,
    /// Current vram address (15 bit)
    v: u16,
    /// Temporary vram address (15 bit)
    t: u16,
    /// Fine x scroll (3 bit)
    x: u8,
    /// Write toggle (1 bit)
    w: u8,
    /// Even / odd frame flag (1 bit)
    f: u8,
    /// Writable communication register
    register: u8,
    // Nim flags
    nmi_occurred: bool,
    nmi_output: bool,
    nmi_previous: bool,
    nmi_delay: u8,
    // Sprite temp variables
    sprite_count: i32,
    sprite_patterns: [u32; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indices: [u8; 8],

    // $2000 PPUCTRL
    // 0: $2000, 1: $2400, 2: $2800, 3: $2C00
    flg_nametable: u8,
    // 0: add 1, 1: add 32
    flg_increment: u8,
    // 0: $0000, 1: $1000
    flg_spritetable: u8,
    // 0: $0000, 1: $1000
    flg_backgroundtable: u8,
    // 0: 8x8, 1: 8x16
    flg_spritesize: u8,
    // 0: read EXT, 1: write EXT
    flg_masterslave: u8,

    // $2001 PPUMASK
    // 0: color, 1: grayscale
    flg_grayscale: u8,
    // 0: hide, 1: show
    flg_showleftbg: u8,
    // 0: hide, 1: show
    flg_showleftsprites: u8,
    // 0: hide, 1: show
    flg_showbg: u8,
    // 0: hide, 1: show
    flg_showsprites: u8,
    // 0: normal, 1: emphasized
    flg_redtint: u8,
    // 0: normal, 1: emphasized
    flg_greentint: u8,
    // 0: normal, 1: emphasized
    flg_bluetint: u8,

    // $2002 PPUSTATUS
    flg_sprite0hit: u8,
    flg_spriteoverflow: u8,

    // $2003 OAMADDR
    oam_address: u8,

    // $2007 PPUDATA
    buffer_data: u8,

    /// Shared with the CPU
    mem: Rc<RefCell<MemoryBus>>
}

impl PPU {
    /// Creates a new PPU
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self {
        let mut ppu = PPU {
            cycle: 0, scanline: 0, palettes: [0; 32],
            nametables: [0; 0x800], oam: [0; 0xFF],
            front: [0xF00000FF; 256 * 240],
            back: [0xF00000FF; 256 * 240],
            v: 0, t: 0, x: 0, w: 0, f: 0, register: 0,
            nmi_occurred: false, nmi_output: false,
            nmi_previous: false, nmi_delay: 0,
            sprite_count: 0, sprite_patterns: [0; 8],
            sprite_positions: [0; 8], sprite_priorities: [0; 8],
            sprite_indices: [0; 8],
            flg_nametable: 0, flg_increment: 0, flg_spritetable: 0,
            flg_backgroundtable: 0, flg_spritesize: 0, flg_masterslave: 0,
            flg_grayscale: 0, flg_showleftbg: 0,
            flg_showleftsprites: 0, flg_showbg: 0, flg_showsprites: 0,
            flg_redtint: 0, flg_greentint: 0, flg_bluetint: 0,
            flg_sprite0hit: 0, flg_spriteoverflow: 0,
            oam_address: 0, buffer_data: 0, mem
        };
        ppu.reset();
        ppu
    }

    /// Resets the PPU to its initial state
    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline =  240;
        self.write_control(0);
        self.write_mask(0);
        self.write_oam_address(0);
    }

    fn write_control(&mut self, value: u8) {
        self.flg_nametable = value & 3;
        self.flg_increment = (value >> 2) & 1;
        self.flg_spritetable = (value >> 3) & 1;
        self.flg_backgroundtable = (value >> 4) & 1;
        self.flg_spritesize = (value >> 5) & 1;
        self.flg_masterslave = (value >> 6) & 1;
        self.nmi_output = (value >> 7) & 1 == 1;
        self.nmi_change();
        self.t = (self.t & 0xF3FF) | (((value as u16) & 0x03) << 10);
    }

    fn nmi_change(&mut self) {
        let is_nmi = self.nmi_output && self.nmi_occurred;
        if is_nmi && !self.nmi_previous {
            self.nmi_delay = 15;
        }
        self.nmi_previous = is_nmi;
    }

    fn write_mask(&mut self, value: u8) {
        self.flg_grayscale = value & 1;
        self.flg_showleftbg = (value >> 1) & 1;
        self.flg_showleftsprites = (value >> 2) & 1;
        self.flg_showbg = (value >> 3) & 1;
        self.flg_showsprites = (value >> 4) & 1;
        self.flg_redtint = (value >> 5) & 1;
        self.flg_greentint = (value >> 6) & 1;
        self.flg_bluetint = (value >> 7) & 1;
    }

    fn write_oam_address(&mut self, value: u8) {
        self.oam_address = value;
    }
}
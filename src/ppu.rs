use super::memory::{Mapper, MemoryBus};

use std::cell::RefCell;
use std::rc::Rc;


type VBuffer = [u32; 256 * 240];

/// Represents openly modifiable PPU state
pub struct PPUState {
    oam: [u8; 0xFF],
    /// Current vram address (15 bit)
    pub v: u16, // Public for access during CPU IO reading
    /// Temporary vram address (15 bit)
    t: u16,
    /// Write toggle (1 bit)
    w: u8,
    register: u8,
    // Nmi flags
    nmi_occurred: bool,
    nmi_output: bool,
    nmi_previous: bool,
    nmi_delay: u8,

    // $2000 PPUCTRL
    // 0: $2000, 1: $2400, 2: $2800, 3: $2C00
    flg_nametable: u8,
    // 0: add 1, 1: add 32
    pub flg_increment: u8, // Pub for access during Bus IO
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
    pub buffer_data: u8, // Pub for Bus access during CPU IO
}

impl PPUState {
    pub fn new() -> Self {
        PPUState {
            oam: [0; 0xFF], v: 0, t: 0, w: 0,
            register: 0,
            nmi_occurred: false, nmi_output: false,
            nmi_previous: false, nmi_delay: 0,
            flg_nametable: 0, flg_increment: 0,
            flg_spritetable: 0, flg_backgroundtable: 0,
            flg_spritesize: 0, flg_masterslave: 0,
            flg_grayscale: 0,
            flg_showleftbg: 0, flg_showleftsprites: 0,
            flg_showbg: 0, flg_showsprites: 0,
            flg_redtint: 0, flg_greentint: 0, flg_bluetint: 0,
            flg_sprite0hit: 0, flg_spriteoverflow: 0,
            oam_address: 0,
            buffer_data: 0,
        }
    }

    fn nmi_change(&mut self) {
        let nmi = self.nmi_output && self.nmi_occurred;
        if nmi && !self.nmi_previous {
            self.nmi_delay = 15;
        }
        self.nmi_previous = nmi;
    }

    /// Needs the wrapper because it might read from CHR data
    pub fn read_register(&mut self, m: &Box<Mapper>, address: u16) -> u8 {
        match address {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam_data(),
            0x2007 => self.read_data(m),
            _ => 0,
        }
    }

    fn read_status(&mut self) -> u8 {
        let mut res = self.register & 0x1F;
        res |= self.flg_spriteoverflow << 5;
        res |= self.flg_sprite0hit << 6;
        if self.nmi_occurred {
            res |= 1 << 7;
        }
        self.nmi_occurred = false;
        self.nmi_change();
        self.w = 0;
        res
    }

    fn read_oam_data(&self) -> u8 {
        self.oam[self.oam_address as usize]
    }

    fn read_data(&mut self, mapper: &Box<Mapper>) -> u8 {
        let v = self.v;
        let mut value = mapper.read(v);
        if v % 0x4000 < 0x3F00 {
            let buffer = self.buffer_data;
            self.buffer_data = value;
            value = buffer;
        } else {
            let read = mapper.read(v.wrapping_sub(0x1000));
            self.buffer_data = read;
        }
        if self.flg_increment == 0 {
            self.v += 1;
        } else {
            self.v += 32;
        }
        value
    }

    fn write_control(&mut self, value: u8) {
        self.flg_nametable = (value >> 0) & 3;
        self.flg_increment = (value >> 2) & 1;
        self.flg_spritetable = (value >> 3) & 1;
        self.flg_backgroundtable = (value >> 4) & 1;
        self.flg_spritesize = (value >> 5) & 1;
        self.nmi_output = (value >> 7) & 1 == 1;
        self.nmi_change();
        self.t = (self.t & 0xF3FF) | (((value as u16) & 0x03) << 10);
    }

    fn write_mask(&mut self, value: u8) {
        self.flg_grayscale = (value >> 0) & 1;
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

/// Represents the PPU
pub struct PPU {
    cycle: i32,
    scanline: i32,
    // Memory
    palettes: [u8; 32],
    nametables: [u8; 0x800],
    // These need to boxed to avoid blowing up the stack
    front: Box<VBuffer>,
    back: Box<VBuffer>,

    /// Fine x scroll (3 bit)
    x: u8,
    /// Even / odd frame flag (1 bit)
    f: u8,
    // Sprite temp variables
    sprite_count: i32,
    sprite_patterns: [u32; 8],
    sprite_positions: [u8; 8],
    sprite_priorities: [u8; 8],
    sprite_indices: [u8; 8],
    /// Shared with the CPU
    mem: Rc<RefCell<MemoryBus>>
}

impl PPU {
    /// Creates a new PPU
    pub fn new(mem: Rc<RefCell<MemoryBus>>) -> Self {
        let mut ppu = PPU {
            cycle: 0, scanline: 0,
            palettes: [0; 32], nametables: [0; 0x800],
            front: Box::new([0xF00000FF; 256 * 240]),
            back: Box::new([0xF00000FF; 256 * 240]),
            x: 0, f: 0,
            sprite_count: 0,
            sprite_patterns: [0; 8], sprite_positions: [0; 8],
            sprite_priorities: [0; 8], sprite_indices: [0; 8],
            mem
        };
        ppu.reset();
        ppu
    }

    /// Resets the PPU to its initial state
    pub fn reset(&mut self) {
        self.cycle = 340;
        self.scanline =  240;
        let ppus = &mut self.mem.borrow_mut().ppustate;
        ppus.write_control(0);
        ppus.write_mask(0);
        ppus.write_oam_address(0);
    }
}
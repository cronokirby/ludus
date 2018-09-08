use super::memory::{Mapper, MemoryBus};

use std::cell::RefCell;
use std::rc::Rc;


type VBuffer = [u32; 256 * 240];

/// Represents openly modifiable PPU state
pub struct PPUState {
    pub oam: [u8; 0xFF], // public to allow cpu DMA transfer
    /// Current vram address (15 bit)
    pub v: u16, // Public for access during CPU IO reading
    /// Temporary vram address (15 bit)
    t: u16,
    /// Write toggle (1 bit)
    w: u8,
    /// Fine x scroll (3 bit)
    x: u8,
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
    // 0: hide, 1: sho
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
    pub oam_address: u8, // Pub for DMA transfer

    // $2007 PPUDATA
    pub buffer_data: u8, // Pub for Bus access during CPU IO
}

impl PPUState {
    pub fn new() -> Self {
        PPUState {
            oam: [0; 0xFF], v: 0, t: 0, w: 0, x: 0,
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
            self.v = self.v.wrapping_add(1);
        } else {
            self.v = self.v.wrapping_add(32);
        }
        value
    }

    pub fn write_register(
        &mut self, mapper: &mut Box<Mapper>,
        address: u16, value: u8)
    {
        self.register = value;
        match address {
            0x2000 => self.write_control(value),
            0x2001 => self.write_mask(value),
            0x2003 => self.write_oam_address(value),
            0x2004 => self.write_oam_data(value),
            0x2005 => self.write_scroll(value),
            0x2006 => self.write_address(value),
            0x2007 => self.write_data(mapper, value),
            // This case can never be reached, since the address is % 8,
            _ => {}
        }
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

    fn write_oam_data(&mut self, value: u8) {
        let a = self.oam_address as usize;
        self.oam[a] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    fn write_scroll(&mut self, value: u8) {
        if self.w == 0 {
            self.t = (self.t & 0xFFE0) | ((value as u16) >> 3);
            self.x = value & 0x7;
            self.w = 1;
        } else {
            self.t = (self.t & 0x8FFF) | (((value as u16) & 0x7) << 12);
            self.t = (self.t & 0xFC1F) | (((value as u16) & 0xF8) << 2);
            self.w = 0;
        }
    }

    fn write_address(&mut self, value: u8) {
        if self.w == 0 {
            self.t = (self.t & 0x80FF) | (((value as u16) & 0x3F) << 8);
            self.w = 1;
        } else {
            self.t = (self.t & 0xFF00) | (value as u16);
            self.v = self.t;
            self.w = 0;
        }
    }

    fn write_data(&mut self, mapper: &mut Box<Mapper>, value: u8) {
        let v = self.v;
        mapper.write(v, value);
        if self.flg_increment == 0 {
            self.v = self.v.wrapping_add(1);
        } else {
            self.v = self.v.wrapping_add(32);
        }
    }

    fn copy_y(&mut self) {
        self.v = (self.v & 0x841F) | (self.t & 0x7BE0);
    }

    fn increment_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= 0xFFE0;
            self.v ^= 0x0400;
        } else {
            self.v = self.v.wrapping_add(1);
        }
    }

    fn increment_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v = self.v.wrapping_add(0x1000);
        } else {
            self.v &= 0x8FFF;
            let y = match (self.v & 0x3E0) >> 5 {
                29 => {
                    self.v ^= 0x800;
                    0
                }
                31 => 0,
                val => val + 1
            };
            self.v = (self.v & 0xFC1F) | (y << 5);
        }
    }

    fn copy_x(&mut self) {
        self.v = (self.v & 0xFBE0) | (self.t & 0x41F);
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

    // Background temporary variables
    nametable_byte: u8,
    attributetable_byte: u8,
    lowtile_byte: u8,
    hightile_byte: u8,
    tiledata: u64,

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
            nametable_byte: 0, attributetable_byte: 0,
            lowtile_byte: 0, hightile_byte: 0,
            tiledata: 0,
            f: 0,
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
        self.scanline = 240;
        let m = &mut self.mem.borrow_mut();
        m.ppu.write_control(0);
        m.ppu.write_mask(0);
        m.ppu.write_oam_address(0);
    }

    fn read(&self, m: &mut MemoryBus, address: u16) -> u8 {
        let wrapped = address % 0x4000;
        match wrapped {
            a if a < 0x2000 => m.mapper.read(address),
            a if a < 0x3F00 => {
                let mode = m.mapper.mirroring_mode();
                let mirrored = mode.mirror_address(address);
                self.nametables[(mirrored % 0x800) as usize]
            }
            a if a < 0x4000 => {
                self.read_palette(a % 32)
            }
            a => {
                panic!("Unhandled PPU memory read at {:X}", a);
            }
        }
    }

    fn write(&mut self, m: &mut MemoryBus, address: u16, value: u8) {
        let wrapped = address % 0x4000;
        match wrapped {
            a if a < 0x2000 => m.mapper.write(address, value),
            a if a < 0x3F00 => {
                let mode = m.mapper.mirroring_mode();
                let mirrored = mode.mirror_address(address);
                self.nametables[(mirrored % 0x800) as usize] = value;
            }
            a if a < 0x4000 => {
                self.write_palette(a % 32, value);
            }
            a => {
                panic!("Unhandled PPU memory write at {:X}", a);
            }
        }
    }

    fn read_palette(&self, address: u16) -> u8 {
        let wrapped = if address >= 16 && address % 4 == 0 {
            address - 16
        } else {
            address
        };
        self.palettes[address as usize]
    }

    fn write_palette(&mut self, address: u16, value: u8) {
        let wrapped = if address >= 16 && address % 4 == 0 {
            address - 16
        } else {
            address
        };
        self.palettes[address as usize] = value;
    }


    fn fetch_nametable_byte(&mut self, m: &mut MemoryBus) {
        let v = m.ppu.v;
        let address = 0x2000 | (v & 0x0FFF);
        self.nametable_byte = self.read(m, address);
    }

    fn fetch_attributetable_byte(&mut self, m: &mut MemoryBus) {
        let v = m.ppu.v;
        let address = 0x23C0 | (v & 0x0C00)
            | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let shift = ((v >> 4) & 4) | (v & 2);
        let read = self.read(m, address);
        self.attributetable_byte = ((read >> shift) & 3) << 2;
    }

    fn fetch_lowtile_byte(&mut self, m: &mut MemoryBus) {
        let fine_y = (m.ppu.v >> 12) & 7;
        let table = m.ppu.flg_backgroundtable;
        let tile = self.nametable_byte as u16;
        let address = 0x1000 * (table as u16) + tile * 16 + fine_y;
        self.lowtile_byte = self.read(m, address);
    }

    fn fetch_hightile_byte(&mut self, m: &mut MemoryBus) {
        let fine_y = (m.ppu.v >> 12) & 7;
        let table = m.ppu.flg_backgroundtable;
        let tile = self.nametable_byte as u16;
        let address = 0x1000 * (table as u16) + tile * 16 + fine_y;
        self.hightile_byte = self.read(m, address + 8);
    }

    fn store_tiledata(&mut self, m: &mut MemoryBus) {
        let mut data: u32 = 0;
        for _ in 0..8 {
            let a = self.attributetable_byte;
            let p1 = (self.lowtile_byte & 0x80) >> 7;
            let p2 = (self.hightile_byte & 0x80) >> 6;
            self.lowtile_byte <<= 1;
            self.hightile_byte <<= 1;
            data <<= 4;
            data |= (a | p1 | p2) as u32;
        }
        self.tiledata |= data as u64;
    }


    /// Steps the ppu forward
    fn do_step(&mut self, m: &mut MemoryBus) {
        self.tick(m);
        let rendering = m.ppu.flg_showbg != 0 || m.ppu.flg_showsprites != 0;
        let preline = self.scanline == 261;
        let visibleline = self.scanline < 240;
        let renderline = preline || visibleline;
        let prefetch_cycle = self.cycle >= 321 && self.cycle <= 336;
        let visible_cycle = self.cycle >= 1 && self.cycle <= 256;
        let fetch_cycle = prefetch_cycle || visible_cycle;

        if rendering {
            if visibleline && visible_cycle {
                // self.render_pixel()
            }
            if renderline && fetch_cycle {
                self.tiledata <<= 4;
                match self.cycle % 8 {
                    1 => self.fetch_nametable_byte(m),
                    3 => self.fetch_attributetable_byte(m),
                    5 => self.fetch_lowtile_byte(m),
                    7 => self.fetch_hightile_byte(m),
                    0 => self.store_tiledata(m),
                    _ => {}
                }
            }
            if preline && self.cycle >= 280 && self.cycle <= 304 {
                m.ppu.copy_y();
            }
            if renderline {
                if fetch_cycle && self.cycle % 8 == 0 {
                    m.ppu.increment_x();
                }
                if self.cycle == 256 {
                    m.ppu.increment_y();
                }
                if self.cycle == 257 {
                    m.ppu.copy_x();
                }
            }
        }
    }

    fn tick(&mut self, m: &mut MemoryBus) {
        if m.ppu.nmi_delay > 0 {
            m.ppu.nmi_delay -= 1;
            let was_nmi = m.ppu.nmi_output && m.ppu.nmi_occurred;
            if m.ppu.nmi_delay == 0 && was_nmi {
                m.cpu.set_nmi();
            }
        }

        if m.ppu.flg_showbg != 0 || m.ppu.flg_showsprites != 0 {
            if self.f == 1 && self.scanline == 261 && self.cycle == 339 {
                self.cycle = 0;
                self.scanline = 0;
                self.f ^= 1;
                return
            }
        }

        self.cycle += 1;
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline > 261 {
                self.scanline = 0;
                self.f ^= 1;
            }
        }
    }
}
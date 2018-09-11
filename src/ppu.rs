use super::memory::{Mapper, MemoryBus};

use super::minifb::Window;


type VBuffer = [u32; 256 * 240];

const PALETTE: [u32; 64] = [
    0xFF757575, 0xFF271B8F, 0xFF0000AB, 0xFF47009F,
    0xFF8F0077, 0xFFAB0013, 0xFFA70000, 0xFF7F0B00,
    0xFF432F00, 0xFF004700, 0xFF005100, 0xFF003F17,
    0xFF1B3F5F, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFBCBCBC, 0xFF0073EF, 0xFF233BEF, 0xFF8300F3,
    0xFFBF00BF, 0xFFE7005B, 0xFFDB2B00, 0xFFCB4F0F,
    0xFF8B7300, 0xFF009700, 0xFF00AB00, 0xFF00933B,
    0xFF00838B, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFFFF, 0xFF3FBFFF, 0xFF5F97FF, 0xFFA78BFD,
    0xFFF77BFF, 0xFFFF77B7, 0xFFFF7763, 0xFFFF9B3B,
    0xFFF3BF3F, 0xFF83D313, 0xFF4FDF4B, 0xFF58F898,
    0xFF00EBDB, 0xFF000000, 0xFF000000, 0xFF000000,
    0xFFFFFFFF, 0xFFABE7FF, 0xFFC7D7FF, 0xFFD7CBFF,
    0xFFFFC7FF, 0xFFFFC7DB, 0xFFFFBFB3, 0xFFFFDBAB,
    0xFFFFE7A3, 0xFFE3FFA3, 0xFFABF3BF, 0xFFB3FFCF,
    0xFF9FFFF3, 0xFF000000, 0xFF000000, 0xFF000000
];


/// Represents openly modifiable PPU state
pub struct PPUState {
     // Memory
    palettes: [u8; 32],
    nametables: [u8; 2048],
    pub oam: [u8; 256], // public to allow cpu DMA transfer
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
            palettes: [0; 32], nametables: [0; 2048],
            oam: [0; 256], v: 0, t: 0, w: 0, x: 0,
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

    fn read(&self, mapper: &Box<Mapper>, address: u16) -> u8 {
        let wrapped = address % 0x4000;
        match wrapped {
            a if a < 0x2000 => mapper.read(a),
            a if a < 0x3F00 => {
                let mode = mapper.mirroring_mode();
                let mirrored = mode.mirror_address(a);
                self.nametables[(mirrored % 2048) as usize]
            }
            a if a < 0x4000 => {
                self.read_palette(a % 32)
            }
            a => {
                panic!("Unhandled PPU memory read at {:X}", a);
            }
        }
    }

    fn write(&mut self, mapper: &mut Box<Mapper>, address: u16, value: u8) {
        let wrapped = address % 0x4000;
        match wrapped {
            a if a < 0x2000 => mapper.write(a, value),
            a if a < 0x3F00 => {
                let mode = mapper.mirroring_mode();
                let mirrored = mode.mirror_address(a);
                self.nametables[(mirrored % 2048) as usize] = value;
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
        self.palettes[wrapped as usize]
    }

    fn write_palette(&mut self, address: u16, value: u8) {
        let wrapped = if address >= 16 && address % 4 == 0 {
            address - 16
        } else {
            address
        };
        self.palettes[wrapped as usize] = value;
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
        let mut value = self.read(mapper, v);
        if v % 0x4000 < 0x3F00 {
            let buffer = self.buffer_data;
            self.buffer_data = value;
            value = buffer;
        } else {
            let read = self.read(&mapper, v - 0x1000);
            self.buffer_data = read;
        }
        if self.flg_increment == 0 {
            self.v += 1;
        } else {
            self.v += 1;
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
        self.flg_masterslave = (value >> 6) & 1;
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
            self.t = (self.t & 0x7FE0) | ((value as u16) >> 3);
            self.x = value & 0x7;
            self.w = 1;
        } else {
            let s1 = ((value as u16) & 0x7) << 12;
            self.t = (self.t & 0xC1F) | (((value as u16) & 0xF8) << 2) | s1;
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
        self.write(mapper, v, value);
        if self.flg_increment == 0 {
            self.v += 1;
        } else {
            self.v += 32;
        }
    }

    fn copy_y(&mut self) {
        let mask = 0b0111_1011_1110_0000;
        self.v = (self.v & !mask) | (self.t & mask);
    }

    fn increment_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= 0xFFE0;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }

    fn increment_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000;
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
        let mask = 0b0000_0100_0001_1111;
        self.v = (self.v & !mask) | (self.t & mask);
    }
}

/// Represents the PPU
pub struct PPU {
    cycle: i32,
    scanline: i32,

    // These need to boxed to avoid blowing up the stack
    front: Box<VBuffer>,
    back: Box<VBuffer>,
    is_front: bool,

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
    sprite_indices: [u8; 8]
    //mem: Rc<RefCell<MemoryBus>>
}

impl PPU {
    /// Creates a new PPU
    pub fn new(m: &mut MemoryBus) -> Self {
        let mut ppu = PPU {
            cycle: 0, scanline: 0,
            front: Box::new([0xFF000000; 256 * 240]),
            back: Box::new([0xFF000000; 256 * 240]),
            is_front: true,
            nametable_byte: 0, attributetable_byte: 0,
            lowtile_byte: 0, hightile_byte: 0,
            tiledata: 0,
            f: 0,
            sprite_count: 0,
            sprite_patterns: [0; 8], sprite_positions: [0; 8],
            sprite_priorities: [0; 8], sprite_indices: [0; 8]
        };
        ppu.reset(m);
        ppu
    }

    /// Resets the PPU to its initial state
    pub fn reset(&mut self, m: &mut MemoryBus) {
        self.cycle = 340;
        self.scanline = 240;
        m.ppu.write_control(0);
        m.ppu.write_mask(0);
        m.ppu.write_oam_address(0);
    }

    pub fn update_window(&self, window: &mut Window) {
        if self.is_front {
            window.update_with_buffer(self.front.as_ref())
        } else {
            window.update_with_buffer(self.back.as_ref())
        }.expect("Failed to update window");
    }

    fn fetch_nametable_byte(&mut self, m: &mut MemoryBus) {
        let v = m.ppu.v;
        let address = 0x2000 | (v & 0x0FFF);
        self.nametable_byte = m.ppu.read(&m.mapper, address);
    }

    fn fetch_attributetable_byte(&mut self, m: &mut MemoryBus) {
        let v = m.ppu.v;
        let address = 0x23C0 | (v & 0x0C00)
            | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
        let shift = ((v >> 4) & 4) | (v & 2);
        let read = m.ppu.read(&m.mapper, address);
        self.attributetable_byte = ((read >> shift) & 3) << 2;
    }

    fn fetch_lowtile_byte(&mut self, m: &mut MemoryBus) {
        let fine_y = (m.ppu.v >> 12) & 7;
        let table = m.ppu.flg_backgroundtable;
        let tile = self.nametable_byte as u16;
        let address = 0x1000 * (table as u16) + tile * 16 + fine_y;
        self.lowtile_byte = m.ppu.read(&m.mapper, address);
    }

    fn fetch_hightile_byte(&mut self, m: &mut MemoryBus) {
        let fine_y = (m.ppu.v >> 12) & 7;
        let table = m.ppu.flg_backgroundtable;
        let tile = self.nametable_byte as u16;
        let address = 0x1000 * (table as u16) + tile * 16 + fine_y;
        self.hightile_byte = m.ppu.read(&m.mapper, address + 8);
    }

    fn store_tiledata(&mut self) {
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

    fn fetch_sprite_pattern(&self, m: &mut MemoryBus,
        i: usize, mut row: i32) -> u32
    {
        let mut tile = m.ppu.oam[i*4 + 1];
        let attributes = m.ppu.oam[i*4 + 2];
        let address = if m.ppu.flg_spritesize == 0 {
            if attributes & 0x80 == 0x80 {
                row -= 7;
            }
            let table = m.ppu.flg_spritetable;
            0x1000 * (table as u16) + (tile as u16) * 16 + (row as u16)
        } else {
            if attributes & 0x80 == 0x80 {
                row -= 15;
            }
            let table = tile & 1;
            tile &= 0xFE;
            if row > 7 {
                tile += 1;
                row -= 8;
            }
            0x1000 * (table as u16) + (tile as u16) * 16 + (row as u16)
        };
        let a = (attributes & 3) << 2;
        let mut lowtile_byte = m.ppu.read(&m.mapper, address);
        let mut hightile_byte = m.ppu.read(&m.mapper, address + 8);
        let mut data: u32 = 0;
        for _ in 0..8 {
            let (p1, p2) = if attributes & 0x40 == 0x40 {
                let p1 = (lowtile_byte & 1) << 0;
                let p2 = (hightile_byte & 1) << 1;
                lowtile_byte >>= 1;
                hightile_byte >>= 1;
                (p1, p2)
            } else {
                let p1 = (lowtile_byte & 0x80) >> 7;
                let p2 = (hightile_byte & 0x80) >> 6;
                lowtile_byte <<= 1;
                hightile_byte <<= 1;
                (p1, p2)
            };
            data <<= 4;
            data |= (a | p1 | p2) as u32;
        }
        data
    }

    fn evaluate_sprites(&mut self, m: &mut MemoryBus) {
        let h: i32 = if m.ppu.flg_spritesize == 0 {
            8
        } else {
            16
        };
        let mut count = 0;
        for i in 0..64 {
            let y = m.ppu.oam[i*4];
            let a = m.ppu.oam[i*4 + 2];
            let x = m.ppu.oam[i*4 + 3];
            let row = self.scanline - (y as i32);
            if row < 0 || row >= h {
                continue;
            }
            if count < 8 {
                let pattern = self.fetch_sprite_pattern(m, i, row);
                self.sprite_patterns[count] = pattern;
                self.sprite_positions[count] = x;
                self.sprite_priorities[count] = (a >> 5) & 1;
                self.sprite_indices[count] = i as u8;
            }
            count += 1;
        }
        if count > 8 {
            count = 8;
            m.ppu.flg_spriteoverflow = 1;
        }
        self.sprite_count = count as i32;
    }

    fn set_vblank(&mut self, m: &mut MemoryBus) {
        self.is_front = !self.is_front;
        m.ppu.nmi_occurred = true;
        m.ppu.nmi_change();
    }

    fn clear_vblank(&self, m: &mut MemoryBus) {
        m.ppu.nmi_occurred = false;
        m.ppu.nmi_change();
    }

    fn fetch_tiledata(&self) -> u32 {
        (self.tiledata >> 32) as u32
    }

    fn background_pixel(&mut self, m: &mut MemoryBus) -> u8 {
        if m.ppu.flg_showbg == 0 {
            0
        } else {
            let data = self.fetch_tiledata() >> ((7 - m.ppu.x) * 4);
            (data & 0x0F) as u8
        }
    }

    fn sprite_pixel(&mut self, m: &mut MemoryBus) -> (u8, u8) {
        if m.ppu.flg_showsprites == 0 {
            (0, 0)
        } else {
            for i in 0..self.sprite_count {
                let sp_off = self.sprite_positions[i as usize] as i32;
                let mut offset = (self.cycle - 1) - sp_off;
                if offset < 0 || offset > 7 {
                    continue
                }
                offset = 7 - offset;
                let shift = (offset * 4) as u8;
                let pattern = self.sprite_patterns[i as usize];
                let color = ((pattern >> shift) & 0x0F) as u8;
                if color % 4 == 0 {
                    continue
                }
                return (i as u8, color)
            }
            (0, 0)
        }
    }

    fn render_pixel(&mut self, m: &mut MemoryBus) {
        let x = self.cycle - 1;
        let y = self.scanline;
        let mut background = self.background_pixel(m);
        let (i, mut sprite) = self.sprite_pixel(m);
        if x < 8 && m.ppu.flg_showleftbg == 0 {
            background = 0;
        }
        if x < 8 && m.ppu.flg_showleftsprites == 0 {
            sprite = 0;
        }
        let b = background % 4 != 0;
        let s = sprite % 4 != 0;
        let color = match (b, s) {
            (false, false) => 0,
            (false, true) => sprite | 0x10,
            (true, false) => background,
            (true, true) =>  {
                let ind = i as usize;
                if self.sprite_indices[ind] == 0 && x < 255 {
                    m.ppu.flg_sprite0hit = 1;
                }
                if self.sprite_priorities[ind] == 0 {
                    sprite | 0x10
                } else {
                    background
                }
            }
        };
        let c = m.ppu.read_palette(color as u16) % 64;
        let rgba = PALETTE[c as usize];
        let pos = (y * 256 + x) as usize;
        if self.is_front {
            self.back[pos] = rgba;
        } else {
            self.front[pos] = rgba;
        }
    }

    /// Steps the ppu forward
    pub fn step(&mut self, m: &mut MemoryBus) {
        self.tick(m);
        let rendering = m.ppu.flg_showbg != 0 || m.ppu.flg_showsprites != 0;
        let preline = self.scanline == 261;
        let visibleline = self.scanline < 240;
        let renderline = preline || visibleline;
        let prefetch_cycle = self.cycle >= 321 && self.cycle <= 336;
        let visible_cycle = self.cycle >= 1 && self.cycle <= 256;
        let fetch_cycle = prefetch_cycle || visible_cycle;

        // Background logic
        if rendering {
            if visibleline && visible_cycle {
                self.render_pixel(m)
            }
            if renderline && fetch_cycle {
                self.tiledata <<= 4;
                match self.cycle % 8 {
                    1 => self.fetch_nametable_byte(m),
                    3 => self.fetch_attributetable_byte(m),
                    5 => self.fetch_lowtile_byte(m),
                    7 => self.fetch_hightile_byte(m),
                    0 => self.store_tiledata(),
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

        // Sprite logic
        if rendering {
            if self.cycle == 257 {
                if visibleline {
                    self.evaluate_sprites(m);
                } else {
                    self.sprite_count = 0;
                }
            }
        }

        // Vblank logic
        if self.scanline == 241 && self.cycle == 1 {
            self.set_vblank(m);
        }
        if preline && self.cycle == 1 {
            self.clear_vblank(m);
            m.ppu.flg_sprite0hit = 0;
            m.ppu.flg_spriteoverflow = 0;
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
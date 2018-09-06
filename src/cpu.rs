use super::memory::MemoryBus;

use std::rc::Rc;
use std::cell::RefCell;


// The various addressing modes of each opcode
const OP_MODES: [u8; 256] = [
    6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
    1, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
    6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
    6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 8, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
    5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3,
    5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3,
    5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
    5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1,
    10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2
];

// The size of each instruction in bytes
// we sacrifice space to avoid casting
const OP_SIZES: [u16; 256] = [
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
    2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0
];

// How many cycles each instruction takes
const OP_CYCLES: [i32; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4,
    2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6,
    2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7
];

// The op codes which add a cycle when crossing pages accessing memory
// doesn't include branch instructions, since the page crossing check
// happens when the branch is known to be successful or not
const EXTRA_PAGECYCLE_OPS: [u8; 23] = [
    0x7D, 0x79, 0x71, 0x3D, 0x39, 0x31, 0xDD, 0xD9, 0xD1,
    0x5D, 0x59, 0x51, 0xBD, 0xB9, 0xB1, 0xBE, 0xBC,
    0x1D, 0x19, 0x11, 0xFD, 0xF9, 0xF1
];

// The various names of each opcode
const OP_NAMES: [&'static str; 256] = [
    "BRK", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO",
    "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
    "BPL", "ORA", "KIL", "SLO", "NOP", "ORA", "ASL", "SLO",
    "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
    "JSR", "AND", "KIL", "RLA", "BIT", "AND", "ROL", "RLA",
    "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
    "BMI", "AND", "KIL", "RLA", "NOP", "AND", "ROL", "RLA",
    "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
    "RTI", "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE",
    "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
    "BVC", "EOR", "KIL", "SRE", "NOP", "EOR", "LSR", "SRE",
    "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
    "RTS", "ADC", "KIL", "RRA", "NOP", "ADC", "ROR", "RRA",
    "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
    "BVS", "ADC", "KIL", "RRA", "NOP", "ADC", "ROR", "RRA",
    "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
    "NOP", "STA", "NOP", "SAX", "STY", "STA", "STX", "SAX",
    "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX", "SAX",
    "BCC", "STA", "KIL", "AHX", "STY", "STA", "STX", "SAX",
    "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "AHX",
    "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "TAY", "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX",
    "BCS", "LDA", "KIL", "LAX", "LDY", "LDA", "LDX", "LAX",
    "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
    "CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP",
    "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
    "BNE", "CMP", "KIL", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
    "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC",
    "INX", "SBC", "NOP", "SBC", "CPX", "SBC", "INC", "ISC",
    "BEQ", "SBC", "KIL", "ISC", "NOP", "SBC", "INC", "ISC",
    "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC"
];


/// Represents the type of addressing an op uses
#[derive(Clone, Copy)]
enum Addressing {
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Accumulator,
    Immediate,
    Implied,
    IndexedIndirect,
    Indirect,
    IndirectIndexed,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY
}

impl Addressing {
    // This can't handle every byte, but is only used
    // for a more compact idea.
    fn from_byte(mode: u8) -> Self {
        use self::Addressing::*;
        let modes = [
            Absolute, AbsoluteX, AbsoluteY,
            Accumulator, Immediate, Implied,
            IndexedIndirect, Indirect, IndirectIndexed,
            Relative,
            ZeroPage, ZeroPageX, ZeroPageY
        ];
        modes[mode as usize - 1]
    }
}


/// Represents the different types of Interrupts the CPU might deal with
enum Interrupt {
    NMI,
    IRQ
}


/// Returns true if two addresses return different pages
fn pages_differ(a: u16, b: u16) -> bool {
    a & 0xFF00 != b & 0xFF
}

/// Returns the number of extra cycles used by a branch instruction
fn branch_cycles(pc: u16, address: u16) -> i32 {
    if pages_differ(pc, address) { 2 } else { 1 }
}


/// Represents possible CPU interrupts
/// Represents the CPU
pub struct CPU {
    /// Program counter
    pc: u16,
    /// Stack pointer
    sp: u8,
    /// Accumulator Register
    a: u8,
    /// X Register
    x: u8,
    /// Y Register
    y: u8,
    // Flags are represented as multiple bytes for more conveniant use
    // bools could be used instead, but bytes make for better arithmetic
    /// Carry Flag
    c: u8,
    /// Zero Flag
    z: u8,
    /// Interrupt disable Flag
    i: u8,
    /// Decimal mode Flag
    d: u8,
    /// Break command Flag
    b: u8,
    /// Unused Flag
    u: u8,
    /// Overflow Flag
    v: u8,
    /// Negative Flag
    n: u8,
    // Interrupts are set to be handled on the next CPU step
    /// Represents the presence of an Interrupt needing to be handled
    interrupt: Option<Interrupt>,
    /// Used to request the CPU stall, mainly for timing purposes
    stall: i32,
    /// Shared acess to the memory bus along with the ppu,
    mem: Rc<RefCell<MemoryBus>>
}

impl CPU {
    /// Creates a new CPU
    /// `reset` should be called if doing this at initialisation of console,
    ///  but cannot be done inside this function, since RAM isn't live.
    pub fn zeroed(mem: Rc<RefCell<MemoryBus>>) -> Self {
        let pc = 0;
        let sp = 0;
        let a = 0;
        let x = 0;
        let y = 0;
        let c = 0;
        let z = 0;
        let i = 0;
        let d = 0;
        let b = 0;
        let u = 0;
        let v = 0;
        let n = 0;
        let interrupt = None;
        let stall = 0;
        let mut cpu = CPU {
            pc, sp, a, x, y, c, z, i, d, b, u, v, n,
            interrupt, stall, mem
        };
        cpu.reset();
        cpu
    }

    /// Resets the CPU to its initial powerup state.
    pub fn reset(&mut self) {
        self.pc = self.read16(0xFFFC) + 0x8000;
        self.sp = 0xFD;
        self.set_flags(0x24);
    }

    fn set_flags(&mut self, flags: u8) {
        self.c = (flags >> 0) & 1;
        self.z = (flags >> 1) & 1;
        self.i = (flags >> 2) & 1;
        self.d = (flags >> 3) & 1;
        self.b = (flags >> 4) & 1;
        self.u = (flags >> 5) & 1;
        self.v = (flags >> 6) & 1;
        self.n = (flags >> 7) & 1;
    }

    fn get_flags(&self) -> u8 {
        let mut r = 0;
        r |= self.c << 0;
        r |= self.z << 1;
        r |= self.i << 2;
        r |= self.d << 3;
        r |= self.b << 4;
        r |= self.u << 5;
        r |= self.v << 6;
        r |= self.n << 7;
        r
    }

    pub fn read(&self, address: u16) -> u8 {
        self.mem.borrow().cpu_read(address)
    }

    pub fn read16(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address + 1) as u16;
        (hi << 8) | lo
    }

    /// Emulates a software bug where only the lower bit wraps around
    fn read16bug(&self, a: u16) -> u16 {
        let b = (a & 0xFF00) | ((a + 1) & 0xFF);
        let lo = self.read(a) as u16;
        let hi = self.read(b) as u16;
        (hi << 8) | lo
    }

    fn write(&mut self, address: u16, value: u8) {
        self.mem.borrow_mut().cpu_write(address, value);
    }

    fn push(&mut self, value: u8) {
        let sp = self.sp as u16;
        self.write(0x100 | sp, value);
        self.sp -= 1;
    }

    fn push16(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.push(hi);
        self.push(lo);
    }

    fn pull(&mut self) -> u8 {
        self.sp += 1;
        let sp = self.sp as u16;
        self.read(0x100 | sp)
    }

    fn pull16(&mut self) -> u16 {
        let lo = self.pull() as u16;
        let hi = self.pull() as u16;
        (hi << 8) | lo
    }

    fn set_z(&mut self, r: u8) {
        self.z = match r {
            0 => 1,
            _ => 0
        }
    }

    fn set_n(&mut self, r: u8) {
        self.n = match r & 0x80 {
            0 => 0,
            _ => 1
        }
    }

    fn set_zn(&mut self, r: u8) {
        self.set_z(r);
        self.set_n(r);
    }

    fn php(&mut self) {
        let flags = self.get_flags();
        self.push(flags | 0x10);
    }

    fn compare(&mut self, a: u8, b: u8) {
        self.set_zn(a.wrapping_sub(b));
        if a >= b {
            self.c = 1;
        } else {
            self.c = 0;
        };
    }

    /// Steps the cpu forward by a single instruction
    /// Returns the number of cycles passed
    pub fn step(&mut self) -> i32 {
        // Stall for a single cycle if stall cycles are still done
        if self.stall > 0 {
            self.stall -= 1;
            return 1;
        }
        let mut cycles = 0;
        let opcode = self.read(self.pc);
        // We now fetch the adress based on what type of addressing the
        // opcode requires, and set the page crossed, in order to
        // increment the cycles if necessary.
        let mut page_crossed = false;
        let address = match Addressing::from_byte(OP_MODES[opcode as usize]) {
            Addressing::Absolute => self.read16(self.pc + 1),
            Addressing::AbsoluteX => {
                let read = self.read16(self.pc + 1);
                let address = read + self.x as u16;
                page_crossed = pages_differ(read, address);
                address
            }
            Addressing::AbsoluteY => {
                let read = self.read16(self.pc + 1);
                let address = read + self.y as u16;
                page_crossed = pages_differ(read, address);
                address
            }
            Addressing::Accumulator => 0,
            Addressing::Immediate => self.pc + 1,
            Addressing::Implied => 0,
            Addressing::IndexedIndirect => {
                let added = self.read(self.pc + 1).wrapping_add(self.x);
                self.read16bug(added as u16)
            }
            Addressing::Indirect => self.read16bug(self.read16(self.pc + 1)),
            Addressing::IndirectIndexed => {
                let added = self.read(self.pc + 1).wrapping_add(self.y);
                let address = self.read16bug(added as u16);
                let old_addr = address - (self.y as u16);
                page_crossed = pages_differ(address, old_addr);
                address
            }
            Addressing::Relative => {
                let offset = self.read(self.pc + 1) as u16;
                // treating this as a signed integer
                if offset < 0x80 {
                    self.pc + 2 + offset
                } else {
                    self.pc + 2 + offset - 0x100
                }
            }
            Addressing::ZeroPage => self.read(self.pc + 1) as u16,
            Addressing::ZeroPageX => {
                let added = self.read(self.pc + 1).wrapping_add(self.x);
                // we don't need to & 0xFF here, since added is a byte
                added as u16
            }
            Addressing::ZeroPageY => {
                let added = self.read(self.pc + 1).wrapping_add(self.y);
                added as u16
            }
        };


        self.pc += OP_SIZES[opcode as usize];
        cycles += OP_CYCLES[opcode as usize];
        if page_crossed && EXTRA_PAGECYCLE_OPS.contains(&opcode) {
            cycles += 1;
        }
        // todo, actually emulate
        match opcode {
            // AND
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                let a = self.a & self.read(address);
                self.a = a;
                self.set_zn(a);
            }
            // BCC
            0x90 => {
                if self.c == 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // BCS
            0xB0 => {
                if self.c != 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address)
                }
            }
            // BEQ
            0xF0 => {
                if self.z != 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // BIT
            0x24 | 0x2C => {
               let value = self.read(address);
               self.v = (value >> 6) & 1;
               let a = self.a;
               self.set_z(value & a);
               self.set_n(value);
            }
            // BNE
            0xD0 => {
                if self.z == 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // BPL
            0x10 => {
                if self.n == 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // BVC
            0x50 => {
                if self.v == 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // BVS
            0x70 => {
                if self.v != 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
            }
            // CLC
            0x18 => self.c = 0,
            // CLD
            0xD8 => self.d = 0,
            // CMP
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                let value = self.read(address);
                let a = self.a;
                self.compare(a, value);
            }
            // JMP
            0x4C | 0x6C => self.pc = address,
            // JSR
            0x20 => {
                let minus = self.pc - 1;
                self.push16(minus);
                self.pc = address;
            }
            // LDA
            0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                let a = self.read(address);
                self.a = a;
                self.set_zn(a);
            }
            // LDX
            0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                let a = self.read(address);
                self.x = a;
                self.set_zn(a);
            }
            // NOP
            0xEA => {},
            // PLA
            0x68 => {
                let a = self.pull();
                self.a = a;
                self.set_zn(a);
            }
            // PHA
            0x48 => {
                let a = self.a;
                self.push(a);
            }
            // PHP
            0x08 => self.php(),
            // PLP
            0x28 => {
                let p = self.pull();
                self.set_flags((p & 0xEF) | 0x20);
            }
            // RTS
            0x60 => self.pc = self.pull16() + 1,
            // SEC
            0x38 => self.c = 1,
            // SED
            0xF8 => self.d = 1,
            // SEI
            0x78 => self.i = 1,
            // STA
            0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                let a = self.a;
                self.write(address, a);
            }
            // STX
            0x86 | 0x96 | 0x8E => {
                let x = self.x;
                self.write(address, x);
            }
            _ => panic!("Unimplented Op {:02X}", opcode)
        }
        cycles
    }


    /// Prints the upcoming operation
    pub fn print_current_op(&self) {
        let opcode = self.read(self.pc);
        let lo = self.read(self.pc + 1);
        let hi = self.read(self.pc + 2);
        print!("{:X} ", self.pc);
        print_op(opcode, lo, hi);
    }

    /// Prints the current state of the CPU
    pub fn print_state(&self) {
        println!("A: {:X} X: {:X} Y: {:X} F: {:X} SP: {:X}",
            self.a, self.x, self.y, self.get_flags(), self.sp);
    }
}


// prints an operation returning the number of cycles
fn print_op(opcode: u8, lo: u8, hi: u8) -> u16 {
    let op = OP_NAMES[opcode as usize];
    let addressing = Addressing::from_byte(OP_MODES[opcode as usize]);
    match addressing {
        Addressing::Absolute => {
            print!("{} ${:02X}{:02X}", op, hi, lo);
            2
        }
        Addressing::AbsoluteX => {
            print!("{} ${:02X}{:02X}", op, hi, lo);
            2
        }
        Addressing::AbsoluteY => {
            print!("{} ${:02X}{:02X}", op, hi, lo);
            2
        }
        Addressing::Accumulator => {
            print!("{} A", op);
            0
        }
        Addressing::Immediate => {
            print!("{} #${:02X}", op, lo);
            1
        }
        Addressing::Implied => {
            print!("{}", op);
            0
        }
        Addressing::IndexedIndirect => {
            print!("{} (${:02X},X)", op, lo);
            1
        }
        Addressing::Indirect => {
            print!("{} (${:02X}{:02X})", op, hi, lo);
            2
        }
        Addressing::IndirectIndexed => {
            print!("{} (${:02X}),Y", op, lo);
            1
        }
        Addressing::Relative => {
            print!("{} *+{:02X}", op, lo);
            1
        }
        Addressing::ZeroPage => {
            print!("{} ${:02X}", op, lo);
            1
        }
        Addressing::ZeroPageX => {
            print!("{} ${:02X},X", op, lo);
            1
        }
        Addressing::ZeroPageY => {
            print!("{} ${:02X},Y", op, lo);
            1
        }
    }
}

pub fn disassemble(in_buf: &[u8]) {
    // we read in the buffer to be able to append the first 2 bytes at the end
    // to simulate wrapped reading. 2 is sufficient because 3 is the largest
    // op size
    let mut buf: Vec<u8> = in_buf.iter().cloned().collect();
    let len = buf.len();
    let a = buf[0].clone();
    let b = buf[1].clone();
    buf.push(a);
    buf.push(b);
    let mut pc = 0;
    while (pc as usize) < len {
        let pcu = pc as usize;
        pc += print_op(buf[pcu], buf[pcu + 1], buf[pcu + 2]);
        pc += 1;
        println!("");
    }
}
use super::memory::MemoryBus;

// The various addressing modes of each opcode
const OP_MODES: [u8; 256] = [
    6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2,
    2, 2, 2, 1, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3,
    6, 3, 2, 2, 2, 2, 6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12,
    12, 6, 3, 6, 3, 2, 2, 2, 2, 6, 7, 6, 7, 11, 11, 11, 11, 6, 5, 4, 5, 8, 1, 1, 1, 10, 9, 6, 9,
    12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1, 1, 1, 10,
    9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6, 5, 1, 1,
    1, 1, 10, 9, 6, 9, 12, 12, 13, 13, 6, 3, 6, 3, 2, 2, 3, 3, 5, 7, 5, 7, 11, 11, 11, 11, 6, 5, 6,
    5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2, 5, 7, 5, 7, 11, 11, 11, 11,
    6, 5, 6, 5, 1, 1, 1, 1, 10, 9, 6, 9, 12, 12, 12, 12, 6, 3, 6, 3, 2, 2, 2, 2,
];

// The size of each instruction in bytes
// we sacrifice space to avoid casting
const OP_SIZES: [u16; 256] = [
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    3, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    1, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 0, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 0, 3, 0, 0,
    2, 2, 2, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
    2, 2, 0, 0, 2, 2, 2, 0, 1, 2, 1, 0, 3, 3, 3, 0, 2, 2, 0, 0, 2, 2, 2, 0, 1, 3, 1, 0, 3, 3, 3, 0,
];

// How many cycles each instruction takes
const OP_CYCLES: [i32; 256] = [
    7, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    6, 6, 2, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 6, 2, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5,
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, 2, 5, 2, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, 2, 5, 2, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7,
];

// The op codes which add a cycle when crossing pages accessing memory
// doesn't include branch instructions, since the page crossing check
// happens when the branch is known to be successful or not
const EXTRA_PAGECYCLE_OPS: [u8; 23] = [
    0x7D, 0x79, 0x71, 0x3D, 0x39, 0x31, 0xDD, 0xD9, 0xD1, 0x5D, 0x59, 0x51, 0xBD, 0xB9, 0xB1, 0xBE,
    0xBC, 0x1D, 0x19, 0x11, 0xFD, 0xF9, 0xF1,
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
    ZeroPageY,
}

impl Addressing {
    // This can't handle every byte, but is only used
    // for a more compact idea.
    fn from_byte(mode: u8) -> Self {
        use self::Addressing::*;
        let modes = [
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
            ZeroPageY,
        ];
        modes[mode as usize - 1]
    }
}

/// Represents the different types of Interrupts the CPU might deal with
#[derive(Clone)]
enum Interrupt {
    NMI,
    IRQ,
}

/// Returns true if two addresses return different pages
fn pages_differ(a: u16, b: u16) -> bool {
    a & 0xFF00 != b & 0xFF
}

/// Returns the number of extra cycles used by a branch instruction
fn branch_cycles(pc: u16, address: u16) -> i32 {
    if pages_differ(pc, address) {
        2
    } else {
        1
    }
}

/// Represents public CPU state
#[derive(Default)]
pub struct CPUState {
    /// Represents the interrupt, None representing no interrupt
    interrupt: Option<Interrupt>,
    /// Used to add stalls to cpu cycles
    stall: i32,
}

impl CPUState {
    pub fn new() -> Self {
        CPUState::default()
    }

    pub fn set_nmi(&mut self) {
        self.interrupt = Some(Interrupt::NMI);
    }

    pub fn set_irq(&mut self) {
        self.interrupt = Some(Interrupt::IRQ);
    }

    pub fn clear_interrupt(&mut self) {
        self.interrupt = None;
    }

    pub fn add_stall(&mut self, amount: i32) {
        self.stall += amount;
    }
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
    /// Shared acess to the memory bus along with the ppu,
    pub mem: MemoryBus,
}

impl CPU {
    /// Creates a new CPU
    pub fn new(mem: MemoryBus) -> Self {
        let mut cpu = CPU {
            pc: 0,
            sp: 0,
            a: 0,
            x: 0,
            y: 0,
            c: 0,
            z: 0,
            i: 0,
            d: 0,
            b: 0,
            u: 0,
            v: 0,
            n: 0,
            mem,
        };
        cpu.reset();
        cpu
    }

    /// Resets the CPU to its initial powerup state.
    pub fn reset(&mut self) {
        self.pc = self.read16(0xFFFC);
        self.sp = 0xFD;
        self.set_flags(0x24);
    }

    /// Sets the buttons for controller 1
    pub fn set_buttons(&mut self, buttons: [bool; 8]) {
        self.mem.controller1.set_buttons(buttons);
    }

    fn set_flags(&mut self, flags: u8) {
        self.c = flags & 1;
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
        r |= self.c;
        r |= self.z << 1;
        r |= self.i << 2;
        r |= self.d << 3;
        r |= self.b << 4;
        r |= self.u << 5;
        r |= self.v << 6;
        r |= self.n << 7;
        r
    }

    pub fn read(&mut self, address: u16) -> u8 {
        self.mem.cpu_read(address)
    }

    fn read16(&mut self, address: u16) -> u16 {
        let lo = self.read(address);
        let hi = self.read(address + 1);
        u16::from_be_bytes([hi, lo])
    }

    /// Emulates a software bug where only the lower bit wraps around
    fn read16bug(&mut self, a: u16) -> u16 {
        let b = (a & 0xFF00) | ((a + 1) & 0xFF);
        let lo = self.read(a);
        let hi = self.read(b);
        u16::from_be_bytes([hi, lo])
    }

    fn write(&mut self, address: u16, value: u8) {
        self.mem.cpu_write(address, value);
    }

    fn push(&mut self, value: u8) {
        let sp = u16::from(self.sp);
        self.write(0x100 | sp, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn push16(&mut self, value: u16) {
        let hi = (value >> 8) as u8;
        let lo = (value & 0xFF) as u8;
        self.push(hi);
        self.push(lo);
    }

    fn pull(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let sp = u16::from(self.sp);
        self.read(0x100 | sp)
    }

    fn pull16(&mut self) -> u16 {
        let lo = self.pull();
        let hi = self.pull();
        u16::from_be_bytes([hi, lo])
    }

    fn set_z(&mut self, r: u8) {
        self.z = match r {
            0 => 1,
            _ => 0,
        }
    }

    fn set_n(&mut self, r: u8) {
        self.n = match r & 0x80 {
            0 => 0,
            _ => 1,
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

    fn nmi(&mut self) {
        let pc = self.pc;
        self.push16(pc);
        self.php();
        self.pc = self.read16(0xFFFA);
        self.i = 1;
    }

    fn irq(&mut self) {
        let pc = self.pc;
        self.push16(pc);
        self.php();
        self.pc = self.read16(0xFFFE);
        self.i = 1;
    }

    /// Steps the cpu forward by a single instruction
    /// Returns the number of cycles passed
    pub fn step(&mut self) -> i32 {
        // Stall for a single cycle if stall cycles are still done
        if self.mem.cpu.stall > 0 {
            self.mem.cpu.stall -= 1;
            return 1;
        }
        let mut cycles = 0;
        let interrupt = {
            let cpustate = &mut self.mem.cpu;
            let i = cpustate.interrupt.clone();
            cpustate.clear_interrupt();
            i
        };
        match interrupt {
            None => {}
            Some(Interrupt::NMI) => {
                self.nmi();
                cycles += 7;
            }
            Some(Interrupt::IRQ) => {
                self.irq();
                cycles += 7;
            }
        }

        let opcode = {
            let pc = self.pc;
            self.read(pc)
        };
        // We now fetch the adress based on what type of addressing the
        // opcode requires, and set the page crossed, in order to
        // increment the cycles if necessary.
        let mut page_crossed = false;
        let addressing = Addressing::from_byte(OP_MODES[opcode as usize]);
        let address = match addressing {
            Addressing::Absolute => {
                let pc = self.pc.wrapping_add(1);
                self.read16(pc)
            }
            Addressing::AbsoluteX => {
                let pc = self.pc.wrapping_add(1);
                let read = self.read16(pc);
                let address = read.wrapping_add(u16::from(self.x));
                page_crossed = pages_differ(read, address);
                address
            }
            Addressing::AbsoluteY => {
                let pc = self.pc.wrapping_add(1);
                let read = self.read16(pc);
                let address = read.wrapping_add(u16::from(self.y));
                page_crossed = pages_differ(read, address);
                address
            }
            Addressing::Accumulator => 0,
            Addressing::Immediate => self.pc.wrapping_add(1),
            Addressing::Implied => 0,
            Addressing::IndexedIndirect => {
                let next = self.pc.wrapping_add(1);
                let added = self.read(next).wrapping_add(self.x);
                self.read16bug(u16::from(added))
            }
            Addressing::Indirect => {
                let next = self.pc.wrapping_add(1);
                let read = self.read16(next);
                self.read16bug(read)
            }
            Addressing::IndirectIndexed => {
                let pc = self.pc.wrapping_add(1);
                let next = u16::from(self.read(pc));
                let read = self.read16bug(next);
                let address = read.wrapping_add(u16::from(self.y));
                page_crossed = pages_differ(address, read);
                address
            }
            Addressing::Relative => {
                let pc = self.pc.wrapping_add(1);
                let offset = u16::from(self.read(pc));
                // treating this as a signed integer
                let nxt = self.pc.wrapping_add(2).wrapping_add(offset);
                if offset < 0x80 {
                    nxt
                } else {
                    nxt.wrapping_sub(0x100)
                }
            }
            Addressing::ZeroPage => {
                let next = self.pc.wrapping_add(1);
                u16::from(self.read(next))
            }
            Addressing::ZeroPageX => {
                let next = self.pc.wrapping_add(1);
                let added = self.read(next).wrapping_add(self.x);
                // we don't need to & 0xFF here, since added is a byte
                u16::from(added)
            }
            Addressing::ZeroPageY => {
                let next = self.pc.wrapping_add(1);
                let added = self.read(next).wrapping_add(self.y);
                u16::from(added)
            }
        };

        self.pc += OP_SIZES[opcode as usize];
        cycles += OP_CYCLES[opcode as usize];
        if page_crossed && EXTRA_PAGECYCLE_OPS.contains(&opcode) {
            cycles += 1;
        }
        // todo, actually emulate
        match opcode {
            // ADC
            0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                let a = self.a;
                let b = self.read(address);
                let c = self.c;
                let a2 = a.wrapping_add(b).wrapping_add(c);
                self.a = a2;
                self.set_zn(a2);
                if u32::from(a) + u32::from(b) + u32::from(c) > 0xFF {
                    self.c = 1;
                } else {
                    self.c = 0;
                }
                if (a ^ b) & 0x80 == 0 && (a ^ a2) & 0x80 != 0 {
                    self.v = 1;
                } else {
                    self.v = 0;
                }
            }
            // AND
            0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                let a = self.a & self.read(address);
                self.a = a;
                self.set_zn(a);
            }
            // ASL
            0x0A | 0x06 | 0x16 | 0x0E | 0x1E => match addressing {
                Addressing::Accumulator => {
                    self.c = (self.a >> 7) & 1;
                    let a = self.a << 1;
                    self.a = a;
                    self.set_zn(a);
                }
                _ => {
                    let mut value = self.read(address);
                    self.c = (value >> 7) & 1;
                    value <<= 1;
                    self.write(address, value);
                    self.set_zn(value);
                }
            },
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
            // BMI
            0x30 => {
                if self.n != 0 {
                    let pc = self.pc;
                    self.pc = address;
                    cycles += branch_cycles(pc, address);
                }
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
            // BRK
            0x00 => {
                let pc = self.pc;
                self.push16(pc);
                self.php();
                self.i = 1;
                self.pc = self.read16(0xFFFE);
            }
            // CLC
            0x18 => self.c = 0,
            // CLD
            0xD8 => self.d = 0,
            // CLI
            0x58 => self.i = 0,
            // CLV
            0xB8 => self.v = 0,
            // CMP
            0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                let value = self.read(address);
                let a = self.a;
                self.compare(a, value);
            }
            // CPX
            0xE0 | 0xE4 | 0xEC => {
                let value = self.read(address);
                let x = self.x;
                self.compare(x, value);
            }
            // CPY
            0xC0 | 0xC4 | 0xCC => {
                let value = self.read(address);
                let y = self.y;
                self.compare(y, value);
            }
            // DEC
            0xC6 | 0xD6 | 0xCE | 0xDE => {
                let value = self.read(address).wrapping_sub(1);
                self.write(address, value);
                self.set_zn(value);
            }
            // DEX
            0xCA => {
                let x = self.x.wrapping_sub(1);
                self.x = x;
                self.set_zn(x);
            }
            // DEY
            0x88 => {
                let y = self.y.wrapping_sub(1);
                self.y = y;
                self.set_zn(y);
            }
            // EOR
            0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                let a = self.a ^ self.read(address);
                self.a = a;
                self.set_zn(a);
            }
            // INC
            0xE6 | 0xF6 | 0xEE | 0xFE => {
                let value = self.read(address).wrapping_add(1);
                self.write(address, value);
                self.set_zn(value);
            }
            // INX
            0xE8 => {
                let x = self.x.wrapping_add(1);
                self.x = x;
                self.set_zn(x);
            }
            // INY
            0xC8 => {
                let y = self.y.wrapping_add(1);
                self.y = y;
                self.set_zn(y);
            }
            // JMP
            0x4C | 0x6C => self.pc = address,
            // JSR
            0x20 => {
                let minus = self.pc.wrapping_sub(1);
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
            // LDY
            0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                let y = self.read(address);
                self.y = y;
                self.set_zn(y);
            }
            // LSR
            0x4A | 0x46 | 0x56 | 0x4E | 0x5E => match addressing {
                Addressing::Accumulator => {
                    self.c = self.a & 1;
                    let a = self.a >> 1;
                    self.a = a;
                    self.set_zn(a);
                }
                _ => {
                    let mut value = self.read(address);
                    self.c = value & 1;
                    value >>= 1;
                    self.write(address, value);
                    self.set_zn(value);
                }
            },
            // NOP
            0xEA => {}
            // ORA
            0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                let a = self.a | self.read(address);
                self.a = a;
                self.set_zn(a);
            }
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
            // ROL
            0x2A | 0x26 | 0x36 | 0x2E | 0x3E => match addressing {
                Addressing::Accumulator => {
                    let c = self.c;
                    self.c = (self.a >> 7) & 1;
                    let a = (self.a << 1) | c;
                    self.a = a;
                    self.set_zn(a);
                }
                _ => {
                    let c = self.c;
                    let mut value = self.read(address);
                    self.c = (value >> 7) & 1;
                    value = (value << 1) | c;
                    self.write(address, value);
                    self.set_zn(value);
                }
            },
            // ROR
            0x6A | 0x66 | 0x76 | 0x6E | 0x7E => match addressing {
                Addressing::Accumulator => {
                    let c = self.c;
                    self.c = self.a & 1;
                    let a = (self.a >> 1) | (c << 7);
                    self.a = a;
                    self.set_zn(a);
                }
                _ => {
                    let c = self.c;
                    let mut value = self.read(address);
                    self.c = value & 1;
                    value = (value >> 1) | (c << 7);
                    self.write(address, value);
                    self.set_zn(value);
                }
            },
            // RTI
            0x40 => {
                let p = self.pull();
                self.set_flags((p & 0xEF) | 0x20);
                self.pc = self.pull16();
            }
            // RTS
            0x60 => self.pc = self.pull16().wrapping_add(1),
            // SBC
            0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                let a = self.a;
                let b = self.read(address);
                let c = self.c;
                let a2 = a.wrapping_sub(b).wrapping_sub(1 - c);
                self.a = a2;
                self.set_zn(a2);
                if i32::from(a) - i32::from(b) - (1 - i32::from(c)) >= 0 {
                    self.c = 1;
                } else {
                    self.c = 0;
                }
                if (a ^ b) & 0x80 != 0 && (a ^ a2) & 0x80 != 0 {
                    self.v = 1;
                } else {
                    self.v = 0;
                }
            }
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
            // STY
            0x84 | 0x94 | 0x8C => {
                let y = self.y;
                self.write(address, y);
            }
            // TAX
            0xAA => {
                let a = self.a;
                self.x = a;
                self.set_zn(a);
            }
            // TAY
            0xA8 => {
                let a = self.a;
                self.y = a;
                self.set_zn(a);
            }
            // TSX
            0xBA => {
                let sp = self.sp;
                self.x = sp;
                self.set_zn(sp);
            }
            // TXA
            0x8A => {
                let x = self.x;
                self.a = x;
                self.set_zn(x);
            }
            // TXS
            0x9A => self.sp = self.x,
            // TYA
            0x98 => {
                let y = self.y;
                self.a = y;
                self.set_zn(y);
            }
            _ => panic!("Unimplented Op {:02X}", opcode),
        }
        cycles
    }

    /// Prints the current state of the CPU
    pub fn print_state(&self) {
        println!(
            "A: {:X} X: {:X} Y: {:X} F: {:X} SP: {:X}",
            self.a,
            self.x,
            self.y,
            self.get_flags(),
            self.sp
        );
    }
}

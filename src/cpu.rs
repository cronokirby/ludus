use super::memory::MemoryBus;


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
    stall: i32
}

impl CPU {
    /// Creates a new CPU
    /// `reset` should be called if doing this at initialisation of console,
    ///  but cannot be done inside this function, since RAM isn't live.
    pub fn zeroed() -> Self {
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
        let cpu = CPU {
            pc, sp, a, x, y, c, z, i, d, b, u, v, n,
            interrupt, stall
        };
        cpu
    }

    /// Resets the CPU to its initial powerup state.
    pub fn reset(&mut self, mem: &MemoryBus) {
        // TODO: set pc to 16bit at 0xFFFC
        self.pc = self.read16(0xFFFC, mem);
        self.sp = 0xFD;
        self.set_flags(0x24);

    }

    fn read16(&self, address: u16, mem: &MemoryBus) -> u16 {
        let lo = mem.cpu_read(address) as u16;
        let hi = mem.cpu_read(address + 1) as u16;
        (hi << 8) | lo
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
}



pub fn disassemble(in_buf: &[u8]) {
    // we read in the buffer to be able to append the first 2 bytes at the end
    // to simulate wrapped reading. 2 is sufficient because 3 is the largest
    // op size
    let mut buf: Vec<u8> = in_buf.iter().cloned().collect();
    let a = buf[0].clone();
    let b = buf[1].clone();
    buf.push(a);
    buf.push(b);
    let mut pc = 0;
    let len = buf.len();
    while pc < len {
        let opcode = buf[pc] as usize;
        let op = OP_NAMES[opcode];
        let addressing = OP_MODES[opcode];
        match Addressing::from_byte(addressing) {
            Addressing::Absolute => {
                pc += 1;
                let lo = buf[pc];
                pc += 1;
                let hi = buf[pc];
                println!("{} ${:02X}{:02X}", op, hi, lo);
            }
            Addressing::AbsoluteX => {
                pc += 1;
                let lo = buf[pc];
                pc += 1;
                let hi = buf[pc];
                println!("{} ${:02X}{:02X}", op, hi, lo);
            }
            Addressing::AbsoluteY => {
                pc += 1;
                let lo = buf[pc];
                pc += 1;
                let hi = buf[pc];
                println!("{} ${:02X}{:02X}", op, hi, lo);
            }
            Addressing::Accumulator => {
                println!("{} A", op);
            }
            Addressing::Immediate => {
                pc += 1;
                println!("{} #${:02X}", op, buf[pc]);
            }
            Addressing::Implied => {
                println!("{}", op);
            }
            Addressing::IndexedIndirect => {
                pc += 1;
                println!("{} (${:02X},X)", op, buf[pc]);
            }
            Addressing::Indirect => {
                pc += 1;
                let lo = buf[pc];
                pc += 1;
                let hi = buf[pc];
                println!("{} (${:02X}{:02X})", op, hi, lo)
            }
            Addressing::IndirectIndexed => {
                pc += 1;
                println!("{} (${:02X}),Y", op, buf[pc]);
            }
            Addressing::Relative => {
                pc += 1;
                println!("{} *+{:02X}", op, buf[pc]);
            }
            Addressing::ZeroPage => {
                pc += 1;
                println!("{} ${:02X}", op, buf[pc]);
            }
            Addressing::ZeroPageX => {
                pc += 1;
                println!("{} ${:02X},X", op, buf[pc]);
            }
            Addressing::ZeroPageY => {
                pc += 1;
                println!("{} ${:02X},Y", op, buf[pc]);
            }
        }
        pc += 1;
    }
}
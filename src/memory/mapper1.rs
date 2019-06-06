use crate::cart::{Cart, Mirroring};
use crate::memory::Mapper;

const PRG_BANK_SIZE: usize = 0x4000;
const CHR_BANK_SIZE: usize = 0x1000;

struct ShiftRegister {
    register: u8,
    count: u8,
}

impl Default for ShiftRegister {
    fn default() -> Self {
        ShiftRegister {
            register: 0x10,
            count: 0,
        }
    }
}

impl ShiftRegister {
    // This returns Some if the shifting is done, and we have a final value
    fn shift(&mut self, value: u8) -> Option<u8> {
        self.register >>= 1;
        self.register |= (value & 1) << 4;
        self.count += 1;
        if self.count == 5 {
            let ret = self.register;
            *self = ShiftRegister::default();
            Some(ret)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PRGSwitching {
    /// We switch 32KB banks at a time
    DoubleBank,
    /// We fix the first bank, and can switch the second
    Fix0,
    /// We fix the second bank, and can switch the first
    Fix1,
}

impl From<u8> for PRGSwitching {
    fn from(mode: u8) -> Self {
        match mode {
            2 => PRGSwitching::Fix0,
            3 => PRGSwitching::Fix1,
            _ => PRGSwitching::DoubleBank,
        }
    }
}

/// Represents the 32KB bank of PRG data
struct PRGBanks {
    /// How many 16KB banks exist
    count: u8,
    /// The index of the first bank
    bank_0: usize,
    /// The index of the second bank
    bank_1: usize,
    /// This tells us how banks are switched around
    switching: PRGSwitching,
    control: u8,
}

impl PRGBanks {
    fn from_cart(cart: &Cart) -> Self {
        let count = cart.prg.len() / PRG_BANK_SIZE;
        PRGBanks {
            count: count as u8,
            bank_0: 0,
            bank_1: count - 1,
            switching: PRGSwitching::Fix1,
            control: 0,
        }
    }

    // Returns an index into cart.prg
    fn index(&self, address: u16) -> usize {
        if address >= 0xC000 {
            let shift = (address - 0xC000) as usize;
            self.bank_1 * PRG_BANK_SIZE + shift
        } else if address >= 0x8000 {
            let shift = (address - 0x8000) as usize;
            self.bank_0 * PRG_BANK_SIZE + shift
        } else {
            unreachable!("Address out of PRG bounds: {:X}", address);
        }
    }

    fn set_switching<S: Into<PRGSwitching>>(&mut self, switching: S) {
        let into = switching.into();
        if self.switching != into {
            self.switching = into;
            let control = self.control;
            self.write(control);
        }
    }

    fn write(&mut self, control: u8) {
        self.control = control;
        match self.switching {
            PRGSwitching::Fix0 => {
                let bank = (control & 0xF) % self.count;
                self.bank_1 = bank as usize;
            }
            PRGSwitching::Fix1 => {
                let bank = (control & 0xF) % self.count;
                self.bank_0 = bank as usize;
            }
            PRGSwitching::DoubleBank => {
                let bank_0 = (control & 0xE) % self.count;
                self.bank_0 = bank_0 as usize;
                self.bank_1 = (bank_0 + 1) as usize;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CHRSwitching {
    Single,
    Double,
}

impl From<u8> for CHRSwitching {
    fn from(mode: u8) -> Self {
        match mode {
            0 => CHRSwitching::Double,
            _ => CHRSwitching::Single,
        }
    }
}

struct CHRBanks {
    count: u8,
    bank_0: usize,
    bank_1: usize,
    switching: CHRSwitching,
    lower_control: u8,
    upper_control: u8,
}

impl CHRBanks {
    fn from_cart(cart: &Cart) -> Self {
        let count = cart.chr.len() / CHR_BANK_SIZE;
        CHRBanks {
            count: count as u8,
            bank_0: 0,
            bank_1: count - 1,
            switching: CHRSwitching::Single,
            lower_control: 0,
            upper_control: 0,
        }
    }

    fn index(&self, address: u16) -> usize {
        let index = if address < 0x1000 {
            let shift = address as usize;
            self.bank_0 * CHR_BANK_SIZE + shift
        } else if address < 0x2000 {
            let shift = (address - 0x1000) as usize;
            self.bank_1 * CHR_BANK_SIZE + shift
        } else {
            unreachable!("Address out of CHR bounds: {:X}", address);
        };
        index
    }

    fn set_switching<S: Into<CHRSwitching>>(&mut self, switching: S) {
        let into = switching.into();
        if self.switching != into {
            self.switching = into;
            let lower = self.lower_control;
            let upper = self.upper_control;
            self.write_lower(lower);
            self.write_upper(upper);
        }
    }

    fn write_lower(&mut self, control: u8) {
        self.lower_control = control;
        if self.switching == CHRSwitching::Double {
            let bank_0 = (control & 0x1E) % self.count;
            self.bank_0 = bank_0 as usize;
            self.bank_1 = (bank_0 + 1) as usize;
        } else {
            let bank = (control & 0x1F) % self.count;
            self.bank_0 = bank as usize;
        }
    }

    fn write_upper(&mut self, control: u8) {
        self.upper_control = control;
        if self.switching == CHRSwitching::Double {
            return;
        }
        let bank = (control & 0x1F) % self.count;
        self.bank_1 = bank as usize;
    }
}

/// The mapper for iNES 1.
///
/// More info: https://wiki.nesdev.com/w/index.php/MMC1
pub struct Mapper1 {
    /// The cartridge data
    cart: Cart,
    /// The program bank structure
    prg: PRGBanks,
    /// The graphical data bank structure
    chr: CHRBanks,
    /// This contains the current value of the shift register.
    ///
    /// This register is part of the behavior of the mapper, and can be used
    /// to control the bank switching behavior.
    shift_register: ShiftRegister,
}

impl Mapper1 {
    pub fn new(cart: Cart) -> Self {
        let prg = PRGBanks::from_cart(&cart);
        let chr = CHRBanks::from_cart(&cart);
        Mapper1 {
            cart,
            prg: prg,
            chr: chr,
            shift_register: ShiftRegister::default(),
        }
    }

    fn write_control(&mut self, control: u8) {
        let mirroring = Mirroring::from(control & 3);
        self.cart.mirroring = mirroring;
        let prg_mode = (control >> 2) & 3;
        self.prg.set_switching(prg_mode);
        let chr_mode = (control >> 4) & 1;
        self.chr.set_switching(chr_mode);
    }

    fn write_shift(&mut self, address: u16, shift: u8) {
        if address >= 0xE000 {
            self.prg.write(shift);
        } else if address >= 0xC000 {
            self.chr.write_upper(shift);
        } else if address >= 0xA000 {
            self.chr.write_lower(shift);
        } else {
            self.write_control(shift);
        }
    }
}

impl Mapper for Mapper1 {
    fn read(&self, address: u16) -> u8 {
        //println!("M1 read {:X}", address);
        if address < 0x2000 {
            self.cart.chr[self.chr.index(address)]
        } else if address >= 0x8000 {
            self.cart.prg[self.prg.index(address)]
        } else if address >= 0x6000 {
            let shift = address - 0x6000;
            self.cart.sram[shift as usize]
        } else {
            panic!("Mapper1 unhandled read at {:X}", address);
        }
    }

    fn mirroring_mode(&self) -> Mirroring {
        self.cart.mirroring
    }

    fn write(&mut self, address: u16, value: u8) {
        //println!("M1 write {:X} value: {:X}", address, value);
        if address < 0x2000 {
            //println!("{:X}", self.cart.prg.len());
            self.cart.chr[self.chr.index(address)] = value;
        } else if address >= 0x8000 {
            if value & 0x80 != 0 {
                self.shift_register = ShiftRegister::default();
                self.write_control(0xC);
            } else if let Some(shift) = self.shift_register.shift(value) {
                self.write_shift(address, shift);
            }
        } else if address >= 0x6000 {
            let shift = address - 0x6000;
            self.cart.sram[shift as usize] = value;
        } else {
            panic!("Mapper1 unhandled write at {:X}", address);
        }
    }
}

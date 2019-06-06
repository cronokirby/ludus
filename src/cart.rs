use std::convert::TryFrom;

/// Represents the possible errors when decoding a Cart
#[derive(Clone, Copy, Debug)]
pub enum CartReadingError {
    UnrecognisedFormat,
    UnknownMapper(u8),
}

/// Represents the type of mirroring present on a cartridge
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mirroring {
    /// Tables start wrapping horizontally
    Horizontal,
    /// Tables start wrapping vertically
    Vertical,
    /// Every mirror points to the first table
    SingleLower,
    /// Every mirror points to the second table
    SingleUpper,
}

impl From<u8> for Mirroring {
    /// Create a mirroring from a boolean, with true representing vertical
    fn from(mirroring: u8) -> Self {
        match mirroring {
            0 => Mirroring::SingleLower,
            1 => Mirroring::SingleUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }
}

impl Mirroring {
    /// Returns true if mirroring is Vertical
    pub fn is_vertical(self) -> bool {
        self == Mirroring::Vertical
    }

    /// Mirrors an address >= 0x2000
    pub(crate) fn mirror_address(self, address: u16) -> u16 {
        let address = (address - 0x2000) % 0x1000;
        let table = match (self, address / 0x400) {
            (Mirroring::Horizontal, 0) => 0,
            (Mirroring::Horizontal, 1) => 0,
            (Mirroring::Horizontal, 2) => 1,
            (Mirroring::Horizontal, 3) => 1,
            (Mirroring::Vertical, 0) => 0,
            (Mirroring::Vertical, 1) => 1,
            (Mirroring::Vertical, 2) => 0,
            (Mirroring::Vertical, 3) => 1,
            (Mirroring::SingleLower, _) => 0,
            (Mirroring::SingleUpper, _) => 1,
            _ => 0,
        };
        0x2000 + table * 0x400 + (address % 0x400)
    }
}

/// This represents the different type of mappers this crate supports.
///
/// In theory, the mapper id in a cart could be any byte, but only a small subset
/// of mappers were actually used.
#[derive(Clone, Copy, Debug)]
pub enum MapperID {
    /// The mapper used for 0x0 and 0x2
    M2,
    /// iNES mapper 0x1
    M1,
}

impl TryFrom<u8> for MapperID {
    type Error = CartReadingError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(MapperID::M2),
            1 => Ok(MapperID::M1),
            2 => Ok(MapperID::M2),
            _ => Err(CartReadingError::UnknownMapper(byte)),
        }
    }
}

/// Represents an NES Cartridge
/// The PRG and CHR roms vary in sizes between carts,
/// which is why they're stored in Vecs.
pub struct Cart {
    /// Represents the PRG ROM, in multiple 16KB chunks
    pub prg: Vec<u8>,
    /// Represents the CHR ROM, in multiple 8KB chunks
    pub chr: Vec<u8>,
    /// The SRAM, always 8KB
    pub sram: [u8; 0x2000],
    /// The ID of the Mapper this cart uses
    pub mapper: MapperID,
    /// What type of mirroring is used in this cart
    pub mirroring: Mirroring,
    /// Indicates whether or not a battery backed RAM is present
    pub has_battery: bool,
}

impl Cart {
    /// Reads a buffer of bytes into a Cart,
    /// detecting and parsing the format automatically.
    pub fn from_bytes(buffer: &[u8]) -> Result<Cart, CartReadingError> {
        if buffer[0..4] == [0x4E, 0x45, 0x53, 0x1A] {
            Cart::from_ines(buffer)
        } else {
            Err(CartReadingError::UnrecognisedFormat)
        }
    }

    /// Reads an INES formatted buffer, including the header
    fn from_ines(buffer: &[u8]) -> Result<Cart, CartReadingError> {
        let prg_chunks = buffer[4] as usize;
        let chr_chunks = buffer[5] as usize;
        let flag6 = buffer[6];
        let flag7 = buffer[7];
        let trainer_offset = if flag6 & 0b100 > 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_offset;
        let prg_end = prg_start + 0x4000 * prg_chunks;
        let chr_end = prg_end + 0x2000 * chr_chunks;
        let mapper = MapperID::try_from((flag6 >> 4) | (flag7 & 0xF0))?;
        let mirroring = if flag6 & 1 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
        Ok(Cart {
            prg: buffer[prg_start..prg_end].to_vec(),
            chr: buffer[prg_end..chr_end].to_vec(),
            mapper,
            sram: [0; 0x2000],
            mirroring,
            has_battery: flag6 & 0b10 > 0,
        })
    }
}

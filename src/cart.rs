/// Represents the possible errors when decoding a Cart
#[derive(Debug)]
pub enum CartReadingError {
    UnrecognisedFormat
}

/// Represents the type of mirroring present on a cartridge
#[derive(Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical
}

impl Mirroring {
    /// Create a mirroring from a boolean, representing whether or not
    /// the mirroring is vertical.
    pub fn from_bool(b: bool) -> Self {
        match b {
            false => Mirroring::Horizontal,
            true => Mirroring::Vertical
        }
    }

    /// Returns true if mirroring is Vertical
    pub fn is_vertical(&self) -> bool {
        match self {
            Horizontal => false,
            Vertical => true
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
    pub mapper: u8,
    /// What type of mirroring is used in this cart
    pub mirroring: Mirroring,
    /// Indicates whether or not a battery backed RAM is present
    pub has_battery: bool
}

impl Cart {
    /// Reads a buffer of bytes into a Cart,
    /// detecting and parsing the format automatically.
    pub fn from_bytes(buffer: &[u8]) -> Result<Cart, CartReadingError> {
        if buffer[0..4] == [0x4E, 0x45, 0x53, 0x1A] {
            Ok(Cart::from_ines(buffer))
        } else {
            Err(CartReadingError::UnrecognisedFormat)
        }
    }

    /// Reads an INES formatted buffer, including the header
    fn from_ines(buffer: &[u8]) -> Cart {
        let prg_chunks = buffer[4] as usize;
        let chr_chunks = buffer[5] as usize;
        let flag6 = buffer[6];
        let flag7 = buffer[7];
        let trainer_offset = if flag6 & 0b100 > 0 { 512 } else { 0 };
        let prg_start = 16 + trainer_offset;
        let prg_end = prg_start + 0x4000 * prg_chunks;
        let chr_end = prg_end + 0x2000 * chr_chunks;
        // todo, check length here
        Cart {
            prg: buffer[prg_start..prg_end].to_vec(),
            chr: buffer[prg_end..chr_end].to_vec(),
            sram: [0; 0x2000],
            mapper: ((flag6 & 0xF0) >> 4) | (flag7 & 0xF0),
            mirroring: Mirroring::from_bool(flag6 & 0b1 > 0),
            has_battery: flag6 & 0b10 > 0
        }
    }
}
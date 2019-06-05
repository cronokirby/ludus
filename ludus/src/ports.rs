/// This represents an audio device we can push samples to.
///
/// The APU will dump its samples into an object implementing
/// this Trait as it generates them.
pub trait AudioDevice {
    fn push_sample(&mut self, sample: f32);
}

/// This represents the width of the display in pixel
pub const NES_WIDTH: usize = 256;
/// This represents the height of the display in pixel
pub const NES_HEIGHT: usize = 240;
const BUFFER_PIXELS: usize = NES_WIDTH * NES_HEIGHT;

/// Represents a buffer of pixels the PPU writes to.
///
/// The pixels can be read as a slice of u32 values in ARGB format, in row order.
///
/// The default value for the pixel buffer is completely transparent.
///
/// This struct is somewhat large, so it should be boxed when included
/// in another struct to avoid blowing up the stack.
pub struct PixelBuffer([u32; BUFFER_PIXELS]);

impl Default for PixelBuffer {
    fn default() -> Self {
        PixelBuffer([0; BUFFER_PIXELS])
    }
}

impl AsRef<[u32]> for PixelBuffer {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl PixelBuffer {
    pub(crate) fn write(&mut self, x: usize, y: usize, argb: u32) {
        let index = NES_WIDTH * y + x;
        self.0[index] = argb;
    }
}

/// This represents a video device we can write a pixel buffer to.
pub trait VideoDevice {
    fn blit_pixels(&mut self, pixels: &PixelBuffer);
}

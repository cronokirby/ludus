/// This represents an audio device we can push samples to.
///
/// The APU will dump its samples into an object implementing
/// this Trait as it generates them.
pub trait AudioDevice {
    fn push_sample(&mut self, sample: f32);
}

/// This represents a video device we can write a pixel buffer to.
///
/// The NES has a 256x240 resolution (width x height). This trait will
/// receive a buffer of ARGB pixels, in big endian order, organized
/// row by row, and should use that to blit them to the screen.
pub trait VideoDevice {
    fn blit_pixels(&mut self, pixels: &[u32]);
}

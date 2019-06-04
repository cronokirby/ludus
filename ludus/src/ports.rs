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
/// receive a buffer of RGBA pixels, in big endian order, organized
/// row by row, and should use that to blit them to the screen.
pub trait VideoDevice {
    fn blit_pixels(&mut self, pixels: &[u32]);
}

/// This is a conveniance struct to pass all the device handles at once.
/// 
/// This exists because when the emulator steps forward, we want to have access
/// to all the devices that may get accessed during that process. The APU
/// and PPU can both write to their corresponding devices at any time, so we need
/// to have access at any time we advance emulation.
pub struct DeviceHandle<'a> {
    pub audio: &'a mut (AudioDevice + 'a),
    pub video: &'a mut (VideoDevice + 'a),
}

impl<'a> DeviceHandle<'a> {
    /// Construct a device handle given references to the devices.
    pub fn from<A: AudioDevice, V: VideoDevice>(audio: &'a mut A, video: &'a mut V) -> Self {
        DeviceHandle { audio, video }
    }
}

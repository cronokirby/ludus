pub trait VideoDevice {
    fn blit_pixels(&mut self, pixels: &[u32]);
}

pub trait AudioDevice {
    fn push_sample(&mut self, sample: f32);
}
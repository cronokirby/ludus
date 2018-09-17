pub struct APU {
    /// Used to time frame ticks
    frame_tick: u16
    /// Used to time sample ticks
    sample_tick: u16
    /// The number of ticks after which to reset the sample tick.
    /// This is determined from the sample rate at runtime
    sample_cap: u16
}

impl APU {
    pub fn new(sample_rate: u32) -> Self {
        let sample_cap = (1_790_000 / sample_rate) as u16;
        APU {
            frame_tick: 0, sample_tick: 0, sample_cap
        }
    }

    /// Steps the apu forward by one CPU tick
    pub fn step(&mut self) {
        // step timer
        self.frame_tick += 1;
        // This is equivalent to firing at roughly 240 hz
        if self.frame_tick >= 7458 {
            self.frame_tick = 0;
            // step frame counter
        }
        self.sample_tick += 1;
        if self.sample_tick >= self.sample_cap {
            self.sample_tick = 0;
            // send sample
        }
    }
}
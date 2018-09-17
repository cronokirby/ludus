/// Represents the Square signal generator of the APU
struct Square {
    /// Whether or not this generator is turned on
    enabled: bool,
    /// Whether or not the generator is on the first channel
    first_channel: bool,
    /// Whether or not to decrement the length value
    length_enabled: bool,
    /// The current point in the length timer
    length_value: u8,
    /// Timer period used for timing sweeps
    timer_period: u16,
    /// The current timer value for sweeps
    timer_value: u16,
    /// Along with `duty_value`, determines what duty to apply
    duty_mode: u8,
    duty_value: u8,
    /// Used to sweep timing
    sweep_reload: bool,
    /// Whether or not sweeps will trigger
    sweep_enabled: bool,
    /// Whether or not to reverse sweeps
    sweep_negate: bool,
    /// Used as a shift for timing sweeps
    sweep_shift: u8,
    /// Used as the period length for sweeps
    sweep_period: u8,
    /// The current value of the sweep
    sweep_value: u8,
    /// Enables envelope
    envelope_enabled: bool,
    /// Uses for timing envelope sounds
    envelope_loop: bool,
    envelope_start: bool,
    /// Used for keeping track of the current position in the envelope
    envelope_period: u8,
    /// Current envelope value
    envelope_value: u8,
    /// Current envelope volume
    envelope_volume: u8,
    /// Base volume
    constant_volume: u8
}

impl Square {
    fn new() -> Self {
        Square {
            enabled: false, first_channel: false,
            length_enabled: false, length_value: 0,
            timer_period: 0, timer_value: 0,
            duty_mode: 0, duty_value: 0,
            sweep_reload: false, sweep_enabled: false,
            sweep_negate: false, sweep_shift: 0,
            sweep_period: 0, sweep_value: 0,
            envelope_enabled: false, envelope_loop: false,
            envelope_start: false, envelope_start: false,
            envelope_period: 0, envelope_value: 0,
            envelope_volume: 0, constant_volume: 0
        }
    }
}


/// Represents the audio processing unit
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
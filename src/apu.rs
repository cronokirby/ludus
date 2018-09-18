const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6,
    160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30
];

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1]
];

//const TRIANGLE_TABLE: [u8; ]


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
    fn new(first_channel: bool) -> Self {
        Square {
            enabled: false, first_channel,
            length_enabled: false, length_value: 0,
            timer_period: 0, timer_value: 0,
            duty_mode: 0, duty_value: 0,
            sweep_reload: false, sweep_enabled: false,
            sweep_negate: false, sweep_shift: 0,
            sweep_period: 0, sweep_value: 0,
            envelope_enabled: false, envelope_loop: false,
            envelope_start: false,
            envelope_period: 0, envelope_value: 0,
            envelope_volume: 0, constant_volume: 0
        }
    }

    fn write_control(&mut self, value: u8) {
        self.duty_mode = (value >> 6) & 3;
        self.length_enabled = (value >> 5) & 1 == 0;
        self.envelope_loop = (value >> 5) & 1 == 1;
        self.envelope_enabled = (value >> 4) & 1 == 0;
        self.envelope_period = value & 15;
        self.constant_volume = value & 15;
        self.envelope_start = true;
    }

    fn write_sweep(&mut self, value: u8) {
        self.sweep_enabled = (value >> 7) & 1 == 1;
        self.sweep_period = (value >> 4) & 7 + 1;
        self.sweep_negate = (value >> 3) & 1 == 1;
        self.sweep_shift = value & 7;
        self.sweep_reload = true;
    }

    fn write_low_timer(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | (value as u16);
    }

    fn write_how_timer(&mut self, value: u8) {
        self.length_value = LENGTH_TABLE[(value >> 3) as usize];
        let shifted = ((value & 7) as u16) << 8;
        self.timer_period = (self.timer_period & 0xFF) | shifted;
        self.envelope_start = true;
        self.duty_value = 0;
    }

    fn step_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            self.duty_value = (self.duty_value + 1) % 8;
        } else {
            self.timer_value -= 1;
        }
    }

    fn step_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_volume = 15;
            self.envelope_value = self.envelope_period;
            self.envelope_start = false;
        } else if self.envelope_value > 0 {
            self.envelope_value -= 1;
        } else {
            if self.envelope_volume > 0 {
                self.envelope_volume -= 1;
            } else if self.envelope_loop {
                self.envelope_volume = 15;
            }
            self.envelope_value = self.envelope_period;
        }
    }

    fn sweep(&mut self) {
        let delta = self.timer_period >> self.sweep_shift;
        if self.sweep_negate {
            self.timer_period -= delta;
            if self.first_channel {
                self.timer_period -= 1;
            }
        } else {
            self.timer_period += delta;
        }
    }

    fn step_sweep(&mut self) {
        if self.sweep_reload {
            if self.sweep_enabled && self.sweep_value == 0 {
                self.sweep();
            }
            self.sweep_value = self.sweep_period;
            self.sweep_reload = false;
        } else if self.sweep_value > 0 {
            self.sweep_value -= 1;
        } else {
            if self.sweep_enabled {
                self.sweep();
            }
            self.sweep_value = self.sweep_period;
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length_value > 0 {
            self.length_value -= 1;
        }
    }

    fn output(&self) -> u8 {
        if self.enabled {
            return 0;
        } else if self.length_value == 0 {
            return 0;
        }
        let (i1, i2) = (self.duty_mode as usize, self.duty_value as usize);
        if DUTY_TABLE[i1][i2] == 0 {
            return 0;
        }
        if self.timer_period < 8 || self.timer_period > 0x7FF {
            return 0;
        }
        if self.envelope_enabled {
            self.envelope_volume
        } else {
            self.constant_volume
        }
    }
}


/// Represents the triangle signal simulator
struct Triangle {
    /// Whether or not output is enabled
    enabled: bool,
    /// Like in Pulse, these are used to control output generation
    length_enabled: bool,
    length_value: u8,
    /// Keeps track of the reset value of the timer
    timer_period: u16,
    /// Counts down to 0 before resetting to timer_period
    timer_value: u16,
    /// Used for manipulating the duty of the signal
    duty_value: u8,
    /// Used to keep track of the max value of the period
    counter_period: u8,
    /// Counts down to 0 before restting to counter_period
    counter_value: u8,
    /// Controls whether or not the value will wrap around
    counter_reload: bool
}

impl Triangle {
    fn new() -> Self {
        Triangle {
            enabled: false,
            length_enabled: false, length_value: 0,
            timer_period: 0, timer_value: 0,
            duty_value: 0,
            counter_period: 0, counter_value: 0,
            counter_reload: false
        }
    }

    fn write_control(&mut self, value: u8) {
        self.length_enabled = (value >> 7) & 1 == 0;
        self.counter_period = value & 0x7F;
    }

    fn write_low_timer(&mut self, value: u8) {
        let low = value as u16;
        self.timer_period = (self.timer_period & 0xFF00) | low;
    }

    fn write_high_timer(&mut self, value: u8) {
        self.length_value = LENGTH_TABLE[(value >> 3) as usize];
        let high = ((value & 7) << 8) as u16;
        self.timer_period = (self.timer_period & 0xFF) | high;
        self.timer_value = self.timer_period;
        self.counter_reload = true;
    }

    fn step_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            if self.length_value > 0 && self.counter_value > 0 {
               self.duty_value = (self.duty_value + 1)  % 32;
            }
        } else {
            self.timer_value -= 1;
        }
    }

    fn step_length(&mut self) {
        if self.length_enabled && self.length_value > 0 {
            self.length_value -= 1;
        }
    }

    fn step_counter(&mut self) {
        if self.counter_reload {
            self.counter_value = self.counter_period;
        } else if self.counter_value > 0 {
            self.counter_value -= 1;
        }
        if self.length_enabled {
            self.counter_reload = false;
        }
    }

    fn output(&self) -> u8 {
        if !self.enabled {
            0
        } else if self.length_value == 0 {
            0
        } else if self.counter_value == 0 {
            0
        } else {
            0
        }
    }
}


/// Represents the audio processing unit
pub struct APU {
    /// Used to time frame ticks
    frame_tick: u16,
    /// Used to time sample ticks
    sample_tick: u16,
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
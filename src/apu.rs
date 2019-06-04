use super::memory::MemoryBus;

use std::f32::consts::PI;
use std::sync::mpsc::Sender;

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

const TRIANGLE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
    13, 14, 15,
];

const NOISE_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

const DMC_TABLE: [u8; 16] = [
    214, 190, 170, 160, 143, 127, 113, 107, 95, 80, 71, 64, 53, 42, 36, 27,
];

/// Constructs a new tnd table
fn make_pulse_table() -> [f32; 31] {
    let mut arr = [0.0; 31];
    for (i, item) in arr.iter_mut().enumerate() {
        *item = 95.52 / (8128.0 / (i as f32) + 100.0);
    }
    arr
}

/// Constructs a new pulse table
fn make_tnd_table() -> [f32; 203] {
    let mut arr = [0.0; 203];
    for (i, item) in arr.iter_mut().enumerate() {
        *item = 163.37 / (24329.0 / (i as f32) + 100.0);
    }
    arr
}

/// Represents a first order filter, implementing the following formula:
/// y_n = b0 * x_n + b1 * x_(n-1) - a y_(n-1)
struct Filter {
    b0: f32,
    b1: f32,
    a: f32,
    /// Caches the previous x value
    prev_x: f32,
    prev_y: f32,
}

/// Used to calculate the frequency constants used in Filters
fn frequency_constants(sample_rate: u32, cutoff: f32) -> (f32, f32) {
    let c = (sample_rate as f32) / PI / cutoff;
    let a0 = 1.0 / (1.0 + c);
    (c, a0)
}

impl Filter {
    /// Constructs a new low pass filter
    fn low_pass(sample_rate: u32, cutoff: f32) -> Filter {
        let (c, a0) = frequency_constants(sample_rate, cutoff);
        Filter {
            b0: a0,
            b1: a0,
            a: (1.0 - c) * a0,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    /// Constructs a new high pass filter
    fn high_pass(sample_rate: u32, cutoff: f32) -> Filter {
        let (c, a0) = frequency_constants(sample_rate, cutoff);
        Filter {
            b0: c * a0,
            b1: -c * a0,
            a: (1.0 - c) * a0,
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }

    fn step(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.prev_x - self.a * self.prev_y;
        self.prev_y = y;
        self.prev_x = x;
        y
    }
}

/// Represents the collection of filters applied to the output of the APU
struct FilterChain {
    high1: Filter,
    high2: Filter,
    low: Filter,
}

impl FilterChain {
    fn new(sample_rate: u32) -> Self {
        FilterChain {
            high1: Filter::high_pass(sample_rate, 90.0),
            high2: Filter::high_pass(sample_rate, 440.0),
            low: Filter::low_pass(sample_rate, 14000.0),
        }
    }

    fn step(&mut self, x: f32) -> f32 {
        let x1 = self.high1.step(x);
        let x2 = self.high2.step(x1);
        self.low.step(x2)
    }
}

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
    constant_volume: u8,
}

impl Square {
    fn new(first_channel: bool) -> Self {
        Square {
            enabled: false,
            first_channel,
            length_enabled: false,
            length_value: 0,
            timer_period: 0,
            timer_value: 0,
            duty_mode: 0,
            duty_value: 0,
            sweep_reload: false,
            sweep_enabled: false,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_period: 0,
            sweep_value: 0,
            envelope_enabled: false,
            envelope_loop: false,
            envelope_start: false,
            envelope_period: 0,
            envelope_value: 0,
            envelope_volume: 0,
            constant_volume: 0,
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
        self.timer_period = (self.timer_period & 0xFF00) | u16::from(value);
    }

    fn write_high_timer(&mut self, value: u8) {
        self.length_value = LENGTH_TABLE[(value >> 3) as usize];
        let shifted = u16::from(value & 7) << 8;
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
        if !self.enabled {
            return 0;
        }
        if self.length_value == 0 {
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
    counter_reload: bool,
}

impl Triangle {
    fn new() -> Self {
        Triangle {
            enabled: false,
            length_enabled: false,
            length_value: 0,
            timer_period: 0,
            timer_value: 0,
            duty_value: 0,
            counter_period: 0,
            counter_value: 0,
            counter_reload: false,
        }
    }

    fn write_control(&mut self, value: u8) {
        self.length_enabled = (value >> 7) & 1 == 0;
        self.counter_period = value & 0x7F;
    }

    fn write_low_timer(&mut self, value: u8) {
        let low = u16::from(value);
        self.timer_period = (self.timer_period & 0xFF00) | low;
    }

    fn write_high_timer(&mut self, value: u8) {
        self.length_value = LENGTH_TABLE[(value >> 3) as usize];
        let high = u16::from(value & 7) << 8;
        self.timer_period = (self.timer_period & 0xFF) | high;
        self.timer_value = self.timer_period;
        self.counter_reload = true;
    }

    fn step_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            if self.length_value > 0 && self.counter_value > 0 {
                self.duty_value = (self.duty_value + 1) % 32;
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
            TRIANGLE_TABLE[self.duty_value as usize]
        }
    }
}

/// Represents the noise signal generator
struct Noise {
    /// Whether or not output is enabled for this component
    enabled: bool,
    /// Which of 2 noise modes the generator is in
    mode: bool,
    shift_register: u16,
    /// Enables sweep timing
    length_enabled: bool,
    /// Value used for sweep timing
    length_value: u8,
    /// Used as the point of reset for the global timer
    timer_period: u16,
    /// Used to keep track of the state of the global timer
    timer_value: u16,
    /// Whether or not an envelope effect is enabled
    envelope_enabled: bool,
    /// Whether or not to loop back around at the end of an envelope
    envelope_loop: bool,
    /// Whether or not to trigger an envelope
    envelope_start: bool,
    /// The point at which to reset the envelope timer
    envelope_period: u8,
    /// Used to control the timing of the envelope effect
    envelope_value: u8,
    /// Used to control the volume of the envelope effect
    envelope_volume: u8,
    /// Background volume
    constant_volume: u8,
}

impl Noise {
    fn new(shift_register: u16) -> Self {
        Noise {
            enabled: false,
            mode: false,
            shift_register,
            length_enabled: true,
            length_value: 0,
            timer_period: 0,
            timer_value: 0,
            envelope_enabled: false,
            envelope_loop: false,
            envelope_start: false,
            envelope_period: 0,
            envelope_value: 0,
            envelope_volume: 0,
            constant_volume: 0,
        }
    }

    fn write_control(&mut self, value: u8) {
        self.length_enabled = (value >> 5) & 1 == 0;
        self.envelope_loop = (value >> 5) & 1 == 1;
        self.envelope_enabled = (value >> 4) & 1 == 0;
        self.envelope_period = value & 15;
        self.constant_volume = value & 15;
        self.envelope_start = true;
    }

    fn write_period(&mut self, value: u8) {
        self.mode = value & 0x80 == 0x80;
        self.timer_period = NOISE_TABLE[(value & 0xF) as usize];
    }

    fn write_length(&mut self, value: u8) {
        self.length_value = LENGTH_TABLE[(value >> 3) as usize];
        self.envelope_start = true;
    }

    fn step_timer(&mut self) {
        if self.timer_value == 0 {
            self.timer_value = self.timer_period;
            let shift = if self.mode { 6 } else { 1 };
            let b1 = self.shift_register & 1;
            let b2 = (self.shift_register >> shift) & 1;
            self.shift_register >>= 1;
            self.shift_register |= (b1 ^ b2) << 14;
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

    fn step_length(&mut self) {
        if self.length_enabled && self.length_value > 0 {
            self.length_value -= 1;
        }
    }

    fn output(&mut self) -> u8 {
        if !self.enabled {
            0
        } else if self.length_value == 0 {
            0
        } else if self.shift_register & 1 == 1 {
            0
        } else if self.envelope_enabled {
            self.envelope_volume
        } else {
            self.constant_volume
        }
    }
}

/// Generator for DMC Samples
struct DMC {
    /// Whether or not output is enabled for this generator
    enabled: bool,
    /// Current output value
    value: u8,
    /// The address of the current sample
    sample_address: u16,
    /// The length of the current sample
    sample_length: u16,
    /// The current address being read from
    current_address: u16,
    /// The current length left to read
    current_length: u16,
    /// Contains the value of shifts for effects
    shift_register: u8,
    bit_count: u8,
    /// The point at which the tick resets
    tick_period: u8,
    /// The current value of the tick
    tick_value: u8,
    /// Whether or not to loop back at the end of a sound cycle
    do_loop: bool,
    /// Whether or not an irq ocurred
    irq: bool,
}

impl DMC {
    fn new() -> Self {
        DMC {
            enabled: false,
            value: 0,
            sample_address: 0,
            sample_length: 0,
            current_address: 0,
            current_length: 0,
            shift_register: 0,
            bit_count: 0,
            tick_period: 0,
            tick_value: 0,
            do_loop: false,
            irq: false,
        }
    }

    fn write_control(&mut self, value: u8) {
        self.irq = value & 0x80 == 0x80;
        self.do_loop = value & 0x40 == 0x40;
        self.tick_period = DMC_TABLE[(value & 0xF) as usize];
    }

    fn write_value(&mut self, value: u8) {
        self.value = value & 0x7F;
    }

    fn write_address(&mut self, value: u8) {
        self.sample_address = 0xC000 | (u16::from(value) << 6);
    }

    fn write_length(&mut self, value: u8) {
        self.sample_length = (u16::from(value) << 4) | 1;
    }

    fn restart(&mut self) {
        self.current_address = self.sample_address;
        self.current_length = self.sample_length;
    }

    fn step_timer(&mut self, read: u8) -> bool {
        if self.enabled {
            let stall = self.step_reader(read);
            if self.tick_value == 0 {
                self.tick_value = self.tick_period;
                self.step_shifter();
            } else {
                self.tick_value -= 1;
            }
            stall
        } else {
            false
        }
    }

    // returns whether or not to stall
    fn step_reader(&mut self, read: u8) -> bool {
        if self.current_length > 0 && self.bit_count == 0 {
            self.shift_register = read;
            self.bit_count = 8;
            self.current_address = self.current_address.wrapping_add(1);
            if self.current_address == 0 {
                self.current_address = 0x8000;
            }
            self.current_length -= 1;
            if self.current_length == 0 && self.do_loop {
                self.restart();
            }
            true
        } else {
            false
        }
    }

    fn step_shifter(&mut self) {
        if self.bit_count != 0 {
            if self.shift_register & 1 == 1 {
                if self.value <= 125 {
                    self.value += 2;
                }
            } else {
                if self.value >= 2 {
                    self.value -= 2;
                }
            }
            self.shift_register >>= 1;
            self.bit_count -= 1;
        }
    }

    fn output(&self) -> u8 {
        self.value
    }
}

/// Contains registers that are written to across the memory bus
pub struct APUState {
    /// The first square output generator
    square1: Square,
    /// The second square output generator
    square2: Square,
    /// The triangle output generator
    triangle: Triangle,
    /// The noise output generator
    noise: Noise,
    /// The DMC sample generator
    dmc: DMC,
    /// The current frame period
    frame_period: u8,
    /// Whether or not to trigger IRQs
    frame_irq: bool,
}

impl Default for APUState {
    fn default() -> Self {
        APUState {
            square1: Square::new(true),
            square2: Square::new(false),
            triangle: Triangle::new(),
            noise: Noise::new(1),
            dmc: DMC::new(),
            frame_period: 0,
            frame_irq: false,
        }
    }
}

impl APUState {
    pub fn new() -> Self {
        APUState::default()
    }

    pub fn read_register(&self, address: u16) -> u8 {
        match address {
            0x4015 => self.read_status(),
            // Some addresses may be read by bad games
            _ => 0,
        }
    }

    fn read_status(&self) -> u8 {
        let mut result = 0;
        if self.square1.length_value > 0 {
            result |= 1;
        }
        if self.square2.length_value > 0 {
            result |= 2;
        }
        if self.triangle.length_value > 0 {
            result |= 4;
        }
        if self.noise.length_value > 0 {
            result |= 8;
        }
        if self.dmc.current_length > 0 {
            result |= 16;
        }
        result
    }

    pub fn write_register(&mut self, address: u16, value: u8) {
        match address {
            0x4000 => self.square1.write_control(value),
            0x4001 => self.square1.write_sweep(value),
            0x4002 => self.square1.write_low_timer(value),
            0x4003 => self.square1.write_high_timer(value),
            0x4004 => self.square2.write_control(value),
            0x4005 => self.square2.write_sweep(value),
            0x4006 => self.square2.write_low_timer(value),
            0x4007 => self.square2.write_high_timer(value),
            0x4008 => self.triangle.write_control(value),
            0x4009 => {}
            0x4010 => self.dmc.write_control(value),
            0x4011 => self.dmc.write_value(value),
            0x4012 => self.dmc.write_address(value),
            0x4013 => self.dmc.write_length(value),
            0x400A => self.triangle.write_low_timer(value),
            0x400B => self.triangle.write_high_timer(value),
            0x400C => self.noise.write_control(value),
            0x400D => {}
            0x400E => self.noise.write_period(value),
            0x400F => self.noise.write_length(value),
            0x4015 => self.write_control(value),
            0x4017 => self.write_frame_counter(value),
            // We may want to panic here
            _ => {}
        }
    }

    fn write_control(&mut self, value: u8) {
        self.square1.enabled = value & 1 == 1;
        self.square2.enabled = value & 2 == 2;
        self.triangle.enabled = value & 4 == 4;
        self.noise.enabled = value & 8 == 8;
        self.dmc.enabled = value & 16 == 16;
        if !self.square1.enabled {
            self.square1.length_value = 0;
        }
        if !self.square2.enabled {
            self.square2.length_value = 0;
        }
        if !self.triangle.enabled {
            self.triangle.length_value = 0;
        }
        if !self.noise.enabled {
            self.noise.length_value = 0;
        }
        if !self.dmc.enabled {
            self.dmc.current_length = 0;
        } else if self.dmc.current_length == 0 {
            self.dmc.restart();
        }
    }

    fn write_frame_counter(&mut self, value: u8) {
        self.frame_period = 4 + ((value >> 7) & 1);
        self.frame_irq = (value >> 6) & 1 == 0;
        // Catching up with the frame period
        if self.frame_period == 5 {
            self.step_envelope();
            self.step_sweep();
            self.step_length();
        }
    }

    fn step_envelope(&mut self) {
        self.square1.step_envelope();
        self.square2.step_envelope();
        self.triangle.step_counter();
        self.noise.step_envelope();
    }

    fn step_sweep(&mut self) {
        self.square1.step_sweep();
        self.square2.step_sweep();
    }

    fn step_length(&mut self) {
        self.square1.step_length();
        self.square2.step_length();
        self.triangle.step_length();
        self.noise.step_length();
    }
}

/// Represents the audio processing unit
pub struct APU {
    /// The channel used to send output to
    channel: Sender<f32>,
    /// The chain of filters used on the output of the generators
    filter: FilterChain,
    // The 2 tables used to find the height of the wave output
    pulse_table: [f32; 31],
    tnd_table: [f32; 203],
    /// Used to time frame ticks
    frame_tick: u16,
    /// Used to time sample ticks
    sample_tick: u16,
    /// The number of ticks after which to reset the sample tick.
    /// This is determined from the sample rate at runtime
    sample_cap: u16,
    /// The current frame value
    frame_value: u8,
}

impl APU {
    pub fn new(tx: Sender<f32>, sample_rate: u32) -> Self {
        let sample_cap = (1_790_000 / sample_rate) as u16;
        let tnd_table = make_tnd_table();
        let pulse_table = make_pulse_table();
        APU {
            channel: tx,
            filter: FilterChain::new(sample_rate),
            tnd_table,
            pulse_table,
            frame_tick: 0,
            sample_tick: 0,
            sample_cap,
            frame_value: 0,
        }
    }

    /// Steps the apu forward by one CPU tick
    pub fn step(&mut self, m: &mut MemoryBus) {
        // step timer
        self.frame_tick += 1;
        // we can use the first bit of the frame_tick as an even odd flag
        let toggle = self.frame_tick & 1 == 0;
        self.step_timer(m, toggle);
        // This is equivalent to firing at roughly 240 hz
        if self.frame_tick >= 7458 {
            self.frame_tick = 0;
            self.step_framecounter(m);
        }
        self.sample_tick += 1;
        if self.sample_tick >= self.sample_cap {
            self.sample_tick = 0;
            self.send_sample(m);
        }
    }

    fn send_sample(&mut self, m: &mut MemoryBus) {
        let output = self.output(m);
        let filtered = self.filter.step(output);
        self.channel.send(filtered).unwrap();
    }

    fn output(&mut self, m: &mut MemoryBus) -> f32 {
        let p1 = m.apu.square1.output();
        let p2 = m.apu.square2.output();
        let t = m.apu.triangle.output();
        let n = m.apu.noise.output();
        let d = m.apu.dmc.output();
        // TODO: figure out if these bound checks are a bug somewhere else
        let pulse_out = self.pulse_table[(p1 + p2) as usize];
        let tnd_out = self.tnd_table[(3 * t + 2 * n + d) as usize];
        pulse_out + tnd_out
    }

    fn step_timer(&mut self, m: &mut MemoryBus, toggle: bool) {
        if toggle {
            m.apu.square1.step_timer();
            m.apu.square2.step_timer();
            m.apu.noise.step_timer();
            let address = m.apu.dmc.current_address;
            let read = m.cpu_read(address);
            if m.apu.dmc.step_timer(read) {
                m.cpu.add_stall(4);
            }
        }
        m.apu.triangle.step_timer();
    }

    fn step_framecounter(&mut self, m: &mut MemoryBus) {
        match m.apu.frame_period {
            4 => {
                self.frame_value = (self.frame_value + 1) % 4;
                match self.frame_value {
                    0 | 2 => m.apu.step_envelope(),
                    1 => {
                        m.apu.step_envelope();
                        m.apu.step_sweep();
                        m.apu.step_length();
                    }
                    3 => {
                        m.apu.step_envelope();
                        m.apu.step_sweep();
                        m.apu.step_length();
                        self.fire_irq(m);
                    }
                    // This can't happen because of the module 4
                    _ => {}
                }
            }
            5 => {
                self.frame_value = (self.frame_value + 1) % 5;
                match self.frame_value {
                    1 | 3 => m.apu.step_envelope(),
                    0 | 2 => {
                        m.apu.step_envelope();
                        m.apu.step_sweep();
                        m.apu.step_length();
                    }
                    // We don't want to do anything for 5
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn fire_irq(&self, m: &mut MemoryBus) {
        if m.apu.frame_irq {
            m.cpu.set_irq();
        }
    }
}

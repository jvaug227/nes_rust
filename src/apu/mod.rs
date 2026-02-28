struct Sequencer {
    sequence: u32,
    new_sequence: u32,
    timer: u16,
    reload: u16,
    output: u8,
}

impl Sequencer {
    fn new() -> Self {
        Self {
            sequence: 0,
            new_sequence: 0,
            timer: 0,
            reload: 0,
            output: 0,
        }
    }
    fn clock<T: Fn(&mut u32)>(&mut self, enabled: bool, seq_fn: T) -> u8 {
        if enabled {
            self.timer = if self.timer == 0 {
                seq_fn(&mut self.sequence);
                self.output = (self.sequence & 0x00000001) as u8;
                self.reload + 1
            } else {
                self.timer - 1
            };
        }

        self.output
    }
}

struct PulseOscillator {
    frequency: f64,
    duty_cycle: f64,
    amplitude: f64,
    harmonics: usize,
    prev_sample: f64,
}

impl PulseOscillator {
    fn new() -> Self {
        Self {
            frequency: 0.0,
            duty_cycle: 0.0,
            amplitude: 1.0,
            harmonics: 20,
            prev_sample: 0.0,
        }
    }
    fn sample(&mut self, enabled: bool, t: f64) -> f64 {
        if !enabled { return self.prev_sample; }
        const TWO_PI: f64 = 2.0 * std::f64::consts::PI;
        let mut a: f64 = 0.0;
        let mut b: f64 = 0.0;
        let p = self.duty_cycle * TWO_PI;
        let sin = |s: f64| {
            let j = (s * 0.15915).fract();
            20.785 * j * (j - 0.5) * (j - 1.0)
        };
        for n in 1..self.harmonics {
            // Every even period, a and b are the same wave.
            // let n = 2 * n - 1;
            let n_f: f64 = n as f64;
            let c = n_f * self.frequency * TWO_PI * t;
            a += -sin(c) / n_f;
            b += -sin(c - (p * n_f)) / n_f;
        }
        self.prev_sample = (2.0 * self.amplitude / std::f64::consts::PI) * (a - b);
        self.prev_sample
    }
    fn sample_weird(&self, t: f64) -> f64 {
        const PI: f64 = std::f64::consts::PI;
        let mut a: f64 = 0.0;
        let mut b: f64 = 0.0;
        let p = self.duty_cycle * PI;
        let sin = |s: f64| {
            let j = (s * 0.15915).fract();
            20.785 * j * (j - 0.5) * (j - 1.0)
            // s.sin()
        };
        for n in 1..self.harmonics {
            let n = 2 * n - 1;
            let n_f: f64 = n as f64;
            let c = n_f * self.frequency * PI * t;
            a += -sin(c) / n_f;
            b += -sin(c - (p * n_f)) / n_f;
        }
        (4.0 * self.amplitude / std::f64::consts::PI) * (a - b -self.duty_cycle*PI/2.0)
    }
}

pub struct ApuPinout {
    pub cpu_addr: u8,
    pub cpu_data: u8,
    pub cpu_rw: bool,
}

impl ApuPinout {
    pub fn new() -> Self {
        Self {
            cpu_addr: 0,
            cpu_data: 0,
            cpu_rw: true,
        }
    }
}

pub struct Apu {
    pulse1: PulseOscillator,
    square1: Sequencer,
    pulse2: PulseOscillator,
    square2: Sequencer,
    triangle: Sequencer,
    noise: Sequencer,

    square1_enable: bool,
    square2_enable: bool,
    triangle_enable: bool,
    noise_enable: bool,

    emulated_time: f64,
    clock_counter: u32,
    frame_clock_counter: u32,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: PulseOscillator::new(),
            square1: Sequencer::new(),
            pulse2: PulseOscillator::new(),
            square2: Sequencer::new(),
            triangle: Sequencer::new(),
            noise: Sequencer::new(),

            square1_enable: false,
            square2_enable: false,
            triangle_enable: false,
            noise_enable: false,

            emulated_time: 0.0,
            clock_counter: 0,
            frame_clock_counter: 0,
        }
    }
    pub fn clock(&mut self, pins: &mut ApuPinout) -> f64 {
        if !pins.cpu_rw {
            let data = pins.cpu_data;
            match pins.cpu_addr {
                // Pulse 1
                // DDLC NNNN
                0x00 => {
                    let duty = (data & 0b11000000) >> 6;
                    let _loop_evelope = (data & 0b00100000) >> 5;
                    let _constant_volume = (data & 0b00010000) >> 4;
                    let _envelope_period = data & 0b00001111;
                    const SEQ_DEFAULTS: [u32; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];
                    const PULSE_SEQ_DEFAULTS: [f64; 4] = [0.125, 0.25, 0.5, 0.75];
                    self.square1.sequence = SEQ_DEFAULTS[duty as usize];
                    self.pulse1.duty_cycle = PULSE_SEQ_DEFAULTS[duty as usize];
                }
                0x01 => {}
                0x02 => {
                    self.square1.reload = (self.square1.reload & 0xFF00) | data as u16;
                }
                0x03 => {
                    let timer_hi = data & 0b111;
                    let reload = (data & 0b11111000) >> 3;
                    self.square1.reload = (self.square1.reload & 0x00FF) | ((timer_hi as u16) << 8);
                    // println!("Setting P1 reload to {}", self.square1.reload);
                    self.square1.timer = self.square1.reload;
                }
                // Pulse 2
                0x04 => {
                    let duty = (data & 0b11000000) >> 6;
                    let _loop_evelope = (data & 0b00100000) >> 5;
                    let _constant_volume = (data & 0b00010000) >> 4;
                    let _envelope_period = data & 0b00001111;
                    const SEQ_DEFAULTS: [u32; 4] = [0b00000001, 0b00000011, 0b00001111, 0b11111100];
                    const PULSE_SEQ_DEFAULTS: [f64; 4] = [0.125, 0.25, 0.5, 0.75];
                    self.square2.sequence = SEQ_DEFAULTS[duty as usize];
                    self.pulse2.duty_cycle = PULSE_SEQ_DEFAULTS[duty as usize];
                }
                0x05 => {}
                0x06 => {
                    self.square2.reload = (self.square2.reload & 0xFF00) | data as u16;
                }
                0x07 => {
                    let timer_hi = data & 0b111;
                    let reload = (data & 0b11111000) >> 3;
                    self.square2.reload = (self.square2.reload & 0x00FF) | ((timer_hi as u16) << 8);
                    self.square2.timer = self.square2.reload;
                }
                // Triangle
                0x08 => {}
                0x09 => {} // Empty
                0x0A => {}
                0x0B => {}
                // Noise
                0x0C => {}
                0x0D => {} // Empty
                0x0E => {}
                0x0F => {}
                // DMC
                0x10 => {}
                0x11 => {}
                0x12 => {}
                0x13 => {}

                0x15 => {
                    self.square1_enable = (data & 0x01) > 0;
                    self.square2_enable = (data & 0x02) > 0;
                    self.triangle_enable = (data & 0x04) > 0;
                    self.noise_enable = (data & 0x08) > 0;
                }
                _ => {}
            }
        } else if pins.cpu_rw && pins.cpu_addr == 0x17 {
            // Put status on data bus
        }

        self.emulated_time += 1.0 / 1789773.0;

        let mut acc_sample = 0.0;
        // APU Timer clocks every 2 cpu cycles
        if self.clock_counter % 2 == 0 {
            self.frame_clock_counter += 1;

            if self.frame_clock_counter == 3729 {
                // quarter frame clock
            }

            if self.frame_clock_counter == 7457 {
                // quarter & half frame clock
            }

            if self.frame_clock_counter == 11186 {
                // quarter frame clock
            }

            if self.frame_clock_counter == 14916 {
                // quarter & half frame clock
                // reset frame_clock_counter
            }

            // if quarter frame ...
            //
            // if half frame ...

            self.square1.clock(self.square1_enable, |s| { *s = s.rotate_right(1); });
            let p1_sample = self.square1.output as f64;
            self.pulse1.frequency = 1789773.0 / (16.0 * (self.square1.reload + 1) as f64);
            let p1_sample = self.pulse1.sample(self.square1_enable, self.emulated_time);

            self.square2.clock(self.square2_enable, |s| { *s = s.rotate_right(1); });
            let p2_sample = self.square1.output as f64;
            self.pulse2.frequency = 1789773.0 / (16.0 * (self.square2.reload + 1) as f64);
            let p2_sample = self.pulse2.sample(self.square2_enable, self.emulated_time);

            self.triangle.clock(self.triangle_enable, |_s| {});
            let t_sample = 0.0;

            self.noise.clock(self.noise_enable, |_s| {});
            let n_sample = 0.0;

            let dmc_sample = 0.0;

            // let pulse_under = p1_sample*15.0 + p2_sample*15.0;
            // let pulse_out = if pulse_under == 0.0 { 0.0 } else { 95.88 / ((8128.0 / pulse_under) + 100.0) };
            // let tnd_under = (t_sample / 8227.0) + (n_sample / 12241.0) + dmc_sample / 22638.0;
            // let tnd_out = if tnd_under == 0.0 { 0.0 } else { 159.79 / (1.0/tnd_under + 100.0) };

            acc_sample += p1_sample + p2_sample;
        }

        self.clock_counter += 1;

        // let square1_sample = self.square1.output as f64;
        // let square2_sample = self.square2.output as f64;

        acc_sample
    }
}

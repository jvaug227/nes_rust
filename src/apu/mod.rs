struct Sequencer {
    timer: u16,
    reload: u16,
    output: u8,
}

impl Sequencer {
    fn new() -> Self {
        Self {
            timer: 0,
            reload: 0,
            output: 0,
        }
    }
    fn clock(&mut self) {
        self.timer = if self.timer == 0 {
            self.output = if self.output == 0 { 7 } else { self.output - 1 };
            self.reload
        } else {
            self.timer - 1
        };
    }
}

struct LengthCounter(u8);
impl LengthCounter {
    pub fn new() -> Self {
        Self(0)
    }
    pub fn clock(&mut self, enabled: bool, halt: bool) {
        // This could possibly be an array lookup
        // index = enabled | halt
        // a = [self.0, self.0, 1, 0]
        // saturating_sub(self.0, a[index])
        self.0 = if enabled {
            if halt {
                self.0
            } else {
                u8::saturating_sub(self.0, 1)
            }
        } else {
            0
        };
    }
    pub fn reload(&mut self, value: u8) {
        let linear = value & 0b1 > 0;
        let value = value >> 1;
        self.0 = match (linear, value) {
            (true, v) => match v {
                0b1111 => 30,
                0b1110 => 28,
                0b1101 => 26,
                0b1100 => 24,
                0b1011 => 22,
                0b1010 => 20,
                0b1001 => 18,
                0b1000 => 16,

                0b0111 => 14,
                0b0110 => 12,
                0b0101 => 10,
                0b0100 => 8,
                0b0011 => 6,
                0b0010 => 4,
                0b0001 => 2,
                0b0000 => 254,
                _ => 0,
            },
            (false, v) => match v {
                0b1111 => 32,
                0b1110 => 16,
                0b1101 => 72,
                0b1100 => 192,
                0b1011 => 96,
                0b1010 => 48,
                0b1001 => 24,
                0b1000 => 12,

                0b0111 => 26,
                0b0110 => 14,
                0b0101 => 60,
                0b0100 => 160,
                0b0011 => 80,
                0b0010 => 40,
                0b0001 => 20,
                0b0000 => 10,
                _ => 0,
            },
        };
    }
    pub fn value(&self) -> u8 {
        self.0
    }
}

struct Sweeper {}

struct PulseChannel {
    duty_cycle: u8,
    length_counter_toggle: bool,
    volume_envelope_toggle: bool,
    volume_envelope_period: u8,
    sweeper: Sweeper,
    sequencer: Sequencer,
    length_counter: LengthCounter,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            duty_cycle: 0,
            length_counter_toggle: false,
            volume_envelope_toggle: false,
            volume_envelope_period: 0,
            sweeper: Sweeper {},
            sequencer: Sequencer::new(),
            length_counter: LengthCounter::new(),
        }
    }

    pub fn clock(&mut self, enabled: bool) {
        if enabled {
            self.sequencer.clock();
        }
    }

    pub fn sample(&self) -> u8 {
        const SEQ_OUTPUTS: [[u8; 8]; 4] = [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 0, 0],
        ];
        // const PULSE_SEQ_DEFAULTS: [f64; 4] = [0.125, 0.25, 0.5, 0.75];
        let frequency = self.duty_cycle as usize;
        let index = self.sequencer.output as usize;
        let sample = SEQ_OUTPUTS[frequency][index];
        let muted = self.length_counter.value() == 0;
        if muted {
            0
        } else {
            sample
        }
    }

    pub fn write_0(
        &mut self,
        duty: u8,
        envelope_loop_length_counter_enable: u8,
        volume_envelope_toggle: u8,
        volume_envelope_period: u8,
    ) {
        self.duty_cycle = duty;
        self.length_counter_toggle = envelope_loop_length_counter_enable > 0;
        self.volume_envelope_toggle = volume_envelope_toggle > 0;
        self.volume_envelope_period = volume_envelope_period;

        // Changes duty cycle without resetting sequencer
    }

    pub fn write_1(&mut self, enabled: bool, negate: bool, period: u8, shift_count: u8) {}
    pub fn write_2(&mut self, timer_low: u8) {
        self.sequencer.reload = (self.sequencer.reload & 0xFF00) | timer_low as u16;
    }
    pub fn write_3(&mut self, length_counter_load: u8, timer_hi: u8) {
        self.sequencer.reload = (self.sequencer.reload & 0x00FF) | ((timer_hi as u16) << 8);
        self.sequencer.timer = self.sequencer.reload;
        self.sequencer.output = 0;

        self.length_counter.reload(length_counter_load);

        // Immediately restart sequencer, restart envelope. Period Divider is NOT reset
    }
}

// Some alternative implementation
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
        if !enabled {
            return self.prev_sample;
        }
        const TWO_PI: f64 = std::f64::consts::TAU;
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
    fn sample_weird(&mut self, enabled: bool, t: f64) -> f64 {
        if !enabled {
            return self.prev_sample;
        }
        const PI: f64 = std::f64::consts::PI;
        let mut a: f64 = 0.0;
        let mut b: f64 = 0.0;
        let p = self.duty_cycle * PI;
        let sin = |s: f64| {
            let j = (s * 0.15915).fract();
            20.785 * j * (j - 0.5) * (j - 1.0)
        };
        for n in 1..self.harmonics {
            let n = 2 * n - 1;
            let n_f: f64 = n as f64;
            let c = n_f * self.frequency * PI * t;
            a += -sin(c) / n_f;
            b += -sin(c - (p * n_f)) / n_f;
        }
        self.prev_sample = (2.0 * self.amplitude / PI) * (a - b - self.duty_cycle * PI / 2.0);
        self.prev_sample
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
    square1: PulseChannel,
    square2: PulseChannel,
    triangle: Sequencer,
    noise: Sequencer,

    square1_enable: bool,
    square2_enable: bool,
    triangle_enable: bool,
    noise_enable: bool,

    emulated_time: f64,
    clock_counter: u64,
    frame_counter: u32,
    frame_mode: bool,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            square1: PulseChannel::new(),
            square2: PulseChannel::new(),
            triangle: Sequencer::new(),
            noise: Sequencer::new(),

            square1_enable: false,
            square2_enable: false,
            triangle_enable: false,
            noise_enable: false,

            emulated_time: 0.0,
            clock_counter: 0,
            frame_counter: 0,
            frame_mode: false,
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
                    let loop_envelope = (data & 0b00100000) >> 5;
                    let volume_envelope_select = (data & 0b00010000) >> 4;
                    let volume_envelope_data = data & 0b00001111;
                    self.square1.write_0(
                        duty,
                        loop_envelope,
                        volume_envelope_select,
                        volume_envelope_data,
                    );
                }
                0x01 => {
                    let enabled = (0x80 & data) > 0;
                    let period = (0b01110000 & data) >> 4;
                    let negate = (0b00001000 & data) > 0;
                    let shift_count = 0b111 & data;
                    self.square1.write_1(enabled, negate, period, shift_count);
                }
                0x02 => {
                    self.square1.write_2(data);
                }
                0x03 => {
                    let timer_hi = data & 0b111;
                    let length_counter_load = (data & 0b11111000) >> 3;
                    self.square1.write_3(length_counter_load, timer_hi);
                }
                // Pulse 2
                0x04 => {
                    let duty = (data & 0b11000000) >> 6;
                    let loop_envelope = (data & 0b00100000) >> 5;
                    let volume_envelope_select = (data & 0b00010000) >> 4;
                    let volume_envelope_data = data & 0b00001111;
                    self.square2.write_0(
                        duty,
                        loop_envelope,
                        volume_envelope_select,
                        volume_envelope_data,
                    );
                }
                0x05 => {
                    let enabled = (0x80 & data) > 0;
                    let period = (0b01110000 & data) >> 4;
                    let negate = (0b00001000 & data) > 0;
                    let shift_count = 0b111 & data;
                    self.square2.write_1(enabled, negate, period, shift_count);
                }
                0x06 => {
                    self.square2.write_2(data);
                }
                0x07 => {
                    let timer_hi = data & 0b111;
                    let length_counter_load = (data & 0b11111000) >> 3;
                    self.square2.write_3(length_counter_load, timer_hi);
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
                    // TODO: Clear dmc irq (same for read)
                }
                0x17 => {
                    self.frame_mode = (data & 0x80) > 0;
                    let irq_inhibit_flag = (data & 0x40) > 0;
                    // TODO: Reset frame counter on the next odd cycle following the next even cycle
                    // (3~4 CPU cycles)
                }
                _ => {}
            }
        } else if pins.cpu_rw && pins.cpu_addr == 0x17 {
            // Put status on data bus
            let p1 = (self.square1_enable as u8);
            let p2 = (self.square2_enable as u8) << 1;
            let t = (self.triangle_enable as u8) << 2;
            let n = (self.noise_enable as u8) << 3;

            pins.cpu_data = p1 | p2 | t | n;

            // clears frame interrupt flag
            // does not clear dmc interrupt flag
        }

        self.emulated_time += 1.0 / 1789773.0;

        let mut acc_sample = 0.0;
        // APU Timer clocks every 2 cpu cycles
        // TODO: This even-cycle check was removed and seems to
        // fix the high-pitch whining. I believe I was double-checking even cycles and therefore only
        // generating 1/2 the samples needed.
        // if self.clock_counter % 2 == 1 {
            // self.clock_counter = 0;
            self.frame_counter += 1;

            let mode = false;

            // TODO: Seems like frame cycle 0 and MAX_CYCLE are the same cycle,
            // and resetting the frame cycle to 0 will trigger the quarter and half frames
            // This might require evaluating/resetting the frame counter early on
            let (clock_envelopes, clock_length_counters, pulls_irq) = match mode {
                false => (
                    matches!(self.frame_counter, 3728 | 7456 | 11185 | 14914),
                    matches!(self.frame_counter, 7456 | 14914),
                    matches!(self.frame_counter, 14914),
                ),
                true => (
                    matches!(self.frame_counter, 3728 | 7456 | 11185 | 18640),
                    matches!(self.frame_counter, 7456 | 18640),
                    matches!(self.frame_counter, 18640),
                ),
            };

            if clock_envelopes {
                // Clock envelopes and triangle linear counter
            }

            if clock_length_counters {
                // Clock linear counters and sweep units
                self.square1
                    .length_counter
                    .clock(self.square1_enable, self.square1.length_counter_toggle);
                self.square2
                    .length_counter
                    .clock(self.square2_enable, self.square2.length_counter_toggle);
            }

            if pulls_irq {
                self.frame_counter = 0;
                if mode {
                    // pull irq on cpu, do not reset until status is written
                }
            }
        // }

        self.square1.clock(self.square1_enable);
        let p1_sample = self.square1.sample() as f64;

        self.square2.clock(self.square2_enable);
        let p2_sample = self.square2.sample() as f64;

        // self.triangle.clock(self.triangle_enable, |_s| {});
        let t_sample = 0.0;

        // self.noise.clock(self.noise_enable, |_s| {});
        let n_sample = 0.0;

        let dmc_sample = 0.0;

        let pulse_under = p1_sample + p2_sample;
        let pulse_out = if pulse_under == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / pulse_under) + 100.0)
        };
        let tnd_under = (t_sample / 8227.0) + (n_sample / 12241.0) + dmc_sample / 22638.0;
        let tnd_out = if tnd_under == 0.0 {
            0.0
        } else {
            159.79 / (1.0 / tnd_under + 100.0)
        };

        acc_sample += pulse_out + tnd_out;

        self.clock_counter += 1;

        acc_sample
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

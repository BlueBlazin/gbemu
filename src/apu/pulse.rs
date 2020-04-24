use crate::apu::AudioRegisters;

const DUTY_TABLE: [[bool; 8]; 4] = [
    [false, false, false, false, false, false, false, true],
    [true, false, false, false, false, false, false, true],
    [true, false, false, false, false, true, true, true],
    [false, true, true, true, true, true, true, false],
];

pub enum SweepType {
    Increase,
    Decrease,
}

impl From<u8> for SweepType {
    fn from(value: u8) -> Self {
        if value > 0 {
            SweepType::Increase
        } else {
            SweepType::Decrease
        }
    }
}

pub struct Period {
    pub wave_duty_period: usize, // in M-cycles
    pub sweep_off: bool,
    pub sweep_time: usize, // in M-cycles
    pub sweep_type: SweepType,
    pub sweep_coeff: u8,
}

impl Default for Period {
    fn default() -> Self {
        Period {
            wave_duty_period: 0,
            sweep_off: true,
            sweep_time: 0,
            sweep_type: SweepType::Decrease,
            sweep_coeff: 0,
        }
    }
}

pub struct SignalInfo {
    pub wave_duty: usize,
    pub sound_length: usize, // in M-cycles
    pub stop_after_length: bool,
    pub stopped: bool,
}

impl SignalInfo {
    pub fn sound_length(value: u8) -> usize {
        ((4194300.0 * (64 - value) as f64) / 256.0) as usize
    }
}

impl Default for SignalInfo {
    fn default() -> Self {
        SignalInfo {
            wave_duty: 0,
            sound_length: 0,
            stop_after_length: true,
            stopped: true,
        }
    }
}

pub enum EnvelopeDirection {
    Increase,
    Decrease,
}

impl From<u8> for EnvelopeDirection {
    fn from(value: u8) -> Self {
        if (value & 0x8) != 0 {
            EnvelopeDirection::Increase
        } else {
            EnvelopeDirection::Decrease
        }
    }
}

pub struct Volume {
    pub volume: u8,
    pub envelope_direction: EnvelopeDirection,
    pub step_length: usize, // in M-cycles
}

impl Volume {
    pub fn step_length(value: u8) -> usize {
        (4194300.0 * (value as f64) / 64.0) as usize
    }
}

impl Default for Volume {
    fn default() -> Self {
        Volume {
            volume: 0,
            envelope_direction: EnvelopeDirection::Decrease,
            step_length: 0,
        }
    }
}

pub struct PulseWave {
    pub output_volume: u8,
    clock: usize,
    volume_clock: usize,
    sweep_clock: usize,
    frequency: usize,
    freq_sweep: bool,
    period: Period,
    signal: SignalInfo,
    volume: Volume,
    registers: AudioRegisters,
    duty_idx: usize,
}

impl PulseWave {
    pub fn new(freq_sweep: bool) -> Self {
        PulseWave {
            output_volume: 0,
            clock: 0,
            volume_clock: 0,
            sweep_clock: 0,
            frequency: 0,
            freq_sweep,
            period: Period::default(),
            signal: SignalInfo::default(),
            volume: Volume::default(),
            registers: AudioRegisters::default(),
            duty_idx: 0,
        }
    }

    pub fn tick(&mut self, cycles: usize) {
        // update volume
        self.update_volume(cycles);
        // update period
        if self.freq_sweep {
            self.update_sweep(cycles);
        }
        // depending on where in the period we currently are, use the wave duty info to set volume on/off
        self.update_duty(cycles);
    }

    fn update_volume(&mut self, cycles: usize) {
        if self.volume.step_length == 0 {
            return;
        }
        self.volume_clock += cycles;
        while self.volume.step_length > 0 && self.volume_clock >= self.volume.step_length {
            self.volume_clock = self.volume_clock - self.volume.step_length;

            match self.volume.envelope_direction {
                EnvelopeDirection::Increase if self.volume.volume < 0xF => self.volume.volume += 1,
                EnvelopeDirection::Decrease if self.volume.volume > 0 => self.volume.volume -= 1,
                _ => (),
            }
        }
    }

    fn update_sweep(&mut self, cycles: usize) {
        self.sweep_clock += cycles;
        // TODO: change period based on sweep
    }

    fn update_duty(&mut self, cycles: usize) {
        self.clock += cycles;
        while self.period.wave_duty_period > 0 && self.clock >= self.period.wave_duty_period {
            self.clock = self.clock - self.period.wave_duty_period;
            self.duty_idx = (self.duty_idx + 1) % 8;
        }

        self.output_volume = if DUTY_TABLE[self.signal.wave_duty][self.duty_idx] {
            self.volume.volume
        } else {
            0
        };
    }

    pub fn get_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.registers.nrx0,
            0xFF11 | 0xFF21 => self.registers.nrx1,
            0xFF12 | 0xFF22 => self.registers.nrx2,
            0xFF13 | 0xFF23 => self.registers.nrx3,
            0xFF14 | 0xFF24 => self.registers.nrx4,
            _ => unreachable!(),
        }
    }

    pub fn set_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF10 => {
                self.registers.nrx0 = value;
                self.period.sweep_off = (value & 0x70) >> 4 == 0;
                self.period.sweep_time = match (value & 0x70) >> 4 {
                    0x0 => 0,
                    0x1 => 32715,
                    0x2 => 65431,
                    0x3 => 98147,
                    0x4 => 131282,
                    0x5 => 163997,
                    0x6 => 196712,
                    0x7 => 229428,
                    _ => unreachable!(),
                };
                self.period.sweep_type = SweepType::from(value & 0x08);
                self.period.sweep_coeff = value & 0x07;
            }
            0xFF11 | 0xFF21 => {
                self.registers.nrx1 = value;
                self.signal.wave_duty = ((value & 0xC0) >> 6) as usize;
                self.signal.sound_length = SignalInfo::sound_length(value & 0x3F);
            }
            0xFF12 | 0xFF22 => {
                self.registers.nrx2 = value;
                self.volume.volume = (value & 0xF0) >> 4;
                self.volume.envelope_direction = EnvelopeDirection::from(value);
                self.volume.step_length = Volume::step_length(value & 0x7);
            }
            0xFF13 | 0xFF23 => {
                self.registers.nrx3 = value;
                self.frequency = (self.frequency & 0x700) | value as usize;
                self.period.wave_duty_period = (2048 - self.frequency) * 4;
            }
            0xFF14 | 0xFF24 => {
                self.registers.nrx4 = value;
                self.frequency = (self.frequency & 0xFF) | (value & 0x7) as usize;
                self.period.wave_duty_period = (2048 - self.frequency) * 4;
                self.signal.stop_after_length = (value & 0x40) != 0;
                if (value & 0x80) != 0 {
                    self.restart();
                }
            }
            _ => unreachable!(),
        }
    }

    fn restart(&mut self) {
        unimplemented!()
    }
}

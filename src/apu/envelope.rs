pub enum EnvelopeDirection {
    Increase,
    Decrease,
}

pub struct VolumeEnvelope {
    pub volume: u8,
    pub direction: EnvelopeDirection,
    pub clock: usize,
    pub period: usize,
}

impl Default for VolumeEnvelope {
    fn default() -> Self {
        Self {
            volume: 0,
            direction: EnvelopeDirection::Decrease,
            clock: 0,
            period: 0,
        }
    }
}

impl VolumeEnvelope {
    pub fn set_direction(&mut self, add: bool) {
        self.direction = if add {
            EnvelopeDirection::Increase
        } else {
            EnvelopeDirection::Decrease
        };
    }
}

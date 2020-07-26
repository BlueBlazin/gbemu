pub enum Event {
    VBlank,
    AudioBufferFull,
    MaxCycles,
}

impl From<Event> for f64 {
    fn from(event: Event) -> f64 {
        match event {
            Event::VBlank => 0.0,
            Event::AudioBufferFull => 1.0,
            Event::MaxCycles => 2.0,
        }
    }
}

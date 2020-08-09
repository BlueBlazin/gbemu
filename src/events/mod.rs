pub enum Event {
    VBlank,
    AudioBufferFull(Vec<f32>, Vec<f32>),
    MaxCycles,
}

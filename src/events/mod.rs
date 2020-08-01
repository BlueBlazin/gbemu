pub enum Event {
    VBlank,
    AudioBufferFull(Vec<f32>, Vec<f32>),
    MaxCycles,
}

// impl From<Event> for f64 {
//     fn from(event: Event) -> f64 {
//         match event {
//             Event::VBlank => 0.0,
//             Event::AudioBufferFull(_, _) => 1.0,
//             Event::MaxCycles => 2.0,
//         }
//     }
// }

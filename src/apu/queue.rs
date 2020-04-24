use std::collections::VecDeque;
use std::mem;

pub const BUFFER_SIZE: usize = 735;

pub struct AudioQueue {
    pub queue: VecDeque<Vec<f32>>,
    current: Vec<f32>,
}

impl AudioQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            current: Vec::new(),
        }
    }

    pub fn push(&mut self, value: f32) {
        if self.current.len() < BUFFER_SIZE {
            self.current.push(value);
        } else {
            let buffer = mem::replace(&mut self.current, Vec::new());
            self.queue.push_back(buffer);
        }
    }

    pub fn dequeue(&mut self) -> Option<Vec<f32>> {
        self.queue.pop_front()
    }
}

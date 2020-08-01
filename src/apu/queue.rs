use std::collections::VecDeque;
use std::mem;

// pub const BUFFER_SIZE: usize = 735;
pub const BUFFER_SIZE: usize = 736;
// pub const BUFFER_SIZE: usize = 4096;

const GAIN: f32 = 0.5;

pub struct AudioQueue {
    pub queue_left: VecDeque<Vec<f32>>,
    pub queue_right: VecDeque<Vec<f32>>,
    current_left: Vec<f32>,
    current_right: Vec<f32>,
}

impl AudioQueue {
    pub fn new() -> Self {
        Self {
            queue_left: VecDeque::new(),
            queue_right: VecDeque::new(),
            current_left: Vec::new(),
            current_right: Vec::new(),
        }
    }

    pub fn push(&mut self, left: f32, right: f32) {
        if self.current_left.len() < BUFFER_SIZE {
            self.current_left.push(left * GAIN);
            self.current_right.push(right * GAIN);
        } else {
            let buffer_left = mem::replace(&mut self.current_left, Vec::new());
            let buffer_right = mem::replace(&mut self.current_right, Vec::new());
            self.queue_left.push_back(buffer_left);
            self.queue_right.push_back(buffer_right);
        }
    }

    pub fn dequeue(&mut self) -> (Option<Vec<f32>>, Option<Vec<f32>>) {
        (self.queue_left.pop_front(), self.queue_right.pop_front())
    }
}

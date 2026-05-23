#![allow(dead_code)]

const MAX_ENTRIES: usize = 200;

pub trait Logger {
    fn log(&mut self, msg: &str);
}

pub struct Log {
    pub entries: Vec<String>,
}

impl Log {
    pub fn new() -> Self {
        Log {
            entries: Vec::with_capacity(MAX_ENTRIES),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Logger for Log {
    fn log(&mut self, msg: &str) {
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push(msg.to_string());
    }
}

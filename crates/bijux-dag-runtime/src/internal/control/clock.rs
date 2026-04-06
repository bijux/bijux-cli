use std::time::{SystemTime, UNIX_EPOCH};

pub trait Clock: Send + Sync {
    fn now_unix_ms(&self) -> u128;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_ms(&self) -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FixedClock {
    now_ms: u128,
}

#[allow(dead_code)]
impl FixedClock {
    pub fn new(now_ms: u128) -> Self {
        Self { now_ms }
    }

    pub fn advance(&mut self, delta_ms: u128) {
        self.now_ms += delta_ms;
    }
}

impl Clock for FixedClock {
    fn now_unix_ms(&self) -> u128 {
        self.now_ms
    }
}

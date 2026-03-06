use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct LocalExecutor {
    capacity: usize,
    inflight: usize,
    queue: VecDeque<String>,
}

impl LocalExecutor {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            inflight: 0,
            queue: VecDeque::new(),
        }
    }

    pub fn submit(&mut self, node_id: String) -> Result<(), String> {
        if self.queue.len() + self.inflight >= self.capacity {
            return Err("executor queue is full".to_string());
        }
        self.queue.push_back(node_id);
        Ok(())
    }

    pub fn start_next(&mut self) -> Option<String> {
        let item = self.queue.pop_front()?;
        self.inflight += 1;
        Some(item)
    }

    pub fn mark_finished(&mut self) {
        self.inflight = self.inflight.saturating_sub(1);
    }

    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }
}

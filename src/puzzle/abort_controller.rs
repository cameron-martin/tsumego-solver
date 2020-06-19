use std::time::{Duration, Instant};

pub trait AbortController {
    fn should_abort(&self) -> bool;
}

pub struct NoAbortController;

impl AbortController for NoAbortController {
    fn should_abort(&self) -> bool {
        false
    }
}

pub struct TimeoutAbortController {
    timeout_at: Instant,
}

impl AbortController for TimeoutAbortController {
    fn should_abort(&self) -> bool {
        Instant::now() >= self.timeout_at
    }
}

impl TimeoutAbortController {
    pub fn duration(duration: Duration) -> Self {
        TimeoutAbortController {
            timeout_at: Instant::now() + duration,
        }
    }
}

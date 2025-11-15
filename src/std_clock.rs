use embedded_timers::clock::Clock;

/// 自己实现一个标准时钟
pub struct StdClock {}

impl StdClock {
    pub fn new() -> Self {
        Self {}
    }
}

impl Clock for StdClock {
    type Instant = std::time::Instant;

    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }

    fn elapsed(&self, instant: Self::Instant) -> std::time::Duration {
        instant.elapsed()
    }
}

// 必须 Send + Sync 才能跨线程
unsafe impl Send for StdClock {}
unsafe impl Sync for StdClock {}

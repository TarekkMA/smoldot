use std::ops::{Add, Sub};
use std::time::Duration;
use nix::time::{clock_gettime, ClockId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    inner: Duration
}

impl Instant {
    pub fn now() -> Self {
        Instant {
            inner: clock_gettime(ClockId::CLOCK_MONOTONIC).unwrap().into()
        }
    }
}

impl Add<Duration> for Instant {
    type Output = Self;

    fn add(self, other: Duration) -> Self {
        Instant {
            inner: self.inner + other
        }
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;

    fn sub(self, other: Instant) -> Duration {
        self.inner - other.inner
    }
}
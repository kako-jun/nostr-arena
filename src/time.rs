//! Cross-platform time helpers for native and WASM targets

pub use std::time::Duration;

/// Sleep for a duration.
///
/// On native platforms, uses `tokio::time::sleep`.
/// On WASM, uses `gloo_timers::future::sleep`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: Duration) {
    gloo_timers::future::sleep(duration).await;
}

/// Create an interval that ticks at the given duration.
///
/// On native platforms, uses `tokio::time::interval`.
/// On WASM, uses a custom implementation with gloo_timers.
#[cfg(not(target_arch = "wasm32"))]
pub fn interval(period: Duration) -> tokio::time::Interval {
    tokio::time::interval(period)
}

/// WASM-compatible interval
#[cfg(target_arch = "wasm32")]
pub struct Interval {
    period: Duration,
}

#[cfg(target_arch = "wasm32")]
impl Interval {
    pub async fn tick(&mut self) {
        gloo_timers::future::sleep(self.period).await;
    }
}

#[cfg(target_arch = "wasm32")]
pub fn interval(period: Duration) -> Interval {
    Interval { period }
}

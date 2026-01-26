//! Cross-platform spawn helper for native and WASM targets

use std::future::Future;

/// Spawn a future to run in the background.
///
/// On native platforms, uses `tokio::spawn` (requires Send).
/// On WASM, uses `wasm_bindgen_futures::spawn_local` (no Send required).
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(future);
}

#[cfg(target_arch = "wasm32")]
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
#[path = "desktop.rs"]
pub mod platform;
#[cfg(target_arch = "wasm32")]
#[path = "web.rs"]
pub mod platform;

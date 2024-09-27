#[cfg(target_arch = "wasm32")]
mod wasm {
    use js_sys::Date;
    pub fn get_time() -> u64 {
        let ts = Date::now() as u64;
        return ts;
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::time;
    pub fn get_time() -> u64 {
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

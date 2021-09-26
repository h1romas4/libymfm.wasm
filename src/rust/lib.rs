// license:BSD-3-Clause
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod sound;
pub mod driver;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[allow(unused_macros)]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (#[allow(unused_unsafe)] unsafe { crate::log(&format_args!($($t)*).to_string()) })
}

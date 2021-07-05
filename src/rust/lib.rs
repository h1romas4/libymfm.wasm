use wasm_bindgen::prelude::*;

#[link(name = "ymfm")]
extern {
    fn ffi_test(count: i32) -> i32;
}

#[wasm_bindgen]
pub fn test() {
    unsafe { ffi_test(10); }
}

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    driver::{self, VgmPlay},
    sound::SoundSlot,
};

///
/// for WebAssembly Instance on thread-local
///
type VgmPlayInstances = Rc<RefCell<Vec<VgmPlay>>>;
std::thread_local!(static VGM_PLAY: VgmPlayInstances = {
    Rc::new(RefCell::new(Vec::new()))
});

///
/// Get thread local value Utility
///
fn get_vgm_vec() -> VgmPlayInstances {
    VGM_PLAY.with(|rc| rc.clone())
}

#[no_mangle]
pub extern "C" fn create_vgm_instance(
    vgm_index_id: u32,
    output_sampling_rate: u32,
    output_sample_chunk_size: u32,
    vgm_file_size: u32,
) {
    get_vgm_vec().borrow_mut().insert(
        vgm_index_id as usize,
        VgmPlay::new(
            SoundSlot::new(
                driver::VGM_TICK_RATE,
                output_sampling_rate,
                output_sample_chunk_size as usize,
            ),
            vgm_file_size as usize,
        ),
    );
}

#[no_mangle]
pub extern "C" fn wasi_interface_test() -> u32 {
    println!("Hello, wasmer-python!");
    1
}

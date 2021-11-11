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
type VgmPlayInsts = Rc<RefCell<Vec<VgmPlay>>>;
std::thread_local!(static VGM_PLAY: VgmPlayInsts = {
    Rc::new(RefCell::new(Vec::new()))
});

///
/// Get thread local value Utility
///
fn get_vgm_vec() -> VgmPlayInsts {
    VGM_PLAY.with(|rc| rc.clone())
}

#[no_mangle]
pub extern "C" fn vgm_create(
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
pub extern "C" fn vgm_get_seq_data_ref(vgm_index_id: u32) -> *mut u8 {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .get_vgmfile_ref()
}

#[no_mangle]
pub extern "C" fn vgm_get_sampling_l_ref(vgm_index_id: u32) -> *const f32 {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .get_sampling_l_ref()
}

#[no_mangle]
pub extern "C" fn vgm_get_sampling_r_ref(vgm_index_id: u32) -> *const f32 {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .get_sampling_r_ref()
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn vgm_get_seq_header(vgm_index_id: u32) {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .get_vgm_header_json();
}

#[no_mangle]
pub extern "C" fn vgm_init(vgm_index_id: u32) -> bool {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .init()
        .is_ok()
}

#[no_mangle]
pub extern "C" fn vgm_play(vgm_index_id: u32) -> usize {
    get_vgm_vec()
        .borrow_mut()
        .get_mut(vgm_index_id as usize)
        .unwrap()
        .play(true)
}

#[no_mangle]
pub extern "C" fn vgm_drop(vgm_index_id: u32) {
    get_vgm_vec()
        .borrow_mut().remove(vgm_index_id as usize);
}

#[no_mangle]
pub extern "C" fn wasi_interface_test() -> u32 {
    println!("Hello, wasmer-python!");
    1
}

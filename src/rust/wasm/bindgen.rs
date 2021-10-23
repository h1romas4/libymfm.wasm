// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use wasm_bindgen::prelude::*;

use crate::{driver::VgmPlay, driver::VGM_TICK_RATE, sound::SoundSlot};

#[wasm_bindgen]
pub struct WgmPlay {
    vgmplay: VgmPlay,
}

///
/// VgmPlay WebAssembly Interface
///
#[wasm_bindgen]
impl WgmPlay {
    ///
    /// constructor
    ///
    #[wasm_bindgen(constructor)]
    pub fn from(
        output_sampling_rate: u32,
        output_sample_chunk_size: usize,
        data_length: usize,
    ) -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        WgmPlay {
            vgmplay: VgmPlay::new(
                SoundSlot::new(
                    VGM_TICK_RATE,
                    output_sampling_rate,
                    output_sample_chunk_size,
                ),
                data_length,
            ),
        }
    }

    ///
    /// Return vgmdata buffer referance.
    ///
    pub fn get_seq_data_ref(&mut self) -> *mut u8 {
        self.vgmplay.get_vgmfile_ref()
    }

    ///
    /// Return sampling_l buffer referance.
    ///
    pub fn get_sampling_l_ref(&self) -> *const f32 {
        self.vgmplay.get_sampling_l_ref()
    }

    ///
    /// Return sampling_r buffer referance.
    ///
    pub fn get_sampling_r_ref(&self) -> *const f32 {
        self.vgmplay.get_sampling_r_ref()
    }

    ///
    /// Get the JSON parsed from the header of the VGM file.
    ///
    pub fn get_seq_header(&self) -> String {
        self.vgmplay.get_vgm_header_json()
    }

    ///
    /// Get the JSON parsed GD3 of the VGM file.
    ///
    pub fn get_seq_gd3(&self) -> String {
        self.vgmplay.get_vgm_gd3_json()
    }

    ///
    /// Initialize sound driver.
    ///
    pub fn init(&mut self) -> bool {
        self.vgmplay.init().is_ok()
    }

    ///
    /// Continue playing until output_sample_chunk_size is satisfied.
    ///
    /// The number of times the song has been looped will be returned.
    /// In the case of an infinite loop, the std::usize::MAX value is always returned.
    ///
    pub fn play(&mut self) -> usize {
        self.vgmplay.play(true)
    }
}

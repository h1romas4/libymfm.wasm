// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use wasm_bindgen::prelude::*;

use crate::{driver::VgmPlay, driver::VGM_TICK_RATE, sound::SoundSlot};

#[wasm_bindgen]
pub struct WgmPlay {
    output_sampling_rate: u32,
    output_sample_chunk_size: usize,
    vgm_file: Vec<u8>,
    vgmplay: Option<VgmPlay>,
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
            output_sampling_rate,
            output_sample_chunk_size,
            vgm_file: vec![0; data_length],
            vgmplay: None,
        }
    }

    ///
    /// Return vgmdata buffer referance.
    ///
    pub fn get_seq_data_ref(&mut self) -> *mut u8 {
        self.vgm_file.as_mut_ptr()
    }

    ///
    /// Return sampling_l buffer referance.
    ///
    pub fn get_sampling_l_ref(&self) -> *const f32 {
        if let Some(vgmplay) = self.vgmplay.as_ref() {
            return vgmplay.get_sampling_l_ref();
        }
        panic!("vgmplay instance not exsist");
    }

    ///
    /// Return sampling_r buffer referance.
    ///
    pub fn get_sampling_r_ref(&self) -> *const f32 {
        if let Some(vgmplay) = self.vgmplay.as_ref() {
            return vgmplay.get_sampling_r_ref();
        }
        panic!("vgmplay instance not exsist");
    }

    ///
    /// Get the JSON parsed from the header of the VGM file.
    ///
    pub fn get_seq_header(&self) -> String {
        if let Some(vgmplay) = self.vgmplay.as_ref() {
            return vgmplay.get_vgm_header_json();
        }
        panic!("vgmplay instance not exsist");
    }

    ///
    /// Get the JSON parsed GD3 of the VGM file.
    ///
    pub fn get_seq_gd3(&self) -> String {
        if let Some(vgmplay) = self.vgmplay.as_ref() {
            return vgmplay.get_vgm_gd3_json();
        }
        panic!("vgmplay instance not exsist");
    }

    ///
    /// Initialize sound driver.
    ///
    pub fn init(&mut self) -> bool {
        let vgmplay = VgmPlay::new(
            SoundSlot::new(
                VGM_TICK_RATE,
                self.output_sampling_rate,
                self.output_sample_chunk_size,
            ),
            self.vgm_file.as_slice()
        );
        if vgmplay.is_err() {
            return false;
        }
        self.vgmplay = Some(vgmplay.unwrap());
        true
    }

    ///
    /// Continue playing until output_sample_chunk_size is satisfied.
    ///
    /// The number of times the song has been looped will be returned.
    /// In the case of an infinite loop, the std::usize::MAX value is always returned.
    ///
    pub fn play(&mut self) -> usize {
        if let Some(vgmplay) = self.vgmplay.as_mut() {
            return vgmplay.play(true)
        }
        panic!("vgmplay instance not exsist");
    }
}

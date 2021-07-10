use wasm_bindgen::prelude::*;

use crate::driver::VgmPlay;

#[wasm_bindgen]
pub struct WgmPlay {
    vgmplay: VgmPlay
}

#[wasm_bindgen]
impl WgmPlay {
    ///
    /// constructor
    ///
    #[wasm_bindgen(constructor)]
    pub fn from(sample_rate: u32, max_sampling_size: usize, data_length: usize) -> Self {
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        WgmPlay {
            vgmplay: VgmPlay::new(sample_rate, max_sampling_size, data_length)
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
    /// get_header
    ///
    pub fn get_seq_header(&self) -> String {
        self.vgmplay.get_vgm_header_json()
    }

    ///
    /// get_gd3
    ///
    pub fn get_seq_gd3(&self) -> String {
        self.vgmplay.get_vgm_gd3_json()
    }

    ///
    /// Initialize sound driver.
    ///
    /// # Arguments
    /// sample_rate - WebAudio sampling rate
    ///
    pub fn init(&mut self) -> bool {
        self.vgmplay.init().is_ok()
    }

    ///
    /// play
    ///
    pub fn play(&mut self) -> usize {
        self.vgmplay.play(true)
    }
}

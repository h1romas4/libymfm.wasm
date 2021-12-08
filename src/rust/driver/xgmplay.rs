// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka

use super::{
    gd3meta::Gd3,
    meta::Jsonlize,
    xgmmeta::{self, VDPMode, XgmHeader},
};
use crate::sound::{DataStreamMode, SoundChipType, SoundSlot};
use flate2::read::GzDecoder;
use std::io::Read;

pub const XGM_NTSC_TICK_RATE: u32 = 60;
const XGM_PCM_SAMPLING_RATE: u32 = 14000;
const XGM_PCM_MAX_CHANNEL: u32 = 4;
const MASTER_CLOCK_NTSC: u32 = 53693175;
const MASTER_CLOCK_PAL: u32 = 53203424;

///
/// XGM Driver
///
pub struct XgmPlay {
    sound_slot: SoundSlot,
    xgm_pos: usize,
    xgm_loop: usize,
    xgm_loop_offset: usize,
    xgm_loop_count: usize,
    xgm_end: bool,
    xgm_data: Vec<u8>,
    xgm_header: Option<XgmHeader>,
    xgm_gd3: Option<Gd3>,
    remain_tick_count: usize,
}

#[allow(dead_code)]
impl XgmPlay {
    ///
    /// Create sound driver.
    ///
    pub fn new(sound_slot: SoundSlot, xgm_file: &[u8]) -> Result<Self, &'static str> {
        let mut xgmplay = XgmPlay {
            sound_slot,
            xgm_pos: 0,
            xgm_loop: 0,
            xgm_loop_offset: 0,
            xgm_loop_count: 0,
            xgm_end: false,
            xgm_data: Vec::new(),
            xgm_header: None,
            xgm_gd3: None,
            remain_tick_count: 0,
        };
        // clone vgm_file and soundchip init
        xgmplay.init(xgm_file)?;

        Ok(xgmplay)
    }

    ///
    /// Return sampling_l buffer referance.
    ///
    pub fn get_sampling_l_ref(&self) -> *const f32 {
        self.sound_slot.get_output_sampling_l_ref()
    }

    ///
    /// Return sampling buffer referance.
    ///
    pub fn get_sampling_r_ref(&self) -> *const f32 {
        self.sound_slot.get_output_sampling_r_ref()
    }

    ///
    /// Return s16le sampling buffer referance.
    ///
    pub fn get_output_sampling_s16le_ref(&mut self) -> *const i16 {
        self.sound_slot.get_output_sampling_s16le_ref()
    }

    ///
    /// Get VGM meta.
    ///
    pub fn get_vgm_meta(&self) -> (&XgmHeader, &Gd3) {
        (
            self.xgm_header.as_ref().unwrap(/* There always is */),
            self.xgm_gd3.as_ref().unwrap(/* There always is */),
        )
    }

    ///
    /// Get VGM header JSON.
    ///
    pub fn get_xgm_header_json(&self) -> String {
        self.xgm_header.as_ref().unwrap(/* There always is */).get_json()
    }

    ///
    /// Extract vgz and initialize sound driver.
    ///
    fn init(&mut self, xgm_file: &[u8]) -> Result<(), &'static str> {
        // try vgz extract to vgm_data
        self.extract(xgm_file);

        // parse vgm header
        match xgmmeta::parse_xgm_meta(&self.xgm_data) {
            Ok((header, gd3)) => {
                self.xgm_header = Some(header);
                self.xgm_gd3 = Some(gd3);
            }
            Err(message) => return Err(message),
        };

        // set sound chip clock
        let header = self.xgm_header.as_ref().unwrap();
        let (clock_ym2612, clock_sn76489) = match header.vdp_mode {
            VDPMode::NTSC => (MASTER_CLOCK_NTSC / 7, MASTER_CLOCK_NTSC / 15),
            VDPMode::PAL => (MASTER_CLOCK_PAL / 7, MASTER_CLOCK_NTSC / 15),
        };
        // change external tick rate
        if header.vdp_mode == VDPMode::PAL {
            self.sound_slot
                .change_external_tick_rate(VDPMode::PAL as u32);
        }

        // add sound chip
        self.sound_slot
            .add_sound_device(SoundChipType::YM2612, 1, clock_ym2612);
        self.sound_slot
            .add_sound_device(SoundChipType::SEGAPSG, 1, clock_sn76489);

        // set up YM2612 data stream
        // 4 PCM channels (8 bits signed at 14 Khz)
        self.sound_slot.set_data_stream_mode(
            SoundChipType::YM2612,
            0,
            DataStreamMode::PCMMerge,
        );
        self.sound_slot.set_data_stream_priority_limit(
            SoundChipType::YM2612,
            0,
            XGM_PCM_MAX_CHANNEL,
        );
        // parse sample table
        for (xgm_sample_id, (address, size)) in header.sample_id_table.iter().enumerate() {
            let start_address: usize =
                *address as usize * 256 + xgmmeta::XGM_SAMPLE_DATA_BLOC_ADDRESS;
            let end_address: usize = *size as usize * 256 + start_address;
            // create data stream into sound device
            self.sound_slot
                .add_data_block(xgm_sample_id, &self.xgm_data[start_address..end_address]);
            self.sound_slot.add_data_stream(
                SoundChipType::YM2612,
                0,
                xgm_sample_id,
                0,
                0x2a,
            );
            self.sound_slot.set_data_stream_frequency(
                SoundChipType::YM2612,
                0,
                xgm_sample_id,
                XGM_PCM_SAMPLING_RATE,
            );
            self.sound_slot.attach_data_block_to_stream(
                SoundChipType::YM2612,
                0,
                xgm_sample_id,
                xgm_sample_id,
            );
        }

        Ok(())
    }

    fn extract(&mut self, xgm_file: &[u8]) {
        let mut d = GzDecoder::new(xgm_file);
        if d.read_to_end(&mut self.xgm_data).is_err() {
            self.xgm_data = xgm_file.to_vec();
        }
    }

    fn get_xgm_u8(&mut self) -> u8 {
        let ret = self.xgm_data[self.xgm_pos];
        self.xgm_pos += 1;
        ret
    }

    fn get_xgm_u16(&mut self) -> u16 {
        u16::from(self.get_xgm_u8()) + (u16::from(self.get_xgm_u8()) << 8)
    }

    fn get_xgm_u32(&mut self) -> u32 {
        u32::from(self.get_xgm_u8())
            + (u32::from(self.get_xgm_u8()) << 8)
            + (u32::from(self.get_xgm_u8()) << 16)
            + (u32::from(self.get_xgm_u8()) << 24)
    }
}

#[cfg(test)]
mod tests {
    use crate::sound::SoundSlot;

    use super::XgmPlay;
    use std::fs::File;
    use std::io::{Read, Write};

    const MAX_SAMPLE_SIZE: usize = 2048;

    #[test]
    fn xgm_1() {
        play("./docs/vgm/sor2.xgm")
    }

    fn play(filepath: &str) {
        println!("Play start! {}", filepath);

        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();

        // read vgm
        let mut xgmplay = XgmPlay::new(
            SoundSlot::new(/* XGM NTSC */ 60, 44100, MAX_SAMPLE_SIZE),
            &buffer,
        )
        .unwrap();

        let mut pcm = File::create("output.pcm").expect("file open error.");
        // play
        // ffplay -f f32le -ar 96000 -ac 2 output.pcm
        // ffmpeg -f f32le -ar 96000 -ac 2 -i output.pcm output-96000.wav
        // #[allow(clippy::absurd_extreme_comparisons)]
        // while xgmplay.play(false) <= 0 {
        //     for i in 0..MAX_SAMPLE_SIZE {
        //         let sampling_l = xgmplay.get_sampling_l_ref();
        //         let sampling_r = xgmplay.get_sampling_r_ref();
        //         unsafe {
        //             let slice_l = std::slice::from_raw_parts(sampling_l.add(i) as *const u8, 4);
        //             let slice_r = std::slice::from_raw_parts(sampling_r.add(i) as *const u8, 4);
        //             pcm.write_all(slice_l).expect("stdout error");
        //             pcm.write_all(slice_r).expect("stdout error");
        //         }
        //     }
        // }
        println!("Play end! {} (xgm instance drop)", filepath);
    }
}
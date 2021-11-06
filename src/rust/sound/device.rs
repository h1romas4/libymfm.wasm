// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::collections::HashMap;
use super::{RomIndex, data_stream::{self, DataBlock, DataStream}, sound_chip::SoundChip, stream::{SoundStream, Tick}};

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: Box<dyn SoundStream>,
    data_stream: HashMap<usize, DataStream>,
}

impl SoundDevice {
    pub fn new(sound_chip: Box<dyn SoundChip>, sound_stream: Box<dyn SoundStream>) -> Self {
        SoundDevice {
            sound_chip,
            sound_stream,
            data_stream: HashMap::new(),
        }
    }

    ///
    /// Generates a waveform for one sample according to
    /// the output sampling rate of the sound stream.
    ///
    pub fn generate(
        &mut self,
        sound_chip_index: usize,
        data_block: &HashMap<usize, DataBlock>,
    ) -> (f32, f32) {
        let mut is_tick;
        #[allow(clippy::blocks_in_if_conditions)]
        while {
            is_tick = self.sound_stream.is_tick();
            is_tick != Tick::No
        } {
            self.sound_chip
                .tick(sound_chip_index, &mut *self.sound_stream);
            if is_tick != Tick::One {
                continue;
            }
            break;
        }
        self.sound_stream.drain()
    }

    ///
    /// Write command to sound chip.
    ///
    pub fn write(&mut self, sound_chip_index: usize, port: u32, data: u32) {
        self.sound_chip
            .write(sound_chip_index, port, data, &mut *self.sound_stream);
    }

    ///
    /// Notify add rom to sound chip.
    ///
    pub fn notify_add_rom(&mut self, rom_index: RomIndex, index_no: usize) {
        self.sound_chip.notify_add_rom(rom_index, index_no);
    }

    ///
    /// Add data stream
    ///
    pub fn add_data_stream(&mut self, data_stream_id: usize, data_stream: DataStream) {
        self.data_stream.insert(data_stream_id,  data_stream);
    }

    ///
    /// Set data stream frequency (re-calc rate)
    ///
    pub fn set_data_stream_frequency(&mut self, data_stream_id: usize, frequency: u32) {
        if let Some(data_stream) = self.data_stream.get_mut(&data_stream_id) {
            data_stream.set_frequency(self.sound_stream.get_sampling_rate(), frequency);
        }
    }

    ///
    /// Get data stream borrow
    ///
    pub fn get_data_stream(&mut self, data_stream_id: usize) -> Option<&mut DataStream> {
        self.data_stream.get_mut(&data_stream_id)
    }
}

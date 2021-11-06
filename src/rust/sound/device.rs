// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::collections::HashMap;
use super::{
    data_stream::{DataBlock, DataStream},
    sound_chip::SoundChip,
    stream::{SoundStream, Tick},
    RomIndex,
};

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: Box<dyn SoundStream>,
    data_stream: Vec<DataStream>,
}

impl SoundDevice {
    pub fn new(sound_chip: Box<dyn SoundChip>, sound_stream: Box<dyn SoundStream>) -> Self {
        SoundDevice {
            sound_chip,
            sound_stream,
            data_stream: Vec::new(),
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
}

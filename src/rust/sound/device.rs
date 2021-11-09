// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use super::{
    data_stream::{DataBlock, DataStream},
    sound_chip::SoundChip,
    stream::{SoundStream, Tick},
    RomIndex,
};
use std::collections::HashMap;

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: Box<dyn SoundStream>,
    data_stream: HashMap<usize, DataStream>,
    write_adjust: Vec<(u32, u32)>,
    adjust_tick: usize,
}

impl SoundDevice {
    pub fn new(sound_chip: Box<dyn SoundChip>, sound_stream: Box<dyn SoundStream>) -> Self {
        SoundDevice {
            sound_chip,
            sound_stream,
            data_stream: HashMap::new(),
            write_adjust: Vec::new(),
            adjust_tick: 0,
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
            // data stream write to sound chip
            for (_, (_, data_stream)) in self.data_stream.iter_mut().enumerate() {
                if let Some((data_block_id, data_block_pos, _write_port, write_reg)) =
                    data_stream.tick()
                {
                    if let Some(data_block) = data_block.get(&data_block_id) {
                        // write stream command
                        self.sound_chip.write(
                            sound_chip_index,
                            write_reg,
                            *data_block.get_data_block().get(data_block_pos).unwrap() as u32,
                            &mut *self.sound_stream,
                        )
                    }
                }
            }
            // write adjust command
            if !self.write_adjust.is_empty() && self.adjust_tick == 1 {
                for (port, data) in self.write_adjust.iter() {
                    self.sound_chip.write(sound_chip_index, *port, *data, &mut *self.sound_stream);
                }
                self.write_adjust.clear();
            }
            if self.adjust_tick > 0 {
                self.adjust_tick -= 1;
            }
            // sound chip update
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
        // If the current playback position is in the future,
        // the command will be delayed to the next tick.
        // However, if there is a data stream, it will not be delayed to synchronize.
        if !self.sound_stream.is_adjust() || /* hack */!self.data_stream.is_empty() {
            self.sound_chip
                .write(sound_chip_index, port, data, &mut *self.sound_stream);
        } else {
            self.write_adjust.push((port, data));
            self.adjust_tick = 2;
        }
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
        self.data_stream.insert(data_stream_id, data_stream);
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
    /// Attach data block to stream
    ///
    pub fn attach_data_block_to_stream(&mut self, data_stream_id: usize, data_block_id: usize) {
        if let Some(data_stream) = self.data_stream.get_mut(&data_stream_id) {
            data_stream.set_data_block_id(data_block_id);
        }
    }

    ///
    /// Start data stream
    ///
    pub fn start_data_stream(
        &mut self,
        data_stream_id: usize,
        data_block_start_offset: usize,
        data_block_length: usize,
    ) {
        if let Some(data_stream) = self.data_stream.get_mut(&data_stream_id) {
            data_stream.start_data_stream(Some(data_block_start_offset), data_block_length);
        }
    }

    ///
    /// Start data stream fast
    ///
    pub fn start_data_stream_fast(
        &mut self,
        data_stream_id: usize,
        data_block_id: usize,
        data_block_length: usize,
    ) {
        if let Some(data_stream) = self.data_stream.get_mut(&data_stream_id) {
            data_stream.set_data_block_id(data_block_id);
            data_stream.start_data_stream(None, data_block_length);
        }
    }

    ///
    /// Stop data stream
    ///
    pub fn stop_data_stream(&mut self, data_stream_id: usize) {
        if let Some(data_stream) = self.data_stream.get_mut(&data_stream_id) {
            data_stream.stop_data_stream();
        }
    }
}

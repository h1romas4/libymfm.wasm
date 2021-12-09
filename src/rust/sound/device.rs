// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use super::{
    data_stream::{DataBlock, DataStream},
    sound_chip::SoundChip,
    stream::{SoundStream, Tick},
    RomIndex,
};
use std::collections::HashMap;

#[derive(std::cmp::PartialEq)]
pub enum DataStreamMode {
    Parallel,
    MergeS8le,
}

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: Box<dyn SoundStream>,
    data_stream_mode: DataStreamMode,
    data_stream: HashMap<usize, DataStream>,
}

impl SoundDevice {
    pub fn new(sound_chip: Box<dyn SoundChip>, sound_stream: Box<dyn SoundStream>) -> Self {
        SoundDevice {
            sound_chip,
            sound_stream,
            data_stream_mode: DataStreamMode::Parallel,
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
        while {
            is_tick = self.sound_stream.is_tick();
            is_tick != Tick::No
        } {
            // write data stream to sound chip
            self.write_data_stream(sound_chip_index, data_block);

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
        self.data_stream.insert(data_stream_id, data_stream);
    }

    ///
    /// Set data stream mode
    ///
    pub fn set_data_stream_mode(&mut self, data_stream_mode: DataStreamMode) {
        self.data_stream_mode = data_stream_mode;
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

    ///
    /// Write data stream
    ///
    fn write_data_stream(
        &mut self,
        sound_chip_index: usize,
        data_block: &HashMap<usize, DataBlock>,
    ) {
        let mut merge_data: Option<i32> = None;
        let mut merge_reg = None;
        for (_, data_stream) in self.data_stream.iter_mut() {
            if let Some((data_block_id, data_block_pos, _write_port, write_reg)) =
                data_stream.tick()
            {
                if let Some(data_block) = data_block.get(&data_block_id) {
                    let data = *data_block.get_data_block().get(data_block_pos).unwrap();
                    match self.data_stream_mode {
                        DataStreamMode::Parallel => {
                            // write stream command each data stream
                            self.sound_chip.write(
                                sound_chip_index,
                                write_reg,
                                data as u32,
                                &mut *self.sound_stream,
                            )
                        }
                        DataStreamMode::MergeS8le => {
                            // merge stream as YM3012 format pcm data
                            let data = data as i8;
                            merge_data = Some(data as i32 + merge_data.unwrap_or_default());
                            merge_reg = Some(write_reg);
                        }
                    }
                }
            }
        }
        // write merged data stream
        if let Some(mut data) = merge_data {
            if self.data_stream_mode == DataStreamMode::MergeS8le {
                if data > i8::MAX.into() {
                    data = i8::MAX.into();
                } else if data < i8::MIN.into() {
                    data = i8::MIN.into();
                }
                // unsigned
                data += 128;
            }
            self.sound_chip.write(
                sound_chip_index,
                merge_reg.unwrap(),
                data as u32,
                &mut *self.sound_stream,
            );
        }
    }
}

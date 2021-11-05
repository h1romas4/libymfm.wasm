// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use super::SoundChipType;
use std::collections::HashMap;

pub struct DataBlock {
    memory: Vec<u8>,
}

pub struct DataStreamSet {
    data_stream: HashMap<SoundChipType, Vec<DataStream>>,
    data_block: HashMap<usize, DataBlock>,
}

impl DataStreamSet {
    pub fn new() -> Self {
        DataStreamSet {
            data_stream: HashMap::new(),
            data_block: HashMap::new(),
        }
    }

    ///
    /// Set data block
    ///
    pub fn set_data_block(&mut self, data_block_id: usize, data_block: &[u8]) {
        self.data_block.insert(
            data_block_id,
            DataBlock {
                memory: data_block.to_vec(), /* clone */
            },
        );
    }

    ///
    /// Add data stream
    ///
    pub fn add_data_stream(&mut self, sound_device_name: SoundChipType, sound_device_index: usize) {

    }

    ///
    /// Find data stream and attached data block by sound device name
    ///
    pub fn find_data_stream_set(
        &mut self,
        sound_device_name: SoundChipType,
        sound_device_index: usize,
    ) -> (Option<&mut DataStream>, Option<&DataBlock>) {
        let mut find_data_stream: Option<&mut DataStream> = None;
        let mut find_data_block: Option<&DataBlock> = None;
        if self.data_stream.contains_key(&sound_device_name) {
            for data_stream in self.data_stream.get_mut(&sound_device_name).unwrap() {
                if data_stream.sound_device_index == sound_device_index {
                    find_data_block = self.data_block.get(&data_stream.data_block_id);
                    find_data_stream = Some(data_stream);
                }
            }
        }

        (find_data_stream, find_data_block)
    }
}

pub struct DataStream {
    sound_device_name: SoundChipType,
    sound_device_index: usize,
    data_block_id: usize,
    frequency: u32,
    pcm_pos: usize,
    pcm_offset: usize,
    pcm_stream_sample_step: f32,
    pcm_stream_sampling_pos: f32,
    pcm_stream_length: usize,
    pcm_stream_pos_init: usize,
    pcm_stream_pos: usize,
    write_port: u32,
    write_reg: u32,
    step_size: u8,
    step_base: u8,
}

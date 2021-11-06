// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
pub struct DataBlock {
    memory: Vec<u8>,
}

impl DataBlock {
    pub fn new(data_block: &[u8]) -> Self {
        DataBlock {
            memory: data_block.to_vec() /* clone */
        }
    }

    pub fn get_data_block(&self) -> &[u8] {
        self.memory.as_slice()
    }
}

pub struct DataStream {
    data_block_id: Option<usize>,
    frequency: u32,
    write_port: u32,
    write_reg: u32,
    data_stream_start_offset: usize,
    data_stream_length: usize,
    data_stream_sampling_pos: f32,
    data_stream_sample_step: f32,
}

impl DataStream {
    pub fn new(write_port: u32, write_reg: u32) -> Self {
        DataStream {
            data_block_id: None,
            frequency: 0,
            write_port,
            write_reg,
            data_stream_start_offset: 0,
            data_stream_length: 0,
            data_stream_sampling_pos: 0_f32,
            data_stream_sample_step: 0_f32,
        }
    }

    pub fn set_frequency(&mut self, sampling_rate: u32, frequency: u32) {
        self.frequency = frequency;
        self.data_stream_sampling_pos = 0_f32;
        self.data_stream_sample_step = frequency as f32 / sampling_rate as f32;
    }
}

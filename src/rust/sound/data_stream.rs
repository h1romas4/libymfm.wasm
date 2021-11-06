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
    data_block_id: usize,
    frequency: u32,
    write_port: u32,
    write_reg: u32,
    data_stream_start_offset: usize,
    data_stream_length: usize,
    data_stream_sampling_pos: f32,
    data_stream_sample_step: f32,
}

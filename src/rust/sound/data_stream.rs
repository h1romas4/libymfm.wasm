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
}

pub struct DataStream {
    data_block_id: usize,
    frequency: u32,
    write_port: u32,
    write_reg: u32,
    pcm_pos: usize,
    pcm_offset: usize,
    pcm_stream_sample_step: f32,
    pcm_stream_sampling_pos: f32,
    pcm_stream_length: usize,
    pcm_stream_pos_init: usize,
    pcm_stream_pos: usize,
}

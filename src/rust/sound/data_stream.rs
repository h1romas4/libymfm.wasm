// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
pub struct DataBlock {
    memory: Vec<u8>,
}

impl DataBlock {
    pub fn new(data_block: &[u8]) -> Self {
        DataBlock {
            memory: data_block.to_vec(), /* clone */
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
    data_block_pos: usize,
    data_block_start_offset: usize,
    data_block_length: usize,
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
            data_block_start_offset: 0,
            data_block_pos: 0,
            data_block_length: 0,
            data_stream_sampling_pos: 0_f32,
            data_stream_sample_step: 0_f32,
        }
    }

    ///
    /// Tick stream
    ///
    pub fn tick(&mut self) -> Option<(usize, usize, u32, u32)> {
        let mut result = None;
        if self.data_block_length > 0 {
            result = if self.data_stream_sampling_pos >= 1_f32 {
                self.data_stream_sampling_pos = 0_f32; /* -= -1_f32 */
                self.data_block_length -= 1;
                self.data_block_pos += 1;
                Some((
                    self.data_block_id.unwrap(/* TODO: */),
                    self.data_block_start_offset + self.data_block_pos,
                    self.write_port,
                    self.write_reg,
                ))
            } else {
                None
            };
            self.data_stream_sampling_pos += self.data_stream_sample_step;
        }
        result
    }

    ///
    /// Set data stream frequency
    ///
    pub fn set_frequency(&mut self, sampling_rate: u32, frequency: u32) {
        self.frequency = frequency;
        self.data_stream_sampling_pos = 0_f32;
        self.data_stream_sample_step = frequency as f32 / sampling_rate as f32;
    }

    ///
    /// Assign data data stream to data block
    ///
    pub fn set_data_block_id(&mut self, data_block_id: usize) {
        self.data_block_id = Some(data_block_id);
    }

    ///
    /// Start data stream
    ///
    pub fn start_data_stream(
        &mut self,
        data_block_start_offset: Option<usize>,
        data_block_length: usize,
    ) {
        if let Some(data_block_start_offset) = data_block_start_offset {
            self.data_block_start_offset = data_block_start_offset;
        }
        self.data_block_pos = 0;
        self.data_block_length = data_block_length - 1;
        self.data_stream_sampling_pos = 0_f32;
    }

    ///
    /// Stop data stream
    ///
    pub fn stop_data_stream(&mut self) {
        if self.data_stream_sampling_pos >= 1_f32 {
            self.data_block_length = 1; /* flash last sample */
        } else {
            self.data_block_length = 0;
        }
    }
}

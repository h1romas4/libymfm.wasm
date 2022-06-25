// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::chip_c140::{C140, C219};
use super::chip_okim6258::OKIM6258;
use super::chip_pwm::PWM;
use super::chip_segapcm::SEGAPCM;
use super::chip_sn76496::SN76496;
use super::chip_ymfm::YmFm;
use super::data_stream::{DataBlock, DataStream};
use super::device::{DataStreamMode, SoundDevice};
use super::rom::{RomBusType, RomIndex, RomSet};
use super::sound_chip::SoundChip;
use super::stream::{
    convert_sample_f2i, LinearUpSamplingStream, NativeStream, NearestDownSampleStream,
    OverSampleStream, Resolution, SampleHoldUpSamplingStream, SoundStream,
};
use super::SoundChipType;

///
/// Sound Slot
///
pub struct SoundSlot {
    output_sampling_rate: u32,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_sample_chunk_size: usize,
    output_sampling_l: Vec<f32>,
    output_sampling_r: Vec<f32>,
    output_sampling_s16le: Vec<i16>,
    output_sampling_buffer_l: VecDeque<f32>,
    output_sampling_buffer_r: VecDeque<f32>,
    sound_device: HashMap<SoundChipType, Vec<SoundDevice>>,
    sound_rom_set: HashMap<RomIndex, Rc<RefCell<RomSet>>>,
    data_block: HashMap<usize, DataBlock>,
}

impl SoundSlot {
    pub fn new(
        external_tick_rate: u32,
        output_sampling_rate: u32,
        output_sample_chunk_size: usize,
    ) -> Self {
        assert!(output_sampling_rate >= external_tick_rate);
        SoundSlot {
            output_sampling_rate,
            output_sampling_pos: 0_f64,
            output_sampling_step: external_tick_rate as f64 / output_sampling_rate as f64,
            output_sample_chunk_size,
            output_sampling_l: vec![0_f32; output_sample_chunk_size],
            output_sampling_r: vec![0_f32; output_sample_chunk_size],
            output_sampling_s16le: vec![0; output_sample_chunk_size * 2],
            output_sampling_buffer_l: VecDeque::with_capacity(output_sample_chunk_size * 2),
            output_sampling_buffer_r: VecDeque::with_capacity(output_sample_chunk_size * 2),
            sound_device: HashMap::new(),
            sound_rom_set: HashMap::new(),
            data_block: HashMap::new(),
        }
    }

    ///
    /// Add sound device (sound chip and sound stream, Rom set)
    ///
    pub fn add_sound_device(
        &mut self,
        sound_chip_type: SoundChipType,
        number_of: usize,
        clock: u32,
    ) {
        for _ in 0..number_of {
            // create sound device
            let mut sound_chip: Box<dyn SoundChip> = match sound_chip_type {
                SoundChipType::YM2149
                | SoundChipType::YM2151
                | SoundChipType::YM2203
                | SoundChipType::YM2413
                | SoundChipType::YM2608
                | SoundChipType::YM2610
                | SoundChipType::YM2612
                | SoundChipType::YM3526
                | SoundChipType::Y8950
                | SoundChipType::YM3812
                | SoundChipType::YMF262
                | SoundChipType::YMF278B => {
                    let mut ymfm = Box::new(YmFm::new(sound_chip_type));
                    let rom_index: Option<Vec<RomIndex>> = match sound_chip_type {
                        SoundChipType::YM2608 => Some(vec![RomIndex::YM2608_DELTA_T]),
                        SoundChipType::YM2610 => {
                            Some(vec![RomIndex::YM2610_ADPCM, RomIndex::YM2610_DELTA_T])
                        }
                        SoundChipType::Y8950 => Some(vec![RomIndex::Y8950_ROM]),
                        SoundChipType::YMF278B => {
                            Some(vec![RomIndex::YMF278B_ROM, RomIndex::YMF278B_RAM])
                        }
                        _ => None,
                    };
                    if let Some(rom_index) = rom_index {
                        self.add_rom_bank(rom_index, &mut *ymfm);
                    }
                    ymfm
                }
                SoundChipType::SEGAPSG => Box::new(SN76496::new(SoundChipType::SEGAPSG)),
                SoundChipType::SN76489 => Box::new(SN76496::new(SoundChipType::SN76489)),
                SoundChipType::PWM => Box::new(PWM::new()),
                SoundChipType::SEGAPCM => {
                    let mut segapcm = Box::new(SEGAPCM::new(SoundChipType::SEGAPCM));
                    self.add_rom_bank(vec![RomIndex::SEGAPCM_ROM], &mut *segapcm);
                    segapcm
                }
                SoundChipType::OKIM6258 => Box::new(OKIM6258::new(SoundChipType::OKIM6258)),
                SoundChipType::C140 => {
                    let mut c140 = Box::new(C140::new(SoundChipType::C140));
                    self.add_rom_bank(vec![RomIndex::C140_ROM], &mut *c140);
                    c140
                },
                SoundChipType::C219 => {
                    let mut c219 = Box::new(C219::new(SoundChipType::C219));
                    self.add_rom_bank(vec![RomIndex::C140_ROM], &mut *c219);
                    c219
                },
            };

            // initialize sound chip
            let sound_chip_sampling_rate = sound_chip.init(clock);
            // select resampling method
            let sound_stream: Box<dyn SoundStream> =
                match sound_chip_sampling_rate.cmp(&self.output_sampling_rate) {
                    Ordering::Equal => Box::new(NativeStream::new()),
                    Ordering::Greater => match sound_chip_type {
                        SoundChipType::SEGAPSG | SoundChipType::SN76489 | SoundChipType::PWM => {
                            Box::new(OverSampleStream::new(
                                sound_chip_sampling_rate,
                                self.output_sampling_rate,
                            ))
                        }
                        _ => Box::new(NearestDownSampleStream::new(
                            sound_chip_sampling_rate,
                            self.output_sampling_rate,
                        )),
                    },
                    _ => match sound_chip_type {
                        SoundChipType::OKIM6258 | SoundChipType::C140 => Box::new(SampleHoldUpSamplingStream::new(
                            sound_chip_sampling_rate,
                            self.output_sampling_rate,
                        )),
                        _ => Box::new(LinearUpSamplingStream::new(
                            sound_chip_sampling_rate,
                            self.output_sampling_rate,
                            Resolution::RangeAll,
                        )),
                    },
                };
            // add sound device
            self.sound_device
                .entry(sound_chip_type)
                .or_insert_with(Vec::new)
                .push(SoundDevice::new(sound_chip, sound_stream));
        }
    }

    ///
    /// Write command to sound chip.
    ///
    pub fn write(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        port: u32,
        data: u32,
    ) {
        match self.find_sound_device(sound_chip_type, sound_chip_index) {
            None => { /* nothing to do */ }
            Some(sound_device) => sound_device.write(sound_chip_index, port, data),
        }
    }

    ///
    /// Update sound chip.
    ///
    pub fn update(&mut self, tick_count: usize) {
        for _ in 0..tick_count {
            while self.output_sampling_pos < 1_f64 {
                self.output_sampling_buffer_l.push_back(0_f32);
                self.output_sampling_buffer_r.push_back(0_f32);
                let buffer_pos = self.output_sampling_buffer_l.len() - 1;
                for (_, sound_devices) in self.sound_device.iter_mut() {
                    for (index, sound_device) in sound_devices.iter_mut().enumerate() {
                        let (l, r) = sound_device.generate(index, &self.data_block);
                        self.output_sampling_buffer_l[buffer_pos] += l;
                        self.output_sampling_buffer_r[buffer_pos] += r;
                    }
                }
                self.output_sampling_pos += self.output_sampling_step;
            }
            self.output_sampling_pos -= 1_f64;
        }
    }

    ///
    /// Remaining tickable in sampling buffers.
    ///
    pub fn is_stream_filled(&self) -> bool {
        self.output_sample_chunk_size as isize - self.output_sampling_buffer_l.len() as isize <= 0
    }

    ///
    /// Stream sampling chunk
    ///
    pub fn stream(&mut self) {
        let mut chunk_size = self.output_sample_chunk_size;
        // Last chunk
        if self.output_sample_chunk_size > self.output_sampling_buffer_l.len() {
            chunk_size = self.output_sampling_buffer_l.len();
            for i in 0..self.output_sample_chunk_size {
                self.output_sampling_l[i] = 0_f32;
                self.output_sampling_r[i] = 0_f32;
            }
        }

        for (i, val) in self
            .output_sampling_buffer_l
            .drain(0..chunk_size)
            .enumerate()
        {
            self.output_sampling_l[i] = val;
        }
        for (i, val) in self
            .output_sampling_buffer_r
            .drain(0..chunk_size)
            .enumerate()
        {
            self.output_sampling_r[i] = val;
        }
    }

    ///
    /// Return sampling_l buffer referance.
    ///
    pub fn get_output_sampling_l_ref(&self) -> *const f32 {
        self.output_sampling_l.as_ptr()
    }

    ///
    /// Return sampling buffer referance.
    ///
    pub fn get_output_sampling_r_ref(&self) -> *const f32 {
        self.output_sampling_r.as_ptr()
    }

    ///
    /// Convert and return s16le sampling buffer
    ///
    pub fn get_output_sampling_s16le_ref(&mut self) -> *const i16 {
        for i in 0..self.output_sample_chunk_size {
            self.output_sampling_s16le[i * 2] = convert_sample_f2i(self.output_sampling_l[i]);
            self.output_sampling_s16le[i * 2 + 1] = convert_sample_f2i(self.output_sampling_r[i]);
        }
        self.output_sampling_s16le.as_ptr()
    }

    ///
    /// Add ROM to sound chip.
    ///
    pub fn add_rom(
        &mut self,
        rom_index: RomIndex,
        memory: &[u8],
        start_address: usize,
        end_address: usize,
    ) {
        if self.sound_rom_set.contains_key(&rom_index) {
            let index_no = self
                .sound_rom_set
                .get(&rom_index)
                .unwrap()
                .borrow_mut()
                .add_rom(memory, start_address, end_address);
            // notify sound chip
            let sound_chip_type = self.get_sound_chip_type_by_rom_index(rom_index);
            if let Some(sound_chip_type) = sound_chip_type {
                if let Some(sound_device) = self.sound_device.get_mut(&sound_chip_type) {
                    for sound_device in sound_device {
                        sound_device.notify_add_rom(rom_index, index_no);
                    }
                };
            }
        }
    }

    ///
    /// Set ROM Bus type
    ///
    pub fn set_rom_bus_type(&mut self, rom_index: RomIndex, rom_bus_type: Option<RomBusType>) {
        if self.sound_rom_set.contains_key(&rom_index) {
            self
                .sound_rom_set
                .get(&rom_index)
                .unwrap()
                .borrow_mut()
                .set_bus_type(rom_bus_type);
        }
    }

    ///
    /// Add data bank for stream data.
    ///
    pub fn add_data_block(&mut self, data_block_id: usize, data_block: &[u8]) {
        self.data_block
            .insert(data_block_id, DataBlock::new(data_block));
    }

    ///
    /// Get data block borrow
    ///
    pub fn get_data_block(&self, data_block_id: usize) -> &[u8] {
        self.data_block.get(&data_block_id).unwrap(/* TODO */).get_data_block()
    }

    ///
    /// Add data stream
    ///
    pub fn add_data_stream(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
        write_port: u32,
        write_reg: u32,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.add_data_stream(data_stream_id, DataStream::new(write_port, write_reg));
        }
    }

    ///
    /// Set data stream mode
    ///
    pub fn set_data_stream_mode(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_mode: DataStreamMode,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.set_data_stream_mode(data_stream_mode);
        }
    }

    ///
    /// Set data stream frequency
    ///
    pub fn set_data_stream_frequency(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
        frequency: u32,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.set_data_stream_frequency(data_stream_id, frequency);
        }
    }

    //
    // Attach data block to data stream
    //
    pub fn attach_data_block_to_stream(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
        data_block_id: usize,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.attach_data_block_to_stream(data_stream_id, data_block_id);
        }
    }

    ///
    /// Start data stream
    ///
    pub fn start_data_stream(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
        data_block_start_offset: usize,
        data_block_length: usize,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.start_data_stream(
                data_stream_id,
                data_block_start_offset,
                data_block_length,
            );
        }
    }

    ///
    /// Start data stream fast call
    ///
    pub fn start_data_stream_fast(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
        data_block_id: usize,
    ) {
        let mut data_block_length = None;
        if let Some(data_block) = self.data_block.get(&data_block_id) {
            data_block_length = Some(data_block.get_data_block().len());
        }
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            if let Some(data_block_length) = data_block_length {
                sound_device.start_data_stream_fast(
                    data_stream_id,
                    data_block_id,
                    data_block_length,
                );
            }
        }
    }

    ///
    /// Stop data stream
    ///
    pub fn stop_data_stream(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
    ) {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            sound_device.stop_data_stream(data_stream_id);
        }
    }

    ///
    /// Return data stream play state
    ///
    pub fn is_stop_data_stream(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
        data_stream_id: usize,
    ) -> bool {
        if let Some(sound_device) = self.find_sound_device(sound_chip_type, sound_chip_index) {
            return sound_device.is_stop_data_stream(data_stream_id);
        }
        true
    }

    ///
    /// Add rom bank
    ///
    fn add_rom_bank<T: SoundChip>(&mut self, rom_index: Vec<RomIndex>, sound_chip: &mut T) {
        for &rom_index in rom_index.iter() {
            // create new romset
            let romset = Rc::new(RefCell::new(RomSet::new()));
            // trancefer romset to soundchip
            sound_chip.set_rom_bank(rom_index, Some(romset.clone()));
            // hold romset in slot
            self.sound_rom_set.insert(rom_index, romset);
        }
    }

    ///
    /// Obtains the SoundChipType corresponding to RomIndex.
    ///
    fn get_sound_chip_type_by_rom_index(&self, rom_index: RomIndex) -> Option<SoundChipType> {
        match rom_index {
            RomIndex::YM2608_DELTA_T => Some(SoundChipType::YM2608),
            RomIndex::YM2610_ADPCM => Some(SoundChipType::YM2610),
            RomIndex::YM2610_DELTA_T => Some(SoundChipType::YM2610),
            RomIndex::YMF278B_ROM => Some(SoundChipType::YMF278B),
            RomIndex::YMF278B_RAM => Some(SoundChipType::YMF278B),
            RomIndex::Y8950_ROM => Some(SoundChipType::Y8950),
            RomIndex::SEGAPCM_ROM => Some(SoundChipType::SEGAPCM),
            RomIndex::C140_ROM => Some(SoundChipType::C140),
            RomIndex::NOT_SUPPOTED => None,
        }
    }

    ///
    /// Change external tick rate.
    ///
    /// This only works correctly when the sound slot is in its initial state.
    ///
    pub fn change_external_tick_rate(&mut self, external_tick_rate: u32) {
        self.output_sampling_step = external_tick_rate as f64 / self.output_sampling_rate as f64;
    }

    ///
    /// Get the sound chip from the sound slot.
    ///
    #[inline]
    fn find_sound_device(
        &mut self,
        sound_chip_type: SoundChipType,
        sound_chip_index: usize,
    ) -> Option<&mut SoundDevice> {
        let sound_device = match self.sound_device.get_mut(&sound_chip_type) {
            None => None,
            Some(vec) => {
                if vec.len() < sound_chip_index {
                    return None;
                }
                Some(vec)
            }
        };
        match sound_device {
            None => None,
            Some(sound_chip) => Some(&mut sound_chip[sound_chip_index]),
        }
    }
}

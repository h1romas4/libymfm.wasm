// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use super::chip_okim6258::OKIM6258;
use super::chip_pwm::PWM;
use super::chip_segapcm::SEGAPCM;
use super::chip_sn76496::SN76496;
use super::chip_ymfm::YmFm;
use super::data_stream::{DataStream, DataStreamSet};
use super::rom::{RomIndex, RomSet};
use super::sound_chip::SoundChip;
use super::stream::{
    LinearUpSamplingStream, NativeStream, NearestDownSampleStream, OverSampleStream, Resolution,
    SoundStream, Tick,
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
    output_sampling_buffer_l: VecDeque<f32>,
    output_sampling_buffer_r: VecDeque<f32>,
    sound_device: HashMap<SoundChipType, Vec<SoundDevice>>,
    sound_rom_set: HashMap<RomIndex, Rc<RefCell<RomSet>>>,
    data_stream_set: DataStreamSet,
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
            output_sampling_buffer_l: VecDeque::with_capacity(output_sample_chunk_size * 2),
            output_sampling_buffer_r: VecDeque::with_capacity(output_sample_chunk_size * 2),
            sound_device: HashMap::new(),
            sound_rom_set: HashMap::new(),
            data_stream_set: DataStreamSet::new(),
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
        for _n in 0..number_of {
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
            };

            // initialize sound chip and select resampling method
            let sound_chip_sampling_rate = sound_chip.init(clock);
            #[allow(clippy::comparison_chain)]
            let sound_stream: Box<dyn SoundStream> =
                if sound_chip_sampling_rate == self.output_sampling_rate {
                    Box::new(NativeStream::new())
                } else if sound_chip_sampling_rate > self.output_sampling_rate {
                    match sound_chip_type {
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
                    }
                } else {
                    Box::new(LinearUpSamplingStream::new(
                        sound_chip_sampling_rate,
                        self.output_sampling_rate,
                        Resolution::RangeAll,
                    ))
                };

            // add sound device
            self.sound_device
                .entry(sound_chip_type)
                .or_insert_with(Vec::new)
                .push(SoundDevice {
                    sound_chip,
                    sound_stream,
                });
        }
    }

    ///
    /// Write command to sound chip.
    ///
    pub fn write(&mut self, sound_device_name: SoundChipType, index: usize, port: u32, data: u32) {
        match self.find_sound_device(sound_device_name, index) {
            None => { /* nothing to do */ }
            Some(sound_device) => sound_device.write(index, port, data),
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
                for (sound_device_name, sound_devices) in self.sound_device.iter_mut() {
                    for (index, sound_device) in sound_devices.iter_mut().enumerate() {
                        let data_stream = self
                            .data_stream_set
                            .find_data_stream_set(*sound_device_name, index);
                        let (l, r) = sound_device.generate(index, data_stream);
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
    /// Add ROM for sound chip.
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
            let sound_device_name = match rom_index {
                RomIndex::YM2608_DELTA_T => Some(SoundChipType::YM2608),
                RomIndex::YM2610_ADPCM => Some(SoundChipType::YM2610),
                RomIndex::YM2610_DELTA_T => Some(SoundChipType::YM2610),
                RomIndex::YMF278B_ROM => Some(SoundChipType::YMF278B),
                RomIndex::YMF278B_RAM => Some(SoundChipType::YMF278B),
                RomIndex::Y8950_ROM => Some(SoundChipType::Y8950),
                RomIndex::SEGAPCM_ROM => Some(SoundChipType::SEGAPCM),
                RomIndex::NOT_SUPPOTED => None,
            };
            if let Some(sound_device_name) = sound_device_name {
                if let Some(sound_device) = self.sound_device.get_mut(&sound_device_name) {
                    for sound_device in sound_device {
                        sound_device.sound_chip.notify_add_rom(rom_index, index_no);
                    }
                };
            }
        }
    }

    ///
    /// Start data stream
    ///
    pub fn start_data_stream(
        &mut self,
        data_stream_id: usize,
        data_block_id: Option<usize>,
        pcm_stream_pos: Option<usize>,
        pcm_stream_length: Option<usize>,
    ) {
    }

    ///
    /// Stop data stream
    ///
    pub fn stop_data_stream(&mut self, data_stream_id: usize) {}

    ///
    /// Add data bank for stream data.
    ///
    pub fn add_data_block(&mut self, memory: &[u8]) -> usize {
        0 /* data_block_id */
    }

    ///
    /// Add data stream
    ///
    pub fn add_data_stream(&mut self, sound_device_name: SoundChipType, index: usize) -> usize {
        0 /* data_stream_id */
    }

    //
    // Attach data block to data stream
    //
    pub fn attach_data_block_to_stream(&mut self, data_stream_id: usize, data_bank_id: usize) {}

    ///
    /// Set data stream frequency
    ///
    pub fn set_data_stream_frequency(&mut self, data_stream_id: usize, frequency: u32) {}

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
    /// Get the sound chip from the sound slot.
    ///
    #[inline]
    fn find_sound_device(
        &mut self,
        sound_device_name: SoundChipType,
        index: usize,
    ) -> Option<&mut SoundDevice> {
        let sound_device = match self.sound_device.get_mut(&sound_device_name) {
            None => None,
            Some(vec) => {
                if vec.len() < index {
                    return None;
                }
                Some(vec)
            }
        };
        match sound_device {
            None => None,
            Some(sound_chip) => Some(&mut sound_chip[index]),
        }
    }
}

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: Box<dyn SoundStream>,
}

impl SoundDevice {
    ///
    /// Generates a waveform for one sample according to
    /// the output sampling rate of the sound stream.
    ///
    fn generate(
        &mut self,
        sound_chip_index: usize,
        data_stream: Option<&mut DataStream>,
    ) -> (f32, f32) {
        let mut is_tick;
        while {
            is_tick = self.sound_stream.is_tick();
            is_tick != Tick::No
        } {
            self.sound_chip.tick(
                sound_chip_index,
                &mut *self.sound_stream,
                &data_stream,
            );
            if is_tick == Tick::One {
                break;
            }
        }
        self.sound_stream.drain()
    }

    ///
    /// Write command to sound chip.
    ///
    fn write(&mut self, sound_chip_index: usize, port: u32, data: u32) {
        self.sound_chip
            .write(sound_chip_index, port, data, &mut *self.sound_stream);
    }
}

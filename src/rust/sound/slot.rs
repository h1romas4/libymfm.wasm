// license:BSD-3-Clause
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::sound::chip_pwm::PWM;
use crate::sound::chip_segapcm::SEGAPCM;
use crate::sound::chip_sn76489::SN76489;
use crate::sound::chip_ymfm::YmFm;
use crate::sound::interface::RomDevice;
use crate::sound::interface::SoundChip;
use crate::sound::stream::NearestDownSampleStream;
use crate::sound::stream::NativeStream;
use crate::sound::stream::LinearUpSamplingStream;

use super::rom::RomSet;
use super::stream::SoundStream;
use super::stream::Tick;
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
    sound_romset: HashMap<usize, Rc<RefCell<RomSet>>>,
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
            sound_romset: HashMap::new(),
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
            let mut sound_chip: Box<dyn SoundChip> = match sound_chip_type {
                SoundChipType::YM2151
                | SoundChipType::YM2203
                | SoundChipType::YM2149
                | SoundChipType::YM2612
                | SoundChipType::YM2413 => Box::new(YmFm::new(sound_chip_type)),
                SoundChipType::YM2602 => todo!(),
                SoundChipType::SEGAPSG => Box::new(SN76489::new(SoundChipType::SEGAPSG)),
                SoundChipType::SN76489 => Box::new(SN76489::new(SoundChipType::SN76489)),
                SoundChipType::PWM => Box::new(PWM::new(SoundChipType::PWM)),
                SoundChipType::SEGAPCM => {
                    // connect PCM ROM
                    let mut segapcm = Box::new(SEGAPCM::new(SoundChipType::SEGAPCM));
                    let segapcm_romset = Rc::new(RefCell::new(RomSet::new()));
                    RomDevice::set_rom(&mut *segapcm, Some(segapcm_romset.clone()));
                    self.sound_romset.insert(0x80, segapcm_romset); // 0x80 segapcm
                    segapcm
                }
            };

            let sound_chip_sampling_rate = sound_chip.init(clock);
            #[allow(clippy::comparison_chain)]
            let sound_stream: Box<dyn SoundStream> = if sound_chip_sampling_rate == self.output_sampling_rate {
                Box::new(NativeStream::new())
            } else if sound_chip_sampling_rate > self.output_sampling_rate {
                Box::new(NearestDownSampleStream::new(
                    sound_chip_sampling_rate,
                    self.output_sampling_rate,
                ))
            } else {
                Box::new(LinearUpSamplingStream::new(
                    sound_chip_sampling_rate,
                    self.output_sampling_rate,
                ))
            };

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
    /// Add ROM for sound chip.
    ///
    pub fn add_rom(&self, index: usize, memory: &[u8], start_address: usize, end_address: usize) {
        if self.sound_romset.contains_key(&index) {
            self.sound_romset.get(&index).unwrap().borrow_mut().add_rom(
                memory,
                start_address,
                end_address,
            );
        }
    }

    ///
    /// Write command to sound chip.
    ///
    pub fn write(&mut self, sound_device_name: SoundChipType, index: usize, port: u32, data: u32) {
        match self.find(sound_device_name, index) {
            None => { /* nothing to do */ }
            Some(sound_device) => sound_device.sound_chip.write(index, port, data),
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
                        let (l, r) = sound_device.generate(index);
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
    pub fn ready(&self) -> bool {
        self.output_sample_chunk_size as isize - self.output_sampling_buffer_l.len() as isize > 0
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
    /// Get the sound chip from the sound slot.
    ///
    #[inline]
    fn find(&mut self, sound_device_name: SoundChipType, index: usize) -> Option<&mut SoundDevice> {
        let sound_chip = match self.sound_device.get_mut(&sound_device_name) {
            None => None,
            Some(vec) => {
                if vec.len() < index {
                    return None;
                }
                Some(vec)
            }
        };
        match sound_chip {
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
    fn generate(&mut self, sound_chip_index: usize) -> (f32, f32) {
        let mut is_tick;
        while {
            is_tick = self.sound_stream.is_tick();
            is_tick != Tick::No
        } {
            self.sound_chip
                .tick(sound_chip_index, &mut *self.sound_stream);
            if is_tick == Tick::One {
                break;
            }
        }
        self.sound_stream.drain()
    }
}

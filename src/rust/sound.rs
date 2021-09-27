// license:BSD-3-Clause
mod pwm;
mod segapcm;
mod sn76489;
mod ymfm;

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::sound::pwm::PWM;
use crate::sound::segapcm::SEGAPCM;
use crate::sound::sn76489::SN76489;
use crate::sound::ymfm::YmFm;

///
/// Sound chip type
///
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum SoundChipType {
    YM2151,
    YM2203,
    YM2149,
    YM2612,
    YM2413,
    YM2602,
    SEGAPSG,
    PWM,
    SEGAPCM,
}

///
/// Sound Chip Interface
///
pub trait SoundChip {
    fn new(sound_device_name: SoundChipType) -> Self
    where
        Self: Sized;
    fn init(&mut self, clock: u32) -> u32;
    fn reset(&mut self);
    fn write(&mut self, index: usize, port: u32, data: u32);
    fn tick(&mut self, index: usize, sound_stream: &mut SoundStream);
}

///
/// Rom Device Interface
///
pub trait RomDevice {
    fn set_rom(&mut self, rombank: RomBank);
    fn read_rom(rombank: &RomBank, address: usize) -> u8 {
        rombank.as_ref().unwrap().borrow().read(address)
    }
}

pub type RomBank = Option<Rc<RefCell<RomSet>>>;

///
/// Sound Slot
///
pub struct SoundSlot {
    _external_tick_rate: u32,
    _output_sampling_rate: u32,
    output_sample_chunk_size: usize,
    output_sampling_l: Vec<f32>,
    output_sampling_r: Vec<f32>,
    output_sampling_buffer_l: VecDeque<f32>,
    output_sampling_buffer_r: VecDeque<f32>,
    internal_sampling_rate: u32,
    sound_device: HashMap<SoundChipType, Vec<SoundDevice>>,
    sound_romset: HashMap<usize, Rc<RefCell<RomSet>>>,
}

// TODO: 44100 -> 96000
const INTERNAL_SAMPLING_RATE: u32 = 44100;

impl SoundSlot {
    pub fn new(
        external_tick_rate: u32,
        output_sampling_rate: u32,
        output_sample_chunk_size: usize,
    ) -> Self {
        SoundSlot {
            // TODO: At present external_tick_rate == output_sampling_rate == internal_sampling_rate
            _external_tick_rate: external_tick_rate,
            _output_sampling_rate: output_sampling_rate,
            output_sample_chunk_size,
            output_sampling_l: vec![0_f32; output_sample_chunk_size],
            output_sampling_r: vec![0_f32; output_sample_chunk_size],
            output_sampling_buffer_l: VecDeque::with_capacity(output_sample_chunk_size * 2),
            output_sampling_buffer_r: VecDeque::with_capacity(output_sample_chunk_size * 2),
            internal_sampling_rate: INTERNAL_SAMPLING_RATE,
            sound_device: HashMap::new(),
            sound_romset: HashMap::new(),
        }
    }

    ///
    /// Add sound device (sound chip and sound stream, Rom set)
    ///
    pub fn add_sound_device(&mut self, sound_chip_type: SoundChipType, number_of: usize, clock: u32) {
        for _n in 0..number_of {
            let mut sound_chip: Box<dyn SoundChip> = match sound_chip_type {
                SoundChipType::YM2151
                | SoundChipType::YM2203
                | SoundChipType::YM2149
                | SoundChipType::YM2612
                | SoundChipType::YM2413 => Box::new(YmFm::new(sound_chip_type)),
                SoundChipType::YM2602 => todo!(),
                SoundChipType::SEGAPSG => Box::new(SN76489::new(SoundChipType::SEGAPSG)),
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

            let sound_chip_tick_rate = sound_chip.init(clock);
            self.sound_device
                .entry(sound_chip_type)
                .or_insert_with(Vec::new)
                .push(SoundDevice {
                    sound_chip,
                    sound_stream: SoundStream::new(
                        sound_chip_tick_rate,
                        self.internal_sampling_rate,
                    ),
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
            self.output_sampling_buffer_l.push_back(0_f32);
            self.output_sampling_buffer_r.push_back(0_f32);
            let buffer_pos = self.output_sampling_buffer_l.len() - 1;
            for (_, sound_devices) in self.sound_device.iter_mut() {
                for (index, sound_device) in sound_devices.iter_mut().enumerate() {
                    let mut is_tick;
                    while {
                        is_tick = sound_device.sound_stream.is_tick();
                        is_tick != Tick::NO
                    } {
                        sound_device
                            .sound_chip
                            .tick(index, &mut sound_device.sound_stream);
                        if is_tick == Tick::ONE {
                            break;
                        }
                    }
                    let (l, r) = sound_device.sound_stream.pop();
                    self.output_sampling_buffer_l[buffer_pos] += l;
                    self.output_sampling_buffer_r[buffer_pos] += r;
                }
            }
        }
    }

    ///
    /// Remaining sampling buffers.
    ///
    pub fn ready(&self) -> usize {
        self.output_sample_chunk_size - self.output_sampling_buffer_l.len()
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
    sound_stream: SoundStream,
}

///
/// Sound stream for sound chip
///
pub struct SoundStream {
    sound_chip_tick_rate: u32,
    sound_chip_tick_pos: u64,
    sound_chip_tick_step: u64,
    output_sampling_rate: u32,
    output_sampling_pos: u64,
    output_sampling_step: u64,
    now_chip_sampling_l: f32,
    now_chip_sampling_r: f32,
}

impl SoundStream {
    pub fn new(sound_chip_tick_rate: u32, output_sampling_rate: u32) -> Self {
        SoundStream {
            sound_chip_tick_rate,
            sound_chip_tick_pos: 0,
            sound_chip_tick_step: 0x100000000_u64 / sound_chip_tick_rate as u64,
            output_sampling_rate,
            output_sampling_pos: 0,
            output_sampling_step: 0x100000000_u64 / output_sampling_rate as u64,
            now_chip_sampling_l: 0_f32,
            now_chip_sampling_r: 0_f32,
        }
    }

    ///
    /// Compare the native tick rate of the sound chip to the output sampling rate
    /// to determine if it needs to be ticked.
    ///
    pub fn is_tick(&self) -> Tick {
        // TODO: better up-sampling
        if self.sound_chip_tick_rate < self.output_sampling_rate
            && self.sound_chip_tick_pos > self.output_sampling_pos
        {
            return Tick::NO;
        }
        // down-sampling
        if self.sound_chip_tick_rate > self.output_sampling_rate {
            #[allow(clippy::comparison_chain)]
            return if self.sound_chip_tick_pos < self.output_sampling_pos {
                Tick::MORE
            } else if self.sound_chip_tick_pos == self.output_sampling_pos {
                Tick::ONE
            } else {
                Tick::NO
            };
        }
        Tick::ONE
    }

    ///
    /// The interface through which the sound chip pushes the stream.
    ///
    pub fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.sound_chip_tick_pos =
            Self::next_pos(self.sound_chip_tick_pos, self.sound_chip_tick_step);
        self.now_chip_sampling_l = sampling_l;
        self.now_chip_sampling_r = sampling_r;
    }

    ///
    /// Get the stream of the sound chip.
    ///
    pub fn pop(&mut self) -> (f32, f32) {
        self.output_sampling_pos =
            Self::next_pos(self.output_sampling_pos, self.output_sampling_step);
        // TODO: better upsampling
        (self.now_chip_sampling_l, self.now_chip_sampling_r)
    }

    ///
    /// Calculate the position of the stream.
    ///
    fn next_pos(now: u64, step: u64) -> u64 {
        let next: u128 = (now + step).into();
        if next > u64::MAX.into() {
            return (u64::MAX as u128 - next) as u64;
        }
        next as u64
    }
}

#[derive(PartialEq)]
pub enum Tick {
    ONE,
    MORE,
    NO,
}

///
/// convert_sample_i2f
///
fn convert_sample_i2f(i32_sample: i32) -> f32 {
    let mut f32_sample: f32;
    if i32_sample < 0 {
        f32_sample = i32_sample as f32 / 32768_f32;
    } else {
        f32_sample = i32_sample as f32 / 32767_f32;
    }
    if f32_sample > 1_f32 {
        f32_sample = 1_f32;
    }
    if f32_sample < -1_f32 {
        f32_sample = -1_f32;
    }
    f32_sample
}

///
/// Rom
///
pub struct Rom {
    start_address: usize,
    end_address: usize,
    memory: Vec<u8>,
}

///
/// Rom set
///
#[derive(Default)]
pub struct RomSet {
    rom: Vec<Rom>,
}

impl RomSet {
    pub fn new() -> RomSet {
        RomSet { rom: Vec::new() }
    }

    ///
    /// Add a ROM to the rom set.
    ///
    pub fn add_rom(&mut self, memory: &[u8], start_address: usize, end_address: usize) {
        // println!("rom: {:<08x} - {:<08x}, {:<08x}, {:<02x}", start_address, end_address, memory.len(), memory[0]);
        // to_vec(clone) is external SPI memory simulation.
        self.rom.push(Rom {
            start_address,
            end_address,
            memory: memory.to_vec(),
        });
    }

    ///
    /// Read the data from the ROM address.
    ///
    pub fn read(&self, address: usize) -> u8 {
        for r in self.rom.iter() {
            if r.start_address <= address && r.end_address >= address {
                return r.memory[address - r.start_address];
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::sound::SoundStream;

    #[test]
    fn make_stream_1() {
        let _stream = SoundStream::new(44100, 44100);
    }
}

mod pwm;
mod segapcm;
mod sn76489;
mod ym3438;
mod ymfm;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub use crate::sound::pwm::PWM;
pub use crate::sound::segapcm::SEGAPCM;
pub use crate::sound::sn76489::SN76489;
pub use crate::sound::ym3438::YM3438;
pub use crate::sound::ymfm::ChipType;
pub use crate::sound::ymfm::YmFm;

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
    fn update(
        &mut self,
        index: usize,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    );
    fn tick(&mut self, index: usize, sound_stream: &mut SoundStream);
}

///
/// Rom Device Interface
///
pub trait RomDevice {
    fn set_rom(&mut self, rombank: RomBank);
    fn read_rom(rombank: &RomBank, address: usize) -> u8 {
        rombank.as_ref().unwrap().borrow_mut().read(address)
    }
}

///
/// Sound Slot
///
pub struct SoundSlot {
    external_tick_rate: u32,
    output_sampling_rate: u32,
    output_sample_chunk_size: usize,
    internal_sampling_rate: u32,
    sound_device: HashMap<SoundChipType, Vec<SoundDevice>>,
    sound_romset: HashMap<usize, Rc<RefCell<RomSet>>>,
}

// TODO: 44100 -> 96000
const INTERNAL_SAMPLING_RATE: u32 = 44100;

///
/// SoundSlot
///
impl SoundSlot {
    pub fn new(
        external_tick_rate: u32,
        output_sampling_rate: u32,
        output_sample_chunk_size: usize,
    ) -> Self {
        SoundSlot {
            external_tick_rate,
            output_sampling_rate,
            output_sample_chunk_size,
            internal_sampling_rate: INTERNAL_SAMPLING_RATE,
            sound_device: HashMap::new(),
            sound_romset: HashMap::new(),
        }
    }

    pub fn get_output_sampling_rate(&self) -> u32 {
        self.output_sampling_rate
    }

    pub fn get_external_tick_rate(&self) -> u32 {
        self.external_tick_rate
    }

    pub fn get_output_sample_chunk_size(&self) -> usize {
        self.output_sample_chunk_size
    }

    pub fn add_device(&mut self, sound_chip_type: SoundChipType, number_of: usize, clock: u32) {
        for _n in 0..number_of {
            let mut sound_chip: Box<dyn SoundChip> = match sound_chip_type {
                SoundChipType::YM2151 => Box::new(YmFm::new(SoundChipType::YM2151)),
                SoundChipType::YM2203 => Box::new(YmFm::new(SoundChipType::YM2203)),
                SoundChipType::YM2149 => Box::new(YmFm::new(SoundChipType::YM2149)),
                SoundChipType::YM2612 => Box::new(YmFm::new(SoundChipType::YM2612)),
                SoundChipType::YM2413 => Box::new(YmFm::new(SoundChipType::YM2413)),
                SoundChipType::YM2602 => todo!(),
                SoundChipType::SEGAPSG => Box::new(SN76489::new(SoundChipType::SEGAPSG)),
                SoundChipType::PWM => Box::new(PWM::new(SoundChipType::PWM)),
                SoundChipType::SEGAPCM => {
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
                    sound_chip_type,
                    sound_stream: SoundStream::new(
                        sound_chip_tick_rate,
                        self.internal_sampling_rate,
                    ),
                });
        }
    }

    pub fn add_rom(&self, index: usize, memory: &[u8], start_address: usize, end_address: usize) {
        if self.sound_romset.contains_key(&index) {
            self.sound_romset.get(&index).unwrap().borrow_mut().add_rom(
                memory,
                start_address,
                end_address,
            );
        }
    }

    #[inline]
    pub fn find(
        &mut self,
        sound_device_name: SoundChipType,
        index: usize,
    ) -> Option<&mut SoundDevice> {
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

    pub fn write(&mut self, sound_device_name: SoundChipType, index: usize, port: u32, data: u32) {
        match self.find(sound_device_name, index) {
            None => { /* nothing to do */ }
            Some(sound_device) => sound_device.sound_chip.write(index, port, data),
        }
    }

    ///
    /// update
    /// TODO: remove arg sound_chip_type & index
    ///
    pub fn update(
        &mut self,
        sound_chip_type: SoundChipType,
        index: usize,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        match self.find(sound_chip_type, index) {
            None => { /* nothing to do */ }
            Some(sound_device) => {
                // TODO: for test
                match &sound_chip_type {
                    // TODO: for test
                    SoundChipType::SEGAPCM => {
                        for i in 0..numsamples {
                            if sound_device.sound_stream.is_tick() {
                                sound_device
                                    .sound_chip
                                    .tick(index, &mut sound_device.sound_stream);
                            }
                            let (l, r) = sound_device.sound_stream.pop();
                            buffer_l[buffer_pos + i] += l;
                            buffer_r[buffer_pos + i] += r;
                        }
                    }
                    _ => {
                        sound_device
                            .sound_chip
                            .update(index, buffer_l, buffer_r, numsamples, buffer_pos);
                    }
                }
            }
        }
    }

    pub fn update_all(
        &mut self,
        sound_chip_type: SoundChipType,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        if self.sound_device.contains_key(&sound_chip_type) {
            for index in 0..self.sound_device.get(&sound_chip_type).unwrap().len() {
                self.update(
                    sound_chip_type,
                    index,
                    buffer_l,
                    buffer_r,
                    numsamples,
                    buffer_pos,
                );
            }
        }
    }
}

///
/// Sound Device
///
pub struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_chip_type: SoundChipType,
    sound_stream: SoundStream,
}

///
/// Sound Stream
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

    pub fn is_tick(&self) -> bool {
        if self.sound_chip_tick_rate != self.output_sampling_rate
            && self.sound_chip_tick_pos > self.output_sampling_pos
        {
            return false;
        }
        true
    }

    pub fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.sound_chip_tick_pos =
            Self::next_pos(self.sound_chip_tick_pos, self.sound_chip_tick_step);
        self.now_chip_sampling_l = sampling_l;
        self.now_chip_sampling_r = sampling_r;
    }

    pub fn pop(&mut self) -> (f32, f32) {
        // TODO: upsampling
        self.output_sampling_pos =
            Self::next_pos(self.output_sampling_pos, self.output_sampling_step);
        (self.now_chip_sampling_l, self.now_chip_sampling_r)
    }

    fn next_pos(now: u64, step: u64) -> u64 {
        let next: u128 = (now + step).into();
        if next > u64::MAX.into() {
            return (u64::MAX as u128 - next) as u64;
        }
        next as u64
    }
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

pub type RomBank = Option<Rc<RefCell<RomSet>>>;

///
/// Rom
///
pub struct Rom {
    start_address: usize,
    end_address: usize,
    memory: Vec<u8>,
}

///
/// Rom sets
///
#[derive(Default)]
pub struct RomSet {
    rom: Vec<Rom>,
}

impl RomSet {
    pub fn new() -> RomSet {
        RomSet { rom: Vec::new() }
    }

    pub fn add_rom(&mut self, memory: &[u8], start_address: usize, end_address: usize) {
        // println!("rom: {:<08x} - {:<08x}, {:<08x}, {:<02x}", start_address, end_address, memory.len(), memory[0]);
        // to_vec(clone) is external SPI memory simulation.
        self.rom.push(Rom {
            start_address,
            end_address,
            memory: memory.to_vec(),
        });
    }

    pub fn read(&self, address: usize) -> u8 {
        for r in self.rom.iter() {
            if r.start_address <= address && r.end_address >= address {
                return r.memory[address - r.start_address];
            }
        }
        0
    }
}

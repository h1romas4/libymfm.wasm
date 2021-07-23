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
pub use crate::sound::ymfm::YmFm;
pub use crate::sound::ymfm::ChipType;

///
/// Device Name
///
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash)]
pub enum SoundChipType {
    YM2151,
    YM2203,
    YM2149,
    YM2612,
    YM2413,
    YM2602,
    SEGA315_5313,
    PWM,
    SEGAPCM,
}

///
/// Device common interface
///
pub trait SoundChip {
    fn new(sound_device_name: SoundChipType) -> Self where Self: Sized;
    fn init(&mut self, clock: u32) -> u32;
    fn reset(&mut self);
    fn write(&mut self, port: u32, data: u32);
    fn update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    );
}

struct SoundDevice {
    sound_chip: Box<dyn SoundChip>,
    sound_stream: SoundStream,
}

pub struct SoundSlot {
    internal_sampling_rate: u32,
    max_output_sample_size: usize,
    sound_device: HashMap<SoundChipType, Vec<SoundDevice>>,
}

impl SoundSlot {
    pub fn new(max_output_sample_size: usize) -> Self {
        SoundSlot {
            internal_sampling_rate: 96000,
            max_output_sample_size,
            sound_device: HashMap::new(),
        }
    }

    pub fn add(&mut self, sound_device_name: SoundChipType, clock: u32) -> usize {
        let mut sound_chip: Box<dyn SoundChip> = match sound_device_name {
            SoundChipType::YM2151 => Box::new(YmFm::new(SoundChipType::YM2151)),
            SoundChipType::YM2203 => Box::new(YmFm::new(SoundChipType::YM2203)),
            SoundChipType::YM2149 => Box::new(YmFm::new(SoundChipType::YM2149)),
            SoundChipType::YM2612 => Box::new(YmFm::new(SoundChipType::YM2612)),
            SoundChipType::YM2413 => Box::new(YmFm::new(SoundChipType::YM2413)),
            SoundChipType::YM2602 => todo!(),
            SoundChipType::SEGA315_5313 => Box::new(SN76489::new(SoundChipType::SEGA315_5313)),
            SoundChipType::PWM => Box::new(PWM::new(SoundChipType::PWM)),
            SoundChipType::SEGAPCM => Box::new(SEGAPCM::new(SoundChipType::SEGAPCM)),
        };
        // TODO: change to chip native sampling rate
        let device_sampling_rate = sound_chip.init(clock);
        // TODO: slot index 0
        self.sound_device.insert(sound_device_name, vec![SoundDevice {
            sound_chip,
            sound_stream: SoundStream::new(device_sampling_rate, self.internal_sampling_rate),
        }]);
        // TODO: return index no
        0
    }

    #[inline]
    pub fn find(&mut self, sound_device_name: SoundChipType, _no: usize) -> &mut dyn SoundChip {
        &mut *self.sound_device.get_mut(&sound_device_name).expect("sound device not found.")[0].sound_chip
    }

    pub fn write(&mut self, sound_device_name: SoundChipType, no: usize, port: u32, data: u32) {
        self.find(sound_device_name, no).write(port, data);
    }

    pub fn update(
        &mut self,
        sound_device_name: SoundChipType,
        no: usize,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        self.find(sound_device_name, no).update(buffer_l, buffer_r, numsamples, buffer_pos);
    }

    // pub fn set_rom(&mut self, sound_device_name: SoundChipType, no: usize, rombank: RomBank) {
    //     if sound_device_name == SoundChipType::SEGAPCM {
    //         RomDevice::set_rom(self.find(&sound_device_name, no), rombank);
    //     }
    // }
}
struct SoundStream {
    device_sampling_rate: u32,
    output_sampling_rate: u32,
    now_device_sampling_l: f32,
    now_device_sampling_r: f32,
    prv_device_sampling_l: f32,
    prv_device_sampling_r: f32,
    device_sampling_pos: usize,
    output_sampling_pos: usize,
}

impl SoundStream {
    pub fn new(device_sampling_rate: u32, output_sampling_rate: u32) -> Self {
        SoundStream {
            device_sampling_rate,
            output_sampling_rate,
            now_device_sampling_l: 0_f32,
            now_device_sampling_r: 0_f32,
            prv_device_sampling_l: 0_f32,
            prv_device_sampling_r: 0_f32,
            device_sampling_pos: 0,
            output_sampling_pos: 0,
        }
    }

    // pub fn push(&mut self, sampling_l: f32, sampling_r: f32) {
    //     // self.output_sampling_l[self.output_sampling_pos] = sampling_l;
    //     // self.output_sampling_r[self.output_sampling_pos] = sampling_r;

    //     // self.output_sampling_pos += 1;
    //     // if self.output_sampling_pos >= N {
    //     //     self.output_sampling_pos = 0;
    //     // }
    // }
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

pub trait RomDevice {
    fn set_rom(&mut self, rombank: RomBank);

    fn read_rom(rombank: &RomBank, address: usize) -> u8 {
        rombank.as_ref().unwrap().borrow_mut().read(address)
    }
}

///
/// Sound rom
///
pub struct Rom {
    start_address: usize,
    end_address: usize,
    memory: Vec<u8>,
}

///
/// Sound rom sets
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

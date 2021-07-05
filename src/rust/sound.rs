mod pwm;
mod segapcm;
mod sn76489;
mod ym3438;
mod ym2151;

use std::cell::RefCell;
use std::rc::Rc;

pub use crate::sound::pwm::PWM;
pub use crate::sound::segapcm::SEGAPCM;
pub use crate::sound::sn76489::SN76489;
pub use crate::sound::ym3438::YM3438;
pub use crate::sound::ym2151::YM2151;

///
/// import ymfm library
///
#[link(name = "ymfm")]
extern {
    fn ym2151_init(clock: u32);
    fn ym2151_write(reg: u32, dat: u8);
    fn ym2151_generate(emulated_time: i64, output_step: i64, buffer: *mut i32);
    // fn ym2203_init(clock: u32);
    // fn ym2203_write(reg: u32, dat: u8);
    // fn ym2203_generate(emulated_time: i64, output_step: i64, buffer: *mut i32);
}

///
/// Device Name
///
pub enum SoundDeviceName {
    YM3438,
    YM2612,
    SN76489,
    PWM,
    SEGAPCM,
    YM2151,
    YM2203,
}

///
/// Device common interface
///
pub trait SoundDevice<T> {
    fn new() -> Self;
    fn get_name(&self) -> SoundDeviceName;
    fn init(&mut self, sample_rate: u32, clock: u32);
    fn reset(&mut self);
    fn write(&mut self, port: u32, data: T);
    fn update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    );
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

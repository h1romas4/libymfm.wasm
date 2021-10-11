// license:BSD-3-Clause
use std::cell::RefCell;
use std::rc::Rc;

use super::rom::RomSet;
use super::stream::SoundStream;

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
    SN76489,
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
    fn tick(&mut self, index: usize, sound_stream: &mut dyn SoundStream);
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

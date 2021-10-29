// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use std::cell::RefCell;
use std::rc::Rc;

use super::rom::RomSet;
use super::rom::RomIndex;
use super::stream::SoundStream;

///
/// Sound chip type
///
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum SoundChipType {
    YM2149,
    YM2151,
    YM2203,
    YM2413,
    YM2608,
    YM2610,
    YM2612,
    YM3526,
    Y8950,
    YM3812,
    YMF262,
    YMF278B,
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
    fn set_rombank(&mut self, rom_index: RomIndex, rom_bank: RomBank);
    fn notify_add_rom(&mut self, rom_index: RomIndex, index_no: usize);
}

///
/// Read RomBank by address utility
///
pub fn read_rombank(rombank: &RomBank, address: usize) -> u8 {
    rombank.as_ref().unwrap().borrow().read(address)
}

///
/// Get Rom referance utility
///
pub fn get_rom_ref(rombank: &RomBank, index_no: usize) -> (*const u8, usize, usize) {
    rombank.as_ref().unwrap().borrow().ger_rom_ref(index_no)
}

pub type RomBank = Option<Rc<RefCell<RomSet>>>;

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use super::rom::RomBank;
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
    OKIM6258,
    C140,
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
    fn write(&mut self, index: usize, port: u32, data: u32, sound_stream: &mut dyn SoundStream);
    fn tick(&mut self, index: usize, sound_stream: &mut dyn SoundStream);
    fn set_rom_bank(&mut self, rom_index: RomIndex, rom_bank: RomBank);
    fn notify_add_rom(&mut self, rom_index: RomIndex, index_no: usize);
}

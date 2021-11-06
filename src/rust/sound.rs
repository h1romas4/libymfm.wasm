// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
mod slot;
mod device;
mod sound_chip;
mod stream;
mod rom;
mod data_stream;

mod chip_ymfm;
mod chip_sn76496;
mod chip_pwm;
mod chip_segapcm;
mod chip_okim6258;

pub use crate::sound::sound_chip::SoundChipType as SoundChipType;
pub use crate::sound::slot::SoundSlot as SoundSlot;
pub use crate::sound::rom::RomIndex as RomIndex;

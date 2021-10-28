// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
mod slot;
mod interface;
mod stream;
mod rom;

mod chip_ymfm;
mod chip_sn76496;
mod chip_pwm;
mod chip_segapcm;

pub use crate::sound::interface::SoundChipType as SoundChipType;
pub use crate::sound::slot::SoundSlot as SoundSlot;
pub use crate::sound::rom::RomIndex as RomIndex;

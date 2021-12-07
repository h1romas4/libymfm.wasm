// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
mod meta;
mod vgmplay;
mod vgmmeta;
mod xgmmeta;
mod gd3meta;

pub use crate::driver::vgmplay::VgmPlay as VgmPlay;
pub use crate::driver::vgmplay::VGM_TICK_RATE as VGM_TICK_RATE;

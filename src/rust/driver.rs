// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
//! There are defined sound chip drivers for several music formats.
mod meta;
mod vgmplay;
mod xgmplay;
mod vgmmeta;
mod xgmmeta;
mod gd3meta;

pub use crate::driver::vgmplay::VgmPlay as VgmPlay;
pub use crate::driver::xgmplay::XgmPlay as XgmPlay;
pub use crate::driver::vgmplay::VGM_TICK_RATE as VGM_TICK_RATE;
pub use crate::driver::xgmplay::XGM_DEFAULT_TICK_RATE as XGM_DEFAULT_TICK_RATE;
pub use crate::driver::xgmmeta::XgmHeader as XgmHeader;
pub use crate::driver::vgmmeta::VgmHeader as VgmHeader;
pub use crate::driver::gd3meta::Gd3 as Gd3;

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
//! A module that defines an interface for WebAssembly.
//! There is a bindgen module for wasm-bindgen, and an extern "C" basic module.
#[cfg(feature = "bindgen")]
mod bindgen;
#[cfg(feature = "basic")]
mod basic;

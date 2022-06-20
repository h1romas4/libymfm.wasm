// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use flate2::read::GzDecoder;
use std::collections::HashMap;
use std::io::prelude::*;

use crate::driver::gd3meta::Gd3;
use crate::driver::meta::Jsonlize;
use crate::driver::vgmmeta;
use crate::driver::vgmmeta::VgmHeader;
use crate::sound::{RomIndex, RomBusType, SoundChipType, SoundSlot};

pub const VGM_TICK_RATE: u32 = 44100;

///
/// VGM Driver
///
pub struct VgmPlay {
    sound_slot: SoundSlot,
    vgm_pos: usize,
    vgm_loop: usize,
    vgm_loop_offset: usize,
    vgm_loop_count: usize,
    vgm_end: bool,
    vgm_data: Vec<u8>,
    vgm_header: Option<VgmHeader>,
    vgm_gd3: Option<Gd3>,
    data_block_id: usize,
    data_stream: HashMap<usize, (SoundChipType, usize)>,
    ym2612_pcm_pos: usize,
    ym2612_pcm_offset: usize,
    remain_tick_count: usize,
}

#[allow(dead_code)]
impl VgmPlay {
    ///
    /// Create sound driver.
    ///
    pub fn new(sound_slot: SoundSlot, vgm_file: &[u8]) -> Result<Self, &'static str> {
        let mut vgmplay = VgmPlay {
            sound_slot,
            vgm_pos: 0,
            vgm_loop: 0,
            vgm_loop_offset: 0,
            vgm_loop_count: 0,
            vgm_end: false,
            vgm_data: Vec::new(),
            vgm_header: None,
            vgm_gd3: None,
            data_block_id: 0,
            data_stream: HashMap::new(),
            ym2612_pcm_pos: 0,
            ym2612_pcm_offset: 0,
            remain_tick_count: 0,
        };
        // clone vgm_file and soundchip init
        vgmplay.init(vgm_file)?;

        Ok(vgmplay)
    }

    ///
    /// Return sampling_l buffer referance.
    ///
    pub fn get_sampling_l_ref(&self) -> *const f32 {
        self.sound_slot.get_output_sampling_l_ref()
    }

    ///
    /// Return sampling buffer referance.
    ///
    pub fn get_sampling_r_ref(&self) -> *const f32 {
        self.sound_slot.get_output_sampling_r_ref()
    }

    ///
    /// Return s16le sampling buffer referance.
    ///
    pub fn get_output_sampling_s16le_ref(&mut self) -> *const i16 {
        self.sound_slot.get_output_sampling_s16le_ref()
    }

    ///
    /// Get VGM meta.
    ///
    pub fn get_vgm_meta(&self) -> (&VgmHeader, &Gd3) {
        (
            self.vgm_header.as_ref().unwrap(/* There always is */),
            self.vgm_gd3.as_ref().unwrap(/* There always is */),
        )
    }

    ///
    /// Get VGM header JSON.
    ///
    pub fn get_vgm_header_json(&self) -> String {
        self.vgm_header.as_ref().unwrap(/* There always is */).get_json()
    }

    ///
    /// Get VGM header GD3 JSON.
    ///
    pub fn get_vgm_gd3_json(&self) -> String {
        self.vgm_gd3.as_ref().unwrap(/* There always is */).get_json()
    }

    ///
    /// Play Sound.
    ///
    pub fn play(&mut self, repeat: bool) -> usize {
        while !self.sound_slot.is_stream_filled() && !self.vgm_end {
            for _ in 0..self.remain_tick_count {
                self.sound_slot.update(1);
                self.remain_tick_count -= 1;
                if self.sound_slot.is_stream_filled() {
                    break;
                }
            }
            if self.remain_tick_count == 0 {
                self.remain_tick_count = self.parse_vgm(repeat) as usize;
            };
        }
        self.sound_slot.stream();

        if self.vgm_loop_count == std::usize::MAX {
            self.vgm_loop_count = 0;
        }
        if self.vgm_end {
            std::usize::MAX
        } else {
            self.vgm_loop_count
        }
    }

    ///
    /// Extract vgz and initialize sound driver.
    ///
    fn init(&mut self, vgm_file: &[u8]) -> Result<(), &'static str> {
        // try vgz extract to vgm_data
        self.extract(vgm_file);

        // parse vgm header
        match vgmmeta::parse_vgm_meta(&self.vgm_data) {
            Ok((header, gd3)) => {
                self.vgm_header = Some(header);
                self.vgm_gd3 = Some(gd3);
            }
            Err(message) => return Err(message),
        };

        let header = self.vgm_header.as_ref().unwrap();

        self.vgm_loop = header.offset_loop as usize;
        self.vgm_loop_offset = (0x1c + header.offset_loop) as usize;
        self.vgm_pos = (0x34 + header.vgm_data_offset) as usize;

        if header.clock_ym2612 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2612,
                self.number_of_chip(header.clock_ym2612),
                header.clock_ym2612 & 0x3fffffff,
            );
        }
        if header.clock_ym2151 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2151,
                self.number_of_chip(header.clock_ym2151),
                header.clock_ym2151 & 0x3fffffff,
            );
        }
        if header.clock_ym2203 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2203,
                self.number_of_chip(header.clock_ym2203),
                header.clock_ym2203 & 0x3fffffff,
            );
        }
        if header.clock_ym2413 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2413,
                self.number_of_chip(header.clock_ym2413),
                header.clock_ym2413 & 0x3fffffff,
            );
        }
        if header.clock_ay8910 != 0 {
            let clock_ay8910: u32;
            if header.clock_ym2151 != 0 {
                // X1 Turbo sync YM2151
                clock_ay8910 = header.clock_ym2151;
            } else if header.clock_ym2413 != 0 {
                // MSX sync YM2413
                clock_ay8910 = header.clock_ym2413;
            } else {
                clock_ay8910 = header.clock_ay8910 * 2;
            }
            self.sound_slot.add_sound_device(
                SoundChipType::YM2149,
                self.number_of_chip(header.clock_ay8910),
                clock_ay8910 & 0x3fffffff,
            );
        }
        if header.clock_ym2608 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2608,
                self.number_of_chip(header.clock_ym2608),
                header.clock_ym2608 & 0x3fffffff,
            );
        }
        if header.clock_ym2610_b != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2610, // TODO:
                self.number_of_chip(header.clock_ym2610_b),
                header.clock_ym2610_b & 0x3fffffff,
            );
        }
        if header.clock_ym3812 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM3812,
                self.number_of_chip(header.clock_ym3812),
                header.clock_ym3812 & 0x3fffffff,
            );
        }
        if header.clock_ym3526 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM3526,
                self.number_of_chip(header.clock_ym3526),
                header.clock_ym3526 & 0x3fffffff,
            );
        }
        if header.clock_y8950 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::Y8950,
                self.number_of_chip(header.clock_y8950),
                header.clock_y8950 & 0x3fffffff,
            );
        }
        if header.clock_ymf262 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YMF262,
                self.number_of_chip(header.clock_ymf262),
                header.clock_ymf262 & 0x3fffffff,
            );
        }
        if header.clock_ymf278_b != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YMF278B,
                self.number_of_chip(header.clock_ymf278_b),
                header.clock_ymf278_b & 0x3fffffff,
            );
        }
        if header.clock_sn76489 != 0 {
            self.sound_slot
                .add_sound_device(SoundChipType::SEGAPSG, 1, header.clock_sn76489);
        }
        if header.clock_pwm != 0 {
            self.sound_slot
                .add_sound_device(SoundChipType::PWM, 1, header.clock_pwm);
        }
        if header.clock_sega_pcm != 0 {
            self.sound_slot
                .add_sound_device(SoundChipType::SEGAPCM, 1, header.clock_sega_pcm);
        }
        if header.clock_okim6258 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::OKIM6258,
                self.number_of_chip(header.clock_okim6258),
                header.clock_okim6258 & 0x3fffffff,
            );
            let flag = header.okmi6258_flag;
            for i in 0..self.number_of_chip(header.clock_okim6258) {
                self.sound_slot.write(
                    SoundChipType::OKIM6258,
                    i,
                    0x10, /* set_divider */
                    (flag & 3) as u32,
                );
                self.sound_slot.write(
                    SoundChipType::OKIM6258,
                    i,
                    0x11, /* set_outbits */
                    if (flag & 4) != 0 { 10 } else { 12 },
                );
                self.sound_slot.write(
                    SoundChipType::OKIM6258,
                    i,
                    0x12, /* set_type */
                    if flag & 2 != 0 { 1 } else { 0 },
                );
            }
        }
        if header.clock_c140 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::C140,
                self.number_of_chip(header.clock_c140),
                header.clock_c140 & 0x3fffffff,
            );
            let rom_bus_type = match header.c140_chip_type {
                0x00 => Some(RomBusType::C140_TYPE_SYSTEM2),
                0x01 => Some(RomBusType::C140_TYPE_SYSTEM21),
                0x02 => Some(RomBusType::C140_TYPE_ASIC219),
                _ => None,
            };
            self.sound_slot.set_rom_bus_type(
                RomIndex::C140_ROM,
                rom_bus_type,
            );
        }

        Ok(())
    }

    fn extract(&mut self, vgm_file: &[u8]) {
        let mut d = GzDecoder::new(vgm_file);
        if d.read_to_end(&mut self.vgm_data).is_err() {
            self.vgm_data = vgm_file.to_vec();
        }
    }

    fn get_vgm_u8(&mut self) -> u8 {
        let ret = self.vgm_data[self.vgm_pos];
        self.vgm_pos += 1;
        ret
    }

    fn get_vgm_u16(&mut self) -> u16 {
        u16::from(self.get_vgm_u8()) + (u16::from(self.get_vgm_u8()) << 8)
    }

    fn get_vgm_u32(&mut self) -> u32 {
        u32::from(self.get_vgm_u8())
            + (u32::from(self.get_vgm_u8()) << 8)
            + (u32::from(self.get_vgm_u8()) << 16)
            + (u32::from(self.get_vgm_u8()) << 24)
    }

    fn number_of_chip(&self, clock: u32) -> usize {
        if clock & 0x40000000 != 0 {
            2
        } else {
            1
        }
    }

    fn parse_vgm(&mut self, repeat: bool) -> u16 {
        let mut wait: u16 = 0;

        let command = self.get_vgm_u8();
        match command {
            0x50 => {
                let dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::SEGAPSG, 0, 0, dat.into());
            }
            0x51 | 0xa1 => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2413,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x52 | 0xa2 => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2612,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x53 | 0xa3 => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2612,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x54 | 0xa4 => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2151,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x55 | 0xa5 => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2203,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x56 | 0xa6 => {
                // YM2608 port 0 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2608,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x57 | 0xa7 => {
                // YM2608 port 1 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2608,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x58 | 0xa8 => {
                // YM2610 port 0 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2610,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x59 | 0xa9 => {
                // YM2610 port 1 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2610,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x5a | 0xaa => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM3812,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x5b | 0xab => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM3526,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x5c | 0xac => {
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::Y8950,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x5e | 0xae => {
                // YMF262 port 0 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YMF262,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x5f | 0xaf => {
                // YMF262 port 1 write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YMF262,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x61 => {
                wait = self.get_vgm_u16();
            }
            0x62 => {
                wait = 735;
            }
            0x63 => {
                wait = 882;
            }
            0x66 => {
                if self.vgm_loop == 0 {
                    self.vgm_end = true;
                } else if repeat {
                    self.vgm_pos = self.vgm_loop_offset;
                    self.vgm_loop_count += 1;
                } else {
                    self.vgm_end = true;
                }
            }
            0x67 => {
                // 0x66 compatibility command to make older players stop parsing the stream
                self.get_vgm_u8();
                let data_type = self.get_vgm_u8();
                let data_length = self.get_vgm_u32() as usize;
                let data_block_pos = self.vgm_pos;
                self.vgm_pos += data_length as usize;
                // handle data block
                if (0x00..=0x3f).contains(&data_type) {
                    // add data block (support uncompressed)
                    self.sound_slot.add_data_block(
                        self.data_block_id,
                        &self.vgm_data[data_block_pos..data_block_pos + data_length],
                    );
                    // data_block_id is a sequence id in vgm
                    self.data_block_id += 1;
                } else if (0x80..=0xbf).contains(&data_type) {
                    // ROM/RAM Image dumps
                    let _real_rom_size = u32::from_le_bytes(
                        self.vgm_data[data_block_pos..(data_block_pos + 4)]
                            .try_into()
                            .unwrap(),
                    );
                    let start_address = u32::from_le_bytes(
                        self.vgm_data[(data_block_pos + 4)..(data_block_pos + 8)]
                            .try_into()
                            .unwrap(),
                    );
                    let mut data_size = data_length - 8;
                    if data_size == 0 {
                        data_size = 1;
                    }
                    let start_address = start_address as usize;
                    let rom_index: RomIndex = Self::get_rom_index(data_type);
                    self.sound_slot.add_rom(
                        rom_index,
                        &self.vgm_data[(data_block_pos + 8)..(data_block_pos + 8) + data_size],
                        start_address,
                        start_address + data_size - 1,
                    );
                }
            }
            0x70..=0x7f => {
                wait = ((command & 0x0f) + 1).into();
            }
            0x80..=0x8f => {
                // YM2612 port 0 address 2A write from the data bank, then wait n samples;
                // n can range from 0 to 15. Note that the wait is n, NOT n+1.
                // See also command 0xE0.
                let ym2612_block = self
                    .sound_slot
                    .get_data_block(/* YM2612 data block 0 fixed */ 0);
                let data = ym2612_block[self.ym2612_pcm_pos + self.ym2612_pcm_offset];
                self.sound_slot
                    .write(SoundChipType::YM2612, 0, 0x2a, data.into());
                self.ym2612_pcm_offset += 1;
                wait = (command & 0x0f).into();
            }
            0x90 => {
                // Setup Stream Control
                // 0x90 ss tt pp cc
                // 0x90 00 02 00 2a
                let data_stream_id = self.get_vgm_u8() as usize;
                let chip_type = self.get_vgm_u8();
                let write_port = self.get_vgm_u8() as u32;
                let write_reg = self.get_vgm_u8() as u32;
                // create new stream
                let sound_chip_type = Self::get_stream_chip_type(chip_type);
                let sound_chip_index = (chip_type >> 7) as usize;
                if let Some(sound_chip_type) = sound_chip_type {
                    self.sound_slot.add_data_stream(
                        sound_chip_type,
                        sound_chip_index,
                        data_stream_id,
                        write_port,
                        write_reg,
                    );
                    self.data_stream
                        .insert(data_stream_id, (sound_chip_type, sound_chip_index));
                }
            }
            0x91 => {
                // Set Stream Data
                // 0x91 ss dd ll bb
                // 0x91 00 00 01 2a
                let data_stream_id = self.get_vgm_u8() as usize;
                let data_block_id = self.get_vgm_u8() as usize;
                let /* TODO: */ _step_base = self.get_vgm_u8();
                let /* TODO: */ _step_size = self.get_vgm_u8();
                // assosiate data block to stream
                if let Some((sound_chip_type, sound_chip_index)) =
                    self.data_stream.get(&data_stream_id)
                {
                    self.sound_slot.attach_data_block_to_stream(
                        *sound_chip_type,
                        *sound_chip_index,
                        data_stream_id,
                        data_block_id,
                    );
                }
            }
            0x92 => {
                // Set Stream Frequency
                // 0x92 ss ff ff ff ff
                // 0x92 00 40 1f 00 00 (8KHz)
                let data_stream_id = self.get_vgm_u8() as usize;
                let frequency = self.get_vgm_u32();
                if let Some((sound_chip_type, sound_chip_index)) =
                    self.data_stream.get(&data_stream_id)
                {
                    self.sound_slot.set_data_stream_frequency(
                        *sound_chip_type,
                        *sound_chip_index,
                        data_stream_id,
                        frequency,
                    );
                }
            }
            0x93 => {
                // Start Stream
                // 0x93 ss aa aa aa aa mm ll ll ll ll
                // 0x93 00 aa aa aa aa 01 ll ll ll ll
                let data_stream_id = self.get_vgm_u8() as usize;
                let data_stream_start_offset = self.get_vgm_u32() as usize;
                let /* TODO */ _length_mode = self.get_vgm_u8();
                let pcm_stream_length = self.get_vgm_u32() as usize;
                // initalize stream and start playback (set pcm_stream_length)
                if let Some((sound_chip_type, sound_chip_index)) =
                    self.data_stream.get(&data_stream_id)
                {
                    self.sound_slot.start_data_stream(
                        *sound_chip_type,
                        *sound_chip_index,
                        data_stream_id,
                        data_stream_start_offset,
                        pcm_stream_length,
                    );
                }
            }
            0x94 => {
                // Stop Stream
                // 0x94 ss
                let data_stream_id = self.get_vgm_u8() as usize;
                // stop stream (set pcm_stream_length to 0)
                if let Some((sound_chip_type, sound_chip_index)) =
                    self.data_stream.get(&data_stream_id)
                {
                    self.sound_slot.stop_data_stream(
                        *sound_chip_type,
                        *sound_chip_index,
                        data_stream_id,
                    );
                }
            }
            0x95 => {
                // Start Stream (fast call)
                // 0x95 ss bb bb ff
                let data_stream_id = self.get_vgm_u8() as usize;
                let data_block_id = self.get_vgm_u16() as usize;
                let /* TODO */ _flags = self.get_vgm_u8();
                // initalize stream and start playback (set pcm_stream_length to data block size)
                if let Some((sound_chip_type, sound_chip_index)) =
                    self.data_stream.get(&data_stream_id)
                {
                    self.sound_slot.start_data_stream_fast(
                        *sound_chip_type,
                        *sound_chip_index,
                        data_stream_id,
                        data_block_id,
                    );
                }
            }
            0xa0 => {
                // TODO: YM2149 as AY8910, write
                let reg = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2149, 0, reg as u32, dat.into());
            }
            0xb2 => {
                // 0xB2 ad dd
                // PWM, write value ddd to register a (d is MSB, dd is LSB)
                let raw1 = self.get_vgm_u8();
                let raw2 = self.get_vgm_u8();
                let channel = (raw1 & 0xf0) >> 4_u8;
                let dat: u16 = (raw1 as u16 & 0x0f) << 8 | raw2 as u16;
                self.sound_slot
                    .write(SoundChipType::PWM, 0, channel as u32, dat.into());
            }
            0xb7 => {
                // 0xb7: aa dd: OKIM6258, write value dd to register aa
                let offset = self.get_vgm_u8();
                let dat = self.get_vgm_u8();
                if offset & 0x80 != 0 {
                    self.sound_slot.write(
                        SoundChipType::OKIM6258,
                        1,
                        (offset & 0x7f) as u32,
                        dat.into(),
                    );
                } else {
                    self.sound_slot.write(
                        SoundChipType::OKIM6258,
                        0,
                        (offset & 0x7f) as u32,
                        dat.into(),
                    );
                }
            }
            0xc0 => {
                let offset = self.get_vgm_u16();
                let dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::SEGAPCM, 0, u32::from(offset), dat.into());
            }
            0xd4 => {
                // 0xd4: pp aa dd: C140, write value dd to register ppaa
                let offset = u16::from(self.get_vgm_u8()) << 8 | u16::from(self.get_vgm_u8());
                let dat = self.get_vgm_u8();
                if offset & 0x8000 != 0 {
                    self.sound_slot.write(
                        SoundChipType::C140,
                        1,
                        (offset & 0x7fff) as u32,
                        dat.into(),
                    );
                } else {
                    self.sound_slot.write(
                        SoundChipType::C140,
                        0,
                        (offset & 0x7fff) as u32,
                        dat.into(),
                    );
                }
            }
            0xe0 => {
                // YM2612 data block 0
                let pcm_pos = self.get_vgm_u32() as usize;
                self.ym2612_pcm_pos = pcm_pos as usize;
                self.ym2612_pcm_offset = 0;
            }
            // unsupport
            0x30..=0x3f | 0x4f => {
                // 0x4f: dd: Game Gear PSG stereo, write dd to port 0x06
                self.get_vgm_u8();
            }
            0x40..=0x4e | 0x5d | 0xb0..=0xb6 | 0xb8..=0xbf => {
                // 0x5d: aa dd: YMZ280B, write value dd to register aa
                // 0xb0: aa dd: RF5C68, write value dd to register aa
                // 0xb1: aa dd: RF5C164, write value dd to register aa
                // 0xb3: aa dd: GameBoy DMG, write value dd to register aa
                // 0xb4: aa dd: NES APU, write value dd to register aa
                // 0xb5: aa dd: MultiPCM, write value dd to register aa
                // 0xb6: aa dd: uPD7759, write value dd to register aa
                // 0xb8: aa dd: OKIM6295, write value dd to register aa
                // 0xb9: aa dd: HuC6280, write value dd to register aa
                // 0xba: aa dd: K053260, write value dd to register aa
                // 0xbb: aa dd: Pokey, write value dd to register aa
                // 0xbc: aa dd: WonderSwan, write value dd to register aa
                // 0xbd: aa dd: SAA1099, write value dd to register aa
                // 0xbe: aa dd: ES5506, write value dd to register aa
                // 0xbf: aa dd: GA20, write value dd to register aa
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0xc9..=0xcf | 0xd7..=0xdf | 0xc1..=0xc8 | 0xd1..=0xd3 | 0xd5..=0xd6 => {
                // 0xc1: bbaa dd: RF5C68, write value dd to memory offset aabb
                // 0xc2: bbaa dd: RF5C164, write value dd to memory offset aabb
                // 0xc3: cc bbaa: MultiPCM, write set bank offset aabb to channel cc
                // 0xc4: mmll rr: QSound, write value mmll to register rr (mm - data MSB, ll - data LSB)
                // 0xc5: mmll dd: SCSP, write value dd to memory offset mmll (mm - offset MSB, ll - offset LSB)
                // 0xc6: mmll dd: WonderSwan, write value dd to memory offset mmll (mm - offset MSB, ll - offset LSB)
                // 0xc7: mmll dd: VSU, write value dd to memory offset mmll (mm - offset MSB, ll - offset LSB)
                // 0xc8: mmll dd: X1-010, write value dd to memory offset mmll (mm - offset MSB, ll - offset LSB)
                // 0xd1: pp aa dd: YMF271, port pp, write value dd to register aa
                // 0xd2: pp aa dd: SCC1, port pp, write value dd to register aa
                // 0xd3: pp aa dd: K054539, write value dd to register ppaa
                // 0xd5: pp aa dd: ES5503, write value dd to register ppaa
                // 0xd6: pp aa dd: ES5506, write value aadd to register pp
                self.get_vgm_u16();
                self.get_vgm_u8();
            }
            _ => {
                #[cfg(not(target_arch = "wasm32"))]
                println!(
                    "unknown cmd at {:x}: {:x}",
                    self.vgm_pos - 1,
                    self.vgm_data[self.vgm_pos - 1]
                );
            }
        }
        wait
    }

    fn get_rom_index(data_type: u8) -> RomIndex {
        match data_type {
            0x80 => RomIndex::SEGAPCM_ROM,
            0x81 => RomIndex::YM2608_DELTA_T,
            0x82 => RomIndex::YM2610_ADPCM,
            0x83 => RomIndex::YM2610_DELTA_T,
            0x84 => RomIndex::YMF278B_ROM,
            0x87 => RomIndex::YMF278B_RAM,
            0x88 => RomIndex::Y8950_ROM,
            0x8d => RomIndex::C140_ROM,
            _ => RomIndex::NOT_SUPPOTED,
        }
    }

    fn get_stream_chip_type(chip_type: u8) -> Option<SoundChipType> {
        match chip_type & 0x7f {
            0 => Some(SoundChipType::SN76489),
            1 => Some(SoundChipType::YM2413),
            2 => Some(SoundChipType::YM2612),
            3 => Some(SoundChipType::YM2151),
            4 => Some(SoundChipType::SEGAPCM),
            5 => None, // rf5c68
            6 => Some(SoundChipType::YM2203),
            7 => Some(SoundChipType::YM2608),
            8 => Some(SoundChipType::YM2610),
            9 => Some(SoundChipType::YM3812),
            10 => Some(SoundChipType::YM3526),
            11 => Some(SoundChipType::Y8950),
            12 => Some(SoundChipType::YMF262),
            13 => Some(SoundChipType::YMF278B),
            14 => None, // ymf271
            15 => None, // ymz280b
            16 => None, // rf5c16
            17 => Some(SoundChipType::PWM),
            18 => Some(SoundChipType::YM2149),
            19 => None, // gb_dmg
            20 => None, // nes_apu
            21 => None, // multi_pcm
            22 => None, // upd7759
            23 => Some(SoundChipType::OKIM6258),
            24 => None, // okim6295
            25 => None, // k051649
            26 => None, // k054539
            27 => None, // huc6280
            28 => None, // c140
            29 => None, // k053260
            30 => None, // pokey
            31 => None, // qsound
            32 => None, // scsp
            33 => None, // wonder_swan
            34 => None, // vsu
            35 => None, // saa1099
            36 => None, // es5503
            37 => None, // es5506
            38 => None, // x1_010
            39 => None, // c352
            40 => None, // ga20
            _ => None,  // not supported stream
        }
    }
}

///
/// cargo test -- --nocapture
///
#[cfg(test)]
mod tests {
    use crate::sound::SoundSlot;

    use super::VgmPlay;
    use std::fs::File;
    use std::io::{Read, Write};

    const MAX_SAMPLE_SIZE: usize = 2048;

    #[test]
    fn sn76489_1() {
        play("./docs/vgm/segapsg-2.vgz")
    }

    #[test]
    fn pwm_1() {
        play("./docs/vgm/pwm.vgz")
    }

    #[test]
    fn ym2612_1() {
        play("./docs/vgm/ym2612.vgm");
    }

    #[test]
    fn ym2612_3() {
        play("./docs/vgm/ym2612-2.vgz")
    }

    #[test]
    fn ym2612_4() {
        play("./docs/vgm/ym2612-datablock.vgm") // data block
    }

    #[test]
    fn ym2151_1() {
        println!("1st vgm instance");
        play("./docs/vgm/ym2151.vgm");
        println!("2nd vgm instance(drop and create)");
        play("./docs/vgm/ym2151.vgm");
    }

    #[test]
    fn ym2608_1() {
        play("./docs/vgm/ym2608-2.vgz")
    }

    #[test]
    fn ym2610_1() {
        play("./docs/vgm/ym2610.vgz")
    }

    #[test]
    fn ym2610_2() {
        play("./docs/vgm/ym2610-2.vgz")
    }

    #[test]
    fn ym2610_3() {
        play("./docs/vgm/ym2610-3.vgz")
    }

    #[test]
    fn segapcm_1() {
        play("./docs/vgm/segapcm-2.vgz")
    }

    #[test]
    fn okim6258_1() {
        play("./docs/vgm/okim6258.vgz")
    }

    #[test]
    fn okim6258_2() {
        play("./docs/vgm/okim6258-2.vgz")
    }

    #[test]
    fn okim6258_3() {
        play("./docs/vgm/okim6258-3.vgz")
    }

    #[test]
    fn okim6258_4() {
        play("./docs/vgm/okim6258-4-ng.vgz")
    }

    #[test]
    fn okim6258_5() {
        play("./docs/vgm/okim6258-5-ng.vgz")
    }

    #[test]
    fn c140_1() {
        play("./docs/vgm/c140-1.vgz")
    }

    #[test]
    fn c140_2() {
        play("./docs/vgm/c140-2.vgz")
    }

    #[test]
    fn c140_3() {
        play("./docs/vgm/c140-3.vgz")
    }

    fn play(filepath: &str) {
        println!("Play start! {}", filepath);

        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();

        // read vgm
        let mut vgmplay =
            VgmPlay::new(SoundSlot::new(44100, 44100, MAX_SAMPLE_SIZE), &buffer).unwrap();

        let mut pcm = File::create("output.pcm").expect("file open error.");
        // play
        // ffplay -f f32le -ar 44100 -ac 2 output.pcm
        // ffplay -f f32le -ar 96000 -ac 2 output.pcm
        // ffmpeg -f f32le -ar 96000 -ac 2 -i output.pcm output-96000.wav
        #[allow(clippy::absurd_extreme_comparisons)]
        while vgmplay.play(false) <= 0 {
            for i in 0..MAX_SAMPLE_SIZE {
                let sampling_l = vgmplay.get_sampling_l_ref();
                let sampling_r = vgmplay.get_sampling_r_ref();
                unsafe {
                    let slice_l = std::slice::from_raw_parts(sampling_l.add(i) as *const u8, 4);
                    let slice_r = std::slice::from_raw_parts(sampling_r.add(i) as *const u8, 4);
                    pcm.write_all(slice_l).expect("stdout error");
                    pcm.write_all(slice_r).expect("stdout error");
                }
            }
        }
        println!("Play end! {} (vgm instance drop)", filepath);
    }
}

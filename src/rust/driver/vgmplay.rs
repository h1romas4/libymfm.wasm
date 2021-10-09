// license:BSD-3-Clause
use flate2::read::GzDecoder;
use std::convert::TryInto;
use std::io::prelude::*;

use crate::driver::vgmmeta::parse_vgm_meta;
use crate::driver::vgmmeta::Gd3;
use crate::driver::vgmmeta::Jsonlize;
use crate::driver::vgmmeta::VgmHeader;
use crate::sound::{SoundChipType, SoundSlot};

pub const VGM_TICK_RATE: u32 = 44100;

pub struct VgmPlay {
    sound_slot: SoundSlot,
    vgm_pos: usize,
    vgm_loop: usize,
    vgm_loop_offset: usize,
    vgm_loop_count: usize,
    vgm_end: bool,
    vgm_file: Vec<u8>,
    vgm_data: Vec<u8>,
    vgm_header: VgmHeader,
    vgm_gd3: Gd3,
    pcm_origin_pos: usize,
    pcm_pos: usize,
    pcm_offset: usize,
    pcm_stream_sample_count: u32,
    pcm_stream_sampling_pos: u32,
    pcm_stream_length: usize,
    pcm_stream_pos_init: usize,
    pcm_stream_pos: usize,
    pcm_stream_offset: usize,
    remain_tick_count: usize,
}

#[allow(dead_code)]
impl VgmPlay {
    ///
    /// Create sound driver.
    ///
    pub fn new(sound_slot: SoundSlot, vgm_file_size: usize) -> Self {
        VgmPlay {
            sound_slot,
            vgm_pos: 0,
            vgm_loop: 0,
            vgm_loop_offset: 0,
            vgm_loop_count: 0,
            vgm_end: false,
            vgm_file: vec![0; vgm_file_size],
            vgm_data: Vec::new(),
            vgm_header: VgmHeader::default(),
            vgm_gd3: Gd3::default(),
            pcm_origin_pos: 0,
            pcm_pos: 0,
            pcm_offset: 0,
            pcm_stream_sample_count: 0,
            pcm_stream_sampling_pos: 0,
            pcm_stream_length: 0,
            pcm_stream_pos_init: 0,
            pcm_stream_pos: 0,
            pcm_stream_offset: 0,
            remain_tick_count: 0,
        }
    }

    ///
    /// Return vgmfile buffer referance.
    ///
    pub fn get_vgmfile_ref(&mut self) -> *mut u8 {
        self.vgm_file.as_mut_ptr()
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
    /// get_vgm_meta
    ///
    pub fn get_vgm_meta(&self) -> (&VgmHeader, &Gd3) {
        (&self.vgm_header, &self.vgm_gd3)
    }

    ///
    /// get_vgm_header_json
    ///
    pub fn get_vgm_header_json(&self) -> String {
        self.vgm_header.get_json()
    }

    ///
    /// get_vgm_header_json
    ///
    pub fn get_vgm_gd3_json(&self) -> String {
        self.vgm_gd3.get_json()
    }

    ///
    /// extract vgz and initialize sound driver.
    ///
    pub fn init(&mut self) -> Result<(), &'static str> {
        // try vgz extract
        self.extract();

        match parse_vgm_meta(&self.vgm_data) {
            Ok((header, gd3)) => {
                self.vgm_header = header;
                self.vgm_gd3 = gd3;
            }
            Err(message) => return Err(message),
        };

        self.vgm_loop = self.vgm_header.offset_loop as usize;
        self.vgm_loop_offset = (0x1c + self.vgm_header.offset_loop) as usize;
        self.vgm_pos = (0x34 + self.vgm_header.vgm_data_offset) as usize;

        if self.vgm_header.clock_ym2612 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2612,
                self.number_of_chip(self.vgm_header.clock_ym2612),
                self.vgm_header.clock_ym2612 & 0x3fffffff,
            );
        }
        if self.vgm_header.clock_ym2151 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2151,
                self.number_of_chip(self.vgm_header.clock_ym2151),
                self.vgm_header.clock_ym2151 & 0x3fffffff,
            );
        }
        if self.vgm_header.clock_ym2203 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2203,
                self.number_of_chip(self.vgm_header.clock_ym2203),
                self.vgm_header.clock_ym2203 & 0x3fffffff,
            );
        }
        if self.vgm_header.clock_ym2413 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::YM2413,
                self.number_of_chip(self.vgm_header.clock_ym2413),
                self.vgm_header.clock_ym2413 & 0x3fffffff,
            );
        }
        if self.vgm_header.clock_ay8910 != 0 {
            // TODO: YM2149 - AY8910 clock hack (* 4 ?)
            let clock_ay8910: u32;
            if self.vgm_header.clock_ym2151 != 0 {
                // TODO: X1 Turbo sync YM2151
                clock_ay8910 = self.vgm_header.clock_ym2151 * 4;
            } else if self.vgm_header.clock_ym2413 != 0 {
                // TODO: MSX sync YM2413
                clock_ay8910 = self.vgm_header.clock_ym2413 * 4;
            } else {
                clock_ay8910 = self.vgm_header.clock_ay8910 * 8;
            }
            self.sound_slot.add_sound_device(
                SoundChipType::YM2149,
                self.number_of_chip(self.vgm_header.clock_ay8910),
                clock_ay8910 & 0x3fffffff,
            );
        }
        if self.vgm_header.clock_sn76489 != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::SEGAPSG,
                1,
                self.vgm_header.clock_sn76489,
            );
        }
        if self.vgm_header.clock_pwm != 0 {
            self.sound_slot
                .add_sound_device(SoundChipType::PWM, 1, self.vgm_header.clock_pwm);
        }
        if self.vgm_header.sega_pcm_clock != 0 {
            self.sound_slot.add_sound_device(
                SoundChipType::SEGAPCM,
                1,
                self.vgm_header.sega_pcm_clock,
            );
        }

        Ok(())
    }

    ///
    /// Play Sound.
    ///
    pub fn play(&mut self, repeat: bool) -> usize {
        while self.sound_slot.ready() && !self.vgm_end {
            for _ in 0..self.remain_tick_count {
                self.update_pcm_stream();
                self.sound_slot.update(1);
                self.remain_tick_count -= 1;
                if !self.sound_slot.ready() {
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

    fn update_pcm_stream(&mut self) {
        // YM2612 straming pcm update
        if self.pcm_stream_pos_init == self.pcm_stream_pos && self.pcm_stream_length > 0 {
            self.pcm_stream_sampling_pos = 0;
        }
        if self.pcm_stream_length > 0
            && (self.pcm_stream_sampling_pos % self.pcm_stream_sample_count) as usize == 0
        {
            self.sound_slot.write(
                SoundChipType::YM2612,
                0,
                0x2a,
                self.vgm_data[self.pcm_origin_pos + self.pcm_stream_pos + self.pcm_stream_offset].into(),
            );
            self.pcm_stream_length -= 1;
            self.pcm_stream_pos += 1;
        }
        if self.pcm_stream_length > 0 {
            self.pcm_stream_sampling_pos += 1;
        }
    }

    fn extract(&mut self) {
        let mut d = GzDecoder::new(self.vgm_file.as_slice());
        if d.read_to_end(&mut self.vgm_data).is_err() {
            self.vgm_data = self.vgm_file.clone();
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
        let command: u8;
        let reg: u8;
        let dat: u8;
        let mut wait: u16 = 0;

        command = self.get_vgm_u8();
        match command {
            0x50 => {
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::SEGAPSG, 0, 0, dat.into());
            }
            0x51 | 0xa1 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2413,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x52 | 0xa2 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2612,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x53 | 0xa3 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2612,
                    (command >> 7) as usize,
                    reg as u32 | 0x100,
                    dat.into(),
                );
            }
            0x54 | 0xa4 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2151,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x55 | 0xa5 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot.write(
                    SoundChipType::YM2203,
                    (command >> 7) as usize,
                    reg as u32,
                    dat.into(),
                );
            }
            0x56 | 0xa6 => {
                // TODO: YM2608 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x57 | 0xa7 => {
                // TODO: YM2608 port 1 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x58 | 0xa8 => {
                // TODO: YM2610 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x59 | 0xa9 => {
                // TODO: YM2610 port 1 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5a | 0xaa => {
                // TODO: YM3812 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5b | 0xab => {
                // TODO: YM3528 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5c | 0xac => {
                // TODO: Y8950 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5e | 0xae => {
                // TODO: YMF262 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5f | 0xaf => {
                // TODO: YMF262 port 1 write
                self.get_vgm_u8();
                self.get_vgm_u8();
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
                let size = self.get_vgm_u32();
                let data_pos = self.vgm_pos;
                self.vgm_pos += size as usize;
                // handle data block
                if (0x00..=0x3f).contains(&data_type) {
                    // data of recorded streams (uncompressed) (for ym2612)
                    self.pcm_origin_pos = data_pos;
                } else if (0x80..=0xbf).contains(&data_type) {
                    // ROM/RAM Image dumps (0x80 segapcm)
                    let rom_size = u32::from_le_bytes(
                        self.vgm_data[data_pos..(data_pos + 4)].try_into().unwrap(),
                    );
                    let start_address = u32::from_le_bytes(
                        self.vgm_data[(data_pos + 4)..(data_pos + 8)]
                            .try_into()
                            .unwrap(),
                    );
                    let mut data_size: usize = 0;
                    if start_address < rom_size {
                        data_size = u32::min(size - 8, rom_size - start_address) as usize;
                    }
                    let start_address = start_address as usize;
                    self.sound_slot.add_rom(
                        data_type as usize,
                        &self.vgm_data[(data_pos + 8)..(data_pos + 8) + data_size],
                        start_address,
                        start_address + data_size - 1,
                    );
                }
            }
            0x70..=0x7f => {
                wait = ((command & 0x0f) + 1).into();
            }
            0x80..=0x8f => {
                // YM2612 PCM
                wait = (command & 0x0f).into();
                self.sound_slot.write(
                    SoundChipType::YM2612,
                    0,
                    0x2a,
                    self.vgm_data[self.pcm_origin_pos + self.pcm_pos + self.pcm_offset].into(),
                );
                self.pcm_offset += 1;
            }
            0x90 => {
                // TODO: respect stream no
                // Setup Stream Control
                // 0x90 ss tt pp cc
                // 0x90 00 02 00 2a
                self.get_vgm_u32();
            }
            0x91 => {
                // Set Stream Data
                // 0x91 ss dd ll bb
                // 0x91 00 00 01 2a
                self.get_vgm_u32();
            }
            0x92 => {
                // Set Stream Frequency
                // 0x92 ss ff ff ff ff
                // 0x92 00 40 1f 00 00 (8KHz)
                self.get_vgm_u8();
                self.pcm_stream_sample_count = VGM_TICK_RATE / self.get_vgm_u32();
            }
            0x93 => {
                // Start Stream
                // 0x93 ss aa aa aa aa mm ll ll ll ll
                // 0x93 00 aa aa aa aa 01 ll ll ll ll
                self.get_vgm_u8();
                self.pcm_stream_pos_init = self.get_vgm_u32() as usize;
                self.pcm_stream_pos = self.pcm_stream_pos_init;
                self.get_vgm_u8();
                self.pcm_stream_length = self.get_vgm_u32() as usize;
                self.pcm_stream_offset = 0;
            }
            0x94 => {
                // Stop Stream
                // 0x94 ss
                self.get_vgm_u8();
                self.pcm_stream_length = 0;
            }
            0x95 => {
                // Start Stream (fast call)
                // 0x95 ss bb bb ff
                self.get_vgm_u8();
                self.get_vgm_u16();
                self.get_vgm_u8();
            }
            0xa0 => {
                // TODO: AY8910, write
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
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
            0xc0 => {
                let offset = self.get_vgm_u16();
                let dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::SEGAPCM, 0, u32::from(offset), dat.into());
            }
            0xe0 => {
                self.pcm_pos = self.get_vgm_u32() as usize;
                self.pcm_offset = 0;
            }
            // unsupport
            0x30..=0x3f | 0x4f => {
                // 0x4f: dd: Game Gear PSG stereo, write dd to port 0x06
                self.get_vgm_u8();
            }
            0x40..=0x4e | 0x5d | 0xb0..=0xbf => {
                // 0x5d: aa dd: YMZ280B, write value dd to register aa
                // 0xb0: aa dd: RF5C68, write value dd to register aa
                // 0xb1: aa dd: RF5C164, write value dd to register aa
                // 0xb2: aa dd: PWM, write value ddd to register a (d is MSB, dd is LSB)
                // 0xb3: aa dd: GameBoy DMG, write value dd to register aa
                // 0xb4: aa dd: NES APU, write value dd to register aa
                // 0xb5: aa dd: MultiPCM, write value dd to register aa
                // 0xb6: aa dd: uPD7759, write value dd to register aa
                // 0xb7: aa dd: OKIM6258, write value dd to register aa
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
            0xc9..=0xcf | 0xd7..=0xdf | 0xc1..=0xc8 | 0xd1..=0xd6 => {
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
                // 0xd4: pp aa dd: C140, write value dd to register ppaa
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
        play("./docs/vgm/segapsg.vgm")
    }

    #[test]
    fn ym2612_1() {
        println!("1st vgm instance");
        play("./docs/vgm/ym2612.vgm");
        println!("2nd vgm instance(drop and create)");
        play("./docs/vgm/ym2612-ng.vgz")
    }

    #[test]
    fn ym2612_2() {
        play("./docs/vgm/2612/20.vgz");
        play("./docs/vgm/2612/21.vgz");
        play("./docs/vgm/2612/22.vgz");
        play("./docs/vgm/2612/23.vgz");
        play("./docs/vgm/2612/24.vgz");
        play("./docs/vgm/2612/25.vgz");
        play("./docs/vgm/2612/26.vgz");
        play("./docs/vgm/2612/27.vgz");
        play("./docs/vgm/2612/28.vgz");
        play("./docs/vgm/2612/29.vgz");
        play("./docs/vgm/2612/30.vgz");
        play("./docs/vgm/2612/31.vgz");
        play("./docs/vgm/2612/32.vgz");
        play("./docs/vgm/2612/33.vgz");
        play("./docs/vgm/2612/34.vgz");
        play("./docs/vgm/2612/35.vgz");
        play("./docs/vgm/2612/36.vgz");
        play("./docs/vgm/2612/37.vgz");
        play("./docs/vgm/2612/38.vgz");
        play("./docs/vgm/2612/39.vgz");
        play("./docs/vgm/2612/40.vgz");
        play("./docs/vgm/2612/41.vgz");
        play("./docs/vgm/2612/42.vgz");
    }

    #[test]
    fn ym2151_1() {
        println!("1st vgm instance");
        play("./docs/vgm/ym2151.vgm");
        println!("2nd vgm instance(drop and create)");
        play("./docs/vgm/ym2151.vgm");
    }

    #[test]
    fn segapcm_1() {
        play("./docs/vgm/segapcm-2.vgz")
    }

    fn play(filepath: &str) {
        println!("Play start! {}", filepath);
        // load sn76489 vgm file
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();

        let mut vgmplay = VgmPlay::new(
            SoundSlot::new(44100, 96000, MAX_SAMPLE_SIZE),
            file.metadata().unwrap().len() as usize,
        );
        // set vgmdata (Wasm simulation)
        let vgmdata_ref = vgmplay.get_vgmfile_ref();
        for (i, buf) in buffer.iter().enumerate() {
            unsafe {
                *vgmdata_ref.add(i) = *buf;
            }
        }

        // init & sample
        vgmplay.init().unwrap();
        let sampling_l = vgmplay.get_sampling_l_ref();
        let sampling_r = vgmplay.get_sampling_r_ref();

        let mut pcm = File::create("output.pcm").expect("file open error.");
        // play
        // ffplay -f f32le -ar 96000 -ac 2 output.pcm
        // ffmpeg -f f32le -ar 96000 -ac 2 -i output.pcm output-96000.wav
        #[allow(clippy::absurd_extreme_comparisons)]
        while vgmplay.play(false) <= 0 {
            for i in 0..MAX_SAMPLE_SIZE {
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

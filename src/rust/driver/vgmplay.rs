use flate2::read::GzDecoder;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::prelude::*;
use std::rc::Rc;

use crate::console_log;
use crate::driver::metadata::parse_vgm_meta;
use crate::driver::metadata::Gd3;
use crate::driver::metadata::Jsonlize;
use crate::driver::metadata::VgmHeader;

use crate::sound::SoundChipType;
use crate::sound::{RomSet, SoundSlot};

pub struct VgmPlay {
    sound_slot: SoundSlot,
    sound_romset: HashMap<usize, Rc<RefCell<RomSet>>>,
    sample_rate: u32,
    vgm_pos: usize,
    data_pos: usize,
    pcm_pos: usize,
    pcm_offset: usize,
    pcm_stream_sample_count: u32,
    pcm_stream_sampling_pos: u32,
    pcm_stream_length: usize,
    pcm_stream_pos_init: usize,
    pcm_stream_pos: usize,
    pcm_stream_offset: usize,
    remain_frame_size: usize,
    vgm_loop: usize,
    vgm_loop_offset: usize,
    vgm_loop_count: usize,
    vgm_end: bool,
    vgm_file: Vec<u8>,
    vgm_data: Vec<u8>,
    max_sample_size: usize,
    sampling_l: Vec<f32>,
    sampling_r: Vec<f32>,
    vgm_header: VgmHeader,
    vgm_gd3: Gd3,
}

#[allow(dead_code)]
impl VgmPlay {
    ///
    /// Create sound driver.
    ///
    pub fn new(sample_rate: u32, max_sample_size: usize, vgm_file_size: usize) -> Self {
        VgmPlay {
            sound_slot: SoundSlot::new(max_sample_size),
            sound_romset: HashMap::new(),
            sample_rate,
            vgm_pos: 0,
            data_pos: 0,
            pcm_pos: 0,
            pcm_offset: 0,
            pcm_stream_sample_count: 0,
            pcm_stream_sampling_pos: 0,
            pcm_stream_length: 0,
            pcm_stream_pos_init: 0,
            pcm_stream_pos: 0,
            pcm_stream_offset: 0,
            remain_frame_size: 0,
            vgm_loop: 0,
            vgm_loop_offset: 0,
            vgm_loop_count: 0,
            vgm_end: false,
            vgm_file: vec![0; vgm_file_size],
            vgm_data: Vec::new(),
            max_sample_size,
            sampling_l: vec![0_f32; max_sample_size],
            sampling_r: vec![0_f32; max_sample_size],
            vgm_header: VgmHeader::default(),
            vgm_gd3: Gd3::default(),
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
        self.sampling_l.as_ptr()
    }

    ///
    /// Return sampling buffer referance.
    ///
    pub fn get_sampling_r_ref(&self) -> *const f32 {
        self.sampling_r.as_ptr()
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

        if self.vgm_header.clock_sn76489 != 0 {
            self.sound_slot
                .add(SoundChipType::SEGAPSG, self.vgm_header.clock_sn76489);
        }
        if self.vgm_header.clock_ym2612 != 0 {
            self.sound_slot
                .add(SoundChipType::YM2612, self.vgm_header.clock_ym2612);
        }
        if self.vgm_header.clock_pwm != 0 {
            self.sound_slot
                .add(SoundChipType::PWM, self.vgm_header.clock_pwm);
        }
        if self.vgm_header.sega_pcm_clock != 0 {
            self.sound_slot
                .add(SoundChipType::SEGAPCM, self.vgm_header.sega_pcm_clock);
            // TODO:
            // let romset = Rc::new(RefCell::new(RomSet::new()));
            // RomDevice::set_rom(&mut self.sound_device_segapcm, Some(romset.clone()));
            // self.sound_romset.insert(0x80, romset); // 0x80 segapcm
        }
        if self.vgm_header.clock_ym2151 != 0 {
            self.sound_slot
                .add(SoundChipType::YM2151, self.vgm_header.clock_ym2151);
        }
        if self.vgm_header.clock_ym2203 != 0 {
            self.sound_slot
                .add(SoundChipType::YM2203, self.vgm_header.clock_ym2203);
        }
        if self.vgm_header.clock_ym2413 != 0 {
            self.sound_slot
                .add(SoundChipType::YM2413, self.vgm_header.clock_ym2413);
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
            self.sound_slot.add(SoundChipType::YM2149, clock_ay8910);
        }

        Ok(())
    }

    ///
    /// play
    ///
    pub fn play(&mut self, repeat: bool) -> usize {
        let mut frame_size: usize;
        let mut update_frame_size: usize;
        let mut buffer_pos: usize;

        // clear buffer
        for i in 0..self.max_sample_size {
            self.sampling_l[i] = 0_f32;
            self.sampling_r[i] = 0_f32;
        }

        buffer_pos = 0;
        while {
            if self.remain_frame_size > 0 {
                frame_size = self.remain_frame_size;
            } else {
                frame_size = self.parse_vgm(repeat) as usize;
            }
            if buffer_pos + frame_size < self.max_sample_size {
                update_frame_size = frame_size;
            } else {
                update_frame_size = self.max_sample_size - buffer_pos;
            }
            if self.pcm_stream_pos_init == self.pcm_stream_pos && self.pcm_stream_length > 0 {
                self.pcm_stream_sampling_pos = 0;
            }
            let base_buffer_pos = buffer_pos;
            for _ in 0..update_frame_size {
                // YM2612 straming pcm update
                if self.vgm_header.clock_ym2612 != 0 {
                    // pcm update
                    if self.pcm_stream_length > 0
                        && (self.pcm_stream_sampling_pos % self.pcm_stream_sample_count) as usize
                            == 0
                    {
                        self.sound_slot.write(
                            SoundChipType::YM2612,
                            0,
                            0x2a,
                            self.vgm_data
                                [self.data_pos + self.pcm_stream_pos + self.pcm_stream_offset]
                                .into(),
                        );
                        self.pcm_stream_length -= 1;
                        self.pcm_stream_pos += 1;
                    }
                    // mix each YM2612 1 sampling
                    self.sound_slot.update(
                        SoundChipType::YM2612,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        1,
                        buffer_pos,
                    );
                }
                if self.remain_frame_size > 0 {
                    self.remain_frame_size -= 1;
                }
                buffer_pos += 1;
                self.pcm_stream_sampling_pos += 1;
            }
            if update_frame_size != 0 {
                if self.vgm_header.clock_sn76489 != 0 {
                    self.sound_slot.update(
                        SoundChipType::SEGAPSG,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.clock_pwm != 0 {
                    self.sound_slot.update(
                        SoundChipType::PWM,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.sega_pcm_clock != 0 {
                    self.sound_slot.update(
                        SoundChipType::SEGAPCM,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.clock_ym2151 != 0 {
                    self.sound_slot.update(
                        SoundChipType::YM2151,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.clock_ym2203 != 0 {
                    self.sound_slot.update(
                        SoundChipType::YM2203,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.clock_ay8910 != 0 {
                    self.sound_slot.update(
                        SoundChipType::YM2149,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
                if self.vgm_header.clock_ym2413 != 0 {
                    self.sound_slot.update(
                        SoundChipType::YM2413,
                        0,
                        &mut self.sampling_l,
                        &mut self.sampling_r,
                        update_frame_size,
                        base_buffer_pos,
                    );
                }
            }
            buffer_pos < self.max_sample_size && !self.vgm_end
        } {}
        self.remain_frame_size = frame_size - update_frame_size;

        if self.vgm_loop_count == std::usize::MAX {
            self.vgm_loop_count = 0;
        }
        if self.vgm_end {
            std::usize::MAX
        } else {
            self.vgm_loop_count
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
            0x51 => {
                // TODO: YM2413 write
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2413, 0, reg as u32, dat.into());
            }
            0x52 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2612, 0, reg as u32, dat.into());
            }
            0x53 => {
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2612, 0, reg as u32 | 0x100, dat.into());
            }
            0x54 => {
                // TODO: YM2151 write
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2151, 0, reg as u32, dat.into());
            }
            0x55 => {
                // TODO: YM2203 write
                reg = self.get_vgm_u8();
                dat = self.get_vgm_u8();
                self.sound_slot
                    .write(SoundChipType::YM2203, 0, reg as u32, dat.into());
            }
            0x56 => {
                // TODO: YM2608 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x57 => {
                // TODO: YM2608 port 1 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x58 => {
                // TODO: YM2610 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x59 => {
                // TODO: YM2610 port 1 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5a => {
                // TODO: YM3812 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5b => {
                // TODO: YM3528 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5c => {
                // TODO: Y8950 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5e => {
                // TODO: YMF262 port 0 write
                self.get_vgm_u8();
                self.get_vgm_u8();
            }
            0x5f => {
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
                    self.data_pos = data_pos;
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
                    self.add_rom(
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
                    self.vgm_data[self.data_pos + self.pcm_pos + self.pcm_offset].into(),
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
                self.pcm_stream_sample_count = self.sample_rate / self.get_vgm_u32();
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
                // 0xc0: bbaa dd: Sega PCM, write value dd to memory offset aabb
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
                #[cfg(feature = "console_error_panic_hook")]
                console_log!(
                    "unknown cmd at {:x}: {:x}",
                    self.vgm_pos - 1,
                    self.vgm_data[self.vgm_pos - 1]
                );
                #[cfg(not(feature = "console_error_panic_hook"))]
                println!(
                    "unknown cmd at {:x}: {:x}",
                    self.vgm_pos - 1,
                    self.vgm_data[self.vgm_pos - 1]
                );
            }
        }
        wait
    }

    fn add_rom(&self, index: usize, memory: &[u8], start_address: usize, end_address: usize) {
        if self.sound_romset.contains_key(&index) {
            self.sound_romset.get(&index).unwrap().borrow_mut().add_rom(
                memory,
                start_address,
                end_address,
            );
        }
    }
}

///
/// cargo test -- --nocapture
///
#[cfg(test)]
mod tests {
    use super::VgmPlay;
    use std::fs::File;
    use std::io::{Read, Write};

    const MAX_SAMPLE_SIZE: usize = 4096;

    #[test]
    fn sn76489_1() {
        play("./docs/vgm/sn76489.vgm")
    }

    #[test]
    fn ym2612_1() {
        println!("1st vgm instance");
        play("./docs/vgm/ym2612.vgm");
        println!("2nd vgm instance(drop and create)");
        play("./docs/vgm/ym2612-ng.vgz")
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
        play("./docs/vgm/segapcm.vgm")
    }

    fn play(filepath: &str) {
        println!("Play start!");
        // load sn76489 vgm file
        let mut file = File::open(filepath).unwrap();
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer).unwrap();

        let mut vgmplay = VgmPlay::new(
            44100,
            MAX_SAMPLE_SIZE,
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
        // ffplay -f f32le -ar 44100 -ac 2 output.pcm
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
        println!("Play end! (vgm instance drop)");
    }
}

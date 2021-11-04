// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
/**
 * Rust SEGAPCM emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Converted from:
 *  MAME
 *  copyright-holders:Hiromitsu Shioya, Olivier Galibert
 *  https://github.com/mamedev/mame/blob/master/src/devices/sound/segapcm.cpp
 *  rev. 70743c6fb2602a5c2666c679b618706eabfca2ad
 */
use crate::sound::SoundChipType;
use super::{
    data_stream::{DataBlock, DataStream},
    rom::{read_rombank, RomBank},
    sound_chip::SoundChip,
    stream::{convert_sample_i2f, SoundStream},
    RomIndex,
};

#[allow(clippy::upper_case_acronyms)]
pub struct SEGAPCM {
    clock: u32,
    ram: [u8; 0x800],
    rombank: RomBank,
    bankshift: u8,
    bankmask: u8,
    low: [u8; 16],
}

impl SEGAPCM {
    fn from() -> Self {
        Self {
            clock: 0,
            bankshift: 12,
            bankmask: 0x70,
            ram: [0xff; 0x800],
            rombank: None,
            low: [0; 16],
        }
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.clock = clock;
        // chip native samplig rate
        self.clock / 128
    }

    fn reset(&mut self) {
        self.ram = [0xff; 0x800];
        self.low = [0; 16];
    }

    fn update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        // reg      function
        // ------------------------------------------------
        // 0x00     ?
        // 0x01     ?
        // 0x02     volume left
        // 0x03     volume right
        // 0x04     loop address (08-15)
        // 0x05     loop address (16-23)
        // 0x06     end address
        // 0x07     address delta
        // 0x80     ?
        // 0x81     ?
        // 0x82     ?
        // 0x83     ?
        // 0x84     current address (08-15), 00-07 is internal?
        // 0x85     current address (16-23)
        // 0x86     bit 0: channel disable?
        //          bit 1: loop disable
        //          other bits: bank
        // 0x87     ?
        for ch in 0..16 {
            let regs = &mut self.ram[ch * 8..];

            /* only process active channels */
            if regs[0x86] & 1 == 0 {
                let offset: i32 = i32::from(regs[0x86] & self.bankmask) << self.bankshift;
                let mut addr: u32 = u32::from(regs[0x85]) << 16
                    | u32::from(regs[0x84]) << 8
                    | u32::from(self.low[ch]);
                let loops: u32 = u32::from(regs[0x05]) << 16 | u32::from(regs[0x04]) << 8;
                let end: u32 = u32::from(regs[6]) + 1;

                for i in 0..numsamples {
                    /* handle looping if we've hit the end */
                    if (addr >> 16) == end {
                        if regs[0x86] & 2 != 0 {
                            regs[0x86] |= 1;
                            break;
                        } else {
                            addr = loops;
                        }
                    }
                    /* fetch the sample */
                    let v = read_rombank(&self.rombank, offset as usize + (addr >> 8) as usize);
                    let v: i32 = i32::from(v) - 0x80;
                    /* apply panning and advance */
                    buffer_l[buffer_pos + i] += convert_sample_i2f(v * (regs[2] & 0x7f) as i32);
                    buffer_r[buffer_pos + i] += convert_sample_i2f(v * (regs[3] & 0x7f) as i32);
                    addr = (addr + regs[7] as u32) & 0xffffff;
                }
                /* store back the updated address */
                regs[0x84] = (addr >> 8) as u8;
                regs[0x85] = (addr >> 16) as u8;
                self.low[ch] = if regs[0x86] & 1 != 0 { 0 } else { addr as u8 };
            }
        }
    }

    fn write(&mut self, offset: u32, data: u8) {
        self.ram[offset as usize & 0x07ff] = data;
    }
}

impl SoundChip for SEGAPCM {
    fn new(_sound_device_name: SoundChipType) -> Self {
        SEGAPCM::from()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.init(clock)
    }

    fn reset(&mut self) {
        self.reset();
    }

    fn write(&mut self, _: usize, offset: u32, data: u32, _: &mut dyn SoundStream) {
        self.write(offset, data as u8);
    }

    fn tick(
        &mut self,
        _: usize,
        sound_stream: &mut dyn SoundStream,
        _data_stream: &Option<&mut DataStream>,
        _data_block: &Option<&DataBlock>,
    ) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.update(&mut l, &mut r, 1, 0);
        sound_stream.push(l[0], r[0]);
    }

    fn set_rom_bank(&mut self, _ /* SEGAPCM has only one RomBank */: RomIndex, rombank: RomBank) {
        self.rombank = rombank;
    }

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {
        /* nothing to do */
    }
}

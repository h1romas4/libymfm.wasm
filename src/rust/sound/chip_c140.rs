// license:BSD-3-Clause
/**
 * Rust C140 ports by
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Porting from:
 *  C140 emulation by R. Belmont
 *  https://github.com/mamedev/mame/blob/master/src/devices/sound/c140.cpp
 *  rev. 1bc5484698bb746c6f9f7dae58b005d2472f1215
 */

/**
 * Original C140 emulation Copyright
 */
// license:BSD-3-Clause
// copyright-holders:R. Belmont

/*
c140.cpp

Simulator based on AMUSE sources.
The C140 sound chip is used by Namco System 2 and System 21
The 219 ASIC (which incorporates a modified C140) is used by Namco NA-1 and NA-2
This chip controls 24 channels (C140) or 16 (219) of PCM.
16 bytes are associated with each channel.
Channels can be 8 bit compressed PCM, or 12 bit signed PCM.

TODO:
- What does the INT0 pin do? Normally Namco tied it to VOL0 (with VOL1 = VCC).
- Acknowledge A9 bit (9th address bit) of host interface
- Verify data bus bits of C219
- Verify C219 LFSR algorithm (same as c352.cpp?)
- Verify unknown mode bits (0x40 for C140, 0x02 for C219)

--------------

    ASIC "219" notes

    On the 219 ASIC used on NA-1 and NA-2, the high registers have the following
    meaning instead:
    0x1f7: bank for voices 0-3
    0x1f1: bank for voices 4-7
    0x1f3: bank for voices 8-11
    0x1f5: bank for voices 12-15

    Some games (bkrtmaq, xday2) write to 0x1fd for voices 12-15 instead.  Probably the bank registers
    mirror at 1f8, in which case 1ff is also 0-3, 1f9 is also 4-7, 1fb is also 8-11, and 1fd is also 12-15.

    Each bank is 0x20000 (128k), and the voice addresses on the 219 are all multiplied by 2.
    Additionally, the 219's base pitch is the same as the C352's (42667).  But these changes
    are IMO not sufficient to make this a separate file - all the other registers are
    fully compatible.

    Finally, the 219 only has 16 voices.
*/
/*
    2000.06.26  CAB     fixed compressed pcm playback
    2002.07.20  R. Belmont   added support for multiple banking types
    2006.01.08  R. Belmont   added support for NA-1/2 "219" derivative
    2020.05.06  cam900       Implement some features from QuattroPlay sources, by superctr
*/
use std::{cell::RefCell, rc::Rc};
use super::{
    rom::{read_word, Decoder, RomBank, set_rom_bus, RomBus},
    sound_chip::SoundChip,
    stream::{convert_sample_i2f, SoundStream},
    RomBusType, RomIndex, SoundChipType,
};

const MAX_VOICE: usize = 24;
const ASIC_219_BANKS: [i16; 4] = [0x1f7, 0x1f1, 0x1f3, 0x1f5];

#[allow(non_snake_case)]
pub struct C140 {
    sample_rate: i32,
    baserate: i32,
    reg: [u8; 0x200],
    pcmtbl: [i16; 256],
    voi: [C140Voice; MAX_VOICE],
    // int1_timer
    rom_bank: RomBank,
    rom_decoder: RomBus<C140RomDecoder>,
}

#[derive(Default, Copy, Clone)]
struct C140Voice {
    ptoffset: i32,
    pos: i32,
    key: i32,
    //--work
    lastdt: i32,
    prevdt: i32,
    dltdt: i32,
    //--reg
    // rvol: i32,
    // lvol: i32,
    // frequency: i32,
    bank: i32,
    mode: i32,

    sample_start: i32,
    sample_end: i32,
    sample_loop: i32,
}

#[repr(C)]
struct VoiceRegisters {
    volume_right: u8,
    volume_left: u8,
    frequency_msb: u8,
    frequency_lsb: u8,
    bank: u8,
    mode: u8,
    start_msb: u8,
    start_lsb: u8,
    end_msb: u8,
    end_lsb: u8,
    loop_msb: u8,
    loop_lsb: u8,
    reserved: [u8; 4],
}

impl C140 {
    fn new() -> Self {
        C140 {
            sample_rate: 0,
            baserate: 0,
            reg: [0; 0x200],
            pcmtbl: [0; 256],
            voi: [C140Voice::default(); MAX_VOICE],
            // int1_timer
            rom_bank: None,
            rom_decoder: None,
        }
    }

    pub fn device_start(&mut self) {
        // generate mulaw table (Verified from Wii Virtual Console Arcade Knuckle Heads)
        for i in 0..256 {
            let j: i8 = i as i8;
            let s1: i8 = j & 7;
            let s2: i8 = (j >> 3).abs() & 31;

            let mut pcmtbl: i64;
            pcmtbl = ((0x80 << s1) as u16 & 0xff00) as i64;
            pcmtbl += (i16::from(s2) << (if s1 != 0 { s1 + 3 } else { 4 })) as i64;

            if j < 0 {
                pcmtbl = -pcmtbl;
            }
            self.pcmtbl[i] = pcmtbl as i16;
        }
    }

    pub fn device_clock_changed(&mut self, clock: i32) -> i32 {
        self.sample_rate = clock;
        self.baserate = self.sample_rate;

        /* allocate a pair of buffers to mix into - 1 second's worth should be more than enough */
        // self.mixer_buffer_left = Some(vec![0; self.sample_rate as usize]);
        // self.mixer_buffer_right = Some(vec![0; self.sample_rate as usize]);

        self.sample_rate
    }

    pub fn rom_bank_updated(&mut self) {}

    pub fn sound_stream_update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        let mut dt: i32;

        let pbase: f32 = self.baserate as f32 * 2.0_f32 / self.sample_rate as f32;

        let mut lmix: i16;
        let mut rmix: i16;

        // let samples: i32 = self.sample_rate;
        /* for libymfm (1 tick) */
        let samples: i32 = 1;

        /* zap the contents of the mixer buffer */
        lmix = 0;
        rmix = 0;

        //--- audio update
        for i in 0..24 {
            let v: &mut C140Voice = &mut self.voi[i];
            // const struct voice_registers *vreg = (struct voice_registers *)&m_REG[i * 16];
            let vreg = &self.reg[i * 16..] as *const [u8] as *const VoiceRegisters;

            if v.key != 0 {
                let frequency: u16 = unsafe {
                    ((((*vreg).frequency_msb) as u16) << 8) | ((*vreg).frequency_lsb) as u16
                };

                /* Abort voice if no frequency value set */
                if frequency == 0 {
                    continue;
                }

                /* Delta =  frequency * ((8MHz/374)*2 / sample rate) */
                let delta: i32 = (frequency as f32 * pbase) as i32;

                /* Calculate left/right channel volumes */
                let lvol: i32 = unsafe { (((*vreg).volume_left) as i32 * 32) / MAX_VOICE as i32 }; //32ch -> 24ch
                let rvol: i32 = unsafe { (((*vreg).volume_right) as i32 * 32) / MAX_VOICE as i32 };

                /* Retrieve sample start/end and calculate size */
                let st = v.sample_start;
                let ed = v.sample_end;
                let sz = ed - st;

                /* Retrieve base pointer to the sample data */
                let sample_data = Self::find_sample(st, v.bank);

                /* Fetch back previous data pointers */
                let mut offset = v.ptoffset;
                let mut pos = v.pos;
                let mut lastdt = v.lastdt;
                let mut prevdt = v.prevdt;
                let mut dltdt = v.dltdt;

                /* linear or compressed 8bit signed PCM */
                for _ in 0..samples {
                    offset += delta;
                    let cnt = (offset >> 16) & 0x7fff;
                    offset &= 0xffff;
                    pos += cnt;
                    /* Check for the end of the sample */
                    if pos >= sz {
                        /* Check if its a looping sample, either stop or loop */
                        if Self::ch_looped(v) {
                            pos = v.sample_loop - st;
                        } else {
                            v.key = 0;
                            break;
                        }
                    }

                    if cnt != 0 {
                        let sample: u16 =
                            read_word(&self.rom_bank, (sample_data + pos) as usize) & 0xfff0; // 12bit
                        prevdt = lastdt;
                        lastdt = ((if Self::ch_mulaw(v) {
                            self.pcmtbl[((sample >> 8) & 0xff) as usize]
                        } else {
                            sample as i16
                        }) >> 4) as i32;
                        dltdt = lastdt - prevdt;
                    }

                    /* Caclulate the sample value */
                    dt = ((dltdt * offset) >> 16) + prevdt;

                    /* Write the data to the sample buffers */
                    lmix += ((dt * lvol) >> (5 + 4)) as i16;
                    rmix += ((dt * rvol) >> (5 + 4)) as i16;
                }

                /* Save positional data for next callback */
                v.ptoffset = offset;
                v.pos = pos;
                v.lastdt = lastdt;
                v.prevdt = prevdt;
                v.dltdt = dltdt;
            }
        }

        /* render to MAME's stream buffer */
        for i in 0..samples as usize {
            buffer_l[i] = convert_sample_i2f(lmix as i32 * 4);
            buffer_r[i] = convert_sample_i2f(rmix as i32 * 4);
        }
    }

    pub fn c140_w(&mut self, offset: usize, data: u8) {
        let offset = offset & 0x1ff;

        self.reg[offset] = data;
        if offset < 0x180 {
            let ch = (offset >> 4) as usize;
            let v: &mut C140Voice = &mut self.voi[ch];

            if offset & 0xf == 0x5 {
                if data & 0x80 != 0 {
                    let vreg = &self.reg[(offset & 0x1f0) as usize..] as *const [u8]
                        as *const VoiceRegisters;
                    v.key = 1;
                    v.ptoffset = 0;
                    v.pos = 0;
                    v.lastdt = 0;
                    v.prevdt = 0;
                    v.dltdt = 0;
                    v.bank = unsafe { (*vreg).bank as i32 };
                    v.mode = data as i32;

                    let sample_loop: u32 =
                        unsafe { (u32::from((*vreg).loop_msb) << 8) + (*vreg).loop_lsb as u32 };
                    let start: u32 =
                        unsafe { (u32::from((*vreg).start_msb) << 8) + (*vreg).start_lsb as u32 };
                    let end: u32 =
                        unsafe { (u32::from((*vreg).end_msb) << 8) + (*vreg).end_lsb as u32 };
                    v.sample_loop = sample_loop as i32;
                    v.sample_start = start as i32;
                    v.sample_end = end as i32;
                } else {
                    v.key = 0;
                }
            }
        } /* else if offset == 0x1fa {
               unimplemented!("init1_timer 1");
          } else if offset == 0x1fe {
               unimplemented!("init1_timer 2");
          } */
    }

    fn find_sample(adrs: i32, bank: i32) -> i32 {
        (bank << 16) + adrs
    }

    #[inline]
    fn ch_looped(v: &C140Voice) -> bool {
        ((v.mode >> 4) & 0x1) == 0x1
    }

    #[inline]
    fn ch_mulaw(v: &C140Voice) -> bool {
        ((v.mode >> 3) & 0x1) == 0x1
    }
}

pub struct C219 {
    sample_rate: i32,
    baserate: i32,
    reg: [u8; 0x200],
    pcmtbl: [i16; 256],
    voi: [C140Voice; MAX_VOICE],
    lfsr: u16,
    // int1_timer
    rom_bank: RomBank,
    rom_decoder: RomBus<C140RomDecoder>,
}

impl C219 {
    fn new() -> Self {
        C219 {
            sample_rate: 0,
            baserate: 0,
            reg: [0; 0x200],
            pcmtbl: [0; 256],
            voi: [C140Voice::default(); MAX_VOICE],
            lfsr: 0,
            // int1_timer
            rom_bank: None,
            rom_decoder: None,
        }
    }

    pub fn device_start(&mut self) {
        // generate mulaw table (Verified from Wii Virtual Console Arcade Knuckle Heads)
        // same as c352.cpp
        let mut j: i16 = 0;
        for i in 0..128 {
            self.pcmtbl[i] = j << 5;
            if i < 16 {
                j += 1;
            } else if i < 24 {
                j += 2;
            } else if i < 48 {
                j += 4;
            } else if i < 100 {
                j += 8;
            } else {
                j += 16;
            }
        }
        for i in 0..128 {
            self.pcmtbl[i + 128] = (!self.pcmtbl[i] as u16 & 0xffe0) as i16;
        }

        self.lfsr = 0x1234;
    }

    pub fn device_clock_changed(&mut self, clock: i32) -> i32 {
        self.sample_rate = clock;
        self.baserate = self.sample_rate;

        /* allocate a pair of buffers to mix into - 1 second's worth should be more than enough */
        // self.mixer_buffer_left = Some(vec![0; self.sample_rate as usize]);
        // self.mixer_buffer_right = Some(vec![0; self.sample_rate as usize]);

        self.sample_rate
    }

    pub fn rom_bank_updated(&mut self) {}

    pub fn sound_stream_update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        let mut dt: i32;

        let pbase: f32 = self.baserate as f32 * 2.0_f32 / self.sample_rate as f32;

        let mut lmix: i16;
        let mut rmix: i16;

        // let samples: i32 = self.sample_rate;
        /* for libymfm (1 tick) */
        let samples: i32 = 1;

        /* zap the contents of the mixer buffer */
        lmix = 0;
        rmix = 0;

        //--- audio update
        for i in 0..16 {
            let v: &mut C140Voice = &mut self.voi[i];
            // const struct voice_registers *vreg = (struct voice_registers *)&m_REG[i * 16];
            let vreg = &self.reg[i * 16..] as *const [u8] as *const VoiceRegisters;

            if v.key != 0 {
                let frequency: u16 = unsafe {
                    ((((*vreg).frequency_msb) as u16) << 8) | ((*vreg).frequency_lsb) as u16
                };

                /* Abort voice if no frequency value set */
                if frequency == 0 {
                    continue;
                }

                /* Delta =  frequency * ((8MHz/374)*2 / sample rate) */
                let delta: i32 = (frequency as f32 * pbase) as i32;

                /* Calculate left/right channel volumes */
                let lvol: i32 = unsafe { (((*vreg).volume_left) as i32 * 32) / MAX_VOICE as i32 }; //32ch -> 24ch
                let rvol: i32 = unsafe { (((*vreg).volume_right) as i32 * 32) / MAX_VOICE as i32 };

                /* Retrieve sample start/end and calculate size */
                let st = v.sample_start;
                let ed = v.sample_end;
                let sz = ed - st;

                /* Retrieve base pointer to the sample data */
                let sample_data = Self::find_sample(st, v.bank, i as i32, &self.reg);

                /* Fetch back previous data pointers */
                let mut offset = v.ptoffset;
                let mut pos = v.pos;
                let mut lastdt = v.lastdt;
                let mut prevdt = v.prevdt;
                let mut dltdt = v.dltdt;

                /* linear or compressed 8bit signed PCM */
                for _ in 0..samples {
                    offset += delta;
                    let cnt = (offset >> 16) & 0x7fff;
                    offset &= 0xffff;
                    pos += cnt;
                    /* Check for the end of the sample */
                    if pos >= sz {
                        /* Check if its a looping sample, either stop or loop */
                        if Self::ch_looped(v) || Self::ch_noise(v) {
                            pos = v.sample_loop - st;
                        } else {
                            v.key = 0;
                            break;
                        }
                    }

                    let shift: i32 = if Self::ch_noise(v) { 8 } else { 3 };

                    if cnt != 0 {
                        prevdt = lastdt;

                        if Self::ch_noise(v) {
                            self.lfsr = ((self.lfsr >> 1) as i16
                                ^ ((-(self.lfsr as i16 & 1)) as u16 & 0xfff6) as i16)
                                as u16;
                            lastdt = self.lfsr as i32;
                        } else {
                            lastdt =
                                ((read_word(&self.rom_bank, ((sample_data + pos) >> 1) as usize)
                                    & 0xff00)
                                    >> 8) as i8 as i32;
                            // 11 bit mulaw
                            if Self::ch_mulaw(v) {
                                lastdt = (self.pcmtbl[(lastdt & 0xff) as usize] as i32) >> 5;
                            } else {
                                lastdt <<= 3;
                            }
                        }

                        // Sign flip
                        if Self::ch_inv_sign(v) {
                            lastdt = -lastdt;
                        }

                        dltdt = lastdt - prevdt;
                    }

                    /* Caclulate the sample value */
                    dt = ((dltdt * offset) >> 16) + prevdt;

                    /* Write the data to the sample buffers */
                    lmix += ((if Self::ch_inv_lout(v) {
                        -(dt * lvol)
                    } else {
                        dt * lvol
                    }) >> (5 + shift)) as i16;
                    rmix += ((dt * rvol) >> (5 + shift)) as i16;
                }

                /* Save positional data for next callback */
                v.ptoffset = offset;
                v.pos = pos;
                v.lastdt = lastdt;
                v.prevdt = prevdt;
                v.dltdt = dltdt;
            }
        }

        /* render to MAME's stream buffer */
        for i in 0..samples as usize {
            // TODO:
            buffer_l[i] = convert_sample_i2f(lmix as i32 * 4);
            buffer_r[i] = convert_sample_i2f(rmix as i32 * 4);
        }
    }

    pub fn c219_w(&mut self, offset: usize, data: u8) {
        let mut offset: usize = offset & 0x1ff;

        // mirror the bank registers on the 219, fixes bkrtmaq (and probably xday2 based on notes in the HLE)
        if offset >= 0x1f8 && (offset & 0x1) == 1 {
            offset = (offset as isize - 8) as usize;
        }

        self.reg[offset] = data;
        if offset < 0x100 {
            // only 16 voices
            let ch = (offset >> 4) as usize;
            let v: &mut C140Voice = &mut self.voi[ch];

            if offset & 0xf == 0x5 {
                if data & 0x80 != 0 {
                    let vreg = &self.reg[(offset & 0x1f0) as usize..] as *const [u8]
                        as *const VoiceRegisters;
                    v.key = 1;
                    v.ptoffset = 0;
                    v.pos = 0;
                    v.lastdt = 0;
                    v.prevdt = 0;
                    v.dltdt = 0;
                    v.bank = unsafe { (*vreg).bank as i32 };
                    v.mode = data as i32;

                    let sample_loop: u32 =
                        unsafe { (u32::from((*vreg).loop_msb) << 8) + (*vreg).loop_lsb as u32 };
                    let start: u32 =
                        unsafe { (u32::from((*vreg).start_msb) << 8) + (*vreg).start_lsb as u32 };
                    let end: u32 =
                        unsafe { (u32::from((*vreg).end_msb) << 8) + (*vreg).end_lsb as u32 };
                    // on the 219 asic, addresses are in words
                    v.sample_loop = (sample_loop as i32) << 1;
                    v.sample_start = (start as i32) << 1;
                    v.sample_end = (end as i32) << 1;
                } else {
                    v.key = 0;
                }
            }
        }
        // TODO: No interrupt/timers?
    }

    fn find_sample(adrs: i32, bank: i32, voice: i32, reg: &[u8]) -> i32 {
        let adrs = (bank << 16) + adrs;

        // ASIC219's banking is fairly simple
        ((reg[(ASIC_219_BANKS[(voice / 4) as usize]) as usize] & 0x3) as u32 * 0x20000) as i32
            + adrs
    }

    #[inline]
    fn ch_noise(v: &C140Voice) -> bool {
        ((v.mode >> 2) & 0x1) == 0x1
    }

    #[inline]
    fn ch_inv_lout(v: &C140Voice) -> bool {
        ((v.mode >> 3) & 0x1) == 0x1
    }

    #[inline]
    fn ch_inv_sign(v: &C140Voice) -> bool {
        ((v.mode >> 6) & 0x1) == 0x1
    }

    #[inline]
    fn ch_looped(v: &C140Voice) -> bool {
        ((v.mode >> 4) & 0x1) == 0x1
    }

    #[inline]
    fn ch_mulaw(v: &C140Voice) -> bool {
        ((v.mode/* >> 0*/) & 0x1) == 0x1
    }
}

struct C140RomDecoder {
    rom_bus_type: Option<RomBusType>,
}

impl C140RomDecoder {
    fn new() -> Self {
        Self { rom_bus_type: None }
    }
}

impl Decoder for C140RomDecoder {
    fn decode(&self, rombank: &super::rom::RomSet, address: usize) -> u32 {
        // address decode
        let data = match self.rom_bus_type {
            Some(RomBusType::C140_TYPE_SYSTEM2) => {
                let offset = ((address & 0x200000) >> 2) | (address & 0x7ffff);
                // high 8 bit only
                u16::from(rombank.read(offset)) << 8
            }
            Some(RomBusType::C140_TYPE_SYSTEM21) => {
                let offset = ((address & 0x300000) >> 1) | (address & 0x7ffff);
                u16::from(rombank.read(offset)) << 8
            }
            None | Some(_) => {
                // C219_TYPE_ASIC219
                (u16::from(rombank.read(address * 2 + 1)) << 8)
                    | u16::from(rombank.read(address * 2))
            }
        };
        data as u32
    }
}

impl SoundChip for C140 {
    fn create(_sound_device_name: SoundChipType) -> Self {
        C140::new()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start();
        self.device_clock_changed(clock as i32) as u32
    }

    fn reset(&mut self) {
        todo!("not impliments");
    }

    fn write(&mut self, _: usize, offset: u32, data: u32, _sound_stream: &mut dyn SoundStream) {
        self.c140_w(offset as usize, data as u8);
    }

    fn tick(&mut self, _: usize, sound_stream: &mut dyn SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.sound_stream_update(&mut l, &mut r);
        sound_stream.push(l[0], r[0]);
    }

    fn set_rom_bank(&mut self, _ /* C140 has only one RomBank */: RomIndex, rombank: RomBank) {
        self.rom_bank = rombank;
        self.rom_bank_updated();
    }

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {
        self.rom_bank_updated();
    }

    fn set_rom_bus(&mut self, rom_bus_type: Option<RomBusType>) {
        // create rom decoder
        let mut rom_decoder = C140RomDecoder::new();
        // set bus type
        rom_decoder.rom_bus_type = rom_bus_type;
        // share bus to rom
        let rom_decoder = Rc::new(RefCell::new(rom_decoder));
        set_rom_bus(&self.rom_bank, Some(rom_decoder.clone()));
        // type state
        self.rom_decoder = Some(rom_decoder);
    }
}

impl SoundChip for C219 {
    fn create(_sound_device_name: SoundChipType) -> Self {
        C219::new()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start();
        self.device_clock_changed(clock as i32) as u32
    }

    fn reset(&mut self) {
        todo!("not impliments");
    }

    fn write(&mut self, _: usize, offset: u32, data: u32, _sound_stream: &mut dyn SoundStream) {
        self.c219_w(offset as usize, data as u8);
    }

    fn tick(&mut self, _: usize, sound_stream: &mut dyn SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.sound_stream_update(&mut l, &mut r);
        sound_stream.push(l[0], r[0]);
    }

    fn set_rom_bank(&mut self, _ /* C140 has only one RomBank */: RomIndex, rombank: RomBank) {
        self.rom_bank = rombank;
        self.rom_bank_updated();
    }

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {
        self.rom_bank_updated();
    }

    fn set_rom_bus(&mut self, rom_bus_type: Option<RomBusType>) {
        // create rom decoder
        let mut rom_decoder = C140RomDecoder::new();
        // set bus type
        rom_decoder.rom_bus_type = rom_bus_type;
        // share bus to rom
        let rom_decoder = Rc::new(RefCell::new(rom_decoder));
        set_rom_bus(&self.rom_bank, Some(rom_decoder.clone()));
        // type state
        self.rom_decoder = Some(rom_decoder);
    }
}

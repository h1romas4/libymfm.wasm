/**
 * Rust YM3438 emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/rust-synth-emulation
 *
 * Converted from:
 *  Nuked-OPN2 (C) Alexey Khokholov (Nuke.YKT)
 *  https://github.com/nukeykt/Nuked-OPN2/blob/master/ym3438.c
 *  rev. a67c34dfa152805bc2e752091e257a98e80ff2ae
 */

/**
 * Original Nuked-OPN2 Copyright
 */
/*
 * Copyright (C) 2017-2018 Alexey Khokholov (Nuke.YKT)
 *
 * This program is free software; you can redistribute it and/or
 * modify it under the terms of the GNU General Public License
 * as published by the Free Software Foundation; either version 2
 * of the License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.
 *
 *
 *  Nuked OPN2(Yamaha YM3438) emulator.
 *  Thanks:
 *      Silicon Pr0n:
 *          Yamaha YM3438 decap and die shot(digshadow).
 *      OPLx decapsulated(Matthew Gambrell, Olli Niemitalo):
 *          OPL2 ROMs.
 *
 * version: 1.0.9
 */

use crate::sound::{SoundDevice, SoundDeviceName, convert_sample_i2f};

use array_macro::*;

#[derive(PartialEq, Copy, Clone)]
pub enum YM3438Mode {
    YM2612 = 0x01,      /* Enables YM2612 emulation (MD1, MD2 VA2) */
    ReadMode = 0x02     /* Enables status read on any port (TeraDrive, MD1 VA7, MD2, etc) */
}

#[derive(PartialEq, Copy, Clone)]
enum EgNum {
    Attack = 0,
    Decay = 1,
    Sustain = 2,
    Release = 3
}

/* logsin table */
const LOGSINROM: [u16; 256]  = [
    0x859, 0x6c3, 0x607, 0x58b, 0x52e, 0x4e4, 0x4a6, 0x471,
    0x443, 0x41a, 0x3f5, 0x3d3, 0x3b5, 0x398, 0x37e, 0x365,
    0x34e, 0x339, 0x324, 0x311, 0x2ff, 0x2ed, 0x2dc, 0x2cd,
    0x2bd, 0x2af, 0x2a0, 0x293, 0x286, 0x279, 0x26d, 0x261,
    0x256, 0x24b, 0x240, 0x236, 0x22c, 0x222, 0x218, 0x20f,
    0x206, 0x1fd, 0x1f5, 0x1ec, 0x1e4, 0x1dc, 0x1d4, 0x1cd,
    0x1c5, 0x1be, 0x1b7, 0x1b0, 0x1a9, 0x1a2, 0x19b, 0x195,
    0x18f, 0x188, 0x182, 0x17c, 0x177, 0x171, 0x16b, 0x166,
    0x160, 0x15b, 0x155, 0x150, 0x14b, 0x146, 0x141, 0x13c,
    0x137, 0x133, 0x12e, 0x129, 0x125, 0x121, 0x11c, 0x118,
    0x114, 0x10f, 0x10b, 0x107, 0x103, 0x0ff, 0x0fb, 0x0f8,
    0x0f4, 0x0f0, 0x0ec, 0x0e9, 0x0e5, 0x0e2, 0x0de, 0x0db,
    0x0d7, 0x0d4, 0x0d1, 0x0cd, 0x0ca, 0x0c7, 0x0c4, 0x0c1,
    0x0be, 0x0bb, 0x0b8, 0x0b5, 0x0b2, 0x0af, 0x0ac, 0x0a9,
    0x0a7, 0x0a4, 0x0a1, 0x09f, 0x09c, 0x099, 0x097, 0x094,
    0x092, 0x08f, 0x08d, 0x08a, 0x088, 0x086, 0x083, 0x081,
    0x07f, 0x07d, 0x07a, 0x078, 0x076, 0x074, 0x072, 0x070,
    0x06e, 0x06c, 0x06a, 0x068, 0x066, 0x064, 0x062, 0x060,
    0x05e, 0x05c, 0x05b, 0x059, 0x057, 0x055, 0x053, 0x052,
    0x050, 0x04e, 0x04d, 0x04b, 0x04a, 0x048, 0x046, 0x045,
    0x043, 0x042, 0x040, 0x03f, 0x03e, 0x03c, 0x03b, 0x039,
    0x038, 0x037, 0x035, 0x034, 0x033, 0x031, 0x030, 0x02f,
    0x02e, 0x02d, 0x02b, 0x02a, 0x029, 0x028, 0x027, 0x026,
    0x025, 0x024, 0x023, 0x022, 0x021, 0x020, 0x01f, 0x01e,
    0x01d, 0x01c, 0x01b, 0x01a, 0x019, 0x018, 0x017, 0x017,
    0x016, 0x015, 0x014, 0x014, 0x013, 0x012, 0x011, 0x011,
    0x010, 0x00f, 0x00f, 0x00e, 0x00d, 0x00d, 0x00c, 0x00c,
    0x00b, 0x00a, 0x00a, 0x009, 0x009, 0x008, 0x008, 0x007,
    0x007, 0x007, 0x006, 0x006, 0x005, 0x005, 0x005, 0x004,
    0x004, 0x004, 0x003, 0x003, 0x003, 0x002, 0x002, 0x002,
    0x002, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
    0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000
];

/* exp table */
const EXPROM: [u16; 256]  = [
    0x000, 0x003, 0x006, 0x008, 0x00b, 0x00e, 0x011, 0x014,
    0x016, 0x019, 0x01c, 0x01f, 0x022, 0x025, 0x028, 0x02a,
    0x02d, 0x030, 0x033, 0x036, 0x039, 0x03c, 0x03f, 0x042,
    0x045, 0x048, 0x04b, 0x04e, 0x051, 0x054, 0x057, 0x05a,
    0x05d, 0x060, 0x063, 0x066, 0x069, 0x06c, 0x06f, 0x072,
    0x075, 0x078, 0x07b, 0x07e, 0x082, 0x085, 0x088, 0x08b,
    0x08e, 0x091, 0x094, 0x098, 0x09b, 0x09e, 0x0a1, 0x0a4,
    0x0a8, 0x0ab, 0x0ae, 0x0b1, 0x0b5, 0x0b8, 0x0bb, 0x0be,
    0x0c2, 0x0c5, 0x0c8, 0x0cc, 0x0cf, 0x0d2, 0x0d6, 0x0d9,
    0x0dc, 0x0e0, 0x0e3, 0x0e7, 0x0ea, 0x0ed, 0x0f1, 0x0f4,
    0x0f8, 0x0fb, 0x0ff, 0x102, 0x106, 0x109, 0x10c, 0x110,
    0x114, 0x117, 0x11b, 0x11e, 0x122, 0x125, 0x129, 0x12c,
    0x130, 0x134, 0x137, 0x13b, 0x13e, 0x142, 0x146, 0x149,
    0x14d, 0x151, 0x154, 0x158, 0x15c, 0x160, 0x163, 0x167,
    0x16b, 0x16f, 0x172, 0x176, 0x17a, 0x17e, 0x181, 0x185,
    0x189, 0x18d, 0x191, 0x195, 0x199, 0x19c, 0x1a0, 0x1a4,
    0x1a8, 0x1ac, 0x1b0, 0x1b4, 0x1b8, 0x1bc, 0x1c0, 0x1c4,
    0x1c8, 0x1cc, 0x1d0, 0x1d4, 0x1d8, 0x1dc, 0x1e0, 0x1e4,
    0x1e8, 0x1ec, 0x1f0, 0x1f5, 0x1f9, 0x1fd, 0x201, 0x205,
    0x209, 0x20e, 0x212, 0x216, 0x21a, 0x21e, 0x223, 0x227,
    0x22b, 0x230, 0x234, 0x238, 0x23c, 0x241, 0x245, 0x249,
    0x24e, 0x252, 0x257, 0x25b, 0x25f, 0x264, 0x268, 0x26d,
    0x271, 0x276, 0x27a, 0x27f, 0x283, 0x288, 0x28c, 0x291,
    0x295, 0x29a, 0x29e, 0x2a3, 0x2a8, 0x2ac, 0x2b1, 0x2b5,
    0x2ba, 0x2bf, 0x2c4, 0x2c8, 0x2cd, 0x2d2, 0x2d6, 0x2db,
    0x2e0, 0x2e5, 0x2e9, 0x2ee, 0x2f3, 0x2f8, 0x2fd, 0x302,
    0x306, 0x30b, 0x310, 0x315, 0x31a, 0x31f, 0x324, 0x329,
    0x32e, 0x333, 0x338, 0x33d, 0x342, 0x347, 0x34c, 0x351,
    0x356, 0x35b, 0x360, 0x365, 0x36a, 0x370, 0x375, 0x37a,
    0x37f, 0x384, 0x38a, 0x38f, 0x394, 0x399, 0x39f, 0x3a4,
    0x3a9, 0x3ae, 0x3b4, 0x3b9, 0x3bf, 0x3c4, 0x3c9, 0x3cf,
    0x3d4, 0x3da, 0x3df, 0x3e4, 0x3ea, 0x3ef, 0x3f5, 0x3fa
];

/* Note table */
const FN_NOTE: [u32; 16]  = [
    0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 3, 3, 3, 3, 3, 3
];

/* Envelope generator */
const EG_STEPHI: [[u32; 4]; 4]  = [
    [ 0, 0, 0, 0 ],
    [ 1, 0, 0, 0 ],
    [ 1, 0, 1, 0 ],
    [ 1, 1, 1, 0 ]
];

const EG_AM_SHIFT: [u8; 4]  = [
    7, 3, 1, 0
];

/* Phase generator */
const PG_DETUNE: [u32; 8]  = [
    16, 17, 19, 20, 22, 24, 27, 29
];

//static const Bit32u pg_lfo_sh1[8][8] = {
const PG_LFO_SH1: [[u32; 8]; 8]  = [
    [ 7, 7, 7, 7, 7, 7, 7, 7 ],
    [ 7, 7, 7, 7, 7, 7, 7, 7 ],
    [ 7, 7, 7, 7, 7, 7, 1, 1 ],
    [ 7, 7, 7, 7, 1, 1, 1, 1 ],
    [ 7, 7, 7, 1, 1, 1, 1, 0 ],
    [ 7, 7, 1, 1, 0, 0, 0, 0 ],
    [ 7, 7, 1, 1, 0, 0, 0, 0 ],
    [ 7, 7, 1, 1, 0, 0, 0, 0 ]
];

const PG_LFO_SH2: [[u32; 8]; 8]  = [
    [ 7, 7, 7, 7, 7, 7, 7, 7 ],
    [ 7, 7, 7, 7, 2, 2, 2, 2 ],
    [ 7, 7, 7, 2, 2, 2, 7, 7 ],
    [ 7, 7, 2, 2, 7, 7, 2, 2 ],
    [ 7, 7, 2, 7, 7, 7, 2, 7 ],
    [ 7, 7, 7, 2, 7, 7, 2, 1 ],
    [ 7, 7, 7, 2, 7, 7, 2, 1 ],
    [ 7, 7, 7, 2, 7, 7, 2, 1 ]
];

/* Address decoder */
const OP_OFFSET: [u32; 12]  = [
    0x000, /* Ch1 OP1/OP2 */
    0x001, /* Ch2 OP1/OP2 */
    0x002, /* Ch3 OP1/OP2 */
    0x100, /* Ch4 OP1/OP2 */
    0x101, /* Ch5 OP1/OP2 */
    0x102, /* Ch6 OP1/OP2 */
    0x004, /* Ch1 OP3/OP4 */
    0x005, /* Ch2 OP3/OP4 */
    0x006, /* Ch3 OP3/OP4 */
    0x104, /* Ch4 OP3/OP4 */
    0x105, /* Ch5 OP3/OP4 */
    0x106  /* Ch6 OP3/OP4 */
];

const CH_OFFSET: [u32; 6]  = [
    0x000, /* Ch1 */
    0x001, /* Ch2 */
    0x002, /* Ch3 */
    0x100, /* Ch4 */
    0x101, /* Ch5 */
    0x102  /* Ch6 */
];

/* LFO */
const LFO_CYCLES: [u32; 8]  = [
    108, 77, 71, 67, 62, 44, 8, 5
];

/* FM algorithm */
const FM_ALGORITHM: [[[u32; 8]; 6]; 4]  = [
    [
        [ 1, 1, 1, 1, 1, 1, 1, 1 ], /* OP1_0         */
        [ 1, 1, 1, 1, 1, 1, 1, 1 ], /* OP1_1         */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP2           */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 0, 0, 0, 1 ]  /* Out           */
    ],
    [
        [ 0, 1, 0, 0, 0, 1, 0, 0 ], /* OP1_0         */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP1_1         */
        [ 1, 1, 1, 0, 0, 0, 0, 0 ], /* OP2           */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 0, 1, 1, 1 ]  /* Out           */
    ],
    [
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP1_0         */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP1_1         */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP2           */
        [ 1, 0, 0, 1, 1, 1, 1, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 0, 0, 1, 1, 1, 1 ]  /* Out           */
    ],
    [
        [ 0, 0, 1, 0, 0, 1, 0, 0 ], /* OP1_0         */
        [ 0, 0, 0, 0, 0, 0, 0, 0 ], /* OP1_1         */
        [ 0, 0, 0, 1, 0, 0, 0, 0 ], /* OP2           */
        [ 1, 1, 0, 1, 1, 0, 0, 0 ], /* Last operator */
        [ 0, 0, 1, 0, 0, 0, 0, 0 ], /* Last operator */
        [ 1, 1, 1, 1, 1, 1, 1, 1 ]  /* Out           */
    ]
];

#[allow(dead_code)]
pub struct YM3438 {
    chip_type: YM3438Mode,
    // Bit32u cycles;
    cycles: usize,
    // Bit32u channel;
    channel: usize,
    // Bit16s mol, mor;
    mol: i16,
    mor: i16,
    // /* IO */
    // Bit16u write_data;
    write_data: u16,
    // Bit8u write_a;
    write_a: u8,
    // Bit8u write_d;
    write_d: u8,
    // Bit8u write_a_en;
    write_a_en: u8,
    // Bit8u write_d_en;
    write_d_en: u8,
    // Bit8u write_busy;
    write_busy: u8,
    // Bit8u write_busy_cnt;
    write_busy_cnt: u8,
    // Bit8u write_fm_address;
    write_fm_address: u8,
    // Bit8u write_fm_data;
    write_fm_data: u8,
    // Bit8u write_fm_mode_a;
    write_fm_mode_a: u8,
    // Bit16u address;
    address: u16,
    // Bit8u data;
    data: u8,
    // Bit8u pin_test_in;
    pin_test_in: u8,
    // Bit8u pin_irq;
    pin_irq: u8,
    // Bit8u busy;
    busy: u8,
    // /* LFO */
    // Bit8u lfo_en;
    lfo_en: u8,
    // Bit8u lfo_freq;
    lfo_freq: u8,
    // Bit8u lfo_pm;
    lfo_pm: u8,
    // Bit8u lfo_am;
    lfo_am: u8,
    // Bit8u lfo_cnt;
    lfo_cnt: u8,
    // Bit8u lfo_inc;
    lfo_inc: u8,
    // Bit8u lfo_quotient;
    lfo_quotient: u8,
    // /* Phase generator */
    // Bit16u pg_fnum;
    pg_fnum: u16,
    // Bit8u pg_block;
    pg_block: u8,
    // Bit8u pg_kcode;
    pg_kcode: u8,
    // Bit32u pg_inc[24];
    pg_inc: [u32; 24],
    // Bit32u pg_phase[24];
    pg_phase: [u32; 24],
    // Bit8u pg_reset[24];
    pg_reset: [u8; 24],
    // Bit32u pg_read;
    pg_read: u32,
    // /* Envelope generator */
    // Bit8u eg_cycle;
    eg_cycle: u8,
    // Bit8u eg_cycle_stop;
    eg_cycle_stop: u8,
    // Bit8u eg_shift;
    eg_shift: u8,
    // Bit8u eg_shift_lock;
    eg_shift_lock: u8,
    // Bit8u eg_timer_low_lock;
    eg_timer_low_lock: u8,
    // Bit16u eg_timer;
    eg_timer: u16,
    // Bit8u eg_timer_inc;
    eg_timer_inc: u8,
    // Bit16u eg_quotient;
    eg_quotient: u16,
    // Bit8u eg_custom_timer;
    eg_custom_timer: u8,
    // Bit8u eg_rate;
    eg_rate: u8,
    // Bit8u eg_ksv;
    eg_ksv: u8,
    // Bit8u eg_inc;
    eg_inc: u8,
    // Bit8u eg_ratemax;
    eg_ratemax: u8,
    // Bit8u eg_sl[2];
    eg_sl: [u8; 2],
    // Bit8u eg_lfo_am;
    eg_lfo_am: u8,
    // Bit8u eg_tl[2];
    eg_tl: [u8; 2],
    // Bit8u eg_state[24];
    eg_state: [EgNum; 24],
    // Bit16u eg_level[24];
    eg_level: [u16; 24],
    // Bit16u eg_out[24];
    eg_out: [u16; 24],
    // Bit8u eg_kon[24];
    eg_kon: [u8; 24],
    // Bit8u eg_kon_csm[24];
    eg_kon_csm: [u8; 24],
    // Bit8u eg_kon_latch[24];
    eg_kon_latch: [u8; 24],
    // Bit8u eg_csm_mode[24];
    eg_csm_mode: [u8; 24],
    // Bit8u eg_ssg_enable[24];
    eg_ssg_enable: [u8; 24],
    // Bit8u eg_ssg_pgrst_latch[24];
    eg_ssg_pgrst_latch: [u8; 24],
    // Bit8u eg_ssg_repeat_latch[24];
    eg_ssg_repeat_latch: [u8; 24],
    // Bit8u eg_ssg_hold_up_latch[24];
    eg_ssg_hold_up_latch: [u8; 24],
    // Bit8u eg_ssg_dir[24];
    eg_ssg_dir: [u8; 24],
    // Bit8u eg_ssg_inv[24];
    eg_ssg_inv: [u8; 24],
    // Bit32u eg_read[2];
    eg_read: [u32; 2],
    // Bit8u eg_read_inc;
    eg_read_inc: u8,
    // /* FM */
    // Bit16s fm_op1[6][2];
    fm_op1: [[i16; 2]; 6],
    // Bit16s fm_op2[6];
    fm_op2: [i16; 6],
    // Bit16s fm_out[24];
    fm_out: [i16; 24],
    // Bit16u fm_mod[24];
    fm_mod: [u16; 24],
    // /* Channel */
    // Bit16s ch_acc[6];
    ch_acc: [i16; 6],
    // Bit16s ch_out[6];
    ch_out: [i16; 6],
    // Bit16s ch_lock;
    ch_lock: i16,
    // Bit8u ch_lock_l;
    ch_lock_l: u8,
    // Bit8u ch_lock_r;
    ch_lock_r: u8,
    // Bit16s ch_read;
    ch_read: i16,
    // /* Timer */
    // Bit16u timer_a_cnt;
    timer_a_cnt: u16,
    // Bit16u timer_a_reg;
    timer_a_reg: u16,
    // Bit8u timer_a_load_lock;
    timer_a_load_lock: u8,
    // Bit8u timer_a_load;
    timer_a_load: u8,
    // Bit8u timer_a_enable;
    timer_a_enable: u8,
    // Bit8u timer_a_reset;
    timer_a_reset: u8,
    // Bit8u timer_a_load_latch;
    timer_a_load_latch: u8,
    // Bit8u timer_a_overflow_flag;
    timer_a_overflow_flag: u8,
    // Bit8u timer_a_overflow;
    timer_a_overflow: u8,

    // Bit16u timer_b_cnt;
    timer_b_cnt: u16,
    // Bit8u timer_b_subcnt;
    timer_b_subcnt: u8,
    // Bit16u timer_b_reg;
    timer_b_reg: u16,
    // Bit8u timer_b_load_lock;
    timer_b_load_lock: u8,
    // Bit8u timer_b_load;
    timer_b_load: u8,
    // Bit8u timer_b_enable;
    timer_b_enable: u8,
    // Bit8u timer_b_reset;
    timer_b_reset: u8,
    // Bit8u timer_b_load_latch;
    timer_b_load_latch: u8,
    // Bit8u timer_b_overflow_flag;
    timer_b_overflow_flag: u8,
    // Bit8u timer_b_overflow;
    timer_b_overflow: u8,

    // /* Register set */
    // Bit8u mode_test_21[8];
    mode_test_21: [u8; 8],
    // Bit8u mode_test_2c[8];
    mode_test_2c: [u8; 8],
    // Bit8u mode_ch3;
    mode_ch3: u8,
    // Bit8u mode_kon_channel;
    mode_kon_channel: u8,
    // Bit8u mode_kon_operator[4];
    mode_kon_operator: [u8; 4],
    // Bit8u mode_kon[24];
    mode_kon: [u8; 24],
    // Bit8u mode_csm;
    mode_csm: u8,
    // Bit8u mode_kon_csm;
    mode_kon_csm: u8,
    // Bit8u dacen;
    dacen: u8,
    // Bit16s dacdata;
    dacdata: i16,

    // Bit8u ks[24];
    ks: [u8; 24],
    // Bit8u ar[24];
    ar: [u8; 24],
    // Bit8u sr[24];
    sr: [u8; 24],
    // Bit8u dt[24];
    dt: [u8; 24],
    // Bit8u multi[24];
    multi: [u8; 24],
    // Bit8u sl[24];
    sl: [u8; 24],
    // Bit8u rr[24];
    rr: [u8; 24],
    // Bit8u dr[24];
    dr: [u8; 24],
    // Bit8u am[24];
    am: [u8; 24],
    // Bit8u tl[24];
    tl: [u8; 24],
    // Bit8u ssg_eg[24];
    ssg_eg: [u8; 24],

    // Bit16u fnum[6];
    fnum: [u16; 6],
    // Bit8u block[6];
    block: [u8; 6],
    // Bit8u kcode[6];
    kcode: [u8; 6],
    // Bit16u fnum_3ch[6];
    fnum_3ch: [u16; 6],
    // Bit8u block_3ch[6];
    block_3ch: [u8; 6],
    // Bit8u kcode_3ch[6];
    kcode_3ch: [u8; 6],
    // Bit8u reg_a4;
    reg_a4: u8,
    // Bit8u reg_ac;
    reg_ac: u8,
    // Bit8u connect[6];
    connect: [u8; 6],
    // Bit8u fb[6];
    fb: [u8; 6],
    // Bit8u pan_l[6], pan_r[6];
    pan_l: [u8; 6],
    pan_r: [u8; 6],
    // Bit8u ams[6];
    ams: [u8; 6],
    // Bit8u pms[6];
    pms: [u8; 6],
    // Bit8u status;
    status: u8,
    // Bit32u status_time;
    status_time: u32,

    // from vgmplay
    // Bit32u mute[7];
    mute: [u32; 7],
    // Bit32s rateratio;
    rateratio: i32,
    // Bit32s samplecnt;
    samplecnt: i32,
    // Bit32s oldsamples[2];
    oldsamples: [i32; 2],
    // Bit32s samples[2];
    samples: [i32; 2],

    // Bit64u writebuf_samplecnt;
    writebuf_samplecnt: u64,
    // Bit32u writebuf_cur;
    writebuf_cur: usize,
    // Bit32u writebuf_last;
    writebuf_last: usize,
    // Bit64u writebuf_lasttime;
    writebuf_lasttime: usize,
    // opn2_writebuf writebuf[OPN_WRITEBUF_SIZE];
    writebuf: [Opn2WriteBuf; OPN_WRITEBUF_SIZE],

    sample_rate: u32,
    clock: u32
}

impl Default for YM3438 {
    fn default() -> YM3438 {
        YM3438 {
            chip_type: YM3438Mode::YM2612,
            cycles: 0,
            channel: 0,
            mol: 0,
            mor: 0,
            write_data: 0,
            write_a: 0,
            write_d: 0,
            write_a_en: 0,
            write_d_en: 0,
            write_busy: 0,
            write_busy_cnt: 0,
            write_fm_address: 0,
            write_fm_data: 0,
            write_fm_mode_a: 0,
            address: 0,
            data: 0,
            pin_test_in: 0,
            pin_irq: 0,
            busy: 0,
            lfo_en: 0,
            lfo_freq: 0,
            lfo_pm: 0,
            lfo_am: 0,
            lfo_cnt: 0,
            lfo_inc: 0,
            lfo_quotient: 0,
            pg_fnum: 0,
            pg_block: 0,
            pg_kcode: 0,
            pg_inc: [0; 24],
            pg_phase: [0; 24],
            pg_reset: [0; 24],
            pg_read: 0,
            eg_cycle: 0,
            eg_cycle_stop: 0,
            eg_shift: 0,
            eg_shift_lock: 0,
            eg_timer_low_lock: 0,
            eg_timer: 0,
            eg_timer_inc: 0,
            eg_quotient: 0,
            eg_custom_timer: 0,
            eg_rate: 0,
            eg_ksv: 0,
            eg_inc: 0,
            eg_ratemax: 0,
            eg_sl: [0; 2],
            eg_lfo_am: 0,
            eg_tl: [0; 2],
            eg_state: [EgNum::Attack; 24],
            eg_level: [0; 24],
            eg_out: [0; 24],
            eg_kon: [0; 24],
            eg_kon_csm: [0; 24],
            eg_kon_latch: [0; 24],
            eg_csm_mode: [0; 24],
            eg_ssg_enable: [0; 24],
            eg_ssg_pgrst_latch: [0; 24],
            eg_ssg_repeat_latch: [0; 24],
            eg_ssg_hold_up_latch: [0; 24],
            eg_ssg_dir: [0; 24],
            eg_ssg_inv: [0; 24],
            eg_read: [0; 2],
            eg_read_inc: 0,
            fm_op1: [[0; 2]; 6],
            fm_op2: [0; 6],
            fm_out: [0; 24],
            fm_mod: [0; 24],
            ch_acc: [0; 6],
            ch_out: [0; 6],
            ch_lock: 0,
            ch_lock_l: 0,
            ch_lock_r: 0,
            ch_read: 0,
            timer_a_cnt: 0,
            timer_a_reg: 0,
            timer_a_load_lock: 0,
            timer_a_load: 0,
            timer_a_enable: 0,
            timer_a_reset: 0,
            timer_a_load_latch: 0,
            timer_a_overflow_flag: 0,
            timer_a_overflow: 0,
            timer_b_cnt: 0,
            timer_b_subcnt: 0,
            timer_b_reg: 0,
            timer_b_load_lock: 0,
            timer_b_load: 0,
            timer_b_enable: 0,
            timer_b_reset: 0,
            timer_b_load_latch: 0,
            timer_b_overflow_flag: 0,
            timer_b_overflow: 0,
            mode_test_21: [0; 8],
            mode_test_2c: [0; 8],
            mode_ch3: 0,
            mode_kon_channel: 0,
            mode_kon_operator: [0; 4],
            mode_kon: [0; 24],
            mode_csm: 0,
            mode_kon_csm: 0,
            dacen: 0,
            dacdata: 0,
            ks: [0; 24],
            ar: [0; 24],
            sr: [0; 24],
            dt: [0; 24],
            multi: [0; 24],
            sl: [0; 24],
            rr: [0; 24],
            dr: [0; 24],
            am: [0; 24],
            tl: [0; 24],
            ssg_eg: [0; 24],
            fnum: [0; 6],
            block: [0; 6],
            kcode: [0; 6],
            fnum_3ch: [0; 6],
            block_3ch: [0; 6],
            kcode_3ch: [0; 6],
            reg_a4: 0,
            reg_ac: 0,
            connect: [0; 6],
            fb: [0; 6],
            pan_l: [0; 6],
            pan_r: [0; 6],
            ams: [0; 6],
            pms: [0; 6],
            status: 0,
            status_time: 0,
            mute: [0; 7],
            rateratio: 0,
            samplecnt: 0,
            oldsamples: [0; 2],
            samples: [0; 2],
            writebuf_samplecnt: 0,
            writebuf_cur: 0,
            writebuf_last: 0,
            writebuf_lasttime: 0,
            writebuf: array![Opn2WriteBuf { time: 0, port: 0, data: 0 } ; OPN_WRITEBUF_SIZE],
            sample_rate: 0,
            clock: 0,
        }
    }
}

const RSM_FRAC: i32 = 10;

const OPN_WRITEBUF_SIZE: usize = 2048;
const OPN_WRITEBUF_DELAY: u32 = 15;

#[derive(Default)]
struct Opn2WriteBuf {
    // Bit64u time;
    time: u64,
    // Bit8u port;
    port: u8,
    // Bit8u data;
    data: u8
}

#[allow(dead_code)]
#[allow(clippy::verbose_bit_mask)]
#[allow(clippy::nonminimal_bool)]
impl YM3438 {
    fn opn2_do_io(&mut self) {
        /* Write signal check */
        self.write_a_en = if self.write_a & 0x03 == 0x01 { 1 } else { 0 };
        self.write_d_en = if self.write_d & 0x03 == 0x01 { 1 } else { 0 };
        self.write_a <<= 1;
        self.write_d <<= 1;
        /* Busy counter */
        self.busy = self.write_busy;
        self.write_busy_cnt += self.write_busy;
        self.write_busy = if (self.write_busy != 0 && (self.write_busy_cnt >> 5) == 0) || self.write_d_en != 0 {
            1
        } else {
            0
        };
        self.write_busy_cnt &= 0x1f;
    }

    fn opn2_do_reg_write(&mut self) {
        let mut slot: usize = self.cycles % 12;
        let mut address: u32;
        let channel: usize = self.channel;

        /* Update registers */
        if self.write_fm_data != 0 {
            /* Slot */
            if OP_OFFSET[slot] == u32::from(self.address) & 0x107 {
                if self.address & 0x08 != 0 {
                    /* OP2, OP4 */
                    slot += 12;
                }
                address = u32::from(self.address) & 0xf0;
                match address {
                    0x30 => {
                        /* DT, MULTI */
                        self.multi[slot] = self.data & 0x0f;
                        if self.multi[slot] == 0 {
                            self.multi[slot] = 1;
                        } else {
                            self.multi[slot] <<= 1;
                        }
                        self.dt[slot] = (self.data >> 4) & 0x07;
                    }
                    0x40 => {
                        /* TL */
                        self.tl[slot] = self.data & 0x7f;
                    }
                    0x50 => {
                        /* KS, AR */
                        self.ar[slot] = self.data & 0x1f;
                        self.ks[slot] = (self.data >> 6) & 0x03;
                    }
                    0x60 => {
                        /* AM, DR */
                        self.dr[slot] = self.data & 0x1f;
                        self.am[slot] = (self.data >> 7) & 0x01;
                    }
                    0x70 => {
                        /* SR */
                        self.sr[slot] = self.data & 0x1f;
                    }
                    0x80 => {
                        /* SL, RR */
                        self.rr[slot] = self.data & 0x0f;
                        self.sl[slot] = (self.data >> 4) & 0x0f;
                        self.sl[slot] |= (self.sl[slot] + 1) & 0x10;
                    }
                    0x90 => {
                        /* SSG-EG */
                        self.ssg_eg[slot] = self.data & 0x0f;
                    }
                    _ => {
                    }
                }
            }

            /* Channel */
            if CH_OFFSET[channel] == u32::from(self.address) & 0x103 {
                address = u32::from(self.address) & 0xfc;
                match address {
                    0xa0 => {
                        self.fnum[channel] = (u16::from(self.data) & 0xff) | ((u16::from(self.reg_a4) & 0x07) << 8);
                        self.block[channel] = (self.reg_a4 >> 3) & 0x07;
                        self.kcode[channel] = (u32::from(self.block[channel] << 2) | FN_NOTE[(self.fnum[channel] >> 7) as usize]) as u8;
                    }
                    0xa4 => {
                        self.reg_a4 = self.data; // & 0xff;
                    }
                    0xa8 => {
                        self.fnum_3ch[channel] = (u16::from(self.data) & 0xff) | ((u16::from(self.reg_ac) & 0x07) << 8);
                        self.block_3ch[channel] = (self.reg_ac >> 3) & 0x07;
                        self.kcode_3ch[channel] = (u32::from(self.block_3ch[channel] << 2) | FN_NOTE[(self.fnum_3ch[channel] >> 7) as usize]) as u8;
                    }
                    0xac => {
                        self.reg_ac = self.data; // & 0xff;
                    }
                    0xb0 => {
                        self.connect[channel] = self.data & 0x07;
                        self.fb[channel] = (self.data >> 3) & 0x07;
                    }
                    0xb4 => {
                        self.pms[channel] = self.data & 0x07;
                        self.ams[channel] = (self.data >> 4) & 0x03;
                        self.pan_l[channel] = (self.data >> 7) & 0x01;
                        self.pan_r[channel] = (self.data >> 6) & 0x01;
                    }
                    _ => {
                    }
                }
            }
        }

        if self.write_a_en != 0 || self.write_d_en != 0 {
            /* Data */
            if self.write_a_en != 0 {
                self.write_fm_data = 0;
            }

            if self.write_fm_address != 0 && self.write_d_en != 0 {
                self.write_fm_data = 1;
            }

            /* Address */
            if self.write_a_en != 0 {
                if (self.write_data & 0xf0) != 0x00 {
                    /* FM Write */
                    self.address = self.write_data;
                    self.write_fm_address = 1;
                } else {
                    /* SSG write */
                    self.write_fm_address = 0;
                }
            }

            /* FM Mode */
            /* Data */
            if self.write_d_en != 0 && (self.write_data & 0x100) == 0 {
                match self.write_fm_mode_a {
                    0x21 => {
                        /* LSI test 1 */
                        for i in 0..8 {
                            self.mode_test_21[i] = ((self.write_data >> i) & 0x01) as u8;
                        }
                    }
                    0x22 => {
                        /* LFO control */
                        if ((self.write_data >> 3) & 0x01) != 0 {
                            self.lfo_en = 0x7f;
                        } else {
                            self.lfo_en = 0;
                        }
                        self.lfo_freq = (self.write_data & 0x07) as u8;
                    }
                    0x24 => {
                        /* Timer A */
                        self.timer_a_reg &= 0x03;
                        self.timer_a_reg |= (self.write_data & 0xff) << 2;
                    }
                    0x25 => {
                        self.timer_a_reg &= 0x3fc;
                        self.timer_a_reg |= self.write_data & 0x03;
                    }
                    0x26 => {
                        /* Timer B */
                        self.timer_b_reg = self.write_data & 0xff;
                    }
                    0x27 => {
                        /* CSM, Timer control */
                        self.mode_ch3 = ((self.write_data & 0xc0) >> 6) as u8;
                        self.mode_csm = if self.mode_ch3 == 2 { 1 } else { 0 };
                        self.timer_a_load = (self.write_data & 0x01) as u8;
                        self.timer_a_enable = ((self.write_data >> 2) & 0x01) as u8;
                        self.timer_a_reset = ((self.write_data >> 4) & 0x01) as u8;
                        self.timer_b_load = ((self.write_data >> 1) & 0x01) as u8;
                        self.timer_b_enable = ((self.write_data >> 3) & 0x01) as u8;
                        self.timer_b_reset = ((self.write_data >> 5) & 0x01) as u8;
                    }
                    0x28 => {
                        /* Key on/off */
                        for i in 0..4 {
                            self.mode_kon_operator[i] = ((self.write_data >> (4 + i)) & 0x01) as u8;
                        }
                        if (self.write_data & 0x03) == 0x03 {
                            /* Invalid address */
                            self.mode_kon_channel = 0xff;
                        } else {
                            self.mode_kon_channel = ((self.write_data & 0x03) + ((self.write_data >> 2) & 1) * 3) as u8;
                        }
                    }
                    0x2a => {
                        /* DAC data */
                        self.dacdata &= 0x01;
                        self.dacdata |= ((self.write_data ^ 0x80) << 1) as i16;
                    }
                    0x2b => {
                        /* DAC enable */
                        self.dacen = (self.write_data >> 7) as u8;
                    }
                    0x2c => {
                        /* LSI test 2 */
                        for i in 0..8 {
                            self.mode_test_2c[i] = ((self.write_data >> i) & 0x01) as u8;
                        }
                        self.dacdata &= 0x1fe;
                        self.dacdata |= i16::from(self.mode_test_2c[3]);
                        self.eg_custom_timer = if self.mode_test_2c[7] == 0 && self.mode_test_2c[6] != 0 {
                            1
                        } else {
                            0
                        };
                    }
                    _ => {}
                }
            }

            /* Address */
            if self.write_a_en != 0 {
                self.write_fm_mode_a = (self.write_data & 0x1ff) as u8;
            }
        }

        if self.write_fm_data != 0 {
            self.data = (self.write_data & 0xff) as u8;
        }
    }

    fn opn2_phase_calc_increment(&mut self) {
        let chan: usize = self.channel;
        let slot: usize = self.cycles;
        let mut fnum: u32 = u32::from(self.pg_fnum);
        let fnum_h: u32 = fnum >> 4;
        let mut fm: u32;
        let mut basefreq: i32;
        let lfo: u8 = self.lfo_pm;
        let mut lfo_l: u8 = lfo & 0x0f;
        let pms: u8 = self.pms[chan];
        let dt: u8 = self.dt[slot];
        let dt_l: u8 = dt & 0x03;
        let mut detune: u8 = 0;
        let block: u8;
        let note: u8;
        let sum: u8;
        let sum_h: u8;
        let sum_l: u8;
        let mut kcode: u8 = self.pg_kcode;

        fnum <<= 1;
        /* Apply LFO */
        if (lfo_l & 0x08) != 0 {
            lfo_l ^= 0x0f;
        }
        fm = (fnum_h >> PG_LFO_SH1[pms as usize][lfo_l as usize]) + (fnum_h >> PG_LFO_SH2[pms as usize][lfo_l as usize]);
        if pms > 5 {
            fm <<= pms - 5;
        }
        fm >>= 2;
        if (lfo & 0x10) != 0 {
            fnum -= fm;
        } else {
            fnum += fm;
        }
        fnum &= 0xfff;

        basefreq = ((fnum << self.pg_block) >> 2) as i32;

        /* Apply detune */
        if dt_l != 0 {
            if kcode > 0x1c {
                kcode = 0x1c;
            }
            block = kcode >> 2;
            note = kcode & 0x03;
            sum = block + 9 + (if dt_l == 3 { 1 } else { 0 } | (dt_l & 0x02));
            sum_h = sum >> 1;
            sum_l = sum & 0x01;
            detune = (PG_DETUNE[((sum_l << 2) | note) as usize] >> (9 - sum_h)) as u8;
        }
        if (dt & 0x04) != 0 {
            basefreq -= i32::from(detune);
        } else {
            basefreq += i32::from(detune);
        }
        basefreq &= 0x1ffff;
        self.pg_inc[slot] = (basefreq as u32 * u32::from(self.multi[slot])) >> 1;
        self.pg_inc[slot] &= 0xfffff;
    }

    fn opn2_phase_generate(&mut self) {
        let mut slot: usize;
        /* Mask increment */
        slot = (self.cycles + 20) % 24;
        if self.pg_reset[slot] != 0 {
            self.pg_inc[slot] = 0;
        }
        /* Phase step */
        slot = (self.cycles + 19) % 24;
        self.pg_phase[slot] += self.pg_inc[slot];
        self.pg_phase[slot] &= 0xfffff;
        if self.pg_reset[slot] != 0 || self.mode_test_21[3] != 0 {
            self.pg_phase[slot] = 0;
        }
    }

    fn opn2_envelope_ssg_eg(&mut self) {
        let slot: usize = self.cycles;
        let mut direction: u8 = 0;
        self.eg_ssg_pgrst_latch[slot] = 0;
        self.eg_ssg_repeat_latch[slot] = 0;
        self.eg_ssg_hold_up_latch[slot] = 0;
        self.eg_ssg_inv[slot] = 0;
        if (self.ssg_eg[slot] & 0x08) != 0 {
            direction = self.eg_ssg_dir[slot];
            if (self.eg_level[slot] & 0x200) != 0 {
                /* Reset */
                if (self.ssg_eg[slot] & 0x03) == 0x00 {
                    self.eg_ssg_pgrst_latch[slot] = 1;
                }
                /* Repeat */
                if (self.ssg_eg[slot] & 0x01) == 0x00 {
                    self.eg_ssg_repeat_latch[slot] = 1;
                }
                /* Inverse */
                if (self.ssg_eg[slot] & 0x03) == 0x02 {
                    direction ^= 1;
                }
                if (self.ssg_eg[slot] & 0x03) == 0x03 {
                    direction = 1;
                }
            }
            /* Hold up */
            if self.eg_kon_latch[slot] != 0
                && ((self.ssg_eg[slot] & 0x07) == 0x05
                || (self.ssg_eg[slot] & 0x07) == 0x03) {
                self.eg_ssg_hold_up_latch[slot] = 1;
            }
            direction &= self.eg_kon[slot];
            self.eg_ssg_inv[slot] =
                (self.eg_ssg_dir[slot] ^ ((self.ssg_eg[slot] >> 2) & 0x01)) & self.eg_kon[slot];
        }
        self.eg_ssg_dir[slot] = direction;
        self.eg_ssg_enable[slot] = (self.ssg_eg[slot] >> 3) & 0x01;
    }

    fn opn2_envelope_adsr(&mut self) {
        let slot: usize = (self.cycles + 22) % 24;

        let nkon: u8 = self.eg_kon_latch[slot];
        let okon: u8 = self.eg_kon[slot];
        let kon_event: u8;
        let koff_event: u8;
        let eg_off: u8;
        let mut level: i16;
        let mut nextlevel: i16;
        let mut ssg_level: i16;
        let mut nextstate: EgNum = self.eg_state[slot];
        let mut inc: i16 = 0;
        self.eg_read[0] = u32::from(self.eg_read_inc);
        self.eg_read_inc = if self.eg_inc > 0 { 1 } else { 0 };

        /* Reset phase generator */
        self.pg_reset[slot] = if (nkon != 0 && okon == 0) || self.eg_ssg_pgrst_latch[slot] != 0 {
            1
        } else {
            0
        };

        /* KeyOn/Off */
        kon_event = if (nkon != 0 && okon == 0 ) || (okon != 0 && self.eg_ssg_repeat_latch[slot] != 0) {
            1
        } else {
            0
        };
        koff_event = if okon != 0 && nkon == 0 {
            1
        } else {
            0
        };

        level = self.eg_level[slot] as i16;
        ssg_level = level;

        if self.eg_ssg_inv[slot] != 0 {
            /* Inverse */
            ssg_level = 512 - level;
            ssg_level &= 0x3ff;
        }
        if koff_event != 0 {
            level = ssg_level;
        }
        if self.eg_ssg_enable[slot] != 0 {
            eg_off = (level >> 9) as u8;
        } else {
            eg_off = if (level & 0x3f0) == 0x3f0 { 1 } else { 0 };
        }
        nextlevel = level;
        if kon_event != 0 {
            nextstate = EgNum::Attack;
            /* Instant attack */
            if self.eg_ratemax != 0 {
                nextlevel = 0;
            }
            else if self.eg_state[slot] == EgNum::Attack
                && level != 0 && self.eg_inc != 0 && nkon != 0 {
                inc = (!level << self.eg_inc) >> 5;
            }
        } else {
            match self.eg_state[slot] {
                EgNum::Attack => {
                    if level == 0 {
                        nextstate = EgNum::Decay;
                    } else if self.eg_inc != 0 && self.eg_ratemax == 0 && nkon != 0 {
                        inc = (!level << self.eg_inc) >> 5;
                    }
                }
                EgNum::Decay => {
                    if (level >> 5) == i16::from(self.eg_sl[1]) {
                        nextstate = EgNum::Sustain;
                    } else if eg_off == 0 && self.eg_inc != 0 {
                        inc = 1 << (self.eg_inc - 1);
                        if self.eg_ssg_enable[slot] != 0 {
                            inc <<= 2;
                        }
                    }
                }
                EgNum::Sustain | EgNum::Release => {
                    if eg_off == 0 && self.eg_inc != 0 {
                        inc = 1 << (self.eg_inc - 1);
                        if self.eg_ssg_enable[slot] != 0 {
                            inc <<= 2;
                        }
                    }
                }
            }
            if nkon == 0 {
                nextstate = EgNum::Release;
            }
        }
        if self.eg_kon_csm[slot] != 0 {
            nextlevel |= i16::from(self.eg_tl[1]) << 3;
        }

        /* Envelope off */
        if kon_event == 0
            && self.eg_ssg_hold_up_latch[slot] == 0
            && self.eg_state[slot] != EgNum::Attack
            && eg_off != 0 {
            nextstate = EgNum::Release;
            nextlevel = 0x3ff;
        }

        nextlevel += inc;

        self.eg_kon[slot] = self.eg_kon_latch[slot];
        self.eg_level[slot] = (nextlevel & 0x3ff) as u16;
        self.eg_state[slot] = nextstate;
    }

    fn opn2_envelope_prepare(&mut self) {
        let mut rate: u8;
        let sum: u8;
        let mut inc: u8 = 0;
        let slot: usize = self.cycles;
        let mut rate_sel: EgNum;

        /* Prepare increment */
        rate = (self.eg_rate << 1) + self.eg_ksv;

        if rate > 0x3f {
            rate = 0x3f;
        }

        sum = ((rate >> 2) + self.eg_shift_lock) & 0x0f;
        if self.eg_rate != 0 && self.eg_quotient == 2 {
            if rate < 48 {
                match sum {
                    12 => {
                        inc = 1;
                    }
                    13 => {
                        inc = (rate >> 1) & 0x01;
                    }
                    14 => {
                        inc = rate & 0x01;
                    }
                    _ => { }
                }
            } else {
                inc = EG_STEPHI[(rate & 0x03) as usize][self.eg_timer_low_lock as usize] as u8 + (rate >> 2) - 11;
                if inc > 4 {
                    inc = 4;
                }
            }
        }
        self.eg_inc = inc;
        self.eg_ratemax = if (rate >> 1) == 0x1f { 1 } else { 0 };

        /* Prepare rate & ksv */
        rate_sel = self.eg_state[slot];
        if (self.eg_kon[slot] != 0 && self.eg_ssg_repeat_latch[slot] != 0)
            || (self.eg_kon[slot] == 0 && self.eg_kon_latch[slot] != 0) {
            rate_sel = EgNum::Attack;
        }
        match rate_sel {
            EgNum::Attack => {
                self.eg_rate = self.ar[slot];
            }
            EgNum::Decay => {
                self.eg_rate = self.dr[slot];
            }
            EgNum::Sustain => {
                self.eg_rate = self.sr[slot];
            }
            EgNum::Release => {
                self.eg_rate = (self.rr[slot] << 1) | 0x01;
            }
        }
        self.eg_ksv = self.pg_kcode >> (self.ks[slot] ^ 0x03);
        if self.am[slot] != 0 {
            self.eg_lfo_am = self.lfo_am >> EG_AM_SHIFT[self.ams[self.channel] as usize];
        } else {
            self.eg_lfo_am = 0;
        }
        /* Delay TL & SL value */
        self.eg_tl[1] = self.eg_tl[0];
        self.eg_tl[0] = self.tl[slot];
        self.eg_sl[1] = self.eg_sl[0];
        self.eg_sl[0] = self.sl[slot];
    }

    fn opn2_envelope_generate(&mut self) {
        let slot: usize = (self.cycles + 23) % 24;
        let mut level: u16;

        level = self.eg_level[slot];

        if self.eg_ssg_inv[slot] != 0 {
            /* Inverse */
            level = 512 - level;
        }
        if self.mode_test_21[5] != 0 {
            level = 0;
        }
        level &= 0x3ff;

        /* Apply AM LFO */
        level += u16::from(self.eg_lfo_am);

        /* Apply TL */
        if !(self.mode_csm != 0 && self.channel == 2 + 1)
        {
            level += u16::from(self.eg_tl[0]) << 3;
        }
        if level > 0x3ff {
            level = 0x3ff;
        }
        self.eg_out[slot] = level;
    }

    fn opn2_update_lfo(&mut self) {
        if (self.lfo_quotient & LFO_CYCLES[self.lfo_freq as usize] as u8) == LFO_CYCLES[self.lfo_freq as usize] as u8 {
            self.lfo_quotient = 0;
            self.lfo_cnt += 1;
        } else {
            self.lfo_quotient += self.lfo_inc;
        }
        self.lfo_cnt &= self.lfo_en;
    }

    fn opn2_fm_prepare(&mut self) {
        let mut slot: usize = (self.cycles + 6) % 24;
        let channel: usize = self.channel;
        let mut mod0: i16;
        let mut mod1: i16;
        let mut mod2: i16;
        let op: usize = slot / 6;
        let connect: usize = self.connect[channel] as usize;
        let prevslot: usize = (self.cycles + 18) % 24;

        /* Calculate modulation */
        mod1 = 0;
        mod2 = 0;

        if FM_ALGORITHM[op][0][connect] != 0 {
            mod2 |= self.fm_op1[channel][0];
        }
        if FM_ALGORITHM[op][1][connect] != 0 {
            mod1 |= self.fm_op1[channel][1];
        }
        if FM_ALGORITHM[op][2][connect] != 0 {
            mod1 |= self.fm_op2[channel];
        }
        if FM_ALGORITHM[op][3][connect] != 0 {
            mod2 |= self.fm_out[prevslot];
        }
        if FM_ALGORITHM[op][4][connect] != 0 {
            mod1 |= self.fm_out[prevslot];
        }
        mod0 = mod1 + mod2;
        if op == 0 {
            /* Feedback */
            mod0 >>= 10 - self.fb[channel];
            if self.fb[channel] == 0 {
                mod0 = 0;
            }
        } else {
            mod0 >>= 1;
        }
        self.fm_mod[slot] = mod0 as u16;

        slot = (self.cycles + 18) % 24;
        /* OP1 */
        if slot / 6 == 0 {
            self.fm_op1[channel][1] = self.fm_op1[channel][0];
            self.fm_op1[channel][0] = self.fm_out[slot];
        }
        /* OP2 */
        if slot / 6 == 2 {
            self.fm_op2[channel] = self.fm_out[slot];
        }
    }

    fn opn2_ch_generate(&mut self) {
        let slot: usize = (self.cycles + 18) % 24;
        let channel: usize = self.channel;
        let op: usize = slot / 6;
        let test_dac: u32 = u32::from(self.mode_test_2c[5]);
        let mut acc: i16 = self.ch_acc[channel];
        let mut add: i16 = test_dac as i16;
        let mut sum: i16;
        if op == 0 && test_dac == 0 {
            acc = 0;
        }
        if FM_ALGORITHM[op][5][self.connect[channel] as usize] != 0 && test_dac == 0 {
            add += self.fm_out[slot] >> 5;
        }
        sum = acc + add;
        /* Clamp */
        if sum > 255 {
            sum = 255;
        }
        else if sum < -256 {
            sum = -256;
        }

        if op == 0 || test_dac != 0 {
            self.ch_out[channel] = self.ch_acc[channel];
        }
        self.ch_acc[channel] = sum;
    }

    fn opn2_ch_output(&mut self) {
        let cycles: usize = self.cycles;
        let slot: usize = self.cycles;
        let mut channel: usize = self.channel;
        let test_dac: u32 = u32::from(self.mode_test_2c[5]);
        let mut out: i16;
        let mut sign: i16;
        let out_en: u32;
        self.ch_read = self.ch_lock;
        if slot < 12 {
            /* Ch 4,5,6 */
            channel += 1;
        }
        if cycles & 3 == 0 {
            if test_dac == 0 {
                /* Lock value */
                self.ch_lock = self.ch_out[channel];
            }
            self.ch_lock_l = self.pan_l[channel];
            self.ch_lock_r = self.pan_r[channel];
        }
        /* Ch 6 */
        if ((cycles >> 2) == 1 && self.dacen != 0) || test_dac != 0 {
            out = self.dacdata;
            out <<= 7;
            out >>= 7;
        } else {
            out = self.ch_lock;
        }
        self.mol = 0;
        self.mor = 0;

        if self.chip_type as u32 & YM3438Mode::YM2612 as u32 != 0 {
            out_en = if cycles & 3 == 3 || test_dac != 0 { 1 } else { 0 };
            /* YM2612 DAC emulation(not verified) */
            sign = out >> 8;
            if out >= 0 {
                out += 1;
                sign += 1;
            }
            if self.ch_lock_l != 0 && out_en != 0 {
                self.mol = out;
            } else {
                self.mol = sign;
            }
            if self.ch_lock_r != 0 && out_en != 0 {
                self.mor = out;
            } else {
                self.mor = sign;
            }
            /* Amplify signal */
            self.mol *= 3;
            self.mor *= 3;
        } else {
            out_en = if ((cycles & 3) != 0) || test_dac != 0 { 1 } else { 0 };
            if self.ch_lock_l != 0 && out_en != 0 {
                self.mol = out;
            }
            if self.ch_lock_r != 0 && out_en != 0 {
                self.mor = out;
            }
        }
    }

    fn opn2_fm_generate(&mut self) {
        let slot: usize = (self.cycles + 19) % 24;
        /* Calculate phase */
        let phase: u16 = ((u32::from(self.fm_mod[slot]) + (self.pg_phase[slot] >> 10)) & 0x3ff) as u16;
        let quarter: usize;
        let mut level: u16;
        let mut output: i16;
        if phase & 0x100 != 0 {
            quarter = ((phase ^ 0xff) & 0xff) as usize;
        } else {
            quarter = (phase & 0xff) as usize;
        }
        level = LOGSINROM[quarter];
        /* Apply envelope */
        level += self.eg_out[slot] << 2;
        /* Transform */
        if level > 0x1fff {
            level = 0x1fff;
        }
        output = ((u32::from((EXPROM[((level & 0xff) ^ 0xff) as usize]) | 0x400) << 2) >> (level >> 8)) as i16;
        if phase & 0x200 != 0 {
            output = ((!output) ^ (u16::from(self.mode_test_21[4]) << 13) as i16) + 1;
        } else {
            output ^= (u16::from(self.mode_test_21[4]) << 13) as i16;
        }
        output <<= 2;
        output >>= 2;
        self.fm_out[slot] = output;
    }

    fn opn2_do_timer_a(&mut self) {
        let mut time: u16;
        let mut load: u8;
        load = self.timer_a_overflow;
        if self.cycles == 2 {
            /* Lock load value */
            load |= if self.timer_a_load_lock == 0 && self.timer_a_load != 0 { 1 } else { 0 };
            self.timer_a_load_lock = self.timer_a_load;
            if self.mode_csm != 0 {
                /* CSM KeyOn */
                self.mode_kon_csm = load;
            } else {
                self.mode_kon_csm = 0;
            }
        }
        /* Load counter */
        if self.timer_a_load_latch != 0 {
            time = self.timer_a_reg;
        } else {
            time = self.timer_a_cnt;
        }
        self.timer_a_load_latch = load;
        /* Increase counter */
        if (self.cycles == 1 && self.timer_a_load_lock != 0) || self.mode_test_21[2] != 0 {
            time += 1;
        }
        /* Set overflow flag */
        if self.timer_a_reset != 0 {
            self.timer_a_reset = 0;
            self.timer_a_overflow_flag = 0;
        } else {
            self.timer_a_overflow_flag |= self.timer_a_overflow & self.timer_a_enable;
        }
        self.timer_a_overflow = (time >> 10) as u8;
        self.timer_a_cnt = time & 0x3ff;
    }

    fn opn2_do_timer_b(&mut self) {
        let mut time: u16;
        let mut load: u8;
        load = self.timer_b_overflow;
        if self.cycles == 2 {
            /* Lock load value */
            load |= if self.timer_b_load_lock == 0 && self.timer_b_load != 0 { 1 } else { 0 };
            self.timer_b_load_lock = self.timer_b_load;
        }
        /* Load counter */
        if self.timer_b_load_latch != 0 {
            time = self.timer_b_reg;
        } else {
            time = self.timer_b_cnt;
        }
        self.timer_b_load_latch = load;
        /* Increase counter */
        if self.cycles == 1 {
            self.timer_b_subcnt += 1;
        }
        if (self.timer_b_subcnt == 0x10 && self.timer_b_load_lock != 0) || self.mode_test_21[2] != 0 {
            time += 1;
        }
        self.timer_b_subcnt &= 0x0f;
        /* Set overflow flag */
        if self.timer_b_reset != 0 {
            self.timer_b_reset = 0;
            self.timer_b_overflow_flag = 0;
        } else {
            self.timer_b_overflow_flag |= self.timer_b_overflow & self.timer_b_enable;
        }
        self.timer_b_overflow = (time >> 8) as u8;
        self.timer_b_cnt = time & 0xff;
    }

    fn opn2_key_on(&mut self) {
        let slot: usize = self.cycles;
        let chan: usize = self.channel;
        /* Key On */
        self.eg_kon_latch[slot] = self.mode_kon[slot];
        self.eg_kon_csm[slot] = 0;
        if self.channel == 2 && self.mode_kon_csm != 0 {
            /* CSM Key On */
            self.eg_kon_latch[slot] = 1;
            self.eg_kon_csm[slot] = 1;
        }
        if self.cycles == self.mode_kon_channel as usize {
            /* OP1 */
            self.mode_kon[chan] = self.mode_kon_operator[0];
            /* OP2 */
            self.mode_kon[chan + 12] = self.mode_kon_operator[1];
            /* OP3 */
            self.mode_kon[chan + 6] = self.mode_kon_operator[2];
            /* OP4 */
            self.mode_kon[chan + 18] = self.mode_kon_operator[3];
        }
    }

    pub fn reset(&mut self, clock: u32, sample_rate: u32) {
        self.clock = clock;
        self.sample_rate = sample_rate;

        let rateratio: i32 = self.rateratio;
        for i in 0..24 {
            self.eg_out[i] = 0x3ff;
            self.eg_level[i] = 0x3ff;
            self.eg_state[i] = EgNum::Release;
            self.multi[i] = 1;
        }
        for i in 0..6 {
            self.pan_l[i] = 1;
            self.pan_r[i] = 1;
        }

        self.chip_type = YM3438Mode::YM2612;
        for i in 0..24 {
            self.eg_state[i] = EgNum::Attack;
        }

        if sample_rate != 0 {
            self.rateratio = ((u64::from(144 * sample_rate) << RSM_FRAC) / clock as u64) as i32;
        } else {
            self.rateratio = rateratio;
        }
    }

    pub fn opn2_set_chip_type(&mut self, chip_type: YM3438Mode) {
        self.chip_type = chip_type;
    }

    pub fn opn2_clock(&mut self, buffer: &mut [i16; 2]) {
        let slot: usize = self.cycles;
        self.lfo_inc = self.mode_test_21[1];
        self.pg_read >>= 1;
        self.eg_read[1] >>= 1;
        self.eg_cycle += 1;
        /* Lock envelope generator timer value */
        if self.cycles == 1 && self.eg_quotient == 2 {
            if self.eg_cycle_stop != 0 {
                self.eg_shift_lock = 0;
            } else {
                self.eg_shift_lock = self.eg_shift + 1;
            }
            self.eg_timer_low_lock = (self.eg_timer & 0x03) as u8;
        }
        /* Cycle specific functions */
        match self.cycles {
            0 => {
                self.lfo_pm = self.lfo_cnt >> 2;
                if self.lfo_cnt & 0x40 != 0 {
                    self.lfo_am = self.lfo_cnt & 0x3f;
                } else {
                    self.lfo_am = self.lfo_cnt ^ 0x3f;
                }
                self.lfo_am <<= 1;
            }
            1 => {
                self.eg_quotient += 1;
                self.eg_quotient %= 3;
                self.eg_cycle = 0;
                self.eg_cycle_stop = 1;
                self.eg_shift = 0;
                self.eg_timer_inc |= (self.eg_quotient >> 1) as u8;
                self.eg_timer += u16::from(self.eg_timer_inc);
                self.eg_timer_inc = (self.eg_timer >> 12) as u8;
                self.eg_timer &= 0xfff;
            }
            2 => {
                self.pg_read = self.pg_phase[21] & 0x3ff;
                self.eg_read[1] = u32::from(self.eg_out[0]);
            }
            13 => {
                self.eg_cycle = 0;
                self.eg_cycle_stop = 1;
                self.eg_shift = 0;
                self.eg_timer += u16::from(self.eg_timer_inc);
                self.eg_timer_inc = (self.eg_timer >> 12) as u8;
                self.eg_timer &= 0xfff;
            }
            23 => {
                self.lfo_inc |= 1;
            }
            _ => { }
        }
        self.eg_timer &= !(u16::from(self.mode_test_21[5]) << self.eg_cycle);
        if ((self.eg_timer >> self.eg_cycle) | u16::from(self.pin_test_in & self.eg_custom_timer)) & u16::from(self.eg_cycle_stop) != 0 {
            self.eg_shift = self.eg_cycle;
            self.eg_cycle_stop = 0;
        }

        self.opn2_do_io();

        self.opn2_do_timer_a();
        self.opn2_do_timer_b();
        self.opn2_key_on();

        self.opn2_ch_output();
        self.opn2_ch_generate();

        self.opn2_fm_prepare();
        self.opn2_fm_generate();

        self.opn2_phase_generate();
        self.opn2_phase_calc_increment();

        self.opn2_envelope_adsr();
        self.opn2_envelope_generate();
        self.opn2_envelope_ssg_eg();
        self.opn2_envelope_prepare();

        /* Prepare fnum & block */
        if self.mode_ch3 != 0 {
            /* Channel 3 special mode */
            match slot {
                1 => {
                    /* OP1 */
                    self.pg_fnum = self.fnum_3ch[1];
                    self.pg_block = self.block_3ch[1];
                    self.pg_kcode = self.kcode_3ch[1];
                }
                7 => {
                    /* OP3 */
                    self.pg_fnum = self.fnum_3ch[0];
                    self.pg_block = self.block_3ch[0];
                    self.pg_kcode = self.kcode_3ch[0];
                }
                13 => {
                    /* OP2 */
                    self.pg_fnum = self.fnum_3ch[2];
                    self.pg_block = self.block_3ch[2];
                    self.pg_kcode = self.kcode_3ch[2];
                }
                // 19 =>
                _ => {
                    /* OP4 */
                    self.pg_fnum = self.fnum[(self.channel + 1) % 6];
                    self.pg_block = self.block[(self.channel + 1) % 6];
                    self.pg_kcode = self.kcode[(self.channel + 1) % 6];
                }
            }
        } else {
            self.pg_fnum = self.fnum[(self.channel + 1) % 6];
            self.pg_block = self.block[(self.channel + 1) % 6];
            self.pg_kcode = self.kcode[(self.channel + 1) % 6];
        }

        self.opn2_update_lfo();
        self.opn2_do_reg_write();

        self.cycles = (self.cycles + 1) % 24;
        self.channel = self.cycles % 6;

        buffer[0] = self.mol;
        buffer[1] = self.mor;

        if self.status_time != 0 {
            self.status_time -= 1;
        }
    }

    pub fn opn2_write(&mut self, port: u32, data: u8) {
        let port = port & 3;
        self.write_data = (((port << 7) & 0x100) | u32::from(data)) as u16;
        if port & 1 != 0 {
            /* Data */
            self.write_d |= 1;
        } else {
            /* Address */
            self.write_a |= 1;
        }
    }

    pub fn opn2_set_test_pin(&mut self, value: u8) {
        self.pin_test_in = value & 1;
    }

    pub fn opn2_read_test_pin(&mut self) -> u32 {
        if self.mode_test_2c[7] == 0 {
            return 0;
        }
        if self.cycles == 23 { 1 } else { 0 }
    }

    pub fn opn2_read_irq_pin(&mut self) -> u32 {
        u32::from(self.timer_a_overflow_flag | self.timer_b_overflow_flag)
    }

    pub fn opn2_read(&mut self, port: u32) -> u8 {
        if (port & 3) == 0 || (self.chip_type as u32 & YM3438Mode::ReadMode as u32) != 0 {
            if self.mode_test_21[6] != 0 {
                /* Read test data */
                let slot: usize = (self.cycles + 18) % 24;
                let mut testdata: u16 = (((self.pg_read & 0x01) << 15)
                    | ((self.eg_read[self.mode_test_21[0] as usize] & 0x01) << 14) as u32) as u16;
                if self.mode_test_2c[4] != 0 {
                    testdata |= (self.ch_read & 0x1ff) as u16;
                } else {
                    testdata |= (self.fm_out[slot] & 0x3fff) as u16;
                }
                if self.mode_test_21[7] != 0 {
                    self.status = (testdata & 0xff) as u8;
                } else {
                    self.status = (testdata >> 8) as u8;
                }
            } else {
                self.status = (self.busy << 7) | (self.timer_b_overflow_flag << 1)
                     | self.timer_a_overflow_flag;
            }
            if self.chip_type as u32 & YM3438Mode::YM2612 as u32 != 0 {
                self.status_time = 300_000;
            } else {
                self.status_time = 40_000_000;
            }
        }
        if self.status_time != 0 {
            return self.status;
        }
        0
    }

    ///
    /// from vgmplay
    ///  https://github.com/vgmrips/vgmplay/blob/master/VGMPlay/chips/ym3438.c#L1433
    ///

    pub fn opn2_write_bufferd(&mut self, port: u32, data: u8) {
        let mut time1: u64;
        let time2: u64;
        let mut buffer: [i16; 2] = [0; 2];
        let mut skip: u64;

        if (self.writebuf[self.writebuf_last].port & 0x04) != 0 {
            self.opn2_write(u32::from(self.writebuf[self.writebuf_last].port & 0x03), self.writebuf[self.writebuf_last].data);

            self.writebuf_cur = (self.writebuf_last + 1) % OPN_WRITEBUF_SIZE;
            skip = self.writebuf[self.writebuf_last].time - self.writebuf_samplecnt;
            self.writebuf_samplecnt = self.writebuf[self.writebuf_last].time;
            while skip > 0 {
                self.opn2_clock(&mut buffer);
                skip -= 1;
            }
        }

        self.writebuf[self.writebuf_last].port = ((port & 0x03) | 0x04) as u8;
        self.writebuf[self.writebuf_last].data = data;
        time1 = (self.writebuf_lasttime + OPN_WRITEBUF_DELAY as usize) as u64;
        time2 = self.writebuf_samplecnt;

        if time1 < time2 {
            time1 = time2;
        }

        self.writebuf[self.writebuf_last].time = time1;
        self.writebuf_lasttime = time1 as usize;
        self.writebuf_last = (self.writebuf_last + 1) % OPN_WRITEBUF_SIZE;
    }

    fn opn2_generate_resampled(&mut self, buf: &mut [i32; 2]) {
        let mut buffer: [i16; 2] = [0; 2];
        let mut mute: u32;

        while self.samplecnt >= self.rateratio {
            self.oldsamples[0] = self.samples[0];
            self.oldsamples[1] = self.samples[1];
            self.samples[0] = 0;
            self.samples[1] = 0;
            for _ in 0..24 {
                match self.cycles >> 2 {
                    0 => {
                        // Ch 2
                        mute = self.mute[1];
                    }
                    1 => {
                        // Ch 6, DAC
                        mute = self.mute[(5 + self.dacen) as usize];
                    }
                    2 => {
                        // Ch 4
                        mute = self.mute[3];
                    }
                    3 => {
                        // Ch 1
                        mute = self.mute[0];
                    }
                    4 => {
                        // Ch 5
                        mute = self.mute[4];
                    }
                    5 => {
                        // Ch 3
                        mute = self.mute[2];
                    }
                    _ => {
                        mute = 0;
                    }
                }
                self.opn2_clock(&mut buffer);
                if mute == 0 {
                    self.samples[0] += i32::from(buffer[0]);
                    self.samples[1] += i32::from(buffer[1]);
                }

                while self.writebuf[self.writebuf_cur].time <= self.writebuf_samplecnt {
                    if (self.writebuf[self.writebuf_cur].port & 0x04) == 0 {
                        break;
                    }
                    self.writebuf[self.writebuf_cur].port &= 0x03;
                    self.opn2_write(u32::from(self.writebuf[self.writebuf_cur].port), self.writebuf[self.writebuf_cur].data);
                    self.writebuf_cur = (self.writebuf_cur + 1) % OPN_WRITEBUF_SIZE;
                }
                self.writebuf_samplecnt += 1;
            }
            self.samples[0] *= 11;
            self.samples[1] *= 11;
            self.samplecnt -= self.rateratio;
        }

        buf[0] = (self.oldsamples[0] * (self.rateratio - self.samplecnt)
            + self.samples[0] * self.samplecnt) / self.rateratio;
        buf[1] = (self.oldsamples[1] * (self.rateratio - self.samplecnt)
            + self.samples[1] * self.samplecnt) / self.rateratio;
        self.samplecnt += 1 << RSM_FRAC;
    }

    pub fn opn2_generate_stream(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        let mut sample: [i32; 2] = [0; 2];

        for i in 0..numsamples {
            self.opn2_generate_resampled(&mut sample);
            buffer_l[buffer_pos + i as usize] += convert_sample_i2f(sample[0]);
            buffer_r[buffer_pos + i as usize] += convert_sample_i2f(sample[1]);
        }
    }
}

impl SoundDevice<u8> for YM3438 {
    fn new() -> Self {
        YM3438::default()
    }

    fn init(&mut self, sample_rate: u32, clock: u32) {
        self.reset(clock, sample_rate);
    }

    fn get_name(&self) -> SoundDeviceName {
        if self.chip_type as u32 & YM3438Mode::YM2612 as u32 != 0 {
            SoundDeviceName::YM2612
        } else {
            SoundDeviceName::YM3438
        }
    }

    fn reset(&mut self) {
        self.reset(self.clock, self.sample_rate);
    }

    fn write(&mut self, port: u32, data: u8) {
        self.opn2_write_bufferd(port, data);
    }

    fn update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        self.opn2_generate_stream(buffer_l, buffer_r, numsamples, buffer_pos);
    }
}

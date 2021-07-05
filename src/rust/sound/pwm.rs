/**
 * Rust PWM emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/rust-synth-emulation
 *
 * Converted from:
 *  Gens: PWM audio emulator
 *  https://github.com/vgmrips/vgmplay/blob/master/VGMPlay/chips/pwm.c
 *  rev. abbe5526a11bb0b159018646ab1cec9e44c3831e
 */

/**
 * Original Gens: PWM audio emulator
 */
/***************************************************************************
 * Gens: PWM audio emulator.                                               *
 *                                                                         *
 * Copyright (c) 1999-2002 by Stéphane Dallongeville                       *
 * Copyright (c) 2003-2004 by Stéphane Akhoun                              *
 * Copyright (c) 2008-2009 by David Korth                                  *
 *                                                                         *
 * This program is free software; you can redistribute it and/or modify it *
 * under the terms of the GNU General Public License as published by the   *
 * Free Software Foundation; either version 2 of the License, or (at your  *
 * option) any later version.                                              *
 *                                                                         *
 * This program is distributed in the hope that it will be useful, but     *
 * WITHOUT ANY WARRANTY; without even the implied warranty of              *
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the           *
 * GNU General Public License for more details.                            *
 *                                                                         *
 * You should have received a copy of the GNU General Public License along *
 * with this program; if not, write to the Free Software Foundation, Inc., *
 * 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301, USA.           *
 ***************************************************************************/

// const CHILLY_WILLY_SCALE: u8 = 1;
// const PWM_BUF_SIZE: usize = 4;

// const PWM_FULL_TAB: [u8; PWM_BUF_SIZE * PWM_BUF_SIZE] = [
// 	0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80,
// 	0x80, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
// 	0x00, 0x80, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
// 	0x00, 0x00, 0x80, 0x40, 0x00, 0x00, 0x00, 0x00,
// 	0x00, 0x00, 0x00, 0x80, 0x40, 0x00, 0x00, 0x00,
// 	0x00, 0x00, 0x00, 0x00, 0x80, 0x40, 0x00, 0x00,
// 	0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x40, 0x00,
// 	0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x40
// ];

// const PWM_FULL_TAB: [u8; PWM_BUF_SIZE * PWM_BUF_SIZE] = [
// 	0x40, 0x00, 0x00, 0x80,
// 	0x80, 0x40, 0x00, 0x00,
// 	0x00, 0x80, 0x40, 0x00,
// 	0x00, 0x00, 0x80, 0x40
// ];

use crate::sound::{SoundDevice, SoundDeviceName, convert_sample_i2f};

const CHIP_SAMPLING_MODE: u8 = 0x00;
const CHIP_SAMPLE_RATE: i32 = 44100;
const MAX_CHIPS: usize = 0x02;
const PWM_LOUDNESS: u8 = 0;

pub struct PWM {
    pwm_chip: [PWMChip; MAX_CHIPS]
}

#[derive(Default)]
struct PWMChip {
    // 	unsigned short PWM_FIFO_R[8];
    pwm_fifo_r: [u16; 8],
    // 	unsigned short PWM_FIFO_L[8];
    pwm_fifo_l: [u16; 8],
    // 	unsigned int PWM_RP_R;
    pwm_rp_r: u32,
    // 	unsigned int PWM_WP_R;
    pwm_wp_r: u32,
    // 	unsigned int PWM_RP_L;
    pwm_rp_l: u32,
    // 	unsigned int PWM_WP_L;
    pwm_wp_l: u32,
    // 	unsigned int PWM_Cycles;
    pwm_cycles: u32,
    // 	unsigned int PWM_Cycle;
    pwm_cycle: u32,
    // 	unsigned int PWM_Cycle_Cnt;
    pwm_cycle_cnt: u32,
    // 	unsigned int PWM_Int;
    pwm_int: u32,
    // 	unsigned int PWM_Int_Cnt;
    pwm_int_cnt: u32,
    // 	unsigned int PWM_Mode;
    pwm_mode: u32,
    // 	//unsigned int PWM_Enable;
    // 	unsigned int PWM_Out_R;
    pwm_out_r: u32,
    // 	unsigned int PWM_Out_L;
    pwm_out_l: u32,

    // 	unsigned int PWM_Cycle_Tmp;
    pwm_cycle_tmp: u32,
    // 	unsigned int PWM_Cycles_Tmp;
    // pwm_cycles_tmp: u32,
    // 	unsigned int PWM_Int_Tmp;
    pwm_int_tmp: u32,
    // 	unsigned int PWM_FIFO_L_Tmp;
    pwm_fifo_l_tmp: u32,
    // 	unsigned int PWM_FIFO_R_Tmp;
    pwm_fifo_r_tmp: u32,

    // #if CHILLY_WILLY_SCALE
    // // TODO: Fix Chilly Willy's new scaling algorithm.
    // 	/* PWM scaling variables. */
    // 	int PWM_Offset;
    pwm_offset: i32,
    // 	int PWM_Scale;
    pwm_scale: i32,
    // 	//int PWM_Loudness;
    // #endif

    // 	int clock;
    clock: i32
}

#[allow(dead_code)]
impl PWM {
    fn pwm_init(chip: &mut PWMChip) {
        chip.pwm_mode = 0;
        chip.pwm_out_r = 0;
        chip.pwm_out_l = 0;

        chip.pwm_fifo_r = [0x00; 8];
        chip.pwm_fifo_l = [0x00; 8];

        chip.pwm_rp_r = 0;
        chip.pwm_wp_r = 0;
        chip.pwm_rp_l = 0;
        chip.pwm_wp_l = 0;
        chip.pwm_cycle_tmp = 0;
        chip.pwm_int_tmp = 0;
        chip.pwm_fifo_l_tmp = 0;
        chip.pwm_fifo_r_tmp = 0;

        PWM::pwm_set_cycle(chip, 0);
        PWM::pwm_set_int(chip, 0);
    }

    fn pwm_recalc_scale(chip: &mut PWMChip) {
        chip.pwm_offset = (chip.pwm_cycle as i32 / 2) + 1;
        chip.pwm_scale = 0x7fff00 / chip.pwm_offset;
    }

    fn pwm_set_cycle(chip: &mut PWMChip, cycle: u32) {
        let cycle: i32 = cycle as i32 - 1;
        chip.pwm_cycle = cycle as u32 & 0xfff;
        chip.pwm_cycle_cnt = chip.pwm_cycles;

        PWM::pwm_recalc_scale(chip);
    }

    fn pwm_set_int(chip: &mut PWMChip, int_time: u32) {
        let int_time = int_time & 0x0f;
        if int_time != 0 {
            chip.pwm_int = int_time;
            chip.pwm_int_cnt = int_time;
        } else {
            chip.pwm_int = 16;
            chip.pwm_int_cnt = 16;
        }
    }

    fn pwm_clear_timer(&self, chip: &mut PWMChip) {
        chip.pwm_cycle_cnt = 0;
    }

    #[inline(always)]
    fn pwm_update_scale(&self, chip: &PWMChip, pwm_in: i32) -> i32 {
        if pwm_in == 0 {
            return 0;
        }
        let mut pwm_in = pwm_in & 0xfff;
        if pwm_in & 0x800 != 0 {
            pwm_in |= !0xfff;
        }
        ((pwm_in - chip.pwm_offset) * chip.pwm_scale) >> (8 - PWM_LOUDNESS)
    }

    fn pwm_update(&self, chip: &PWMChip, buffer_l: &mut [f32], buffer_r: &mut [f32], length: usize) {
        let tmp_out_l: i32;
        let tmp_out_r: i32;

        if chip.pwm_out_l == 0 && chip.pwm_out_r == 0 {
            return;
        }

        let pwm_out_l = chip.pwm_out_l as i32;
        let pwm_out_r = chip.pwm_out_r as i32;

        tmp_out_l = self.pwm_update_scale(&chip, pwm_out_l);
        tmp_out_r = self.pwm_update_scale(&chip, pwm_out_r);

        for i in 0..length {
            buffer_l[i] += convert_sample_i2f(tmp_out_l);
            buffer_r[i] += convert_sample_i2f(tmp_out_r);
        }
    }

    pub fn pwm_update_chip(&self, chipid: usize, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        self.pwm_update(&self.pwm_chip[chipid], &mut buffer_l[buffer_pos..], &mut buffer_r[buffer_pos..], numsamples);
    }

    pub fn device_start_pwm(&mut self, chipid: usize, clock: i32) -> i32 {
        if chipid >= MAX_CHIPS {
            return 0;
        }

        let mut chip = &mut self.pwm_chip[chipid];
        let mut rate: i32 = 22020;
        if (CHIP_SAMPLING_MODE & 0x01 != 0 && rate < CHIP_SAMPLE_RATE) || CHIP_SAMPLING_MODE == 0x20 {
            rate = CHIP_SAMPLE_RATE;
        }
        chip.clock = clock;

        PWM::pwm_init(&mut chip);

        rate
    }

    pub fn device_stop_pwm(&self, _chipid: usize) {
    }

    pub fn device_reset_pwm(&mut self, chipid: usize) {
        let mut chip = &mut self.pwm_chip[chipid];
        PWM::pwm_init(&mut chip);
    }

    pub fn pwm_chn_w(&mut self, chipid: usize, channel: u8, data: u16) {
        let mut chip = &mut self.pwm_chip[chipid];
        let data = data as u32;

        if chip.clock == 1 {
            match channel {
                0x00 => {
                    chip.pwm_out_l = data;
                }
                0x01 => {
                    chip.pwm_out_r = data;
                }
                0x02 => {
                    PWM::pwm_set_cycle(chip, data);
                }
                0x03 => {
                    chip.pwm_out_l = data;
                    chip.pwm_out_r = data;
                }
                _ => {}
            }
        } else {
            match channel {
                0x00 => {
                    // control register
                    PWM::pwm_set_int(chip, data >> 8);
                }
                0x01 => {
                    // cycle register
                    PWM::pwm_set_cycle(chip, data);
                }
                0x02 => {
                    // l ch
                    chip.pwm_out_l = data;
                }
                0x03 => {
                    // r ch
                    chip.pwm_out_r = data;
                    if chip.pwm_mode == 0 && chip.pwm_out_l == chip.pwm_out_r {
                        // fixes these terrible pops when
                        // starting/stopping/pausing the song
                        chip.pwm_offset = data as i32;
                        chip.pwm_mode = 0x01;
                    }
                }
                0x04 => {
                    // mono ch
                    chip.pwm_out_l = data;
                    chip.pwm_out_r = data;
                    if chip.pwm_mode == 0 {
                        chip.pwm_offset = data as i32;
                        chip.pwm_mode = 0x01;
                    }
                }
                _ => {}
            }
        }
    }
}

impl SoundDevice<u16> for PWM {
    fn new() -> Self {
        PWM {
            pwm_chip: [PWMChip::default(), PWMChip::default()]
        }
    }

    fn init(&mut self, _: u32, clock: u32) {
        self.device_start_pwm(0, clock as i32);
    }

    fn get_name(&self) -> SoundDeviceName {
        SoundDeviceName::PWM
    }

    fn reset(&mut self) {
        self.device_reset_pwm(0);
    }

    fn write(&mut self, port: u32, data: u16) {
        self.pwm_chn_w(0, port as u8, data);
    }

    fn update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        self.pwm_update_chip(0, buffer_l, buffer_r, numsamples, buffer_pos);
    }
}

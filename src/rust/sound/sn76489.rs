/**
 * Rust SN76489 emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/rust-synth-emulation
 *
 * Converted from:
 *  SN76489 emulation by Maxim
 *  https://github.com/vgmrips/vgmplay/blob/master/VGMPlay/chips/sn76489.c
 */

/**
 * Original SN76489 emulation Copyright
 */
/*
	SN76489 emulation
	by Maxim in 2001 and 2002
	converted from my original Delphi implementation
	I'm a C newbie so I'm sure there are loads of stupid things
	in here which I'll come back to some day and redo
	Includes:
	- Super-high quality tone channel "oversampling" by calculating fractional positions on transitions
	- Noise output pattern reverse engineered from actual SMS output
	- Volume levels taken from actual SMS output
	07/08/04  Charles MacDonald
	Modified for use with SMS Plus:
	- Added support for multiple PSG chips.
	- Added reset/config/update routines.
	- Added context management routines.
	- Removed SN76489_GetValues().
	- Removed some unused variables.
*/

use std::f32;
use std::i32;

use crate::sound::{SoundDevice, SoundDeviceName, convert_sample_i2f};

// More testing is needed to find and confirm feedback patterns for
// SN76489 variants and compatible chips.
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
enum FeedbackPatterns {
    // Texas Instruments TMS SN76489N (original) from BBC Micro computer
    FbBbcmicro =   0x8005,
    // Texas Instruments TMS SN76489AN (rev. A) from SC-3000H computer
    FbSc3000   =   0x0006,
    // SN76489 clone in Sega's VDP chips (315-5124, 315-5246, 315-5313, Game Gear)
    FbSegavdp  =   0x0009
}

#[allow(dead_code)]
enum SrWidths {
    SrwSc3000bbcmicro = 15,
    SrwSegavdp = 16
}

#[allow(dead_code)]
enum VolumeModes {
    // Volume levels 13-15 are identical
    VolTrunc   =   0,
    // Volume levels 13-15 are unique
    VolFull    =   1
}

#[allow(dead_code)]
enum MuteValues {
    // All channels muted
    AllOff   =   0,
    // Tone 1 mute control
    Tone1On  =   1,
    // Tone 2 mute control
    Tone2On  =   2,
    // Tone 3 mute control
    Tone3On  =   4,
    // Noise mute control
    NoiseOn  =   8,
    // All channels enabled
    AllOn    =   15
}

// Initial state of shift register
const NOISE_INITIAL_STATE: i32 = 0x8000;
const PSG_CUTOFF: i32 = 0x6;

const PSG_VOLUME_VALUES: [i32; 16] = [
    // These values are taken from a real SMS2's output
    //	{892,892,892,760,623,497,404,323,257,198,159,123,96,75,60,0}, // I can't remember why 892...:P some scaling I did at some point
    // these values are true volumes for 2dB drops at each step (multiply previous by 10^-0.1)
    //	1516,1205,957,760,603,479,381,303,240,191,152,120,96,76,60,0
    // The MAME core uses 0x2000 as maximum volume (0x1000 for bipolar output)
    4096, 3254, 2584, 2053, 1631, 1295, 1029, 817, 649, 516, 410, 325, 258, 205, 163, 0
];

#[derive(Default)]
pub struct SN76489 {
    // per-channel muting
    mute: i32,

    // Variables
    clock: f32,
    d_clock: f32,
    psg_stereo: i32,
    num_clocks_for_sample: i32,
    white_noise_feedback: i32,
    sr_width: i32,

    // PSG registers:
    // Tone, vol x4
    registers: [i32; 8],
    latched_register: i32,
    noise_shift_register: i32,
    // Noise channel signal generator frequency
    noise_freq: i32,

    // Output calculation variables
    // Frequency register values (counters)
    tone_freq_vals: [i32; 4],
    // Frequency channel flip-flops
    tone_freq_pos: [i32; 4],
    // Value of each channel, before stereo is applied
    channels: [i32; 4],
    // intermediate values used at boundaries between + and - (does not need double accuracy)
    intermediate_pos: [f32; 4],
}

impl SN76489 {
    pub fn init(&mut self, psg_clock_value: i32, sampling_rate: i32) {
        self.d_clock = (psg_clock_value & 0x07ff_ffff) as f32 / 16_f32 / sampling_rate as f32;

        self.mute(MuteValues::AllOn);
        self.config(FeedbackPatterns::FbSegavdp, SrWidths::SrwSegavdp);
    }

    pub fn reset(&mut self) {
        self.psg_stereo = 0xff;

        for i in 0..4 {
            // Initialise PSG state
            self.registers[2 * i] = 1;
            self.registers[2 * i + 1] = 0xf;
            self.noise_freq = 0x10;

            // Set counters to 0
            self.tone_freq_vals[i] = 0;

            // Set flip-flops to 1
            self.tone_freq_pos[i] = 1;

            // Set intermediate positions to do-not-use value
            self.intermediate_pos[i] = f32::MIN;
        }

        self.latched_register = 0;

        // Initialise noise generator
        self.noise_shift_register = NOISE_INITIAL_STATE;

        // Zero clock
        self.clock = 0_f32;
    }

    pub fn write(&mut self, data: u8) {
        let data : u16 = u16::from(data);
        if data & 0x80 != 0 {
            self.latched_register = i32::from(data >> 4) & 0x07;
            self.registers[self.latched_register as usize] =
                // zero low 4 bits
                (self.registers[self.latched_register as usize] & 0x3f0)
                // and replace with data
                | i32::from(data) & 0xf;
        } else {
            // Data byte %0 - dddddd
            if (self.latched_register % 2) == 0 && (self.latched_register < 5) {
                // Tone register
                self.registers[self.latched_register as usize] =
                    // zero high 6 bits
                    (self.registers[self.latched_register as usize] & 0x00f)
                    // and replace with data
                    | i32::from(data & 0x3f) << 4;
            } else {
                // Other register
                // Replace with data
                self.registers[self.latched_register as usize] = i32::from(data) & 0x0f;
            }
        }

        match self.latched_register {
            0 | 2 | 4 => {
                // Tone channels
                if self.registers[self.latched_register as usize] == 0 {
                    // Zero frequency changed to 1 to avoid div/0
                    self.registers[self.latched_register as usize] = 1;
                }
            }
            6 => {
                // Noise
                // reset shift register
                self.noise_shift_register = NOISE_INITIAL_STATE;
                // set noise signal generator frequency
                self.noise_freq = 0x10 << (self.registers[6] & 0x3);
            }
            _ => {
                // println!("through latched_register value {:x}", self.latched_register);
            }
        }
    }

    pub fn update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], length: usize, buffer_pos: usize) {
        for j in 0..length {
            // Tone channels
            for i in 0..3 {
                if (self.mute >> i) & 1 != 0 {
                    if self.intermediate_pos[i] != f32::MIN {
                        // Intermediate position (antialiasing)
                        self.channels[i] =
                            (PSG_VOLUME_VALUES[self.registers[2 * i + 1] as usize] as f32
                                * self.intermediate_pos[i]) as i32;
                    } else {
                        // Flat (no antialiasing needed)
                        self.channels[i] =
                            (PSG_VOLUME_VALUES[self.registers[2 * i + 1] as usize]
                                * self.tone_freq_pos[i]) as i32;
                    }
                } else {
                    // Muted channel
                    self.channels[i] = 0;
                }
            }

            // Noise channel
            if (self.mute >> 3) & 1 != 0 {
                // Now the noise is bipolar, too. -Valley Bell
                self.channels[3] = PSG_VOLUME_VALUES[self.registers[7] as usize]
                    * ((self.noise_shift_register & 0x01) * 2 - 1);
                // due to the way the white noise works here, it seems twice as loud as it should be
                if (self.registers[6] & 0x4) != 0 {
                    self.channels[3] >>= 1;
                }
            } else {
                self.channels[3] = 0;
            }

            // Build stereo result into buffer (clear buffer)
            let mut buffer_li: i32  = 0;
            let mut buffer_ri: i32  = 0;
            for i in 0..4 {
                buffer_li +=  self.channels[i];
                buffer_ri +=  self.channels[i];
            }
            buffer_l[j + buffer_pos] += convert_sample_i2f(buffer_li / 2);
            buffer_r[j + buffer_pos] += convert_sample_i2f(buffer_ri / 2);

            // Increment clock by 1 sample length
            self.clock += self.d_clock;
            self.num_clocks_for_sample = self.clock as i32;
            self.clock -= self.num_clocks_for_sample as f32;

            // Decrement tone channel counters
            for i in 0..3 {
                self.tone_freq_vals[i] -= self.num_clocks_for_sample;
            }

            // Noise channel: match to tone2 or decrement its counter
            if self.noise_freq == 0x80 {
                self.tone_freq_vals[3] = self.tone_freq_vals[2];
            } else {
                self.tone_freq_vals[3] -= self.num_clocks_for_sample;
            }

            // Tone channels:
            for i in 0..3 {
                // If the counter gets below 0...
                if self.tone_freq_vals[i] <= 0 {
                    if self.registers[i * 2] >= PSG_CUTOFF {
                        // For tone-generating values, calculate how much of the sample is + and how much is -
                        // This is optimised into an even more confusing state than it was in the first place...
                        self.intermediate_pos[i] =
                            (self.num_clocks_for_sample as f32 - self.clock + 2_f32 * self.tone_freq_vals[i] as f32)
                                * self.tone_freq_pos[i] as f32
                                / (self.num_clocks_for_sample as f32 + self.clock);
                        // Flip the flip-flop
                        self.tone_freq_pos[i] = -self.tone_freq_pos[i];
                    } else {
                        // stuck value
                        self.tone_freq_pos[i] = 1;
                        self.intermediate_pos[i] = f32::MIN;
                    }
                    self.tone_freq_vals[i] += self.registers[i * 2] *
                        (self.num_clocks_for_sample / self.registers[i * 2] + 1);
                } else {
                    // signal no antialiasing needed
                    self.intermediate_pos[i] = f32::MIN;
                }
            }

            // Noise channel
            if self.tone_freq_vals[3] <= 0 {
                // If the counter gets below 0...
                // Flip the flip-flop
                self.tone_freq_pos[3] = -self.tone_freq_pos[3];
                if self.noise_freq != 0x80 {
                    // If not matching tone2, decrement counter
                    self.tone_freq_vals[3] += self.noise_freq * (self.num_clocks_for_sample / self.noise_freq + 1);
                }
                if self.tone_freq_pos[3] == 1 {
                    // On the positive edge of the square wave (only once per cycle)
                    let mut feedback: i32;
                    if self.registers[6] & 0x4 != 0 {
                        // White noise
                        // Calculate parity of fed-back bits for feedback
                        match self.white_noise_feedback {
                            // Do some optimised calculations for common (known) feedback values
                            // SC-3000, BBC %00000011
                            // SMS, GG, MD  %00001001
                            0x0003 | 0x0009 => {
                                // If two bits fed back, I can do Feedback=(nsr & fb) && (nsr & fb ^ fb)
                                // since that's (one or more bits set) && (not all bits set)
                                let f1 = self.noise_shift_register & self.white_noise_feedback;
                                let f2 = (self.noise_shift_register & self.white_noise_feedback) ^ self.white_noise_feedback;
                                if f1 != 0 && f2 != 0 {
                                    feedback = 1;
                                } else {
                                    feedback = 0;
                                }
                            }
                            _ => {
                                // Default handler for all other feedback values
                                // XOR fold bits into the final bit
                                feedback = self.noise_shift_register & self.white_noise_feedback;
                                feedback ^= feedback >> 8;
                                feedback ^= feedback >> 4;
                                feedback ^= feedback >> 2;
                                feedback ^= feedback >> 1;
                                feedback &= 1;
                            }
                        }
                    } else {
                        feedback = self.noise_shift_register & 1;
                    }
                    self.noise_shift_register = (self.noise_shift_register >> 1)
                        | (feedback << (self.sr_width - 1));
                }
            }
        }
    }

    fn config(&mut self, feedback: FeedbackPatterns, sr_width: SrWidths) {
        self.white_noise_feedback = feedback as i32;
        self.sr_width = sr_width as i32;
    }

    fn mute(&mut self, mask: MuteValues) {
        self.mute = mask as i32;
    }
}

impl SoundDevice<u8> for SN76489 {
    fn new() -> Self {
        SN76489::default()
    }

    fn init(&mut self, sample_rate: u32, clock: u32) {
        self.init(clock as i32, sample_rate as i32);
        self.reset();
    }

    fn get_name(&self) -> SoundDeviceName {
        SoundDeviceName::SN76489
    }

    fn reset(&mut self) {
        self.reset();
    }

    fn write(&mut self, _: u32, data: u8) {
        self.write(data);
    }

    fn update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        self.update(buffer_l, buffer_r, numsamples, buffer_pos);
    }
}

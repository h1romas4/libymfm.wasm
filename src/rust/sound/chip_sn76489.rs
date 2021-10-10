// license:BSD-3-Clause
/**
 * Rust SN76489 emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Converted from:
 *  SN76489 emulation by Nicola Salmoria
 *  https://github.com/mamedev/mame/blob/ee1e4f9683a4953cb9d88f9256017fcbc38e3144/src/devices/sound/sn76496.cpp
 */

/**
 * Original SN76489 emulation Copyright
 */
/***************************************************************************

  sn76496.c
  by Nicola Salmoria
  with contributions by others

  Routines to emulate the:
  Texas Instruments SN76489, SN76489A, SN76494/SN76496
  ( Also known as, or at least compatible with, the TMS9919 and SN94624.)
  and the Sega 'PSG' used on the Master System, Game Gear, and Megadrive/Genesis
  This chip is known as the Programmable Sound Generator, or PSG, and is a 4
  channel sound generator, with three squarewave channels and a noise/arbitrary
  duty cycle channel.

  Noise emulation for all verified chips should be accurate:

  ** SN76489 uses a 15-bit shift register with taps on bits D and E, output on E,
  XOR function.
  It uses a 15-bit ring buffer for periodic noise/arbitrary duty cycle.
  Its output is inverted.
  ** SN94624 is the same as SN76489 but lacks the /8 divider on its clock input.
  ** SN76489A uses a 15-bit shift register with taps on bits D and E, output on F,
  XOR function.
  It uses a 15-bit ring buffer for periodic noise/arbitrary duty cycle.
  Its output is not inverted.
  ** SN76494 is the same as SN76489A but lacks the /8 divider on its clock input.
  ** SN76496 is identical in operation to the SN76489A, but the audio input on pin 9 is
  documented.
  All the TI-made PSG chips have an audio input line which is mixed with the 4 channels
  of output. (It is undocumented and may not function properly on the sn76489, 76489a
  and 76494; the sn76489a input is mentioned in datasheets for the tms5200)
  All the TI-made PSG chips act as if the frequency was set to 0x400 if 0 is
  written to the frequency register.
  ** Sega Master System III/MD/Genesis PSG uses a 16-bit shift register with taps
  on bits C and F, output on F
  It uses a 16-bit ring buffer for periodic noise/arbitrary duty cycle.
  (whether it uses an XOR or XNOR needs to be verified, assumed XOR)
  (whether output is inverted or not needs to be verified, assumed to be inverted)
  ** Sega Game Gear PSG is identical to the SMS3/MD/Genesis one except it has an
  extra register for mapping which channels go to which speaker.
  The register, connected to a z80 port, means:
  for bits 7  6  5  4  3  2  1  0
           L3 L2 L1 L0 R3 R2 R1 R0
  Noise is an XOR function, and audio output is negated before being output.
  All the Sega-made PSG chips act as if the frequency was set to 0 if 0 is written
  to the frequency register.
  ** NCR8496 (as used on the Tandy 1000TX) is similar to the SN76489 but with a
  different noise LFSR pattern: taps on bits A and E, output on E, XNOR function
  It uses a 15-bit ring buffer for periodic noise/arbitrary duty cycle.
  Its output is inverted.
  ** PSSJ-3 (as used on the later Tandy 1000 series computers) is the same as the
  NCR8496 with the exception that its output is not inverted.

  28/03/2005 : Sebastien Chevalier
  Update th SN76496Write func, according to SN76489 doc found on SMSPower.
   - On write with 0x80 set to 0, when LastRegister is other then TONE,
   the function is similar than update with 0x80 set to 1

  23/04/2007 : Lord Nightmare
  Major update, implement all three different noise generation algorithms and a
  set_variant call to discern among them.

  28/04/2009 : Lord Nightmare
  Add READY line readback; cleaned up struct a bit. Cleaned up comments.
  Add more TODOs. Fixed some unsaved savestate related stuff.

  04/11/2009 : Lord Nightmare
  Changed the way that the invert works (it now selects between XOR and XNOR
  for the taps), and added R->OldNoise to simulate the extra 0 that is always
  output before the noise LFSR contents are after an LFSR reset.
  This fixes SN76489/A to match chips. Added SN94624.

  14/11/2009 : Lord Nightmare
  Removed STEP mess, vastly simplifying the code. Made output bipolar rather
  than always above the 0 line, but disabled that code due to pending issues.

  16/11/2009 : Lord Nightmare
  Fix screeching in regulus: When summing together four equal channels, the
  size of the max amplitude per channel should be 1/4 of the max range, not
  1/3. Added NCR8496.

  18/11/2009 : Lord Nightmare
  Modify Init functions to support negating the audio output. The gamegear
  psg does this. Change gamegear and sega psgs to use XOR rather than XNOR
  based on testing. Got rid of R->OldNoise and fixed taps accordingly.
  Added stereo support for game gear.

  15/01/2010 : Lord Nightmare
  Fix an issue with SN76489 and SN76489A having the wrong periodic noise periods.
  Note that properly emulating the noise cycle bit timing accurately may require
  extensive rewriting.

  24/01/2010: Lord Nightmare
  Implement periodic noise as forcing one of the XNOR or XOR taps to 1 or 0 respectively.
  Thanks to PlgDavid for providing samples which helped immensely here.
  Added true clock divider emulation, so sn94624 and sn76494 run 8x faster than
  the others, as in real life.

  15/02/2010: Lord Nightmare & Michael Zapf (additional testing by PlgDavid)
  Fix noise period when set to mirror channel 3 and channel 3 period is set to 0 (tested on hardware for noise, wave needs tests) - MZ
  Fix phase of noise on sn94624 and sn76489; all chips use a standard XOR, the only inversion is the output itself - LN, Plgdavid
  Thanks to PlgDavid and Michael Zapf for providing samples which helped immensely here.

  23/02/2011: Lord Nightmare & Enik
  Made it so the Sega PSG chips have a frequency of 0 if 0 is written to the
  frequency register, while the others have 0x400 as before. Should fix a bug
  or two on sega games, particularly Vigilante on Sega Master System. Verified
  on SMS hardware.

  27/06/2012: Michael Zapf
  Converted to modern device, legacy devices were gradually removed afterwards.

  16/09/2015: Lord Nightmare
  Fix PSG chips to have volume reg inited on reset to 0x0 based on tests by
  ValleyBell. Made Sega PSG chips start up with register 0x3 selected (volume
  for channel 2) based on hardware tests by Nemesis.

  03/09/2018: Lord Nightmare, Qbix, ValleyBell, NewRisingSun
  * renamed the NCR8496 to its correct name, based on chip pictures on VGMPF
  * fixed NCR8496's noise LFSR behavior so it is only reset if the mode bit in
  register 6 is changed.
  * NCR8496's LFSR feedback function is an XNOR, which is now supported.
  * add PSSJ-3 support for the later Tandy 1000 series computers.
  * NCR8496's output is inverted, PSSJ-3's output is not.

  10/12/2019: Michael Zapf
  * READY line handling by own emu_timer, not depending on sound_stream_update


  TODO: * Implement the TMS9919 - any difference to sn94624?
        * Implement the T6W28; has registers in a weird order, needs writes
          to be 'sanitized' first. Also is stereo, similar to game gear.
        * Factor out common code so that the SAA1099 can share some code.
        * verify NCR8496/PSSJ-3 behavior on write to mirrored registers; unlike the
          other variants, the NCR-derived variants are implied to ignore writes to
          regs 1,3,5,6,7 if 0x80 is not set. This needs to be verified on real hardware.

***************************************************************************/
use super::{
    interface::SoundChip,
    stream::{convert_sample_i2f, SoundStream},
    SoundChipType,
};

const MAX_OUTPUT: i32 = 0x7fff;

#[allow(non_snake_case)]
pub struct SN76489 {
    clock: u32,
    feedback_mask: u32,   // mask for feedback
    whitenoise_tap1: u32, // mask for white noise tap 1 (higher one, usually bit 14)
    whitenoise_tap2: u32, // mask for white noise tap 2 (lower one, usually bit 13)
    negate: bool,         // output negate flag
    stereo: bool,         // whether we're dealing with stereo or not
    clock_divider: i32,   // clock divider
    ncr_style_psg: bool,  // flag to ignore writes to regs 1,3,5,6,7 with bit 7 low
    sega_style_psg: bool, // flag to make frequency zero acts as if it is one more than max (0x3ff+1) or if it acts like 0; the initial register is pointing to 0x3 instead of 0x0; the volume reg is preloaded with 0xF instead of 0x0
    vol_table: [i32; 16], // volume table (for 4-bit to db conversion)
    register: [i32; 8],   // registers
    last_register: i32,   // last register written
    volume: [i32; 4],     // db volume of voice 0-2 and noise
    RNG: u32,             // noise generator LFSR
    current_clock: i32,
    stereo_mask: i32, // the stereo output mask
    period: [i32; 4], // Length of 1/2 of waveform
    count: [i32; 4],  // Position within the waveform
    output: [i32; 4], // 1-bit output of each channel, pre-volume
}

impl SN76489 {
    #[allow(clippy::too_many_arguments)]
    fn new(
        feedback_mask: u32,
        whitenoise_tap1: u32,
        whitenoise_tap2: u32,
        negate: bool,
        stereo: bool,
        clock_divider: i32,
        ncr_style_psg: bool,
        sega_style_psg: bool,
    ) -> Self {
        SN76489 {
            clock: 0,
            feedback_mask,
            whitenoise_tap1,
            whitenoise_tap2,
            negate,
            stereo,
            clock_divider,
            ncr_style_psg,
            sega_style_psg,
            vol_table: [0; 16],
            register: [0; 8],
            last_register: 0,
            volume: [0; 4],
            RNG: 0,
            current_clock: 0,
            stereo_mask: 0,
            period: [0; 4],
            count: [0; 4],
            output: [0; 4],
        }
    }

    ///
    /// device start
    ///
    pub fn device_start(&mut self, clock: u32) -> u32 {
        self.clock = clock;

        for i in 0..4 {
            self.volume[i] = 0;
        }

        // Sega VDP PSG defaults to selected period reg for 2nd channel
        self.last_register = if self.sega_style_psg { 3 } else { 0 };

        for i in (0..8).step_by(2) {
            self.register[i] = 0;
            // volume = 0x0 (max volume) on reset; this needs testing on chips other than SN76489A and Sega VDP PSG
            self.register[i + 1] = 0;
        }

        for i in 0..4 {
            self.output[i] = 0;
            self.period[i] = 0;
            self.count[i] = 0;
        }

        self.RNG = self.feedback_mask as u32;
        self.output[3] = self.RNG as i32 & 1;

        self.stereo_mask = 0xff; // all channels enabled
        self.current_clock = self.clock_divider - 1;

        let mut gain: i32 = 0;
        gain &= 0xff;

        // increase max output basing on gain (0.2 dB per step)
        let mut out: f64 = (MAX_OUTPUT / 4) as f64; // four channels, each gets 1/4 of the total range
        while gain > 0_i32 {
            out *= 1.023292992_f64; // = (10 ^ (0.2/20))
            gain -= 1;
        }

        // build volume table (2dB per step)
        for i in 0..15 {
            // limit volume to avoid clipping
            if out > (MAX_OUTPUT / 4) as f64 {
                self.vol_table[i] = MAX_OUTPUT / 4;
            } else {
                self.vol_table[i] = out as i32;
            }
            out /= 1.258925412_f64; /* = 10 ^ (2/20) = 2dB */
        }
        self.vol_table[15] = 0;

        // return sampling rate
        self.clock / 2
    }

    pub fn write(&mut self, data: u8) {
        let n: i32;
        let r: i32;
        let c: i32;

        if data & 0x80 != 0 {
            r = (data as i32 & 0x70) >> 4;
            self.last_register = r;
            if (self.ncr_style_psg && r == 6) && (data & 0x04 != self.register[6] as u8 & 0x04) {
                // NCR-style PSG resets the LFSR only on a mode write which actually changes the state of bit 2 of register 6
                self.RNG = self.feedback_mask as u32;
            }
            self.register[r as usize] = (self.register[r as usize] & 0x3f0) | (data as i32 & 0x0f);
        } else {
            r = self.last_register;
        }

        c = r >> 1;
        match r {
            0 | 2 | 4 => {
                // tone 0: frequency
                // tone 1: frequency
                // tone 2: frequency
                if data & 0x80 == 0 {
                    self.register[r as usize] =
                        (self.register[r as usize] & 0x0f) | ((data as i32 & 0x3f) << 4);
                }
                if self.register[r as usize] != 0 || !self.sega_style_psg {
                    self.period[c as usize] = self.register[r as usize];
                } else {
                    self.period[c as usize] = 0x400;
                }

                if r == 4 {
                    // update noise shift frequency
                    if self.register[6] & 0x03 == 0x03 {
                        self.period[3] = self.period[2] << 1;
                    }
                }
            }
            1 | 3 | 5 | 7 => {
                // tone 0: volume
                // tone 1: volume
                // tone 2: volume
                // noise: volume
                self.volume[c as usize] = self.vol_table[(data & 0x0f) as usize];
                if data & 0x80 == 0 {
                    self.register[r as usize] =
                        (self.register[r as usize] & 0x3f0) | (data & 0x0f) as i32;
                }
            }
            6 => {
                // noise: frequency, mode
                if data & 0x80 == 0 {
                    // println!("sn76496_base_device: write to reg 6 with bit 7 clear; data was {}, new write is {}! report this to LN!\n", self.register[6], data);
                }
                if data & 0x80 == 0 {
                    self.register[r as usize] =
                        (self.register[r as usize] & 0x3f0) | (data as i32 & 0x0f);
                }
                n = self.register[6];
                // N/512,N/1024,N/2048,Tone #3 output
                self.period[3] = if n & 3 == 3 {
                    self.period[2] << 1
                } else {
                    1 << (5 + (n & 3))
                };
                if !self.ncr_style_psg {
                    self.RNG = self.feedback_mask;
                }
            }
            _ => {
                // Nonexistent data patterns
            }
        }
    }

    pub fn sound_stream_update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        length: usize,
        buffer_pos: usize,
    ) {
        let mut out: i32;
        let mut out2: i32 = 0;

        for sampindex in 0..length {
            // clock chip once
            if self.current_clock > 0 {
                // not ready for new divided clock
                self.current_clock -= 1;
            } else {
                // ready for new divided clock, make a new sample
                self.current_clock = self.clock_divider - 1;

                // handle channels 0,1,2
                for i in 0..3 {
                    self.count[i] -= 1;
                    if self.count[i] <= 0 {
                        self.output[i] ^= 1;
                        self.count[i] = self.period[i];
                    }
                }

                // handle channel 3
                self.count[3] -= 1;
                if self.count[3] <= 0 {
                    // if noisemode is 1, both taps are enabled
                    // if noisemode is 0, the lower tap, whitenoisetap2, is held at 0
                    // The != was a bit-XOR (^) before
                    let tap2 = if self.ncr_style_psg {
                        self.whitenoise_tap2
                    } else {
                        0
                    };
                    let rng = self.RNG;
                    self.RNG >>= 1;
                    if ((rng & self.whitenoise_tap1) != 0)
                        != (((rng & self.whitenoise_tap2) != tap2) && self.in_noise_mode())
                    {
                        self.RNG |= self.feedback_mask;
                    }
                    self.output[3] = (self.RNG & 1) as i32;

                    self.count[3] = self.period[3];
                }
            }

            if self.stereo {
                out = if self.stereo_mask & 0x10 != 0 && self.output[0] != 0 {
                    self.volume[0]
                } else {
                    0
                } + if self.stereo_mask & 0x20 != 0 && self.output[1] != 0 {
                    self.volume[1]
                } else {
                    0
                } + if self.stereo_mask & 0x40 != 0 && self.output[2] != 0 {
                    self.volume[2]
                } else {
                    0
                } + if self.stereo_mask & 0x80 != 0 && self.output[3] != 0 {
                    self.volume[3]
                } else {
                    0
                };
                out2 = if self.stereo_mask & 0x1 != 0 && self.output[0] != 0 {
                    self.volume[0]
                } else {
                    0
                } + if self.stereo_mask & 0x2 != 0 && self.output[1] != 0 {
                    self.volume[1]
                } else {
                    0
                } + if self.stereo_mask & 0x4 != 0 && self.output[2] != 0 {
                    self.volume[2]
                } else {
                    0
                } + if self.stereo_mask & 0x8 != 0 && self.output[3] != 0 {
                    self.volume[3]
                } else {
                    0
                };
            } else {
                out = if self.output[0] != 0 {
                    self.volume[0]
                } else {
                    0
                } + if self.output[1] != 0 {
                    self.volume[1]
                } else {
                    0
                } + if self.output[2] != 0 {
                    self.volume[2]
                } else {
                    0
                } + if self.output[3] != 0 {
                    self.volume[3]
                } else {
                    0
                };
            }

            if self.negate {
                out = -out;
                out2 = -out2;
            }

            if self.stereo {
                buffer_l[sampindex + buffer_pos] += convert_sample_i2f(out);
                buffer_r[sampindex + buffer_pos] += convert_sample_i2f(out2);
            } else {
                buffer_l[sampindex + buffer_pos] += convert_sample_i2f(out / 2);
                buffer_r[sampindex + buffer_pos] += convert_sample_i2f(out / 2);
            }
        }
    }

    #[inline]
    fn in_noise_mode(&self) -> bool {
        self.register[6] & 4 != 0
    }
}

impl SoundChip for SN76489 {
    fn new(sound_device_name: SoundChipType) -> Self {
        match sound_device_name {
            SoundChipType::SEGAPSG => {
                SN76489::new(0x8000, 0x01, 0x08, true, false, 8, false, false)
            }
            SoundChipType::SN76489 => SN76489::new(0x4000, 0x01, 0x02, true, false, 8, false, true),
            _ => {
                panic!("not supported sound chip type");
            }
        }
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start(clock)
    }

    fn reset(&mut self) {
        todo!("WIP");
    }

    fn write(&mut self, _: usize, _: u32, data: u32) {
        self.write(data as u8);
    }

    fn tick(&mut self, _: usize, sound_stream: &mut dyn SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.sound_stream_update(&mut l, &mut r, 1, 0);
        sound_stream.push(l[0], r[0]);
    }
}

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
use super::{RomIndex, interface::{RomBank, SoundChip}, stream::{OutputChannel, SoundStream, convert_sample_i2f}};
/**
 * Rust OKI MSM6258 emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Converted from:
 *  MAME
 *  copyright-holders:Barry Rodewald
 *  https://github.com/mamedev/mame/blob/master/src/devices/sound/okim6258.cpp
 *  rev. 70743c6fb2602a5c2666c679b618706eabfca2ad
 */

/**
 * Original SN76489 emulation Copyright
 */
/**********************************************************************************************
 *
 *   OKI MSM6258 ADPCM
 *
 *   TODO:
 *   3-bit ADPCM support
 *   Recording?
 *
 **********************************************************************************************/
use crate::sound::SoundChipType;

const COMMAND_STOP: u8 = 1 << 0;
const COMMAND_PLAY: u8 = 1 << 1;
const COMMAND_RECORD: u8 = 1 << 2;
const STATUS_PLAYING: u8 = 1 << 1;
const STATUS_RECORDING: u8 = 1 << 2;

const DIVIDERS: [u32; 4] = [1024, 768, 512, 512];
/* step size index shift table */
const INDEX_SHIFT: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

pub struct OKIM6258 {
    clock: u32,
    status: u8,
    start_divider: u32,
    divider: u32,     /* master clock divider */
    adpcm_type: u8,   /* 3/4 bit ADPCM select */
    data_in: u8,      /* ADPCM data-in register */
    nibble_shift: u8, /* nibble select */
    output_bits: u8, /* D/A precision is 10-bits but 12-bit data can be output serially to an external DAC */
    signal: i32,
    step: i32,
    diff_lookup: [i32; 49 * 16], /* lookup table for the precomputed difference */
}

impl OKIM6258 {
    pub fn default() -> Self {
        OKIM6258 {
            clock: 0,
            status: 0,
            start_divider: 0,
            divider: 512,
            adpcm_type: 0,
            data_in: 0,
            nibble_shift: 0,
            output_bits: 0,
            signal: 0,
            step: 0,
            diff_lookup: [0; 49 * 16],
            // addtional
        }
    }

    pub fn device_start(&mut self, clock: u32) -> u32 {
        self.clock = clock;
        self.compute_tables();

        self.divider = DIVIDERS[self.start_divider as usize];

        self.signal = -1;
        self.step = 0;

        // return sampling rate
        // 7812, 8000000, 1024
        // 15625, 8000000, 512
        self.clock / self.divider
    }

    pub fn device_reset(&mut self) {
        self.signal = -2;
        self.step = 0;
        self.status = 0;
    }

    pub fn sound_stream_update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        length: usize,
        buffer_pos: usize,
    ) {
        if self.status & STATUS_PLAYING != 0 {
            let mut nibble_shift = self.nibble_shift;

            for sampindex in 0..length {
                /* Compute the new amplitude and update the current step */
                let nibble: u8 = (self.data_in >> nibble_shift) & 0xf;

                /* Output to the buffer */
                let sample: i16 = self.clock_adpcm(nibble);

                nibble_shift ^= 4;

                buffer_l[sampindex + buffer_pos] = convert_sample_i2f(sample as i32) / 2_f32;
                buffer_r[sampindex + buffer_pos] = convert_sample_i2f(sample as i32) / 2_f32;
            }

            /* Update the parameters */
            self.nibble_shift = nibble_shift;
        } else {
            buffer_l.fill(0_f32);
            buffer_r.fill(0_f32);
        }
    }

    pub fn data_w(&mut self, data: u8) {
        // println!("data_w: {}", data);
        // printf("data_w: %d\n", data);
        self.data_in = data;
        self.nibble_shift = 0;
    }

    pub fn ctrl_w(&mut self, data: u8) {
        // println!("ctrl_w: {}", data);
        // printf("ctrl_w: %x\n", data);
        if (data & COMMAND_STOP) != 0 {
            self.status &= !(STATUS_PLAYING | STATUS_RECORDING);
            return;
        }

        if (data & COMMAND_PLAY) != 0 && (self.status & STATUS_PLAYING) == 0 {
            self.status |= STATUS_PLAYING;

            /* Also reset the ADPCM parameters */
            self.signal = -2;
            self.step = 0;
            self.nibble_shift = 0;
        } else {
            self.status &= !STATUS_PLAYING;
        }

        if (data & COMMAND_RECORD) != 0 {
            // logerror("M6258: Record enabled\n");
            self.status |= STATUS_RECORDING;
        } else {
            self.status &= !STATUS_RECORDING;
        }
    }

    pub fn set_divider(&mut self, val: u32) -> u32 {
        self.divider = DIVIDERS[val as usize];
        // return sampling rate
        self.clock / self.divider
    }

    pub fn set_outbits(&mut self, outbit: u8) {
        self.output_bits = outbit;
    }

    pub fn set_type(&mut self, typ: u8) {
        self.adpcm_type = typ;
    }

    fn clock_adpcm(&mut self, nibble: u8) -> i16 {
        let max: i32 = (1 << (self.output_bits - 1)) as i32 - 1;
        let min: i32 = -(1 << (self.output_bits - 1) as i32);

        self.signal += self.diff_lookup[(self.step * 16 + (nibble & 15) as i32) as usize];

        /* clamp to the maximum */
        if self.signal > max {
            self.signal = max;
        } else if self.signal < min {
            self.signal = min;
        }

        /* adjust the step size and clamp */
        self.step += INDEX_SHIFT[(nibble & 7) as usize];
        if self.step > 48 {
            self.step = 48;
        } else if self.step < 0 {
            self.step = 0;
        }

        // println!("{}, {}, {}, {}, {}, {}, {}", self.data_in, self.signal, self.step, max, min, nibble, self.output_bits);
        // printf("%d, %d, %d, %d, %d, %d, %d\n", m_data_in, m_signal, m_step, max, min, nibble, m_output_bits);

        (self.signal << 4) as i16
    }

    fn compute_tables(&mut self) {
        /* nibble to bit map */
        let nbl2bit: [[i32; 4]; 16] = [
            [1, 0, 0, 0],
            [1, 0, 0, 1],
            [1, 0, 1, 0],
            [1, 0, 1, 1],
            [1, 1, 0, 0],
            [1, 1, 0, 1],
            [1, 1, 1, 0],
            [1, 1, 1, 1],
            [-1, 0, 0, 0],
            [-1, 0, 0, 1],
            [-1, 0, 1, 0],
            [-1, 0, 1, 1],
            [-1, 1, 0, 0],
            [-1, 1, 0, 1],
            [-1, 1, 1, 0],
            [-1, 1, 1, 1],
        ];

        /* loop over all possible steps */
        for step in 0..=48 {
            /* compute the step value */
            let stepval = f32::floor(16.0_f32 * f32::powf(11.0_f32 / 10.0_f32, step as f32)) as i32;

            /* loop over all nibbles and compute the difference */
            #[allow(clippy::needless_range_loop)]
            for nib in 0..16 {
                self.diff_lookup[(step * 16 + nib) as usize] = nbl2bit[nib][0]
                    * (stepval * nbl2bit[nib][1]
                        + stepval / 2 * nbl2bit[nib][2]
                        + stepval / 4 * nbl2bit[nib][3]
                        + stepval / 8) as i32;
            }
        }
    }
}

impl SoundChip for OKIM6258 {
    fn new(_sound_device_name: SoundChipType) -> Self {
        OKIM6258::default()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start(clock)
    }

    fn reset(&mut self) {
        self.device_reset();
    }

    fn write(&mut self, _: usize, offset: u32, data: u32, sound_stream: &mut dyn SoundStream) {
        match offset {
            0x0 => self.ctrl_w((data & 0xff) as u8),
            0x1 => self.data_w((data & 0xff) as u8),
            0x2 => {
                match data {
                    0 => sound_stream.set_output_channel(OutputChannel::Stereo),
                    1 => sound_stream.set_output_channel(OutputChannel::Left),
                    2 => sound_stream.set_output_channel(OutputChannel::Right),
                    3 => sound_stream.set_output_channel(OutputChannel::Stereo), /* TODO: Mute ? */
                    _ => { panic!("ignore set_output_channel ({})", data) }
                }
            }
            0x8 => todo!("change data clock"),
            0xc => todo!("restore initial divider"),
            // (hack) addtional port map offset for lib
            0x10 => sound_stream.change_sapmling_rate(self.set_divider(data)),
            0x11 => self.set_outbits((data & 0xff) as u8),
            0x12 => self.set_type((data & 0xff) as u8),
            _ => {
                panic!("chip_okim6258 unknown offset")
            }
        }
    }

    fn tick(&mut self, _: usize, sound_stream: &mut dyn SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.sound_stream_update(&mut l, &mut r, 1, 0);
        sound_stream.push(l[0], r[0]);
    }

    fn set_rombank(&mut self, _: RomIndex, _: RomBank) {
        /* nothing to do */
    }

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {
        /* nothing to do */
    }
}

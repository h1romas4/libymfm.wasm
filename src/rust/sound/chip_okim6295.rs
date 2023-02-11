// license:BSD-3-Clause
/**
 * Rust OKIM 6295 ports by
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Porting from:
 *  MAME
 *  copyright-holders:Mirko Buffoni,Aaron Giles
 *  https://github.com/mamedev/mame/blob/master/src/devices/sound/okim6295.cpp
 *  rev. 92ffd8d0269cd49b0ef256cd3912269387f8d318
 */

/**
 * Original SN76489 emulation Copyright
 */
// license:BSD-3-Clause
// copyright-holders:Mirko Buffoni,Aaron Giles

/***************************************************************************

    okim6295.cpp

    OKIM 6295 ADCPM sound chip.

****************************************************************************

    Library to transcode from an ADPCM source to raw PCM.
    Written by Buffoni Mirko in 08/06/97
    References: various sources and documents.

    R. Belmont 31/10/2003
    Updated to allow a driver to use both MSM6295s and "raw" ADPCM voices
    (gcpinbal). Also added some error trapping for MAME_DEBUG builds

****************************************************************************

    OKIM 6295 ADPCM chip:

    Command bytes are sent:

        1xxx xxxx = start of 2-byte command sequence, xxxxxxx is the sample
                    number to trigger
        abcd vvvv = second half of command; one of the abcd bits is set to
                    indicate which voice the v bits seem to be volumed

        0abc d000 = stop playing; one or more of the abcd bits is set to
                    indicate which voice(s)

    Status is read:

        ???? abcd = one bit per voice, set to 0 if nothing is playing, or
                    1 if it is active

    OKI Semiconductor produced this chip in two package variants. The
    44-pin QFP version, MSM6295GS, is the original one and by far the more
    common of the two. The 42-pin DIP version, MSM6295VRS, omits A17 and
    RD, which limits its ROM addressing to one megabit instead of two.

***************************************************************************/
use std::{cell::RefCell, rc::Rc};
use super::{
    rom::{read_byte, set_rom_bus, Decoder, RomBank, RomBus},
    sound_chip::SoundChip,
    stream::{SoundStream, convert_int},
    RomBusType, RomIndex, SoundChipType,
};

const OKIM6295_VOICES: usize = 4;
const PIN7_LOW: u8 = 0;
const PIN7_HIGH: u8 = 1;

struct OkiAdpcmState {
    signal: i32,
    step: i32,
    loop_signal: i32,
    loop_step: i32,
    saved: bool,
    index_shift: [i8; 8],
    diff_lookup: [i32; 49 * 16],
    tables_computed: bool,
}

#[allow(dead_code)]
impl OkiAdpcmState {
    pub fn new() -> Self {
        let mut state = OkiAdpcmState {
            signal: 0,
            step: 0,
            loop_signal: 0,
            loop_step: 0,
            saved: false,
            index_shift: [-1, -1, -1, -1, 2, 4, 6, 8],
            diff_lookup: [0; 49 * 16],
            tables_computed: false,
        };
        state.compute_tables();
        state.reset();
        state
    }

    pub fn reset(&mut self) {
        // reset the signal/step
        self.signal = 0;
        self.loop_signal = self.signal;
        self.step = 0;
        self.loop_step = self.step;
        self.saved = false;
    }

    pub fn clock(&mut self, nibble: u8) -> i32 {
        // update the signal
        self.signal += self.diff_lookup[(self.step * 16 + (nibble & 15) as i32) as usize];

        // clamp to the maximum
        self.signal = self.signal.clamp(-2048, 2047);

        // adjust the step size and clamp
        self.step += (self.index_shift[(nibble & 7) as usize]) as i32;
        self.step = self.step.clamp(0, 48);

        // return the signal
        self.signal
    }

    pub fn output(&self) -> i32 {
        self.signal
    }

    pub fn save(&mut self) {
        if !self.saved {
            self.loop_signal = self.signal;
            self.loop_step = self.step;
            self.saved = true;
        }
    }

    pub fn restore(&mut self) {
        self.signal = self.loop_signal;
        self.step = self.loop_step;
    }

    pub fn compute_tables(&mut self) {
        // skip if we already did it
        if self.tables_computed {
            return;
        }
        self.tables_computed = true;

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

        // loop over all possible steps
        for step in 0..=48 {
            // compute the step value
            let stepval = f32::floor(16.0_f32 * f32::powf(11.0_f32 / 10.0_f32, step as f32)) as i32;

            // loop over all nibbles and compute the difference
            #[allow(clippy::needless_range_loop)]
            for nib in 0..16 {
                self.diff_lookup[step * 16 + nib] = nbl2bit[nib][0]
                    * (stepval * nbl2bit[nib][1]
                        + stepval / 2 * nbl2bit[nib][2]
                        + stepval / 4 * nbl2bit[nib][3]
                        + stepval / 8);
            }
        }
    }
}

struct OkiVoice {
    adpcm: OkiAdpcmState,
    playing: bool,
    base_offset: usize,
    sample: u32,
    count: usize,
    volume: f32,
}

impl OkiVoice {
    pub fn new() -> Self {
        OkiVoice {
            adpcm: OkiAdpcmState::new(),
            playing: false,
            base_offset: 0,
            sample: 0,
            count: 0,
            volume: 0_f32,
        }
    }

    pub fn generate_adpcm(&mut self, rombank: &RomBank) -> f32 {
        let mut buffer: f32 = 0_f32;
        // skip if not active
        if !self.playing {
            return buffer;
        }

        // fetch the next sample byte
        let nibble = read_byte(
            rombank,
            self.base_offset + self.sample as usize / 2,
        ) >> (((self.sample & 1) << 2) ^ 4);
        // output to the buffer, scaling by the volume
        // signal in range -2048..2047
        buffer = convert_int((self.adpcm.clock(nibble) as f32 * self.volume) as i32, 2048);

        // next!
        self.sample += 1;
        if self.sample >= self.count as u32 {
            self.playing = false;
        }

        buffer
    }
}

pub struct OKIM6295 {
    voice: [OkiVoice; OKIM6295_VOICES],
    command: i32,
    pin7_state: u8,
    volume_table: [f32; 16],
    // add by libymfm.wasm
    clock: u32,
    rom_bank: RomBank,
    rom_decoder: RomBus<OKIM6295RomDecoder>,
}

#[allow(dead_code)]
impl OKIM6295 {
    #[allow(clippy::eq_op)]
    #[allow(clippy::erasing_op)]
    pub fn from() -> Self {
        OKIM6295 {
            voice: [
                OkiVoice::new(),
                OkiVoice::new(),
                OkiVoice::new(),
                OkiVoice::new(),
            ],
            command: -1,
            pin7_state: 0,
            volume_table: [
                (0x20 as f32 / 0x20 as f32), //   0 dB
                (0x16 as f32 / 0x20 as f32), //  -3.2 dB
                (0x10 as f32 / 0x20 as f32), //  -6.0 dB
                (0x0b as f32 / 0x20 as f32), //  -9.2 dB
                (0x08 as f32 / 0x20 as f32), // -12.0 dB
                (0x06 as f32 / 0x20 as f32), // -14.5 dB
                (0x04 as f32 / 0x20 as f32), // -18.0 dB
                (0x03 as f32 / 0x20 as f32), // -20.5 dB
                (0x02 as f32 / 0x20 as f32), // -24.0 dB
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
                (0x00 as f32 / 0x20 as f32),
            ],
            clock: 0,
            rom_bank: None,
            rom_decoder: None,
        }
    }

    pub fn device_start(&mut self, clock: u32) -> u32 {
        self.clock = clock;

        if self.pin7_state != PIN7_LOW && self.pin7_state != PIN7_HIGH {
            self.pin7_state = 0;
        }

        let divisor = if self.pin7_state != 0 { 132 } else { 165 };

        self.clock / divisor
    }

    pub fn device_reset(&mut self) {
        for elem in self.voice.as_mut() {
            elem.playing = false;
        }
    }

    pub fn device_clock_changed(&mut self) -> u32 {
        let divisor = if self.pin7_state != 0 { 132 } else { 165 };
        self.clock / divisor
    }

    pub fn sound_stream_update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32]) {
        // reset the output stream
        let mut output: f32 = 0_f32;

        // iterate over voices and accumulate sample data
        for elem in self.voice.as_mut() {
            output += elem.generate_adpcm(&self.rom_bank);
        }

        // TODO:
        buffer_l[0] = output / 4_f32;
        buffer_r[0] = output / 4_f32;
    }

    pub fn rom_bank_updated(&mut self) {}

    pub fn set_pin7(&mut self, pin7: u8) {
        self.pin7_state = if pin7 != 0 { 1 } else { 0 };
        self.device_clock_changed();
    }

    pub fn read(&self) -> u8 {
        let mut result: u8 = 0xf0; // naname expects bits 4-7 to be 1

        // set the bit to 1 if something is playing on a given channel
        for voicenum in 0..OKIM6295_VOICES {
            if self.voice[voicenum].playing {
                result |= 1 << voicenum as u8;
            }
        }

        result
    }

    pub fn write(&mut self, command: u8) {
        // if a command is pending, process the second half
        if self.command != -1 {
            // the manual explicitly says that it's not possible to start multiple voices at the same time
            let mut voicemask = command >> 4;
            //if (voicemask != 0 && voicemask != 1 && voicemask != 2 && voicemask != 4 && voicemask != 8)
            //  popmessage("OKI6295 start %x contact MAMEDEV", voicemask);

            // determine which voice(s) (voice is set by a 1 bit in the upper 4 bits of the second byte)
            for voicenum in 0..OKIM6295_VOICES {
                if voicemask & 1 != 0 {
                    let voice = &mut self.voice[voicenum];

                    if !voice.playing {
                        // fixes Got-cha and Steel Force
                        // determine the start/stop positions
                        let base: usize = (self.command * 8) as usize;

                        let mut start = ((read_byte(&self.rom_bank, base)) as usize) << 16;
                        start |= (read_byte(&self.rom_bank, base + 1) as usize) << 8;
                        start |= read_byte(&self.rom_bank, base + 2) as usize;
                        start &= 0x3ffff;

                        let mut stop = ((read_byte(&self.rom_bank, base + 3)) as usize) << 16;
                        stop |= (read_byte(&self.rom_bank, base + 4) as usize) << 8;
                        stop |= read_byte(&self.rom_bank, base + 5) as usize;
                        stop &= 0x3ffff;

                        if start < stop {
                            // set up the voice to play this sample
                            voice.playing = true;
                            voice.base_offset = start;
                            voice.sample = 0;
                            voice.count = 2 * (stop - start + 1);

                            // also reset the ADPCM parameters
                            voice.adpcm.reset();
                            voice.volume = self.volume_table[(command & 0x0f) as usize];
                        } else {
                            // invalid samples go here
                            // logerror("Requested to play invalid sample %02x\n", m_command);
                        }
                    } else {
                        // logerror("Requested to play sample %02x on non-stopped voice\n", m_command);
                    }
                }
                voicemask >>= 1;
            }
            // reset the command
            self.command = -1;
        } else if command & 0x80 != 0 {
            // if this is the start of a command, remember the sample number for next time
            self.command = (command & 0x7f) as i32;
        } else {
            // otherwise, see if this is a silence command
            let mut voicemask = command >> 3;
            for voicenum in 0..OKIM6295_VOICES {
                if voicemask & 1 != 0 {
                    self.voice[voicenum].playing = false;
                }
                voicemask >>= 1;
            }
        }
    }
}

struct OKIM6295RomDecoder {
    nmk112_enable: u8,
    bank: u8,
    nmk112_bank: [u8; 4],
}

impl OKIM6295RomDecoder {
    pub fn new() -> Self {
        Self {
            nmk112_enable: 0,
            bank: 0,
            nmk112_bank: [0; 4],
        }
    }
}

impl Decoder for OKIM6295RomDecoder {
    fn decode(&self, rombank: &super::rom::RomSet, address: usize) -> u32 {
        rombank.read(if self.nmk112_enable != 0 {
            if address < 0x400 && (self.nmk112_enable & 0x80) != 0 {
                ((self.nmk112_bank[(address >> 8) & 0x3]) as usize) << 16
                    | address & 0x3ff
            } else {
                ((self.nmk112_bank[(address >> 16) & 0x3]) as usize) << 16
                    | address & 0xffff
            }
        } else {
            (self.bank as usize * 0x40000) | address
        }) as u32
    }
}

impl SoundChip for OKIM6295 {
    fn create(_sound_device_name: SoundChipType) -> Self {
        OKIM6295::from()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start(clock);
        self.device_reset();
        self.device_clock_changed()
    }

    fn reset(&mut self) {
        todo!("not impliments");
    }

    fn write(&mut self, _: usize, offset: u32, data: u32, sound_stream: &mut dyn SoundStream) {
        let mut data = (data & 0xff) as u8;
        match offset {
            0x0 => self.write(data),
            0x8 => {
                self.clock &= !0x000000ff;
                self.clock |= data as u32;
            }
            0x9 => {
                self.clock &= !0x0000ff00;
                self.clock |= u32::from(data) << 8;
            }
            0xa => {
                self.clock &= !0x00ff0000;
                self.clock |= u32::from(data) << 16;
            }
            0xb => {
                data &= 0x7f; // fix a bug in MAME VGM logs
                self.clock &= !0xff000000;
                self.clock |= u32::from(data) << 24;
                sound_stream.change_sampling_rate(self.device_clock_changed());
            }
            0xc => self.set_pin7(data),
            0xe => {
                if let Some(decoder) = &self.rom_decoder {
                    decoder.as_ref().borrow_mut().nmk112_enable = data;
                }
            }
            0xf => {
                if let Some(decoder) = &self.rom_decoder {
                    decoder.as_ref().borrow_mut().bank = data;
                }
            }
            0x10..=0x13 => {
                if let Some(decoder) = &self.rom_decoder {
                    decoder.as_ref().borrow_mut().nmk112_bank[(offset & 0x03) as usize] = data;
                }
            }
            _ => { /* panic!("chip_okim6295 unknown offset") */ }
        }
    }

    fn tick(&mut self, _: usize, sound_stream: &mut dyn SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.sound_stream_update(&mut l, &mut r);
        sound_stream.push(l[0], r[0]);
    }

    fn set_rom_bank(&mut self, _ /* OKIM6295 has only one RomBank */: RomIndex, rombank: RomBank) {
        self.rom_bank = rombank;
        self.rom_bank_updated();
    }

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {
        self.rom_bank_updated();
    }

    fn set_rom_bus(&mut self, rom_bus_type: Option<RomBusType>) {
        if let Some(RomBusType::OKIM6295) = rom_bus_type {
            // create rom decoder
            let rom_decoder = OKIM6295RomDecoder::new();
            // share bus to rom
            let rom_decoder = Rc::new(RefCell::new(rom_decoder));
            set_rom_bus(&self.rom_bank, Some(rom_decoder.clone()));
            // type state
            self.rom_decoder = Some(rom_decoder);
        }
    }
}

use crate::sound::{SoundChip, convert_sample_i2f};

use super::{SoundChipType, SoundStream};

///
/// FFI interface
///
#[link(name = "ymfm")]
extern {
    fn ymfm_add_chip(chip_num: u16, clock: u32) -> u32;
    fn ymfm_write(chip_num: u16, index: u16, reg: u32, data: u8);
    fn ymfm_generate(chip_num: u16, index: u16, output_start: i64, output_step: i64, buffer: *const i32);
    fn ymfm_remove_chip(chip_num: u16);
}

type EmulatedTime = i64;

#[allow(non_camel_case_types)]
#[derive(Clone)]
#[derive(Copy)]
pub enum ChipType {
    CHIP_YM2149 = 0,
    CHIP_YM2151 = 1,
    CHIP_YM2203 = 2,
    CHIP_YM2413 = 3,
    CHIP_YM2608 = 4,
    CHIP_YM2610 = 5,
    CHIP_YM2612 = 6,
    CHIP_YM3526 = 7,
    CHIP_Y8950 = 8,
    CHIP_YM3812 = 9,
    CHIP_YMF262 = 10,
    CHIP_YMF278B = 11,
}

pub struct YmFm {
    chip_type: ChipType,
    clock: u32,
    sampling_rate: u32,
    output_pos: EmulatedTime,
    output_step: EmulatedTime,
}

impl YmFm {
    fn init(&mut self, clock: u32) -> u32 {
        unsafe { self.sampling_rate = ymfm_add_chip(self.chip_type as u16, clock); }
        self.clock = clock;
        // TODO: 44100 yet
        self.output_step = 0x100000000 / i64::from(44100);
        self.sampling_rate
    }

    fn write_chip(&self, index: usize, offset: u32, data: u8) {
        unsafe { ymfm_write(self.chip_type as u16, index as u16, offset, data); }
    }

    #[allow(clippy::missing_safety_doc)]
    fn generate(&mut self, index: usize, buffer: &mut [i32; 2]) {
        let generate_buffer: [i32; 2] = [0, 0];
        unsafe { ymfm_generate(self.chip_type as u16, index as u16, self.output_pos, self.output_step, generate_buffer.as_ptr()); }
        buffer[0] = generate_buffer[0];
        buffer[1] = generate_buffer[1];

        // vgm == sampling_rate == 44100Hz
        if self.output_pos as u64 + self.output_step as u64 > i64::MAX as u64 {
           self.output_pos = ((self.output_pos as u64 + self.output_step as u64) - i64::MAX as u64) as i64;
        } else {
           self.output_pos += self.output_step;
        }
    }
}

impl Drop for YmFm {
    fn drop(&mut self) {
        if self.clock != 0 {
            unsafe { ymfm_remove_chip(self.chip_type as u16) }
        }
    }
}

impl SoundChip for YmFm {
    fn new(sound_device_name: SoundChipType) -> Self {
        let chip_type = match sound_device_name {
            SoundChipType::YM2151 => ChipType::CHIP_YM2151,
            SoundChipType::YM2203 => ChipType::CHIP_YM2203,
            SoundChipType::YM2149 => ChipType::CHIP_YM2149,
            SoundChipType::YM2612 => ChipType::CHIP_YM2612,
            SoundChipType::YM2413 => ChipType::CHIP_YM2413,
            _ => todo!(),
        };
        YmFm {
            chip_type,
            clock: 0,
            sampling_rate: 0,
            output_pos: 0,
            output_step: 0
        }
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.init(clock)
    }

    fn reset(&mut self) {
    }

    fn write(&mut self, index: usize, offset: u32, data: u32) {
        self.write_chip(index, offset, data as u8);
    }

    fn update(
        &mut self,
        index: usize,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        let mut buffer: [i32; 2] = [0, 0];
        for i in 0..numsamples {
            self.generate(index, &mut buffer);
            buffer_l[buffer_pos + i] += convert_sample_i2f(buffer[0]);
            buffer_r[buffer_pos + i] += convert_sample_i2f(buffer[1]);
        }
    }

    fn tick(&mut self, index: usize, sound_stream: &mut SoundStream) {
        let mut l: [f32; 1] = [0_f32];
        let mut r: [f32; 1] = [0_f32];
        self.update(index, &mut l, &mut r, 1, 0);
        sound_stream.push(l[0], r[0]);
    }
}

// license:BSD-3-Clause
use super::{SoundChipType, interface::SoundChip, stream::{SoundStream, convert_sample_i2f}};

///
/// FFI interface
///
#[link(name = "ymfm")]
extern "C" {
    fn ymfm_add_chip(chip_num: u16, clock: u32) -> u32;
    fn ymfm_write(chip_num: u16, index: u16, reg: u32, data: u8);
    fn ymfm_generate(chip_num: u16, index: u16, buffer: *const i32);
    fn ymfm_remove_chip(chip_num: u16);
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Clone, Copy)]
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
}

impl YmFm {
    fn init(&mut self, clock: u32) -> u32 {
        unsafe {
            self.sampling_rate = ymfm_add_chip(self.chip_type as u16, clock);
        }
        self.clock = clock;
        self.sampling_rate
    }

    fn write_chip(&self, index: usize, offset: u32, data: u8) {
        unsafe {
            ymfm_write(self.chip_type as u16, index as u16, offset, data);
        }
    }

    #[allow(clippy::missing_safety_doc)]
    fn generate(&mut self, index: usize, buffer: &mut [i32; 2]) {
        let generate_buffer: [i32; 2] = [0, 0];
        unsafe {
            ymfm_generate(
                self.chip_type as u16,
                index as u16,
                generate_buffer.as_ptr(),
            );
        }
        buffer[0] = generate_buffer[0];
        buffer[1] = generate_buffer[1];
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
        }
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.init(clock)
    }

    fn reset(&mut self) {}

    fn write(&mut self, index: usize, offset: u32, data: u32) {
        self.write_chip(index, offset, data as u8);
    }

    fn tick(&mut self, index: usize, sound_stream: &mut dyn SoundStream) {
        let mut buffer: [i32; 2] = [0, 0];
        self.generate(index, &mut buffer);
        sound_stream.push(convert_sample_i2f(buffer[0]), convert_sample_i2f(buffer[1]));
    }
}

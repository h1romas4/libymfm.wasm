use crate::sound::{SoundDevice, SoundDeviceName, convert_sample_i2f};

///
/// FFI interface
///
#[link(name = "ymfm")]
extern {
    fn ymfm_add_chip(chip_num: u16, clock: u32);
    fn ymfm_write(chip_num: u16, reg: u32, data: u8);
    fn ymfm_generate(chip_num: u16, output_start: i64, output_step: i64, buffer: *const i32);
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
    pub fn from(chip_type: ChipType) -> Self {
        YmFm {
            chip_type,
            clock: 0,
            sampling_rate: 0,
            output_pos: 0,
            output_step: 0
        }
    }

    fn init(&mut self, sampling_rate: u32, clock: u32) {
        match self.chip_type {
            ChipType::CHIP_YM2149 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2149 as u16, clock) }
            },
            ChipType::CHIP_YM2151 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2151 as u16, clock) }
            },
            ChipType::CHIP_YM2203 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2203 as u16, clock) }
            },
            ChipType::CHIP_YM2413 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2413 as u16, clock) }
            },
            ChipType::CHIP_YM2608 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2608 as u16, clock) }
            },
            ChipType::CHIP_YM2610 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2610 as u16, clock) }
            },
            ChipType::CHIP_YM2612 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM2612 as u16, clock) }
            },
            ChipType::CHIP_YM3526 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM3526 as u16, clock) }
            },
            ChipType::CHIP_Y8950 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_Y8950 as u16, clock) }
            },
            ChipType::CHIP_YM3812 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YM3812 as u16, clock) }
            },
            ChipType::CHIP_YMF262 => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YMF262 as u16, clock) }
            },
            ChipType::CHIP_YMF278B => {
                unsafe { ymfm_add_chip(ChipType::CHIP_YMF278B as u16, clock) }
            },
        }
        self.sampling_rate = sampling_rate;
        self.clock = clock;
        self.output_step = 0x100000000 / i64::from(sampling_rate);
    }

    fn write_chip(&self, offset: u32, data: u8) {
        unsafe { ymfm_write(self.chip_type as u16, offset, data); }
    }

    #[allow(clippy::missing_safety_doc)]
    fn generate(&mut self, buffer: &mut [i32; 2]) {
        let generate_buffer: [i32; 2] = [0, 0];
        unsafe { ymfm_generate(self.chip_type as u16, self.output_pos, self.output_step, generate_buffer.as_ptr()); }
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

impl SoundDevice<u8> for YmFm {
    fn new() -> Self {
        YmFm::from(ChipType::CHIP_YM2149) // TODO: intarfece change
    }

    fn init(&mut self, sample_rate: u32, clock: u32) {
        self.init(sample_rate, clock);
    }

    fn get_name(&self) -> SoundDeviceName {
        SoundDeviceName::YmFm // TODO: intarfece change
    }

    fn reset(&mut self) {
    }

    fn write(&mut self, offset: u32, data: u8) {
        self.write_chip(offset, data);
    }

    fn update(
        &mut self,
        buffer_l: &mut [f32],
        buffer_r: &mut [f32],
        numsamples: usize,
        buffer_pos: usize,
    ) {
        let mut buffer: [i32; 2] = [0, 0];
        for i in 0..numsamples {
            self.generate(&mut buffer);
            buffer_l[buffer_pos + i] += convert_sample_i2f(buffer[0]);
            buffer_r[buffer_pos + i] += convert_sample_i2f(buffer[1]);
        }
    }
}

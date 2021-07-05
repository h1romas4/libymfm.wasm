use crate::sound::{SoundDevice, SoundDeviceName, ym2151_init, ym2151_write, ym2151_generate};

pub struct YM2151 {
    clock: u32,
    sample_rate: u32
}

impl SoundDevice<u8> for YM2151 {
    fn new() -> YM2151 {
        YM2151 {
            clock: 0,
            sample_rate: 0,
        }
    }

    fn init(&mut self, sample_rate: u32, clock: u32) {
        self.clock = clock;
        self.sample_rate = sample_rate;
        unsafe {
            ym2151_init(clock);
        }
    }

    fn get_name(&self) -> SoundDeviceName {
        SoundDeviceName::YM2151
    }

    fn reset(&mut self) {
    }

    fn write(&mut self, port: u32, data: u8) {
        unsafe {
            ym2151_write(port, data);
        }
    }

    fn update(&mut self, buffer_l: &mut [f32], buffer_r: &mut [f32], numsamples: usize, buffer_pos: usize) {
        let mut buffer: Vec<i32> = Vec::new();
        unsafe {
            ym2151_generate(0, 0, buffer.as_mut_ptr());
        }
    }
}

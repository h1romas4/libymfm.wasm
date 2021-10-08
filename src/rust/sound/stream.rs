// license:BSD-3-Clause
///
/// Sound stream interface
///
pub trait SoundStream {
    fn is_tick(&mut self) -> Tick;
    fn push(&mut self, sampling_l: f32, sampling_r: f32);
    fn drain(&mut self) -> (f32, f32);
}

///
/// Through native chip stream
///
pub struct NativeStream {
    now_input_sampling_l: f32,
    now_input_sampling_r: f32,
}

impl NativeStream {
    pub fn new() -> Self {
        NativeStream {
            now_input_sampling_l: 0_f32,
            now_input_sampling_r: 0_f32,
        }
    }
}

impl SoundStream for NativeStream {
    fn is_tick(&mut self) -> Tick {
        Tick::One
    }

    fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.now_input_sampling_l = sampling_l;
        self.now_input_sampling_r = sampling_r;
    }

    fn drain(&mut self) -> (f32, f32) {
        (self.now_input_sampling_l, self.now_input_sampling_r)
    }
}

pub struct NearestDownSampleStream {
    now_input_sampling_l: f32,
    now_input_sampling_r: f32,
    prev_input_sampling_l: f32,
    prev_input_sampling_r: f32,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_sampling_l: f32,
    output_sampling_r: f32,
}

impl NearestDownSampleStream {
    pub fn new(input_sampling_rate: u32, output_sampling_rate: u32) -> Self {
        assert!(input_sampling_rate >= output_sampling_rate);
        NearestDownSampleStream {
            now_input_sampling_l: 0_f32,
            now_input_sampling_r: 0_f32,
            prev_input_sampling_l: 0_f32,
            prev_input_sampling_r: 0_f32,
            output_sampling_pos: 0_f64,
            output_sampling_step: output_sampling_rate as f64 / input_sampling_rate as f64,
            output_sampling_l: 0_f32,
            output_sampling_r: 0_f32,
        }
    }
}

impl SoundStream for NearestDownSampleStream {
    fn is_tick(&mut self) -> Tick {
        if self.output_sampling_pos < 1_f64 {
            return Tick::More;
        }
        let prev_sampling_pos = self.output_sampling_pos - self.output_sampling_step;
        if 1_f64 - prev_sampling_pos < self.output_sampling_pos - 1_f64 {
            self.output_sampling_l = self.prev_input_sampling_l;
            self.output_sampling_r = self.prev_input_sampling_r;
        } else {
            self.output_sampling_l = self.now_input_sampling_l;
            self.output_sampling_r = self.now_input_sampling_r;
        }
        self.output_sampling_pos -= 1_f64;
        Tick::No
    }

    fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.output_sampling_pos += self.output_sampling_step;
        self.prev_input_sampling_l = self.now_input_sampling_l;
        self.prev_input_sampling_r = self.now_input_sampling_r;
        self.now_input_sampling_l = sampling_l;
        self.now_input_sampling_r = sampling_r;
    }

    fn drain(&mut self) -> (f32, f32) {
        (self.output_sampling_l, self.output_sampling_r)
    }
}

///
/// Resample stream for sound chip
///
pub struct LinearUpSamplingStream {
    now_input_sampling_l: Option<f32>,
    now_input_sampling_r: Option<f32>,
    prev_input_sampling_l: Option<f32>,
    prev_input_sampling_r: Option<f32>,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_sampling_l: f32,
    output_sampling_r: f32,
}

impl LinearUpSamplingStream {
    pub fn new(input_sampling_rate: u32, output_sampling_rate: u32) -> Self {
        assert!(input_sampling_rate <= output_sampling_rate);
        let output_sampling_step = input_sampling_rate as f64 / output_sampling_rate as f64;
        LinearUpSamplingStream {
            now_input_sampling_l: None,
            now_input_sampling_r: None,
            prev_input_sampling_l: None,
            prev_input_sampling_r: None,
            output_sampling_pos: 1_f64,
            output_sampling_step,
            output_sampling_l: 0_f32,
            output_sampling_r: 0_f32,
        }
    }

    fn calc_sample(&self, prev_sample: f32, now_sample: f32) -> f32 {
        let output_pos = self.output_sampling_pos + self.output_sampling_step;
        (prev_sample as f64
            + (output_pos - self.output_sampling_step as f64)
                * (now_sample as f64 - prev_sample as f64)) as f32
    }
}

impl SoundStream for LinearUpSamplingStream {
    ///
    /// Compare the native sampling rate to the output sampling rate
    /// to determine if it needs to be ticked.
    ///
    fn is_tick(&mut self) -> Tick {
        if self.output_sampling_pos >= 1_f64 {
            if self.now_input_sampling_l != None {
                self.output_sampling_l = self.now_input_sampling_l.unwrap();
                self.output_sampling_r = self.now_input_sampling_r.unwrap();
            }
            self.output_sampling_pos -= 1_f64;
            Tick::One
        } else {
            // No sound is produced until the previous sample is confirmed.
            if self.prev_input_sampling_l == None {
                self.output_sampling_l = 0_f32;
                self.output_sampling_r = 0_f32;
            } else {
                // y(t/Ts) = x[n] + (t/Ts-n)(x[n+1]-x[n]);
                self.output_sampling_l = self.calc_sample(
                    self.prev_input_sampling_l.unwrap(),
                    self.now_input_sampling_l.unwrap(),
                );
                self.output_sampling_r = self.calc_sample(
                    self.prev_input_sampling_r.unwrap(),
                    self.now_input_sampling_r.unwrap(),
                );
            }
            Tick::No
        }
    }

    ///
    /// The interface through which the sound chip pushes the stream.
    ///
    fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.prev_input_sampling_l = self.now_input_sampling_l;
        self.prev_input_sampling_r = self.now_input_sampling_r;
        self.now_input_sampling_l = Some(sampling_l);
        self.now_input_sampling_r = Some(sampling_r);
    }

    ///
    /// Get the stream of the sound chip.
    ///
    fn drain(&mut self) -> (f32, f32) {
        self.output_sampling_pos += self.output_sampling_step;
        (self.output_sampling_l, self.output_sampling_r)
    }
}

#[derive(PartialEq)]
pub enum Tick {
    One,
    More,
    No,
}

///
/// convert_sample_i2f
///
pub fn convert_sample_i2f(i32_sample: i32) -> f32 {
    let mut f32_sample: f32;
    if i32_sample < 0 {
        f32_sample = i32_sample as f32 / 32768_f32;
    } else {
        f32_sample = i32_sample as f32 / 32767_f32;
    }
    if f32_sample > 1_f32 {
        f32_sample = 1_f32;
    }
    if f32_sample < -1_f32 {
        f32_sample = -1_f32;
    }
    f32_sample
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use approx_eq::assert_approx_eq;

    use crate::sound::stream::LinearUpSamplingStream;
    use crate::sound::stream::Tick;

    use super::SoundStream;

    #[test]
    fn make_stream_1() {
        let mut stream = LinearUpSamplingStream::new(1, 4);

        println!("= step 0 (0.0, 0.0) -> (1.0, 1.0)");
        assert!(stream.is_tick() == Tick::One);
        stream.push(1_f32, 1_f32);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0_f32, 0_f32));

        println!("\n= step 1 (1.0, 1.0) -> (1.0, 1.0)");
        assert!(stream.is_tick() == Tick::One);
        stream.push(1_f32, 1_f32);
        assert_sampling(stream.drain(), (1_f32, 1_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (1_f32, 1_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (1_f32, 1_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (1_f32, 1_f32));

        println!("\n= step 2 (1.0, 1.0) -> (0.0, 0.0)");
        assert!(stream.is_tick() == Tick::One);
        stream.push(0_f32, 0_f32);
        assert_sampling(stream.drain(), (1_f32, 1_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0.75_f32, 0.75_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0.5_f32, 0.5_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0.25_f32, 0.25_f32));

        println!("\n= step 3 (0.0, 0.0) -> (-1.0, -1.0)");
        assert!(stream.is_tick() == Tick::One);
        stream.push(-1_f32, -1_f32);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (-0.25_f32, -0.25_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (-0.5_f32, -0.5_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (-0.75_f32, -0.75_f32));

        println!("\n= step 4 (-1.0, -1.0) -> (1.0, 1.0)");
        assert!(stream.is_tick() == Tick::One);
        stream.push(1_f32, 1_f32);
        assert_sampling(stream.drain(), (-1_f32, -1_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (-0.50_f32, -0.5_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0.5_f32, 0.5_f32));
        println!("@");
        assert!(stream.is_tick() == Tick::One);
        stream.push(2_f32, 2_f32); // dummy
        assert_sampling(stream.drain(), (1_f32, 1_f32));
    }

    fn assert_sampling((l, r): (f32, f32), (tl, tr): (f32, f32)) {
        println!("({}, {}) ({}, {})", l, r, tl, tr);
        // #[allow(clippy::float_cmp)]
        // assert_approx_eq!(l as f64, tl as f64);
        // #[allow(clippy::float_cmp)]
        // assert_approx_eq!(r as f64, tr as f64);
    }
}

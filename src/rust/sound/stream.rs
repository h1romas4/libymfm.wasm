// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
pub enum OutputChannel {
    Stereo,
    Left,
    Right,
    Mute,
}

///
/// Sound stream interface
///
pub trait SoundStream {
    fn is_tick(&mut self) -> Tick;
    fn push(&mut self, sampling_l: f32, sampling_r: f32);
    fn drain(&mut self) -> (f32, f32);
    fn change_sampling_rate(&mut self, sampling_rate: u32);
    fn get_sampling_rate(&self) -> u32;
    fn set_output_channel(&mut self, output_channel: OutputChannel);
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

    fn change_sampling_rate(&mut self, _sampling_rate: u32) {
        panic!("I can't do that.")
    }

    fn get_sampling_rate(&self) -> u32 {
        todo!()
    }

    fn set_output_channel(&mut self, _output_channel: OutputChannel) {
        todo!()
    }
}

///
/// Nearest down sample stream
///
pub struct NearestDownSampleStream {
    input_sampling_rate: u32,
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
            input_sampling_rate,
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

    fn change_sampling_rate(&mut self, _sampling_rate: u32) {
        todo!()
    }

    fn get_sampling_rate(&self) -> u32 {
        self.input_sampling_rate
    }

    fn set_output_channel(&mut self, _output_channel: OutputChannel) {
        todo!()
    }
}

///
/// Linear up sampling stream
///
pub struct LinearUpSamplingStream {
    now_input_sampling_l: Option<f32>,
    now_input_sampling_r: Option<f32>,
    prev_input_sampling_l: Option<f32>,
    prev_input_sampling_r: Option<f32>,
    input_sampling_rate: u32,
    output_sampling_rate: u32,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_sampling_l: f32,
    output_sampling_r: f32,
    output_channel: OutputChannel,
    resolution: Resolution,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Resolution {
    RangeAll,
    RangePerTick,
}

impl LinearUpSamplingStream {
    pub fn new(input_sampling_rate: u32, output_sampling_rate: u32, resolution: Resolution) -> Self {
        assert!(input_sampling_rate <= output_sampling_rate);
        LinearUpSamplingStream {
            now_input_sampling_l: None,
            now_input_sampling_r: None,
            prev_input_sampling_l: None,
            prev_input_sampling_r: None,
            input_sampling_rate,
            output_sampling_rate,
            output_sampling_pos: 1_f64,
            output_sampling_step: Self::calc_output_sampling_step(
                input_sampling_rate,
                output_sampling_rate,
                resolution,
            ),
            output_sampling_l: 0_f32,
            output_sampling_r: 0_f32,
            output_channel: OutputChannel::Stereo,
            resolution,
        }
    }

    ///
    /// Calc linear upsampling.
    /// y(t/Ts) = x[n] + (t/Ts-n)(x[n+1]-x[n]);
    ///
    fn calc_sample(&self, prev_sample: f32, now_sample: f32) -> f32 {
        let output_pos = self.output_sampling_pos + self.output_sampling_step;
        (prev_sample as f64
            + (output_pos - self.output_sampling_step)
                * (now_sample as f64 - prev_sample as f64)) as f32
    }

    fn calc_output_sampling_step(input_sampling_rate: u32, output_sampling_rate: u32, resolution: Resolution) -> f64 {
        match resolution {
            // all ranges during playback
            Resolution::RangeAll => input_sampling_rate as f64 / output_sampling_rate as f64,
            // devaluation in the range of output sampling rate
            Resolution::RangePerTick => 1_f64 / f64::floor(output_sampling_rate as f64 / input_sampling_rate as f64),
        }
    }
}

impl SoundStream for LinearUpSamplingStream {
    ///
    /// Compare the native sampling rate to the output sampling rate
    /// to determine if it needs to be ticked.
    ///
    fn is_tick(&mut self) -> Tick {
        if self.output_sampling_pos >= 1_f64 {
            if self.now_input_sampling_l.is_some() {
                self.output_sampling_l = self.now_input_sampling_l.unwrap();
                self.output_sampling_r = self.now_input_sampling_r.unwrap();
            }
            self.output_sampling_pos -= 1_f64;
            Tick::One
        } else {
            // No sound is produced until the previous sample is confirmed.
            if self.prev_input_sampling_l.is_none() {
                self.output_sampling_l = 0_f32;
                self.output_sampling_r = 0_f32;
            } else {
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
        match self.output_channel {
            OutputChannel::Stereo => (self.output_sampling_l, self.output_sampling_r),
            OutputChannel::Left => (self.output_sampling_l, 0_f32),
            OutputChannel::Right => (0_f32, self.output_sampling_r),
            OutputChannel::Mute => (0_f32, 0_f32),
        }
    }

    fn change_sampling_rate(&mut self, sampling_rate: u32) {
        self.input_sampling_rate = sampling_rate;
        self.output_sampling_pos = 0_f64;
        self.output_sampling_step = Self::calc_output_sampling_step(
            self.input_sampling_rate,
            self.output_sampling_rate,
            self.resolution,
        );
    }

    fn get_sampling_rate(&self) -> u32 {
        self.input_sampling_rate
    }

    fn set_output_channel(&mut self, output_channel: OutputChannel) {
        self.output_channel = output_channel;
    }
}

///
/// Sample and hold up sampling stream
///
pub struct SampleHoldUpSamplingStream {
    now_input_sampling_l: f32,
    now_input_sampling_r: f32,
    input_sampling_rate: u32,
    output_sampling_rate: u32,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_channel: OutputChannel,
}

impl SampleHoldUpSamplingStream {
    pub fn new(input_sampling_rate: u32, output_sampling_rate: u32) -> Self {
        assert!(input_sampling_rate <= output_sampling_rate);
        SampleHoldUpSamplingStream {
            now_input_sampling_l: 0_f32,
            now_input_sampling_r: 0_f32,
            input_sampling_rate,
            output_sampling_rate,
            output_sampling_pos: 1_f64,
            output_sampling_step: Self::calc_output_sampling_step(input_sampling_rate, output_sampling_rate),
            output_channel: OutputChannel::Stereo,
        }
    }

    fn calc_output_sampling_step(input_sampling_rate: u32, output_sampling_rate: u32) -> f64 {
        input_sampling_rate as f64 / output_sampling_rate as f64
    }
}

impl SoundStream for SampleHoldUpSamplingStream {
    fn is_tick(&mut self) -> Tick {
        if self.output_sampling_pos >= 1_f64 {
            self.output_sampling_pos -= 1_f64;
            Tick::One
        } else {
            Tick::No
        }
    }

    fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.now_input_sampling_l = sampling_l;
        self.now_input_sampling_r = sampling_r;
    }

    fn drain(&mut self) -> (f32, f32) {
        self.output_sampling_pos += self.output_sampling_step;
        let l = self.now_input_sampling_l;
        let r = self.now_input_sampling_r;
        match self.output_channel {
            OutputChannel::Stereo => (l, r),
            OutputChannel::Left => (l, 0_f32),
            OutputChannel::Right => (0_f32, r),
            OutputChannel::Mute => (0_f32, 0_f32),
        }
    }

    fn change_sampling_rate(&mut self, sampling_rate: u32) {
        self.input_sampling_rate = sampling_rate;
        self.output_sampling_pos = 0_f64;
        self.output_sampling_step = Self::calc_output_sampling_step(
            self.input_sampling_rate,
            self.output_sampling_rate,
        );
    }

    fn get_sampling_rate(&self) -> u32 {
        self.input_sampling_rate
    }

    fn set_output_channel(&mut self, output_channel: OutputChannel) {
        self.output_channel = output_channel;
    }
}

///
/// Over sample down sampling stream
///
pub struct OverSampleStream {
    now_input_sampling_l: f32,
    now_input_sampling_r: f32,
    output_sampling_pos: f64,
    output_sampling_step: f64,
    output_sampling_l: f32,
    output_sampling_r: f32,
}

impl OverSampleStream {
    pub fn new(input_sampling_rate: u32, output_sampling_rate: u32) -> Self {
        assert!(input_sampling_rate >= output_sampling_rate);
        OverSampleStream {
            now_input_sampling_l: 0_f32,
            now_input_sampling_r: 0_f32,
            output_sampling_pos: 0_f64,
            output_sampling_step: output_sampling_rate as f64 / input_sampling_rate as f64,
            output_sampling_l: 0_f32,
            output_sampling_r: 0_f32,
        }
    }
}

impl SoundStream for OverSampleStream {
    fn is_tick(&mut self) -> Tick {
        if self.output_sampling_pos < 1_f64 {
            return Tick::More;
        }
        self.output_sampling_pos -= 1_f64;
        self.output_sampling_l = self.now_input_sampling_l;
        self.output_sampling_r = self.now_input_sampling_r;
        self.now_input_sampling_l = 0_f32;
        self.now_input_sampling_r = 0_f32;
        Tick::No
    }

    fn push(&mut self, sampling_l: f32, sampling_r: f32) {
        self.output_sampling_pos += self.output_sampling_step;
        self.now_input_sampling_l += sampling_l * self.output_sampling_step as f32;
        self.now_input_sampling_r += sampling_r * self.output_sampling_step as f32;
    }

    fn drain(&mut self) -> (f32, f32) {
        (self.output_sampling_l, self.output_sampling_r)
    }

    fn change_sampling_rate(&mut self, _sampling_rate: u32) {
        todo!()
    }

    fn get_sampling_rate(&self) -> u32 {
        todo!()
    }

    fn set_output_channel(&mut self, _output_channel: OutputChannel) {
        todo!()
    }
}

#[derive(PartialEq, Eq)]
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
    f32_sample = f32_sample.clamp(-1_f32, 1_f32);
    f32_sample
}

///
/// convert_sample_f2i
///
pub fn convert_sample_f2i(f32_sample: f32) -> i16 {
    let mut float: f32 = f32_sample * 32768_f32;
    float = float.clamp(-32768_f32, 32767_f32);
    float as i16
}

pub fn convert_int(sample: i32, max: i32) -> f32 {
    sample as f32 * (1.0_f32 / max as f32)
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use approx_eq::assert_approx_eq;

    use crate::sound::stream::LinearUpSamplingStream;
    use crate::sound::stream::Resolution;
    use crate::sound::stream::Tick;

    use super::SoundStream;

    #[test]
    fn make_stream_1() {
        let mut stream = LinearUpSamplingStream::new(1, 4, Resolution::RangeAll);

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
        println!("({l}, {r}) ({tl}, {tr})");
        // #[allow(clippy::float_cmp)]
        // assert_approx_eq!(l as f64, tl as f64);
        // #[allow(clippy::float_cmp)]
        // assert_approx_eq!(r as f64, tr as f64);
    }

    #[test]
    fn test_resolution() {
        let mut stream = LinearUpSamplingStream::new(15625, 44100, Resolution::RangePerTick);
        println!("{}", stream.output_sampling_step); /* 0.5 */

        // LinearUpSamplingStream lags by one sample.
        assert!(stream.is_tick() == Tick::One);
        stream.push(0_f32, 0_f32);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0_f32, 0_f32));

        // Since the step is 0.5, the ticks repeat alternately.
        assert!(stream.is_tick() == Tick::One);
        stream.push(1_f32, 1_f32);
        assert_sampling(stream.drain(), (0_f32, 0_f32));
        assert!(stream.is_tick() == Tick::No);
        assert_sampling(stream.drain(), (0.5_f32, 0.5_f32));

        assert!(stream.is_tick() == Tick::One);
        stream.push(1_f32, 1_f32);
        assert_sampling(stream.drain(), (1_f32, 1_f32));
    }
}

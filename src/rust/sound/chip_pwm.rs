// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
/**
 * Rust PWM emulation
 *  Hiromasa Tanaka <h1romas4@gmail.com>
 *  https://github.com/h1romas4/libymfm.wasm
 *
 * Porting from:
 *  MAME
 *  copyright-holders:David Haywood
 *  https://github.com/mamedev/mame/blob/master/src/mame/machine/mega32x.cpp
 *  rev. ee1e4f9683a4953cb9d88f9256017fcbc38e3144
 */
use super::{
    data_stream::DataStream,
    rom::RomBank,
    sound_chip::SoundChip,
    stream::SoundStream,
    RomIndex, SoundChipType,
};

const PWM_FIFO_SIZE: usize = 3;
const EMU_SAMPLING_RATE: u32 = 22050; /* 22050 / 15611 After Burner Complete */

#[allow(clippy::upper_case_acronyms)]
pub struct PWM {
    clock: u32,
    pwm_ctrl: u16,
    pwm_cycle: u16,
    pwm_tm_reg: u16,
    cur_lch: [u16; PWM_FIFO_SIZE],
    cur_rch: [u16; PWM_FIFO_SIZE],
    pwm_cycle_reg: u16,
    pwm_timer_tick: u8,
    lch_size: usize,
    rch_size: usize,
    lch_fifo_state: u16,
    rch_fifo_state: u16,
    emu_timer_up_hz: Option<u32>,
    emu_timer_pos: f32,
    emu_out_l: f32,
    emu_out_r: f32,
    emu_rasio: f32,
}

impl PWM {
    pub fn new() -> Self {
        PWM {
            clock: 0,
            pwm_ctrl: 0,
            pwm_cycle: 0,
            pwm_tm_reg: 0,
            cur_lch: [0; PWM_FIFO_SIZE],
            cur_rch: [0; PWM_FIFO_SIZE],
            pwm_cycle_reg: 0,
            pwm_timer_tick: 0,
            lch_size: 0,
            rch_size: 0,
            lch_fifo_state: 0,
            rch_fifo_state: 0,
            emu_timer_up_hz: None,
            emu_timer_pos: 1_f32,
            emu_out_l: 0_f32,
            emu_out_r: 0_f32,
            emu_rasio: 0_f32,
        }
    }

    pub fn device_start(&mut self, clock: u32) -> u32 {
        self.clock = clock;
        EMU_SAMPLING_RATE
    }

    pub fn pwm_w(&mut self, offset: u32, data: u16) {
        #[allow(clippy::identity_op)]
        match offset {
            0x00 /* 0x00/2 */ => {
                self.pwm_ctrl = data & 0xffff;
                self.pwm_tm_reg = (self.pwm_ctrl & 0xf00) >> 8;
                self.calculate_pwm_timer();
            }
            0x01 /* 0x02/2 */ => {
                self.pwm_cycle = data & 0xffff;
                self.pwm_cycle_reg = self.pwm_ctrl;
                self.calculate_pwm_timer();
            }
            0x02 /* 0x04/2 */ => {
                if self.lch_size == PWM_FIFO_SIZE {
                    self.lch_pop();
                }
                self.cur_lch[self.lch_size] = data;
                self.lch_size += 1;

                self.lch_fifo_state = if self.lch_size == PWM_FIFO_SIZE { 0x8000 } else { 0x0000 };
            }
            0x03 /* 0x06/2 */ => {
                if self.rch_size == PWM_FIFO_SIZE {
                    self.rch_pop();
                }
                self.cur_rch[self.rch_size] = data;
                self.rch_size += 1;

                self.rch_fifo_state = if self.rch_size == PWM_FIFO_SIZE { 0x8000 } else { 0x0000 };
            }
            0x04 /* 0x08/2 */ => {
                if self.lch_size == PWM_FIFO_SIZE {
                    self.lch_pop();
                }
                self.cur_lch[self.lch_size] = data;
                self.lch_size += 1;

                self.lch_fifo_state = if self.lch_size == PWM_FIFO_SIZE { 0x8000 } else { 0x0000 };

                if self.rch_size == PWM_FIFO_SIZE {
                    self.rch_pop();
                }
                self.cur_rch[self.rch_size] = data;
                self.rch_size += 1;

                self.rch_fifo_state = if self.rch_size == PWM_FIFO_SIZE { 0x8000 } else { 0x0000 };
            }
            _ => {
                panic!("Write at undefined PWM register {:>02x} {:>04x}\n", offset, data);
            }
        }

        // for debug (matches MAME)
        // println!("{:<06x} {:<06x} : {:<06x} {:<06x} {:<06}", offset, data, self.pwm_ctrl, self.pwm_tm_reg, self.pwm_cycle);
        // printf("%06x %06x : %06x %06x %06d\n", offset, data, m_pwm_ctrl, m_pwm_tm_reg, m_pwm_cycle);
    }

    pub fn handle_pwm_callback(&mut self) -> (u32, u32) {
        let mut ldac = 0;
        let mut rdac = 0;

        if self.lch_size > 0 {
            match self.pwm_ctrl & 3 {
                0 => { /*Speaker OFF*/ }
                1 => {
                    ldac = self.cur_lch[0];
                }
                2 => {
                    rdac = self.cur_lch[0];
                }
                3 => {
                    panic!("Undefined PWM Lch value 3, contact MESSdev");
                }
                _ => {
                    panic!("Undefined PWM command");
                }
            }

            self.lch_pop();
        }

        self.lch_fifo_state = if self.lch_fifo_state == 0 {
            0x4000
        } else {
            0x0000
        };

        if self.rch_size > 0 {
            match (self.pwm_ctrl & 0xc) >> 2 {
                0 => { /*Speaker OFF*/ }
                1 => rdac = self.cur_rch[0],
                2 => ldac = self.cur_rch[0],
                3 => panic!("Undefined PWM Lch value 3, contact MESSdev"),
                _ => panic!("Undefined PWM command"),
            }

            self.rch_pop();
        }

        self.rch_fifo_state = if self.rch_fifo_state == 0 {
            0x4000
        } else {
            0x0000
        };

        self.pwm_timer_tick += 1;

        if self.pwm_timer_tick as u16 == self.pwm_tm_reg {
            self.pwm_timer_tick = 0;
            // if(sh2_master_pwmint_enable) { m_master_cpu->set_input_line(SH2_PINT_IRQ_LEVEL,ASSERT_LINE); }
            // if(sh2_slave_pwmint_enable) { m_slave_cpu->set_input_line(SH2_PINT_IRQ_LEVEL,ASSERT_LINE); }
        }

        self.emu_timer_adjust(Some(self.clock / (self.pwm_cycle as u32 - 1)));

        // for 12 bit DAC
        (ldac as u32, rdac as u32)
    }

    fn calculate_pwm_timer(&mut self) {
        if self.pwm_tm_reg == 0 {
            // zero gives max range
            self.pwm_tm_reg = 16;
        }
        if self.pwm_cycle == 0 {
            // zero gives max range
            self.pwm_cycle = 4095;
        }

        /* if both RMD and LMD are set to OFF or pwm cycle register is one, then PWM timer ticks doesn't occur */
        if self.pwm_cycle == 1 || self.pwm_ctrl & 0xf == 0 {
            self.emu_timer_adjust(None);
        } else {
            self.pwm_timer_tick = 0;
            self.lch_fifo_state = 0x4000;
            self.rch_fifo_state = 0x4000;
            self.lch_size = 0;
            self.rch_size = 0;
            self.emu_timer_adjust(Some(self.clock / (self.pwm_cycle as u32 - 1)));
        }
    }

    fn emu_timer_adjust(&mut self, time: Option<u32>) {
        self.emu_timer_up_hz = time;
        if let Some(time) = time {
            self.emu_rasio = time as f32 / EMU_SAMPLING_RATE as f32;
        }
    }

    fn lch_pop(&mut self) {
        for i in 0..PWM_FIFO_SIZE - 1 {
            self.cur_lch[i] = self.cur_lch[i + 1];
        }
        self.lch_size -= 1;
    }

    fn rch_pop(&mut self) {
        for i in 0..PWM_FIFO_SIZE - 1 {
            self.cur_rch[i] = self.cur_rch[i + 1];
        }
        self.rch_size -= 1;
    }

    fn convert_dac_12bit_unsigned(input: u32, bit: u8) -> f32 {
        let scale = 1.0 / ((1 << bit) as u32) as f32;
        let input = input & (((1 << bit) as u32) - 1);
        // output level hack * 4_f32
        input as f32 * scale * 4_f32
    }
}

impl SoundChip for PWM {
    fn new(_sound_device_name: SoundChipType) -> Self {
        PWM::new()
    }

    fn init(&mut self, clock: u32) -> u32 {
        self.device_start(clock)
    }

    fn reset(&mut self) {
        todo!("WIP");
    }

    fn write(&mut self, _: usize, port: u32, data: u32, _: &mut dyn SoundStream) {
        self.pwm_w(port as u32, data as u16);
    }

    fn tick(
        &mut self,
        _: usize,
        sound_stream: &mut dyn SoundStream,
        _data_stream: &Option<&mut DataStream>,
    ) {
        let mut timer_step = 0_f32;
        if let Some(emu_timer_up_hz) = self.emu_timer_up_hz {
            timer_step = emu_timer_up_hz as f32 / EMU_SAMPLING_RATE as f32;
        }

        if timer_step != 0_f32 {
            while self.emu_timer_pos >= 1_f32 {
                let out = self.handle_pwm_callback();
                self.emu_out_l = PWM::convert_dac_12bit_unsigned((out.0 as f32) as u32, 12);
                self.emu_out_r = PWM::convert_dac_12bit_unsigned((out.1 as f32) as u32, 12);
                self.emu_timer_pos -= 1_f32;
            }
            self.emu_timer_pos += timer_step;
        }

        sound_stream.push(self.emu_out_l, self.emu_out_r);
    }

    fn set_rom_bank(&mut self, _: RomIndex, _: RomBank) {}

    fn notify_add_rom(&mut self, _: RomIndex, _: usize) {}
}

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
#[macro_use]
extern crate clap;
extern crate libymfm;

use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::{env, io, process};
use clap::{App, Arg};
use crate::libymfm::driver::{VgmPlay, VGM_TICK_RATE, XgmPlay, XGM_NTSC_TICK_RATE};
use crate::libymfm::sound::SoundSlot;

const MAX_SAMPLE_SIZE: usize = 2048;

fn main() {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("filename")
                .help("Play .vgm/.vzg/.xgm/.xgz file path")
                .required(true),
        )
        .arg(
            Arg::with_name("rate")
                .help("Output sampling rate")
                .short("r")
                .long("rate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("loop")
                .help("Loop count")
                .long("loop")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output filepath")
                .help("Output file path")
                .short("o")
                .long("output")
                .takes_value(true),
        );

    let matches = app.get_matches();

    // sampling rate
    let sampling_rate: u32 = match matches.value_of("rate") {
        Some(rate) => String::from(rate).parse().unwrap(),
        None => 44100,
    };

    // loop count
    let loop_count: usize = match matches.value_of("loop") {
        Some(loop_count) => String::from(loop_count).parse().unwrap(),
        None => 1,
    };

    // filename
    let file_name = matches.value_of("filename").unwrap();
    let mut file = match File::open(file_name) {
        Ok(file) => file,
        Err(error) => {
            eprintln!("There was a problem opening the file: {:?}", error);
            process::exit(1);
        }
    };

    // output file
    let output_file: Option<File>;
    if let Some(filepath) = matches.value_of("output filepath") {
        // wasmer run libymfm-cli.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm -o ym2612.pcm
        // ffplay -f f32le -ar 44100 -ac 2 ../../docs/vgm/ym2612.pcm
        output_file = match File::create(filepath) {
            Ok(file) => Some(file),
            Err(error) => {
                eprintln!("There was a problem opening the file: {:?}", error);
                process::exit(1);
            }
        };
    } else {
        // stdout direct play
        // wasmer run libymfm-cli.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm | ffplay -f f32le -ar 44100 -ac 2 -i -
        output_file = None;
    }

    // read file
    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer).unwrap();

    // get file type
    let file_type = Path::new(file_name).extension().and_then(OsStr::to_str);

    match file_type {
        Some("vgm") | Some("vgz") => {
            let mut vgmplay = VgmPlay::new(
                SoundSlot::new(VGM_TICK_RATE, sampling_rate, MAX_SAMPLE_SIZE),
                buffer.as_slice(),
            ).expect("vgm file is not valid error.");
            play(&mut vgmplay, output_file, loop_count);
        },
        Some("xgm") | Some("xgz") => {
            let mut xgmplay = XgmPlay::new(
                SoundSlot::new(XGM_NTSC_TICK_RATE, sampling_rate, MAX_SAMPLE_SIZE),
                buffer.as_slice(),
            ).expect("xgm file is not valid error.");
            play(&mut xgmplay, output_file, loop_count);
        },
        Some(_) | None => eprintln!("Known extention type: {:?}", file_type),
    }
}

fn play(player: &mut impl Player, mut output_file: Option<File>, loop_count: usize) {
    loop {
        let loop_now = player.play(true);
        for i in 0..MAX_SAMPLE_SIZE {
            let sampling_l = player.get_sampling_l_ref();
            let sampling_r = player.get_sampling_r_ref();
            unsafe {
                let slice_l = std::slice::from_raw_parts(sampling_l.add(i) as *const u8, 4);
                let slice_r = std::slice::from_raw_parts(sampling_r.add(i) as *const u8, 4);
                if let Some(ref mut output_file) = output_file {
                    output_file.write_all(slice_l).expect("file write error");
                    output_file.write_all(slice_r).expect("file write error");
                } else {
                    io::stdout().write_all(slice_l).expect("stdout error");
                    io::stdout().write_all(slice_r).expect("stdout error");
                }
            }
        }
        if loop_now >= loop_count {
            break;
        }
    }
}

trait Player {
    fn new(sound_slot: SoundSlot, file: &[u8]) -> Result<Self, &'static str> where Self: std::marker::Sized;
    fn get_sampling_l_ref(&self) -> *const f32;
    fn get_sampling_r_ref(&self) -> *const f32;
    fn play(&mut self, repeat: bool) -> usize;
}

impl Player for VgmPlay {
    fn new(sound_slot: SoundSlot, file: &[u8]) -> Result<Self, &'static str> {
        VgmPlay::new(sound_slot, file)
    }

    fn get_sampling_l_ref(&self) -> *const f32 {
        self.get_sampling_l_ref()
    }

    fn get_sampling_r_ref(&self) -> *const f32 {
        self.get_sampling_r_ref()
    }

    fn play(&mut self, repeat: bool) -> usize {
        self.play(repeat)
    }
}

impl Player for XgmPlay {
    fn new(sound_slot: SoundSlot, file: &[u8]) -> Result<Self, &'static str> {
        XgmPlay::new(sound_slot, file)
    }

    fn get_sampling_l_ref(&self) -> *const f32 {
        self.get_sampling_l_ref()
    }

    fn get_sampling_r_ref(&self) -> *const f32 {
        self.get_sampling_r_ref()
    }

    fn play(&mut self, repeat: bool) -> usize {
        self.play(repeat)
    }
}

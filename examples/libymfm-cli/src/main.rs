// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
#[macro_use]
extern crate clap;
extern crate libymfm;

use std::fs::File;
use std::io::{Read, Write};
use std::{env, io, process};

use clap::{App, Arg};

use crate::libymfm::driver::{VgmPlay, VGM_TICK_RATE};
use crate::libymfm::sound::SoundSlot;

const MAX_SAMPLE_SIZE: usize = 2048;

fn main() {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("vgm filename")
                .help("Play .vgm/.vzg file path")
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

    // vgm file
    let vgmfile = match File::open(matches.value_of("vgm filename").unwrap()) {
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

    play(vgmfile, output_file, sampling_rate);
}

fn play(mut file: File, mut output_file: Option<File>, sampling_rate: u32) {
    // load sn76489 vgm file
    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer).unwrap();

    let mut vgmplay = VgmPlay::new(
        SoundSlot::new(VGM_TICK_RATE, sampling_rate, MAX_SAMPLE_SIZE),
        file.metadata().unwrap().len() as usize,
    );
    // set vgmdata (Wasm simulation)
    let vgmdata_ref = vgmplay.get_vgmfile_ref();
    for (i, buf) in buffer.iter().enumerate() {
        unsafe {
            *vgmdata_ref.add(i) = *buf;
        }
    }

    // init & sample
    vgmplay.init().unwrap();
    let sampling_l = vgmplay.get_sampling_l_ref();
    let sampling_r = vgmplay.get_sampling_r_ref();

    // play
    // ffplay -f f32le -ar 44100 -ac 2 output.pcm
    #[allow(clippy::absurd_extreme_comparisons)]
    while vgmplay.play(false) <= 0 {
        for i in 0..MAX_SAMPLE_SIZE {
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
    }
}

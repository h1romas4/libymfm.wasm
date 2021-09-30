extern crate libymfm;

use std::env;
use std::fs::File;
use std::io::{Read, Write};

use crate::libymfm::sound::SoundSlot;
use crate::libymfm::driver::{VgmPlay, VGM_TICK_RATE};

const MAX_SAMPLE_SIZE: usize = 2048;

fn main() {
    // wasmer run target/wasm32-wasi/release/rust-wasm32-wasi.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm
    // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Custom { kind: Uncategorized, error: "failed to find a pre-opened file descriptor through which \"/ym2612.vgm\" could be opened" }', src/main.rs:25:41
    // WASI: Cannot open paths with nightly >= 2021-03-11 when linked with LLD 11.1
    //  https://github.com/rust-lang/rust/issues/85840
    let args: Vec<String> = env::args().collect();
    play(&args[1]);
}

fn play(filepath: &str) {
    println!("Play start! {}", filepath);
    // load sn76489 vgm file
    let mut file = File::open(filepath).unwrap();
    let mut buffer = Vec::new();
    let _ = file.read_to_end(&mut buffer).unwrap();

    let mut vgmplay = VgmPlay::new(
        SoundSlot::new(VGM_TICK_RATE, VGM_TICK_RATE, MAX_SAMPLE_SIZE),
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

    let mut pcm = File::create("output.pcm").expect("file open error.");
    // play
    // ffplay -f f32le -ar 44100 -ac 2 output.pcm
    #[allow(clippy::absurd_extreme_comparisons)]
    while vgmplay.play(false) <= 0 {
        for i in 0..MAX_SAMPLE_SIZE {
            unsafe {
                let slice_l = std::slice::from_raw_parts(sampling_l.add(i) as *const u8, 4);
                let slice_r = std::slice::from_raw_parts(sampling_r.add(i) as *const u8, 4);
                pcm.write_all(slice_l).expect("stdout error");
                pcm.write_all(slice_r).expect("stdout error");
            }
        }
    }
    println!("Play end! {} (vgm instance drop)", filepath);
}

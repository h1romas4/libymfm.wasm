# libymfm.wasm

![](https://github.com/h1romas4/libymfm.wasm/workflows/Build/badge.svg)

This repository is an experimental WebAssembly build of the [ymfm](https://github.com/aaronsgiles/ymfm) Yamaha FM sound cores library.

> [aaronsgiles / ymfm](https://github.com/aaronsgiles/ymfm)
>
> BSD-licensed Yamaha FM sound cores (OPM, OPN, OPL, and others)

We provide high-level and low-level WebAssembly interfaces to sound chips.

The high-level interface provides the vgm/xgm sequencer, while the low-level interface provides direct access to the sound chip.
Both can get PCM binary at a specified sampling rate and number of frames.

The WebAssembly interface can be called from many computer languages by using Wasmer.

## Supported Sound Chips

|chip|from|note|
|----|----|----|
|YM2149|ymfm||
|YM2151|ymfm||
|YM2203|ymfm||
|YM2413|ymfm||
|YM2608|ymfm||
|YM2610/YM2610B|ymfm||
|YM2612|ymfm||
|YM3526|ymfm||
|Y8950|ymfm||
|YM3812|ymfm||
|YMF262|ymfm||
|YMF278B|ymfm||
|SN76489|MAME|Rust ports|
|SEGAPCM|MAME|Rust ports|
|PWM|MAME|Rust ports|
|OKIM6285|MAME|Rust ports|
|C140/C219|MAME|Rust ports|

## Web Browser Interface

[WebAssembly VGM Player](https://chipstream.netlify.app/)

[![](https://github.com/h1romas4/libymfm.wasm/raw/main/public/images/ogp.png)](https://chipstream.netlify.app/)

Firefox or Chromium is recommended. Currently, Safari does not support SharedArrayBuffer because it is not available.

- Web Worker/Worklet architecture
- WASI build on browser

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web)

## Python Example

![](https://github.com/h1romas4/libymfm.wasm/raw/main/docs/images/pyxel.png)

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/python](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/python)

## WASI Commnad Line Interface

- Install [Wasmer](https://wasmer.io/) runtime
- Download [libymfm-cli.wasm](https://github.com/h1romas4/libymfm.wasm/releases/tag/v0.9.3) from pre-build release

Options

```
$ wasmer run libymfm-cli.wasm -- -h
libymfm-cli 0.9.3
Hiromasa Tanaka <h1romas4@gmail.com>
libymfm CLI

USAGE:
    libymfm-cli.wasm [OPTIONS] <filename>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --loop <loop>                 Loop count
    -o, --output <output filepath>    Output file path
    -r, --rate <rate>                 Output sampling rate

ARGS:
    <filename>    Play .vgm/.vzg/.xgm/.xgz file path
```

Example 1 - Specify output file name

```
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm -o ym2612.pcm
$ ffplay -f f32le -ar 44100 -ac 2 ./docs/vgm/ym2612.pcm
```

Example 2 - Direct play

```
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm | ffplay -f f32le -ar 44100 -ac 2 -i -
```

Example 3 - Specify samplig rate

```
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm -r 96000 | ffplay -f f32le -ar 96000 -ac 2 -i -
```

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/libymfm-cli](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/libymfm-cli)

## Build

Build require Rust 2021 edition and +nightly.

```
rustup install nightly
```

`Cargo.toml`

```
[package]
edition = "2021"
rust-version = "1.61"
```

Setup [wasi-sdk-16](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-16)

`.bashrc`

```
export WASI_SDK_PATH=/home/hiromasa/devel/toolchain/wasi-sdk-16.0
export CARGO_TARGET_WASM32_WASI_LINKER=${WASI_SDK_PATH}/bin/lld
export CARGO_TARGET_WASM32_WASI_RUSTFLAGS="-L ${WASI_SDK_PATH}/share/wasi-sysroot/lib/wasm32-wasi"
```

```
$ echo ${WASI_SDK_PATH}
/home/hiromasa/devel/toolchain/wasi-sdk-16.0
$ ls -alF ${WASI_SDK_PATH}
drwxr-xr-x 2 hiromasa hiromasa 4096 12月  3  2020 bin/
drwxr-xr-x 3 hiromasa hiromasa 4096 12月  3  2020 lib/
drwxr-xr-x 6 hiromasa hiromasa 4096 12月  3  2020 share/
$ ${WASI_SDK_PATH}/bin/clang -v
clang version 14.0.4 (https://github.com/llvm/llvm-project 29f1039a7285a5c3a9c353d054140bf2556d4c4d)
Target: wasm32-unknown-wasi
Thread model: posix
InstalledDir: /home/hiromasa/devel/toolchain/wasi-sdk-15.0/bin
```

cmake / make

```
git clone --recursive https://github.com/h1romas4/libymfm.wasm
cd libymfm.wasm
mkdir build && cd build
cmake -DCMAKE_TOOLCHAIN_FILE=../cmake/wasi.cmake  ..
make -j4
```

### Web Browser Interface (`examples/web`)

Install wasm-bindgen require (`--version 0.2.78`)

```
cargo install wasm-bindgen-cli --version 0.2.78
```

Rust build and wasm-bindgen

Always add the **+nightly** flag.

```
rustup target add wasm32-wasi
cargo +nightly build --release --target wasm32-wasi --features bindgen
wasm-bindgen target/wasm32-wasi/release/libymfm.wasm --out-dir ./examples/web/src/wasm/
```

npm

```
cd examples/web
npm install
npm run start
```

### WASI Commnad Line Interface (`examples/libymfm-cli`)

@see [libymfm command line interface](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/libymfm-cli/README.md)

### Python Binding Test (`examples/python`)

Rust build and copy .wasm to Python project

Always add the **+nightly** flag.

```
rustup target add wasm32-wasi
cargo +nightly build --release --target wasm32-wasi
cp -p target/wasm32-wasi/release/libymfm.wasm ./examples/python/src/wasm/
```

Run Python

```
cd examples/python
# Install require
pip3 install -r requirements.txt
# Simple VGM Player
python src/sample_vgmplay.py
# Pyxel impliments example
python src/sample_pyxel.py
# Sound chip direct access example
python src/sample_direct.py
```

### Native Debug & Test

Since Rust currently does not allow create-type switching, the following modification to the source code is required for native debugging.

> [Cargo --crate-type CLI Argument](https://github.com/rust-lang/rfcs/pull/3180/files)

It is also required if you want to use this library as a simple native library.

These are the codes needed to make the library the "WASI Library".

Pacth `Cargo.toml`

```
[lib]
# https://github.com/rust-lang/rust/pull/79997
# https://github.com/bazelbuild/rules_rust/issues/771
# crate-type = ["bin"] # disable this line
crate-type = ["cdylib", "rlib"] # enable this line
path = "src/rust/lib.rs"
```

Pacth `src/rust/lib.rs`

```
// #![no_main] // disable this line
```

Buile or test on native

```
mkdir build && cd build
cmake -DCMAKE_TOOLCHAIN_FILE=../cmake/x86-64.cmake ..
make -j4
```

```
cargo build --release
cargo test ym2612_1 -- --nocapture
```

### Build Note

WASI Library

- [Add a -Zwasi-exec-model codegen option for emitting WASI reactors #79997](https://github.com/rust-lang/rust/pull/79997)
- [Reactor support. #74](https://github.com/WebAssembly/wasi-libc/pull/74)
- [WASI Libraries #24](https://github.com/WebAssembly/WASI/issues/24)
- [New-style command support](https://reviews.llvm.org/D81689)

Essentially, wasm-bindgen is incompatible with wasm32-wasi.

> [improve panic message when compiling to wasi #2554](https://github.com/rustwasm/wasm-bindgen/issues/2554)
>
> `panicked at 'unknown instruction LocalTee`

## License

BSD 3-Clause License

## Thanks

- [ymfm](https://github.com/aaronsgiles/ymfm)
- [MAME](https://github.com/mamedev/mame)

## TODO / Known Issues

- [ ] System
    - [ ] fix ROM bus architecture.
    - [ ] add support sound mixer.
- [x] VGM driver
    - [x] YM2141 clock worng?
    - [x] Is there a problem with the file parser? The beginning of the song may be wrong.
    - [x] Support all data stream (now only support YM2612 and OKIM6285)
- [x] Non-vgm driver support
    - [x] XGM
        - [x] There is still a bug with multi-channel PCM.
- [x] Multilingual Interface
    - [x] CLI
    - [x] Web/JavaScript
    - [x] Python [wasmer-python](https://github.com/wasmerio/wasmer-python)
    - [x] Add an interface that does not depend on wasm-bindgen
- [x] ymfm
    - [x] Add direct ymfm intarfece
    - [x] Support yfmf's all sound chips
- [x] Refactoring
    - [x] Better upsampling
    - [x] Separate the sound stream from the sound driver.
    - [x] Support for arbitrary input tick rate and output sampling rate.
    - [x] Support data stream.
- [X] Add support sound chip
    - [x] Fix SEGAPCM
    - [x] OKIM6285
    - [x] C140
    - [x] C219
    - [ ] OKIM6295
    - [ ] Next to be determined
- [ ] Examples source
    - [ ] Web Frontend: Safari now supports SharedArrayBuffer, but it does not work well.
    - [ ] Web Frontend: Support YM2608 ADPCM ROM (wasmer-js WASI fopen)
    - [x] Web Frontend: AudioWorklet
    - [x] Web Frontend: Web Worker AudioWorklet and SharedArrayBuffer (The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers cannot be set in github pages, so they cannot be deployed)
    - [x] Web Frontend: Add buffering mode
    - [x] CLI: Support loop
- [x] To BSD license
    - [x] SN76489
    - [x] PWM

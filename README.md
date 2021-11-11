# libymfm.wasm

![](https://github.com/h1romas4/libymfm.wasm/workflows/Build/badge.svg)

This repository is an experimental WebAssembly build of the [ymfm](https://github.com/aaronsgiles/ymfm) Yamaha FM sound cores library.

> [aaronsgiles / ymfm](https://github.com/aaronsgiles/ymfm)
>
> BSD-licensed Yamaha FM sound cores (OPM, OPN, OPL, and others)

## Supported sound chips

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
|SN76489|mame|Rust ports|
|SEGAPCM|mame|Rust ports|
|PWM|mame|Rust ports|
|OKIM6285|mame|Rust ports|

## Web Browser Interface

[WebAssembly VGM Player](https://chipstream.netlify.app/)

[![](https://github.com/h1romas4/libymfm.wasm/raw/main/public/images/ogp.png)](https://chipstream.netlify.app/)

Firefox or Chromium is recommended. Currently, Safari does not support SharedArrayBuffer because it is not available.

- Web Worker/Worklet architecture
- WASI build on browser

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web)

## WASI Commnad Line Interface

- Install [Wasmer](https://wasmer.io/) runtime
- Download [libymfm-cli.wasm](https://github.com/h1romas4/libymfm.wasm/releases/tag/v0.3.1) from pre-build release

Options

```
$ wasmer run libymfm-cli.wasm -- -h
libymfm-cli 0.3.1
h1romas4 <h1romas4@gmail.com>
libymfm CLI

USAGE:
    libymfm-cli.wasm [OPTIONS] <vgm filename>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output filepath>    Output file path
    -r, --rate <rate>                 Output sampling rate

ARGS:
    <vgm filename>    Play .vgm/.vzg file path
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

Build require Rust 2021 edition

`Cargo.toml`

```
[package]
edition = "2021"
rust-version = "1.56"
```

Setup [wasi-sdk-12](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-12)

`.bashrc`

```
export WASI_SDK_PATH=/home/hiromasa/devel/toolchain/wasi-sdk-12.0
export CARGO_TARGET_WASM32_WASI_LINKER=${WASI_SDK_PATH}/bin/lld
export CARGO_TARGET_WASM32_WASI_RUSTFLAGS="-L ${WASI_SDK_PATH}/share/wasi-sysroot/lib/wasm32-wasi"
```

```
$ echo ${WASI_SDK_PATH}
/home/hiromasa/devel/toolchain/wasi-sdk-12.0
$ ls -alF ${WASI_SDK_PATH}
drwxr-xr-x 2 hiromasa hiromasa 4096 12月  3  2020 bin/
drwxr-xr-x 3 hiromasa hiromasa 4096 12月  3  2020 lib/
drwxr-xr-x 6 hiromasa hiromasa 4096 12月  3  2020 share/
$ ${WASI_SDK_PATH}/bin/clang -v
clang version 11.0.0 (https://github.com/llvm/llvm-project 176249bd6732a8044d457092ed932768724a6f06)
Target: wasm32-unknown-wasi
Thread model: posix
InstalledDir: /home/hiromasa/devel/toolchain/wasi-sdk-12.0/bin
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

Install wasm-bindgen

```
cargo install wasm-bindgen-cli
```

Rust build and wasm-bindgen

```
rustup target add wasm32-wasi
cargo build --release --target wasm32-wasi --features bindgen
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

### Build Note

Essentially, wasm-bindgen is incompatible with wasm32-wasi.

> [improve panic message when compiling to wasi #2554](https://github.com/rustwasm/wasm-bindgen/issues/2554)
>
> `panicked at 'unknown instruction LocalTee`

To link Rust 1.55 with C/C++ using wasm32-wasi, you need LLD for LLVM 12.

> [WASI: Cannot open paths with nightly >= 2021-03-11 when linked with LLD 11.1 #85840](https://github.com/rust-lang/rust/issues/85840)
>
> `failed to find a pre-opened file descriptor`

wasm-bindgen outputs a TextEncoder TextDecoder function that cannot be used in a Worklet.

> [Unblock AudioWorklets: Find an alternative to TextEncoder / TextDecoder #2367](https://github.com/rustwasm/wasm-bindgen/issues/2367)

## License

BSD 3-Clause License

## Thanks

- [ymfm](https://github.com/aaronsgiles/ymfm)
- [MAME](https://github.com/mamedev/mame)

## TODO / Known Issues

- [x] VGM driver
    - [x] YM2141 clock worng?
    - [x] Is there a problem with the file parser? The beginning of the song may be wrong.
    - [ ] Support all data stream (now only support YM2612 and OKIM6285)
- [ ] Non-vgm driver support
    - [ ] XGM
- [ ] Multilingual Interface
    - [x] CLI
    - [x] Web/JavaScript
    - [ ] Python [wasmer-python](https://github.com/wasmerio/wasmer-python)
- [ ] ymfm
    - [ ] Add direct ymfm intarfece
    - [x] Support yfmf's all sound chips
- [ ] Refactoring
    - [x] Better upsampling
    - [x] Separate the sound stream from the sound driver.
    - [x] Support for arbitrary input tick rate and output sampling rate.
    - [x] Support data stream.
- [X] Add support sound chip
    - [x] Fix SEGAPCM
    - [x] OKIM6285
    - [ ] Next to be determined
- [ ] Examples source
    - [ ] Web Frontend: Support YM2608 ADPCM ROM (wasmer-js WASI fopen)
    - [x] Web Frontend: AudioWorklet
    - [x] Web Frontend: Web Worker AudioWorklet and SharedArrayBuffer (The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers cannot be set in github pages, so they cannot be deployed)
    - [x] Web Frontend: Add buffering mode
    - [ ] CLI: Support loop and feedout
- [x] To BSD license
    - [x] SN76489
    - [x] PWM

# libymfm.wasm

![](https://github.com/h1romas4/libymfm.wasm/workflows/Build/badge.svg)

This repository is an experimental WebAssembly build of the [ymfm](https://github.com/aaronsgiles/ymfm) Yamaha FM sound cores library.

> [aaronsgiles / ymfm](https://github.com/aaronsgiles/ymfm)
>
> BSD-licensed Yamaha FM sound cores (OPM, OPN, OPL, and others)

|chip|from|note|
|----|----|----|
|YM2151|ymfm||
|YM2203|ymfm||
|YM2149|ymfm||
|YM2612|ymfm||
|YM2413|ymfm||
|SN76489|mame|Rust ports for demo|
|SEGAPCM|mame|Rust ports for demo|
|PWM|mame|Rust ports for demo|

## Web browser interface

[WebAssembly VGM Player](https://h1romas4.github.io/libymfm.wasm/)

[![](https://raw.githubusercontent.com/h1romas4/libymfm.wasm/main/docs/assets/example-web-01.png)](https://h1romas4.github.io/libymfm.wasm/)

Firefox or iOS Safari is recommended. Currently, Chromium and other browsers are out of tune (crackings).

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web)

## WASI commnad line interface

- Install [Wasmer](https://wasmer.io/) runtime
- Download [libymfm-cli.wasm](https://github.com/h1romas4/libymfm.wasm/releases/tag/v0.1.0) from pre-build release

Options

```
$ wasmer run libymfm-cli.wasm -- -h
libymfm-cli 0.1.0
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

### Web browser interface (`examples/web`)

Install wasm-bindgen

```
cargo install wasm-bindgen-cli
```

Rust build and wasm-bindgen

```
rustup target add wasm32-wasi
cargo build --release --target wasm32-wasi
wasm-bindgen target/wasm32-wasi/release/libymfm.wasm --out-dir ./examples/web/src/wasm/
```

npm

```
cd examples/web
npm install
npm run start
```

### WASI commnad line interface (`examples/libymfm-cli`)

@see [libymfm command line interface](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/libymfm-cli/README.md)

### Build note

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

## TODO / known issues

- [x] Better upsampling
- [x] To BSD license
    - [x] SN76489
    - [x] PWM
- [x] Add buffering mode
- [ ] Non-vgm driver support
    - [ ] XGM
- [ ] Multilingual Interface
    - [x] CLI
    - [x] Web/JavaScript
    - [ ] Python [wasmer-python](https://github.com/wasmerio/wasmer-python)
- [ ] Add direct ymfm intarfece
- [ ] Support yfmf's all sound chips
- [x] YM2141 clock worng?
- [x] Fix SEGAPCM
- [ ] Refactoring
    - [x] Separate the sound stream from the sound driver.
    - [x] Support for arbitrary input tick rate and output sampling rate.
    - [ ] Examples

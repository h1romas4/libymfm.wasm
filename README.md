# libymfm.wasm

![](https://github.com/h1romas4/libymfm.wasm/workflows/Build/badge.svg)

This repository is an experimental WebAssembly build of the [ymfm](https://github.com/aaronsgiles/ymfm) Yamaha FM sound cores library.

> [aaronsgiles / ymfm](https://github.com/aaronsgiles/ymfm)
>
> BSD-licensed Yamaha FM sound cores (OPM, OPN, OPL, and others)

`libymfm.wasm` provide high-level and low-level WebAssembly interfaces to ymfm's sound chips and additional sound chips.

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
|OKIM6258|MAME|Rust ports|
|C140/C219|MAME|Rust ports|
|OKIM6295|MAME|Rust ports|

### Special Thanks

- [ymfm](https://github.com/aaronsgiles/ymfm)
- [MAME](https://github.com/mamedev/mame)

## License

BSD 3-Clause License

## Web Browser Interface

[WebAssembly VGM Player](https://chipstream.netlify.app/)

[![](https://github.com/h1romas4/libymfm.wasm/raw/main/public/images/ogp.png)](https://chipstream.netlify.app/)

Firefox or Chromium or Safari `16` or higher is recommended.

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/web)

## WASI Commnad Line Interface

- Install [Wasmer](https://wasmer.io/) runtime
- Download [libymfm-cli.wasm](https://github.com/h1romas4/libymfm.wasm/releases/tag/v0.14.0) from pre-build release

Options

```bash
$ wasmer run libymfm-cli.wasm -- -h
libymfm-cli 0.14.0
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

```bash
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm -o ym2612.pcm
$ ffplay -f f32le -ar 44100 -ac 2 ./docs/vgm/ym2612.pcm
```

Example 2 - Direct play

```bash
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm | ffplay -f f32le -ar 44100 -ac 2 -i -
```

Example 3 - Specify samplig rate

```bash
$ wasmer run libymfm-cli.wasm --mapdir /:./docs/vgm -- /ym2612.vgm -r 96000 | ffplay -f f32le -ar 96000 -ac 2 -i -
```

Source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/main/examples/libymfm-cli](https://github.com/h1romas4/libymfm.wasm/tree/main/examples/libymfm-cli)

## Python Binding

Install dependencies

```bash
cd examples/python
pip install -r requirements.txt
```

Run examples

- Simple VGM Player - [src/sample_vgmplay.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/sample_vgmplay.py)
- Simple XGM Player - [src/sample_xgmplay.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/sample_xgmplay.py)
- Sound chip direct access example - [src/sample_direct.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/sample_direct.py)
- Pyxel impliments example - [src/sample_pyxel.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/sample_pyxel.py)

```bash
# Simple VGM Player
python src/sample_vgmplay.py
# Simple XGM Player
python src/sample_xgmplay.py
# Sound chip direct access example
python src/sample_direct.py
# Pyxel impliments example
python src/sample_pyxel.py
```

VGM Play Example: [sample_vgmplay.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/sample_vgmplay.py)

```python
#
# VGM Play Example
#
import pygame
from wasm.chipstream import ChipStream

# VGM instance index
VGM_INDEX = 0
# Output sampling rate settings
SAMPLING_RATE = 44100
SAMPLING_CHUNK_SIZE = 4096

# Sound device init (signed 16bit)
pygame.mixer.pre_init(frequency=SAMPLING_RATE, size=-16, channels=2, buffer=SAMPLING_CHUNK_SIZE)
pygame.init()

# Create Wasm instance
chip_stream = ChipStream()

# Setup VGM
header, gd3 = chip_stream.create_vgm_instance(VGM_INDEX, "./vgm/ym2612.vgm", SAMPLING_RATE, SAMPLING_CHUNK_SIZE)
# Print VGM meta
print(header)
print(gd3)

# Play
while chip_stream.vgm_play(VGM_INDEX) == 0:
    # Get sampling referance
    s16le = chip_stream.vgm_get_sampling_ref(VGM_INDEX)
    # Sounds
    sample = pygame.mixer.Sound(buffer=s16le)
    pygame.mixer.Sound.play(sample)
    # Wait pygame mixer
    while pygame.mixer.get_busy() == True:
        pass

# PyGame quit
pygame.quit()

# Drop instance
chip_stream.drop_vgm_instance(VGM_INDEX)
```

## Bindings from other computer languages

libymfm.wasm has a super basic `extern c` Wasm interface.

> [src/rust/wasm/basic.rs](https://github.com/h1romas4/libymfm.wasm/blob/main/src/rust/wasm/basic.rs)

```rust
#[no_mangle]
pub extern "C" fn vgm_create(
    vgm_index_id: u32,
    output_sampling_rate: u32,
    output_sample_chunk_size: u32,
    memory_index_id: u32,
) -> bool {
    let vgmplay = VgmPlay::new(
        SoundSlot::new(
            driver::VGM_TICK_RATE,
            output_sampling_rate,
            output_sample_chunk_size as usize,
        ),
        get_memory_bank()
            .borrow_mut()
            .get(memory_index_id as usize)
            .unwrap(),
    );
    if vgmplay.is_err() {
        return false;
    }
    get_vgm_bank()
        .borrow_mut()
        .insert(vgm_index_id as usize, vgmplay.unwrap());
    true
}
```

As with the Python Binding example, you could easily create an interface. It would also be possible to create a more type-structural interface.

> [examples/python/src/wasm/chipstream.py](https://github.com/h1romas4/libymfm.wasm/blob/main/examples/python/src/wasm/chipstream.py)

## Build

### Setup Rust toolchaine

Build require Rust 2021 edition and +nightly.

```bash
rustup install nightly
rustup target add wasm32-wasi
```

### Setup wasi-sdk

Setup [wasi-sdk-17](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-17)

Setup enviroment values:

`.bashrc`

```bash
export WASI_SDK_PATH=/home/hiromasa/devel/toolchain/wasi-sdk-17.0
export CARGO_TARGET_WASM32_WASI_LINKER=${WASI_SDK_PATH}/bin/lld
export CARGO_TARGET_WASM32_WASI_RUSTFLAGS="-L ${WASI_SDK_PATH}/share/wasi-sysroot/lib/wasm32-wasi"
```

Verify:

```bash
$ echo ${WASI_SDK_PATH}
/home/hiromasa/devel/toolchain/wasi-sdk-17.0
$ ls -alF ${WASI_SDK_PATH}
drwxr-xr-x 2 hiromasa hiromasa 4096 12月  3  2020 bin/
drwxr-xr-x 3 hiromasa hiromasa 4096 12月  3  2020 lib/
drwxr-xr-x 6 hiromasa hiromasa 4096 12月  3  2020 share/
$ ${WASI_SDK_PATH}/bin/clang -v
clang version 15.0.6 (https://github.com/llvm/llvm-project 088f33605d8a61ff519c580a71b1dd57d16a03f8)
Target: wasm32-unknown-wasi
Thread model: posix
InstalledDir: /home/hiromasa/devel/toolchain/wasi-sdk-17.0/bin
```

### Clone source

Require `--recursive`

```bash
git clone --recursive https://github.com/h1romas4/libymfm.wasm
cd libymfm.wasm
```

### Build C/C++ (ymfm)

```bash
mkdir build && cd build
cmake -DCMAKE_TOOLCHAIN_FILE=../cmake/wasi.cmake  ..
make -j4
```

### Build Rust

#### Web Browser Interface (`examples/web`)

Install wasm-bindgen require (`--version 0.2.78`)

```bash
cargo install wasm-bindgen-cli --version 0.2.78
```

Rust build and wasm-bindgen

Always add the **+nightly** flag.

```bash
cargo +nightly build --release --target wasm32-wasi --features bindgen
wasm-bindgen target/wasm32-wasi/release/libymfm.wasm --out-dir ./examples/web/src/wasm/
```

npm

```bash
cd examples/web
npm install
npm run start
```

#### Python Binding (`examples/python`)

Rust build and copy .wasm to Python project

Always add the **+nightly** flag.

```bash
cargo +nightly build --release --target wasm32-wasi
cp -p target/wasm32-wasi/release/libymfm.wasm ./examples/python/src/wasm/
```

#### WASI Commnad Line Interface (`examples/libymfm-cli`)

Building the WASI command line interface requires disabling the library's `WASI reactor` mode, so the build requires a patch of the source code.

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

Pacth `.cargo/config`

```
[target.wasm32-wasi]
rustflags = [
  "-Ctarget-feature=+bulk-memory",
  # "-Z", "wasi-exec-model=reactor", # disable this line
```

Build

```bash
cd examples/libymfm-cli
cargo +nightly build --target=wasm32-wasi --release
```

Verify:

```bash
ls -laF target/wasm32-wasi/release/*.wasm
-rwxrwxr-x  2 hiromasa hiromasa 3292594  7月 26 21:31 libymfm-cli.wasm*
```

#### Native Debug & Test

Since Rust currently does not allow create-type switching, the following modification to the source code is required for native debugging.

> [Cargo --crate-type CLI Argument](https://github.com/rust-lang/rfcs/pull/3180/files)

It is also required if you want to use this library as a simple native library.

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

```bash
mkdir build && cd build
cmake -DCMAKE_TOOLCHAIN_FILE=../cmake/x86-64.cmake ..
make -j4
```

Native debugging can now be performed.

```bash
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

## TODO / Known Issues

- [ ] System
    - [ ] Fix ROM bus architecture.
    - [ ] Add support sound mixers with multi-channel output.
    - [ ] Remove the dependency on wasm-bindgen to have only extern "C" interface.
    - [ ] Allow the header meta parser to be used independently.
    - [ ] Split the sequence parser and player.
- [x] VGM driver
    - [x] YM2141 clock worng?
    - [x] Is there a problem with the file parser? The beginning of the song may be wrong.
    - [x] Support all data stream (now only support YM2612 and OKIM6285)
    - [x] Support dual chip ROM blocks.
    - [x] Add support parse v1.70 extra header.
    - [ ] Respect the sound chip volume value of the extra header.
    - [ ] Respect seccond sound chip clock value of the extra header.
    - [ ] Implement more of the unimplemented.
- [x] XGM driver
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
    - [x] OKIM6295
    - [ ] RF5C164
    - [ ] Next to be determined
- [ ] Examples source
    - [x] Web Frontend: Safari now supports SharedArrayBuffer, but it does not work well. [SharedArrayBuffer posted to AudioWorkletProcessor is not actually shared with the main thread](https://bugs.webkit.org/show_bug.cgi?id=237144)
    - [ ] Web Frontend: Support YM2608 ADPCM ROM (wasmer-js WASI fopen)
    - [ ] Web Frontend: Remove the wasm-bindgen dependency. Provide a TypeScript-based API wrapper.
    - [x] Web Frontend: AudioWorklet
    - [x] Web Frontend: Web Worker AudioWorklet and SharedArrayBuffer (The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy headers cannot be set in github pages, so they cannot be deployed)
    - [x] Web Frontend: Add buffering mode
    - [x] CLI: Support loop

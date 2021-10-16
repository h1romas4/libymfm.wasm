# libymfm-cli

libymfm command line interface.

## Install lld-12

```
sudo apt install lld-12
```

`.bashrc`

```
export CARGO_TARGET_WASM32_WASI_LINKER=/usr/lib/llvm-12/bin/lld
```

## Build

```
cargo build --target=wasm32-wasi --release
```

## Run

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

Example 1 - specify output file name

```
$ wasmer run libymfm-cli.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm -o ym2612.pcm
$ ffplay -f f32le -ar 44100 -ac 2 ../../docs/vgm/ym2612.pcm
```

Example 2 - direct play

```
$ wasmer run libymfm-cli.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm | ffplay -f f32le -ar 44100 -ac 2 -i -
```


Example 3 - specify samplig rate

```
$ wasmer run libymfm-cli.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm -r 96000 | ffplay -f f32le -ar 96000 -ac 2 -i -
```

## Problem

wasi-sdk-12 lld (wasm-ld) is version 11.0. This linker and Rust >=2021-03-11 have wasi problem.

This should be fixed when wasi-sdk becomes version 13 (lld 12).

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Custom { kind: Uncategorized, error: "failed to find a pre-opened file descriptor through which \"/ym2612.vgm\" could be opened" }', src/main.rs:25:41
```

WASI: Cannot open paths with nightly >= 2021-03-11 when linked with LLD 11.1

https://github.com/rust-lang/rust/issues/85840

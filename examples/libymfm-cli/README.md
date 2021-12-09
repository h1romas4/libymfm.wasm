# libymfm-cli

libymfm command line interface.

## Build

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

```
cargo +nightly build --target=wasm32-wasi --release
```

## Run

Options

```
$ wasmer run libymfm-cli.wasm -- -h
libymfm-cli 0.9.0
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

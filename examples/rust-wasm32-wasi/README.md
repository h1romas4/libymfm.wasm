# rust-wasm32-wasi

libymfm wasi32-wasi benchmark

## Build

```
cargo build --target=wasm32-wasi --release
```

## Run

```
wasmer run target/wasm32-wasi/release/rust-wasm32-wasi.wasm --mapdir /:../../docs/vgm -- /ym2612.vgm
```

## Problem

wasi-sdk-12 lld (wasm-ld) is version 11.0. This linker and Rust >=2021-03-11 have wasi problem.

This should be fixed when wasi-sdk becomes version 13 (lld 12).

```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Custom { kind: Uncategorized, error: "failed to find a pre-opened file descriptor through which \"/ym2612.vgm\" could be opened" }', src/main.rs:25:41
```

WASI: Cannot open paths with nightly >= 2021-03-11 when linked with LLD 11.1

https://github.com/rust-lang/rust/issues/85840

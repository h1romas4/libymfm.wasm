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

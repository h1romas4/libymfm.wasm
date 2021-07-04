# libymfm.wasm

## Play

Install [Wasmer](https://wasmer.io/) runtime

```
$ wasmer -v
wasmer 2.0.0
```

Play vgm file (This repository includes pre-build `dist/libymfm.wasi` and sample vgm file)

```
wasmer run ./dist/libymfm.wasi --mapdir /:./docs/vgm/ -- /ym2612.vgm -o ym2612.wav
ffplay ./docs/vgm/ym2612.wav
```

## Build `libymfm.wasi`

Setup [wasi-sdk-12](https://github.com/WebAssembly/wasi-sdk/releases/tag/wasi-sdk-12)

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
mkdir build && cd build
cmake ..
make -j4
```

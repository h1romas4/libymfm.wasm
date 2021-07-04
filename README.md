# libymfm.wasm

## require

wasi-sdk

```
$ echo ${WASI_SDK_PATH}
/home/hiromasa/devel/toolchain/wasi-sdk-12.0
$ ls -lAF ${WASI_SDK_PATH}
drwxr-xr-x 2 hiromasa hiromasa 4096 12月  3  2020 bin/
drwxr-xr-x 3 hiromasa hiromasa 4096 12月  3  2020 lib/
drwxr-xr-x 6 hiromasa hiromasa 4096 12月  3  2020 share/
$ ${WASI_SDK_PATH}/bin/clang -v
clang version 11.0.0 (https://github.com/llvm/llvm-project 176249bd6732a8044d457092ed932768724a6f06)
Target: wasm32-unknown-wasi
Thread model: posix
InstalledDir: /home/hiromasa/devel/toolchain/wasi-sdk-12.0/bin
```

## build

```
mkdir build && cd build
cmake ..
make -j4
```

## run

```
wasmer run ../dist/libymfm.wasi --mapdir /:../docs/vgm/ -- /ym2612.vgm -o ym2612.pcm
ffplay ../docs/vgm/ym2612.pcm
```

# libymfm.wasm

## require

Emscripten

```
$ emcc -v
emcc (Emscripten gcc/clang-like replacement + linker emulating GNU ld) 2.0.24 (416685fb964c14cde4be3e8a45ad26d75bac3e33)
clang version 13.0.0 (https://github.com/llvm/llvm-project 91f147792e815d401ae408989992f3c1530cc18a)
Target: wasm32-unknown-emscripten
Thread model: posix
InstalledDir: /home/hiromasa/devel/toolchain/emsdk/upstream/bin
```

## build

Emscripten

```
mkdir build && cd build
emcmake cmake ..
emmake make -j4
```

# libymfm.wasm

This repository is an experimental WebAssembly build of the [ymfm](https://github.com/aaronsgiles/ymfm) Yamaha FM sound cores library.

> [aaronsgiles / ymfm](https://github.com/aaronsgiles/ymfm)
>
> BSD-licensed Yamaha FM sound cores (OPM, OPN, OPL, and others)

## WebAssembly example

[WebAssembly VGM Player](https://h1romas4.github.io/libymfm.wasm/)

[![](https://raw.githubusercontent.com/h1romas4/libymfm.wasm/main/docs/assets/example-web-01.png)](https://h1romas4.github.io/libymfm.wasm/)

|chip|from|note|
|----|----|----|
|YM2151|ymfm||
|YM2203|ymfm||
|YM2149|ymfm|vgmplayer needs clock hack?|
|YM2612|ymfm||
|YM2413|ymfm||
|PWM|vgmplay|for demo|
|SN76489|vgmplay|for demo|
|segapcm|mame|for demo (wrong sound. my port miss)|

example source code:

> [https://github.com/h1romas4/libymfm.wasm/tree/wasm](https://github.com/h1romas4/libymfm.wasm/tree/wasm)

## WASI vgm2wav render

Install [Wasmer](https://wasmer.io/) runtime

```
$ wasmer -v
wasmer 2.0.0
```

Play vgm file (This repository includes pre-build `dist/vgmrender.wasi` and sample vgm file)

```
wasmer run ./dist/vgmrender.wasi --mapdir /:./docs/vgm/ -- /ym2612.vgm -o ym2612.wav
ffplay ./docs/vgm/ym2612.wav
```

## Build `vgmrender.wasi`

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
cd libymfm.wasm
mkdir build && cd build
cmake ..
make -j4
```

## WebAssembly VGM Player (`examples/web`)

fix linker path

```
$ cat .cargo/config # fix linker path
[target.wasm32-unknown-unknown]
linker = "/home/hiromasa/devel/toolchain/wasi-sdk-12.0/bin/lld"
rustflags = [
  "-L", "/home/hiromasa/devel/toolchain/wasi-sdk-12.0/share/wasi-sysroot/lib/wasm32-wasi",
```

wasm-pack

```
wasm-pack build
```

npm

```
cd examples/web
npm install
```

workaround patch webpack/lib/wasm-sync/WebAssemblyParser.js

```
alias: {
    // Import "fd_seek" from "wasi_snapshot_preview1" with Non-JS-compatible Func Signature (i64 as parameter)
    //  can only be used for direct wasm to wasm dependencies
    // webpack/lib/wasm-sync/WebAssemblyParser.js
    //  const JS_COMPAT_TYPES = new Set(["i32", "f32", "f64"]);
    // build for workaround patch examples/web/node_modules/webpack/lib/wasm-sync/WebAssemblyParser.js
    //  const JS_COMPAT_TYPES = new Set(["i32", "i64", "f32", "f64"]);
    "wasi_snapshot_preview1": path.resolve(__dirname, './src/js/wasi_snapshot_preview1.js'), // eslint-disable-line
}
```

```
npm run start
```

## TODO / known issues

- [ ] Add direct ymfm intarfece.
- [ ] Use yfmf's YM2612 chip and other.
- [ ] It is not a BSD license.
- [x] YM2141 clock worng?
- [ ] Fix segapcm.
- [ ] Refactoring.

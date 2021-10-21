import { WASI } from '@wasmer/wasi';
import { lowerI64Imports } from "@wasmer/wasm-transformer";
import { WasmFs } from '@wasmer/wasmfs';

// wasi instance
let wasi;
let wasiFs;

// // import * as wasm from './libymfm_bg.wasm';
// let wasm;
// export function setWasmExport(exports) {
//     wasm = exports;
// }
export async function initWasi() {
    // create WASI instance
    wasiFs = new WasmFs();
    wasi = new WASI({
        args: [""],
        env: {},
        bindings: {
            ...WASI.defaultBindings,
            fs: wasiFs,
        }
    });
    // fetch wasm module
    const response = await fetch(new URL('../wasm/libymfm_bg.wasm', import.meta.url));
    const responseArrayBuffer = new Uint8Array(await response.arrayBuffer())
    // compile wasm
    const wasm_bytes = new Uint8Array(responseArrayBuffer).buffer;
    const lowered_wasm = await lowerI64Imports(wasm_bytes);
    let module = await WebAssembly.compile(lowered_wasm);
    // get WASI imports
    let imposts = wasi.getImports(module);
    // merge wasm imports
    //   (import "wasi_snapshot_preview1" "fd_seek" (func $__wasi_fd_seek (type $t25)))
    //   (import "./libymfm_bg.js" "__wbg_new_59cb74e423758ede"...)
    imposts['./libymfm_bg.js'] = await import('../wasm/libymfm_bg');
    const instance = await WebAssembly.instantiate(module, {
        ...imposts
    });
    // start wasi
    wasi.start(instance);

    // return wasm exports
    return instance.exports;
}

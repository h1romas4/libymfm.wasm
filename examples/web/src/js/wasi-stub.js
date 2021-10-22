import { WASI } from '@wasmer/wasi';
import { lowerI64Imports } from "@wasmer/wasm-transformer";
import { WasmFs } from '@wasmer/wasmfs';

/**
 * Initialize WebAssembly with wasmer-js
 *
 * need wasm-bindgen generate source code patch for insert wasm export.
 *
 * > import * as wasm from './libymfm_bg.wasm';
 * < let wasm; export function setWasmExport(exports) { wasm = exports; }
 *
 * @see scripts/wasm_bindgen_patch.js
 * @returns instance.exports
 */
export async function initWasi() {
    // fetch wasm module
    const response = await fetch(new URL('../wasm/libymfm_bg.wasm', import.meta.url));
    const responseArrayBuffer = new Uint8Array(await response.arrayBuffer())
    // compile wasm
    let module = await WebAssembly.compile(responseArrayBuffer);
    // merge wasm imports
    //   (import "wasi_snapshot_preview1" "fd_seek" (func $__wasi_fd_seek (type $t25)))
    //   (import "./libymfm_bg.js" "__wbg_new_59cb74e423758ede"...)
    let imposts;
    imposts['./libymfm_bg.js'] = await import('../wasm/libymfm_bg');
    const instance = await WebAssembly.instantiate(module, {
        ...imposts
    });

    // return wasm exports(for call setWasmExport())
    return instance.exports;
}

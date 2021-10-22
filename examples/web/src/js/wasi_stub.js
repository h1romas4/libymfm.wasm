// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import * as wasi from './wasi_stub_snapshot_preview1';
import * as libymfm from '../wasm/libymfm_bg';

/**
 * Initialize WebAssembly with Worklet and wasi-stub
 *
 * Need wasm-bindgen generate source code patch for insert wasm export.
 * TextEncoder/Decoder cannot be used in Worklets
 *
 * > import * as wasm from './libymfm_bg.wasm';
 * < let wasm; export function setWasmExport(exports) { wasm = exports; }
 * < import '../js/polyfill/TextEncoderTextDecoder.js';
 *
 * @see scripts/wasm_bindgen_patch.js
 * @returns instance.exports
 */
export async function initWasi(module) {
    let wasm = await WebAssembly.compile(module);
    // merge wasm imports
    //   (import "wasi_snapshot_preview1" "fd_seek" (func $__wasi_fd_seek (type $t25)))
    //   (import "./libymfm_bg.js" "__wbg_new_59cb74e423758ede"...)
    let imposts = {};
    imposts['wasi_snapshot_preview1'] = wasi;
    imposts['./libymfm_bg.js'] = libymfm;
    const instance = await WebAssembly.instantiate(wasm, {
        ...imposts
    });

    // return wasm exports(for call setWasmExport())
    return instance.exports;
}

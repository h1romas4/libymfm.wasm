// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import { WASI } from '@wasmer/wasi';
import { WasmFs } from "@wasmer/wasmfs";
import { lowerI64Imports } from "@wasmer/wasm-transformer";
import { spy } from 'spyfs';

// wasi instance
export let wasi;
export let wasmFs;
export let memFs;

/**
 * Initialize WebAssembly with wasmer-js
 *
 * Need wasm-bindgen generate source code patch for insert wasm export.
 * A patch is needed to insert wasmer-js WASI instance.
 *
 * > import * as wasm from './libymfm_bg.wasm';
 * < let wasm; export function setWasmExport(exports) { wasm = exports; }
 *
 * @see scripts/wasm_bindgen_patch.js
 * @returns instance.exports
 */
export async function initWasi() {
    // memfs + spy
    wasmFs = new WasmFs();
    // WasmFs not working ?
    memFs = spy(wasmFs.fs, async (action) => {
        console.log({ [action.method] : {
            "isAsync": action.isAsync,
            "args": action.args,
        }});
        if(action.args[1] instanceof Uint8Array) {
            console.log(new TextDecoder().decode(action.args[1]));
        }
        await action;
    });
    // It is not read by fopen.
    // wasmFs.fs.writeFileSync('test.bin', 'test');
    // create WASI instance
    wasi = new WASI({
        args: [""],
        env: {},
        bindings: {
            ...WASI.defaultBindings,
            fs: memFs
        }
    });
    // fetch wasm module
    const response = await fetch(new URL('../wasm/libymfm_bg.wasm', import.meta.url));
    const responseArrayBuffer = new Uint8Array(await response.arrayBuffer());
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
    // init wasi (Isn't this necessary?)
    instance.exports._initialize();

    // return wasm exports(for call setWasmExport())
    return instance.exports;
}

import { WASI } from '@wasmer/wasi'
import browserBindings from '@wasmer/wasi/lib/bindings/browser'
import { WasmFs } from '@wasmer/wasmfs'

// wasi instance
let wasi;

// // import * as wasm from './libymfm_bg.wasm';
// let wasm;
// export function setWasmExport(exports) {
//     wasm = exports;
// }
export async function initWasi() {
    // create WASI instance
    wasi = new WASI({
        args: [""],
        env: {},
        bindings: {
            ...browserBindings,
            fs: new WasmFs(),
        }
    });
    // fetch wasm module
    const res = await fetch(new URL('../wasm/libymfm_bg.wasm', import.meta.url));
    const bytes = new Uint8Array(await res.arrayBuffer())
    // compile wasm
    let module = await WebAssembly.compile(bytes);
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

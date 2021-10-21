/**
 * wasm-bindgen source code patch for wasmer-js/Parcel
 */

const replace = require('replace-in-file');

try {
    const result = replace.sync({
        files: 'src/wasm/libymfm_bg.js',
        from: "import * as wasm from './libymfm_bg.wasm';",
        to: "let wasm; export function setWasmExport(exports) { wasm = exports; }\nimport {TextEncoder, TextDecoder} from '../js/TextEncoderTextDecoder.js';",
    });
    console.log("[wasm-bindgen source patch] Success", result);
} catch(error) {
    console.log("[wasm-bindgen source patch] Error occurred:", error);
}

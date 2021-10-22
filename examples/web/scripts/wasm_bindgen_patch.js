/**
 * wasm-bindgen source code patch for Worklet/Bundler
 */

const replace = require('replace-in-file');

try {
    const result = replace.sync({
        files: 'src/wasm/libymfm_bg.js',
        from: "import * as wasm from './libymfm_bg.wasm';",
        to: "let wasm; export function setWasmExport(exports) { wasm = exports; }\n"
            + "import {TextEncoder, TextDecoder} from '../js/TextEncoderTextDecoder.js';",
    });
    console.log("[wasm-bindgen source patch] Success", result);
} catch(error) {
    console.log("[wasm-bindgen source patch] Error occurred:", error);
}

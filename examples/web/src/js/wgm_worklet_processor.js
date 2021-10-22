import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from './wasi_stub';

// license:BSD-3-Clause
class WgmWorkletProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        setWasmExport(initWasi());
    }

    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {

    }
}

console.log("aaaaa");

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

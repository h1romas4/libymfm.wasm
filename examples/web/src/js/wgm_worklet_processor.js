import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from "./wasi";

class WgmWorkletProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
    }

    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {

    }
}

console.log("aaaaa");

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

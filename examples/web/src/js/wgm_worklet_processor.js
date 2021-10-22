import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from './wasi_stub';

// license:BSD-3-Clause
class WgmWorkletProcessor extends AudioWorkletProcessor {
    constructor(options) {
        super();
        // get option
        this.module = options.processorOptions.module;
        this.samplingRate = options.processorOptions.samplingRate;
        this.chunkSize = options.processorOptions.chunkSize;
        // wgm instance
        this.wgmplay = null;
        this.memory = null;
        // event dispatch
        this.port.onmessage = (event) => this.onMessage(event);
    }

    async onMessage(event) {
        console.log(event.data);
        switch(event.data.message) {
            case 'compile': {
                await this.compile();
                this.port.postMessage("OK");
                break;
            }
            case 'create': {
                this.port.postMessage(this.create(event.data.vgmdata));
                break;
            }
        }
    }

    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {

    }

    async compile() {
        const exports = await initWasi(this.module);
        setWasmExport(exports);
        this.memory = exports.memory;
    }

    create(vgmdata) {
        // init instance (init sound devicies)
        if(this.wgmplay != null) {
            this.wgmplay.free();
        }
        // create and set data
        this.wgmplay = new WgmPlay(this.samplingRate, this.chunkSize, vgmdata.byteLength);
        let seqdata = new Uint8Array(this.memory.buffer, this.wgmplay.get_seq_data_ref(), vgmdata.byteLength);
        seqdata.set(new Uint8Array(vgmdata));
        if(!this.wgmplay.init()) {
            this.wgmplay.free();
            this.wgmplay = null;
        }
        // return music meta
        return JSON.parse(this.wgmplay.get_seq_gd3());
    }
}

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

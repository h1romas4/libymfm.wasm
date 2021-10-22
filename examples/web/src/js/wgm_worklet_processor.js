// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from './wasi_stub';

/**
 * WgmWorkletProcessor
 */
class WgmWorkletProcessor extends AudioWorkletProcessor {
    /**
     * Constructor
     *
     * @param {*} options
     */
    constructor(options) {
        super();
        // get option
        const opt = options.processorOptions;
        this.module = opt.module;
        this.samplingRate = opt.samplingRate;
        this.chunkSize = opt.chunkSize;
        this.loopMaxCount = opt.loopMaxCount;
        this.feedOutRemain = opt.feedOutRemain;
        // wgm instance
        this.wgmplay = null;
        this.memory = null;
        // instance status
        this.play = false;
        // event dispatch
        this.port.onmessage = (event) => this.dispatch(event);
    }

    /**
     * Waveform generation process
     *
     * @param {*} inputs
     * @param {*} outputs
     * @param {*} parameters
     * @return {boolean} next stage
     */
    process(inputs, outputs, parameters) { // eslint-disable-line no-unused-vars
        if(!this.play) return true;
        try {
            // create wave
            const loop = this.wgmplay.play();
            // clone buffer
            let bufferL = new Float32Array(this.chunkSize);
            let bufferR = new Float32Array(this.chunkSize);
            bufferL.set(new Float32Array(this.memory.buffer, this.wgmplay.get_sampling_l_ref(), this.chunkSize));
            bufferR.set(new Float32Array(this.memory.buffer, this.wgmplay.get_sampling_r_ref(), this.chunkSize));
            // output
            outputs[0][0].set(bufferL);
            outputs[0][1].set(bufferR);
            if(loop >= this.loopMaxCount) {
                this.play = false;
            }
            // next stage
            return true;
        } catch(e) {
            this.play = false;
            console.log(`An unexpected error has occurred. System has stoped. Please reload brwoser.\n${e}`);
            return false;
        }
    }

    /**
     * Message dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
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
            case 'play': {
                this.play = true;
                break;
            }
        }
    }

    /**
     * Compile and setting WebAssembly
     */
    async compile() {
        const exports = await initWasi(this.module);
        setWasmExport(exports);
        this.memory = exports.memory;
    }

    /**
     * Create or recreate WgmPlay instance for play VGM
     *
     * @param {*} vgmdata
     * @returns music GD3 meta
     */
    create(vgmdata) {
        // init instance (init sound devicies)
        if(this.wgmplay != null) {
            this.wgmplay.free();
            this.wgmplay = null; // force GC
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

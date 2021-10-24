// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from './wasi_wasmer';

class WgmWorker {
    constructor(worker) {
        // Worker and WebAssembly
        this.worker = worker;
        this.memory = null;
        // shared memory
        this.ring1 = null;
        this.ring2 = null;
        this.status = null;
        // wgm instance
        this.wgmplay = null;
        this.memory = null;
        this.viewL = null;
        this.viewR = null;
        // state
        this.chankSize;
        this.loopMaxCount,
        this.feedOutRemain,
        // event dispatch
        this.worker.onmessage = (event) => this.dispatch(event);
    }

    async dispatch(event) {
        console.log(`worker: ${event.data.message}`);
        switch(event.data.message) {
            case 'compile': {
                await this.compile();
                this.ring1 = new Float32Array(event.data.shared.ring1);
                this.ring2 = new Float32Array(event.data.shared.ring2);
                this.status = new Int32Array(event.data.shared.status);
                this.worker.postMessage({
                    "message": "callback",
                    "data": "OK"
                });
                break;
            }
            case 'create': {
                this.worker.postMessage({
                    "message": "callback",
                    "data": this.create(event.data.vgmdata, event.data.options)
                });
                break;
            }
        }
    }

    async compile() {
        const exports = await initWasi();
        setWasmExport(exports);
        this.memory = exports.memory;
    }

    /**
     * Create or recreate WgmPlay instance for play VGM
     *
     * @param {*} vgmdata
     * @returns music GD3 meta
     */
     create(vgmdata, options) {
        // init status
        Atomics.store(this.status, 0, 0); // now play ring
        // init instance (init sound devicies)
        if(this.wgmplay != null) {
            this.wgmplay.free();
            this.wgmplay = null; // force GC
        }
        // init state
        this.feedOutCount = 0;
        this.loopMaxCount = options.loopMaxCount;
        this.feedOutRemain = options.feedOutRemain;
        this.chunkSize = options.chunkSize;
        // create and set data
        this.wgmplay = new WgmPlay(options.samplingRate, this.chunkSize, vgmdata.byteLength);
        let seqdata = new Uint8Array(this.memory.buffer, this.wgmplay.get_seq_data_ref(), vgmdata.byteLength);
        seqdata.set(new Uint8Array(vgmdata));
        if(!this.wgmplay.init()) {
            this.wgmplay.free();
            this.wgmplay = null;
        }
        // set view
        this.viewL = new Float32Array(this.memory.buffer, this.wgmplay.get_sampling_l_ref(), this.chunkSize);
        this.viewR = new Float32Array(this.memory.buffer, this.wgmplay.get_sampling_r_ref(), this.chunkSize);
        // sharedbuffer test
        this.buffering();
        // return music meta
        return JSON.parse(this.wgmplay.get_seq_gd3());
    }

    buffering() {
        // create wave
        const loop = this.wgmplay.play();
        // output
        console.log(this.ring1);
        console.log(new Float32Array(this.memory.buffer, this.viewL, this.chunkSize));
        this.ring1.set(new Float32Array(this.memory.buffer, this.viewL, this.chunkSize));
        this.ring1.set(new Float32Array(this.memory.buffer, this.viewR, this.chunkSize));
        // loop
        if(loop >= this.loopMaxCount) {
            if(this.feedOutCount == 0 && loop > this.loopMaxCount) {
                // no loop track
                this.play = false;
                this.port.postMessage({"message": "callback", "data": "OK"});
            } else {
                // feed out start
                if(this.feedOutCount == 0) {
                    this.port.postMessage({"message": "feedout"});
                }
                this.feedOutCount++;
                // feed out end and next track
                if(this.feedOutCount >= this.feedOutRemain) {
                    this.play = false;
                    this.port.postMessage({"message": "callback", "data": "OK"});
                }
            }
        }
    }
}

new WgmWorker(self);

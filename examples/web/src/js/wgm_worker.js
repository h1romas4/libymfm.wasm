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
        this.loopMaxCount;
        this.feedOutRemain;
        this.chunkCount;
        // event dispatch
        this.worker.onmessage = (event) => this.dispatch(event);
    }

    /**
     * WebAssembly compile and WASI/wasmer-js setup
     */
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
        // init instance (init sound devicies)
        if(this.wgmplay != null) {
            this.wgmplay.free();
            this.wgmplay = null; // force GC
        }
        // init state
        this.buffering = true;
        this.feedOutCount = 0;
        this.chunkCount = 0;
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
        // init shared status
        this.status[0] = 0; // playing ring
        this.status[1] = 0; // end of chunk
        this.status[2] = 0; // feedout chunk
        // create first buffer ring 1
        this.generate(1);
        // return music meta
        return JSON.parse(this.wgmplay.get_seq_gd3());
    }

    /**
     * Buffering loop
     */
    loop() {
        let waitRing = 0;
        while(this.buffering) {
            // wait notify (first step 0 -> 1)
            Atomics.wait(this.status, 0, waitRing);
            // It's not atomic loading, but there is a time lag between next updates.
            waitRing = this.status[0];
            // stop event
            if(waitRing == 3) break;
            // create buffer
            this.generate(waitRing == 1? 2: 1);
        }
    }

    /**
     * Generate sound buffer
     *
     * @param {*} ring
     */
    generate(ring) {
        // wait messege
        // create wave
        const loop = this.wgmplay.play();

        // clone view
        let bufferL = new Float32Array(this.chunkSize);
        let bufferR = new Float32Array(this.chunkSize);
        bufferL.set(new Float32Array(this.viewL));
        bufferR.set(new Float32Array(this.viewR));
        if(ring == 1) {
            this.ringL1.set(bufferL);
            this.ringR1.set(bufferR);
        } else {
            this.ringL2.set(bufferL);
            this.ringR2.set(bufferR);
        }

        // loop
        if(loop >= this.loopMaxCount) {
            // this.status is always updated before the playback
            if(this.feedOutCount == 0 && loop > this.loopMaxCount) {
                // no loop track
                this.buffering = false;
                // end of play chunk
                this.status[1] = this.chunkCount
            } else {
                // feed out start
                if(this.feedOutCount == 0) {
                    // feedout start chunk
                    this.status[2] = this.chunkCount
                }
                this.feedOutCount++;
                // feed out end and next track
                if(this.feedOutCount >= this.feedOutRemain) {
                    this.buffering = false;
                    // end of play chunk
                    this.status[1] = this.chunkCount
                }
            }
        }
        this.chunkCount++;
    }

    /**
     * Event dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
        console.log(`worker: ${event.data.message}`);
        switch(event.data.message) {
            case 'compile': {
                await this.compile();
                this.ringL1 = new Float32Array(event.data.shared.ringL1);
                this.ringR1 = new Float32Array(event.data.shared.ringR1);
                this.ringL2 = new Float32Array(event.data.shared.ringL2);
                this.ringR2 = new Float32Array(event.data.shared.ringR2);
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
            case 'start': {
                // start buffering loop (Atomic status wait)
                this.loop();
                break;
            }
            case 'clear': {
                // stop loop
                this.buffering = false
                // stop Atomic wait
                Atomics.store(this.status, 0, /* break loop */ 3);
                Atomics.notify(this.status, 0, /* watcher count */ 1);
                break;
            }
        }
    }
}

new WgmWorker(self);

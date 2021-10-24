// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import { INIT_NOW_PLAYING_RING, BUFFER_RING_COUNT, NOW_PLAYING_RING, END_OF_MUSIC_CHUNK, FEED_OUT_START_CHUNK } from './const.js'
import { WgmPlay, setWasmExport } from "../wasm/libymfm_bg";
import { initWasi } from './wasi_wasmer';

class WgmWorker {
    constructor(worker) {
        // Worker and WebAssembly
        this.worker = worker;
        this.memory = null;
        // shared memory
        this.ringL = [];
        this.ringR = [];
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
        this.status[NOW_PLAYING_RING] = INIT_NOW_PLAYING_RING; // playing ring
        this.status[END_OF_MUSIC_CHUNK] = 0; // end of chunk
        this.status[FEED_OUT_START_CHUNK] = 0; // feedout chunk
        // create first buffer ring 0
        this.generate(0);
        // return music meta
        return JSON.parse(this.wgmplay.get_seq_gd3());
    }

    /**
     * Buffering loop
     */
    loop() {
        let waitRing = INIT_NOW_PLAYING_RING;
        while(this.buffering) {
            // wait notify (first step INIT_NOW_PLAYING_RING -> 0)
            Atomics.wait(this.status, 0, waitRing);
            // It's not atomic loading, but there is a time lag between next updates.
            waitRing = this.status[NOW_PLAYING_RING];
            // stop event
            if(waitRing == INIT_NOW_PLAYING_RING) {
                this.buffering = false;
                break;
            }
            this.generate(waitRing == 0? 1: 0);
        }
    }

    /**
     * Generate sound buffer
     *
     * @param {*} ring
     */
    generate(ring) {
        // create wave
        const loop = this.wgmplay.play();

        // clone view
        let bufferL = new Float32Array(this.chunkSize);
        let bufferR = new Float32Array(this.chunkSize);
        bufferL.set(new Float32Array(this.viewL));
        bufferR.set(new Float32Array(this.viewR));
        // set clone
        this.ringL[ring].set(bufferL);
        this.ringR[ring].set(bufferR);

        this.chunkCount++;

        // loop
        if(loop >= this.loopMaxCount) {
            // this.status is always updated before the playback
            if(this.feedOutCount == 0 && loop > this.loopMaxCount) {
                // no loop track
                this.buffering = false;
                // end of play chunk
                this.status[END_OF_MUSIC_CHUNK] = this.chunkCount;
            } else {
                // feed out start
                if(this.feedOutCount == 0) {
                    // feedout start chunk
                    this.status[FEED_OUT_START_CHUNK] = this.chunkCount
                }
                // feed out end and next track
                if(this.feedOutCount >= this.feedOutRemain) {
                    this.buffering = false;
                    // end of play chunk
                    this.status[END_OF_MUSIC_CHUNK] = this.chunkCount
                }
                this.feedOutCount++;
            }
        }
    }

    /**
     * Event dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
        switch(event.data.message) {
            case 'compile': {
                await this.compile();
                for(let i = 0; i < BUFFER_RING_COUNT; i++) {
                    this.ringL[i] = new Float32Array(event.data.shared.ringL[i]);
                    this.ringR[i] = new Float32Array(event.data.shared.ringR[i]);
                }
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
        }
    }
}

new WgmWorker(self);

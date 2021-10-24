// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import { INIT_NOW_PLAYING_RING, BUFFER_RING_COUNT, AUDIO_WORKLET_SAMPLING_CHUNK, NOW_PLAYING_RING, END_OF_MUSIC_CHUNK, FEED_OUT_START_CHUNK } from './const.js'

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
        // instance status
        this.play = false;
        this.playring = null;
        this.playringBefore = null;
        this.chunkStep = null;
        this.chunkCount = null;
        this.chunkSteps = options.processorOptions.chunkSteps;
        // shared memory
        this.ringL = [];
        this.ringR = [];
        for(let i = 0; i < BUFFER_RING_COUNT; i++) {
            this.ringL[i] = new Float32Array(options.processorOptions.ringL[i]);
            this.ringR[i] = new Float32Array(options.processorOptions.ringR[i]);
        }
        this.status = new Int32Array(options.processorOptions.status);
        // message dispatch
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
        // stop music
        if(!this.play) return true;

        // notify buffering next ring
        if(this.playring != this.playringBefore) {
            Atomics.store(this.status, NOW_PLAYING_RING, this.playring);
            Atomics.notify(this.status, NOW_PLAYING_RING, /* watcher count */ 1);
            this.playringBefore = this.playring;
        }

        let chunkL = this.ringL[this.playring];
        let chunkR = this.ringR[this.playring];

        // set sampling
        let pointer = this.chunkStep * AUDIO_WORKLET_SAMPLING_CHUNK;
        outputs[0][0].set(chunkL.slice(pointer, pointer + AUDIO_WORKLET_SAMPLING_CHUNK));
        outputs[0][1].set(chunkR.slice(pointer, pointer + AUDIO_WORKLET_SAMPLING_CHUNK));

        // step chunk step per AudioWorklet chunk
        this.chunkStep++;
        // next chunk
        if(this.chunkStep >= this.chunkSteps) {
            // end of music
            if(this.status[END_OF_MUSIC_CHUNK] != 0
                && this.status[END_OF_MUSIC_CHUNK] <= this.chunkCount) {
                this.play = false;
                this.port.postMessage({"message": "callback", "data": "endofplay"});
            } else if(this.status[FEED_OUT_START_CHUNK] != 0
                && this.status[FEED_OUT_START_CHUNK] <= this.chunkCount) {
                // feedout
                this.port.postMessage({"message": "feedout"});
            }
            // change ring
            this.playring++;
            if(this.playring >= BUFFER_RING_COUNT) {
                this.playring = 0;
            }
            // clear chunk step
            this.chunkStep = 0;
            // count chunk
            this.chunkCount++;
        }

        return true;
    }

    /**
     * Message dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
        switch(event.data.message) {
            case 'play': {
                // init status
                this.playring = 0;
                this.playringBefore = null;
                this.chunkCount = 1; // 1:first buffer
                this.chunkStep = 0;
                // start play
                this.play = true;
                break;
            }
            case 'stop': {
                this.play = false;
                Atomics.store(this.status, NOW_PLAYING_RING, INIT_NOW_PLAYING_RING);
                Atomics.notify(this.status, NOW_PLAYING_RING, /* watcher count */ 1);
                this.port.postMessage({"message": "callback", "data": "clear wait"});
            }
        }
    }
}

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

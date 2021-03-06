// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import * as def from './const.js'
import worklet from 'worklet:./wgm_worklet_processor.js'; // worklet: Parcel

/**
 * AudioWorklet Controller
 */
export class WgmController {
    /**
     * Constructor
     *
     * @param {ArrayBuffer} module WebAssembly module binary
     * @param {number} samplingRate Sampling rate
     * @param {number} loopMaxCount Max loop count
     */
    constructor(module, samplingRate, loopMaxCount) {
        // WebAssembly binary
        this.module = module;
        // Worker and Worklet
        this.worklet = null;
        this.worker = null;
        this.callback = null;
        // shared memory Worker, Worklet
        this.sharedRingL = [];
        this.sharedRingR = [];
        this.sharedStatus = null;
        // sampling rate
        this.samplingRate = samplingRate;
        this.loopMaxCount = loopMaxCount;
        this.chunkSize = def.AUDIO_WORKLET_SAMPLING_CHUNK * def.BUFFERING_CHUNK_COUNT;
        this.feedOutRemain = 1; // 1chunk
        this.feedOutSecond = Math.floor(this.chunkSize * this.feedOutRemain / samplingRate);
        // init audio contexts
        this.context = null;
        this.gain = null;
        this.analyser = null;
        this.analyserBuffer = null;
        this.analyserBufferLength = null;
    }

    /**
     * prepare AudioContext and AudioWorklet
     *
     * Create Worklet and compile Webassembly in Worklet
     *
     * @param {AudioContext} context AudioContext
     */
    async prepare(context, callback) {
        // set audio context
        this.context = context;
        // create shared memory
        try {
            for(let i = 0; i < def.BUFFER_RING_COUNT; i++) {
                this.sharedRingL[i] = new SharedArrayBuffer(this.chunkSize * 4); // * 4: Float32Array;
                this.sharedRingR[i] = new SharedArrayBuffer(this.chunkSize * 4); // * 4: Float32Array;
            }
            this.sharedStatus = new SharedArrayBuffer(1024); // Int32Array
        } catch(e) {
            return false;
        }
        // create Worker
        this.worker = new Worker(new URL('wgm_worker.js', import.meta.url), {type: 'module'});
        this.worker.onmessage = (event) => this.dispatch(event);
        // create and compile Wasm Worker
        this.sendWorker({
            "message": "compile",
            "shared": {
                "ringL": this.sharedRingL,
                "ringR": this.sharedRingR,
                "status": this.sharedStatus,
            }
        }, async () => {
            // create worklet
            await this.context.audioWorklet.addModule(worklet);
            callback();
        });

        return true;
    }

    /**
     * Initialize Controller
     *
     * Initialize AudioNode Worklet and analyser
     */
    init() {
        this.worklet = new AudioWorkletNode(this.context, "wgm-worklet-processor", {
            "numberOfInputs": 1,
            "numberOfOutputs": 1,
            "outputChannelCount": [2], // 2ch stereo
            "processorOptions": {
                "ringL": this.sharedRingL,
                "ringR": this.sharedRingR,
                "status": this.sharedStatus,
                "chunkSteps": def.BUFFERING_CHUNK_COUNT
            }
        });
        // message dispatch
        this.worklet.port.onmessage = (event) => this.dispatch(event);
        // connect gain
        this.gain = this.context.createGain();
        this.gain.connect(this.context.destination);
        // connect node
        this.worklet.connect(this.gain);
        // connect fft
        this.analyser = this.context.createAnalyser();
        this.analyserBufferLength = this.analyser.frequencyBinCount;
        this.analyserBuffer = new Uint8Array(this.analyserBufferLength);
        this.analyser.getByteTimeDomainData(this.analyserBuffer);
        this.gain.connect(this.analyser);
    }

    /**
     * Instance ready?
     *
     * @returns {boolean}
     */
    ready() {
        if(this.worklet == null) {
            return false;
        }
        return true;
    }

    /**
     * Create playable instance
     *
     * @param {ArrayBuffer} wgmdata
     * @param {string} type(vgm|xgm)
     * @param {Function} callback(gd3meta)
     */
    create(wgmdata, type, callback) {
        // Stop the current loop if there is one
        // Stop Atomic wait via Worklet
        this.sendWorklet({"message": "stop"}, () => {
            this.sendWorker({
                "message": "create",
                "wgmdata": wgmdata,
                "type": type,
                "options": {
                    "samplingRate": this.samplingRate,
                    "chunkSize": this.chunkSize,
                    "loopMaxCount": this.loopMaxCount,
                    "feedOutRemain": this.feedOutRemain,
                }
            }, callback);
        });
    }

    /**
     * Start playback
     *
     * @param {Function} callback end music callback
     */
    play(callback) {
        // return to 1.0
        this.gain.gain.setValueAtTime(1, this.context.currentTime);
        // start buffering
        this.sendWorker({"message": "start"});
        // start playback
        this.sendWorklet({"message": "play"}, callback);
    }

    /**
     * Get FFT data current time
     *
     * @returns FFT array buffer
     */
    getByteFrequencyData() {
        this.analyser.getByteFrequencyData(this.analyserBuffer);
        return this.analyserBuffer;
    }

    /**
     * Get FFT data length
     *
     * @returns FFT array length
     */
    getAnalyserBufferLength() {
        return this.analyserBufferLength;
    }

    /**
     * Feed out music
     */
    feedout() {
        const now = this.context.currentTime;
        // feed out to 0.0
        this.gain.gain.setValueAtTime(1, now);
        this.gain.gain.linearRampToValueAtTime(0, now + this.feedOutSecond);
    }

    /**
     * Message dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
        switch(event.data.message) {
            case "callback": {
                if(this.callback != null) {
                    await this.callback(event.data.data);
                }
                break;
            }
            case "feedout": {
                this.feedout();
                break;
            }
        }
    }

    /**
     * Send message to Worklet
     *
     * @param {*} message
     * @param {Function} callback
     */
    sendWorklet(message, callback) {
        // wait for a reply from the worklet
        if(callback != null) {
            this.callback = callback;
        } else {
            this.callback = null;
        }
        // sends a message to the Worklet
        this.worklet.port.postMessage(message);
    }

    /**
     * Send message to Worklet
     *
     * @param {*} message
     * @param {Function} callback
     */
     sendWorker(message, callback) {
        // wait for a reply from the worklet
        if(callback != null) {
            this.callback = callback;
        } else {
            this.callback = null;
        }
        // sends a message to the Worklet
        this.worker.postMessage(message);
    }
}

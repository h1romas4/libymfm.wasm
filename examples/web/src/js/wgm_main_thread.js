// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import worklet from 'worklet:./wgm_worklet_processor.js'; // worklet: Parcel

const AUDIO_WORKLET_SAMPLING_CHUNK = 128;
const BUFFERING_CHUNK_COUNT = 768;

/**
 * AudioWorklet Controller
 */
export class WgmController {
    /**
     * Constructor
     *
     * @param {*} module WebAssembly module binary
     * @param {*} samplingRate Sampling rate
     * @param {*} loopMaxCount Max loop count
     */
    constructor(module, samplingRate, loopMaxCount) {
        // WebAssembly binary
        this.module = module;
        // Worker and Worklet
        this.worklet = null;
        this.worker = null;
        this.callback = null;
        // shared memory Worker, Worklet
        this.sharedRingL1 = null;
        this.sharedRingR1 = null;
        this.sharedRingL2 = null;
        this.sharedRingR2 = null;
        this.sharedStatus = null;
        // sampling rate
        this.samplingRate = samplingRate;
        this.loopMaxCount = loopMaxCount;
        this.chunkSize = AUDIO_WORKLET_SAMPLING_CHUNK * BUFFERING_CHUNK_COUNT;
        this.feedOutRemain = 1; // 1chunk
        this.feedOutSecond = Math.ceil(this.chunkSize * this.feedOutRemain / samplingRate);
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
     * @param {*} context AudioContext
     */
    async prepare(context, callback) {
        // set audio context
        this.context = context;
        // create shared memory
        try {
            this.sharedRingL1 = new SharedArrayBuffer(this.chunkSize * 4); // Float32Array
            this.sharedRingR1 = new SharedArrayBuffer(this.chunkSize * 4); // Float32Array
            this.sharedRingL2 = new SharedArrayBuffer(this.chunkSize * 4); // Float32Array
            this.sharedRingR2 = new SharedArrayBuffer(this.chunkSize * 4); // Float32Array
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
                "ringL1": this.sharedRingL1,
                "ringR1": this.sharedRingR1,
                "ringL2": this.sharedRingL2,
                "ringR2": this.sharedRingR2,
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
                "ringL1": this.sharedRingL1,
                "ringR1": this.sharedRingR1,
                "ringL2": this.sharedRingL2,
                "ringR2": this.sharedRingR2,
                "status": this.sharedStatus,
                "chunkSteps": BUFFERING_CHUNK_COUNT
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
     * @param {*} vgmdata
     * @param {*} callback(gd3meta)
     */
    create(vgmdata, callback) {
        // Stop the current loop if there is one
        this.sendWorklet({"message": "stop"}); // stop Atomic wait via Worklet
        // Interval of one event
        setTimeout(() => this.sendWorker({
            "message": "create",
            "vgmdata": vgmdata,
            "options": {
                "samplingRate": this.samplingRate,
                "chunkSize": this.chunkSize,
                "loopMaxCount": this.loopMaxCount,
                "feedOutRemain": this.feedOutRemain,
            }
        }, callback), 1);
    }

    /**
     * Start playback
     *
     * @param {*} callback end music callback
     */
    play(callback) {
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
        this.gain.gain.linearRampToValueAtTime(0, now + this.feedOutSecond / 2);
        // return to 1.0
        this.gain.gain.setValueAtTime(1, now + this.feedOutSecond);
    }

    /**
     * Message dispatcher
     *
     * @param {*} event
     */
    async dispatch(event) {
        console.log(event.data);
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
     * @param {function} callback
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
     * @param {function} callback
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

// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka
import worklet from 'worklet:./wgm_worklet_processor.js'; // worklet: Parcel

const AUDIO_WORKLET_SAMPLING_CHUNK = 128;

/**
 * AudioWorklet Controller
 */
export class WgmController {
    constructor(module, samplingRate, loopMaxCount, feedOutSecond) {
        // WebAssembly binary
        this.module = module;
        // sampling rate
        this.samplingRate = samplingRate;
        this.loopMaxCount = loopMaxCount;
        this.feedOutRemain = (samplingRate * feedOutSecond) / AUDIO_WORKLET_SAMPLING_CHUNK;
        // init audio contexts
        this.context = null;
        this.worklet = null;
        this.gain = null;
        this.analyser = null;
        this.analyserBuffer = null;
        this.analyserBufferLength = null;
    }

    /**
     * Initialize Controller
     *
     * - Initialize AudioNode Worklet and analyser
     * - Create Worklet and compile Webassembly in Worklet
     *
     * @param {*} context AudioContext
     */
    async init(context, callback) {
        // set audio context
        this.context = context;
        // create and compile Wasm AudioWorklet
        await this.context.audioWorklet.addModule(worklet);
        this.worklet = new AudioWorkletNode(context, "wgm-worklet-processor", {
            "numberOfInputs": 2,
            "numberOfOutputs": 2,
            "processorOptions": {
                "module": this.module,
                "samplingRate": this.samplingRate,
                "chunkSize": AUDIO_WORKLET_SAMPLING_CHUNK
            }
        });
        this.send({ "message": "compile" }, () => {
            // connect gain
            this.gain = context.createGain();
            this.gain.connect(this.context.destination);
            this.gain.gain.setValueAtTime(1, this.context.currentTime);
            // connect node
            this.worklet.connect(this.gain);
            // connect fft
            this.analyser = this.context.createAnalyser();
            this.analyserBufferLength = this.analyser.frequencyBinCount;
            this.analyserBuffer = new Uint8Array(this.analyserBufferLength);
            this.analyser.getByteTimeDomainData(this.analyserBuffer);
            this.gain.connect(this.analyser);
            callback();
        });
    }

    /**
     * Create playable instance
     *
     * @param {*} vgmdata
     * @param {*} callback(gd3meta)
     */
    create(vgmdata, callback) {
        this.send({
            "message": "create",
            "vgmdata": vgmdata
        }, callback);
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
     * Send message to Worklet
     *
     * @param {*} message
     * @param {function} callback
     */
    send(message, callback) {
        // wait for a reply from the worklet
        if(callback != null) {
            this.worklet.port.onmessage = (event) => {
                callback(event.data);
            }
        } else {
            this.worklet.port.onmessage = null;
        }
        // sends a message to the Worklet
        this.worklet.port.postMessage(message);
    }
}

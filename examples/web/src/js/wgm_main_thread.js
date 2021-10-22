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

    create(vgmdata, callback) {
        this.send({
            "message": "create",
            "vgmdata": vgmdata
        }, callback);
    }

    send(message, callback) {
        if(callback != null) {
            this.worklet.port.onmessage = (event) => {
                callback(event.data);
            }
        } else {
            this.worklet.port.onmessage = null;
        }
        this.worklet.port.postMessage(message);
    }

    getByteFrequencyData() {
        this.analyser.getByteFrequencyData(this.analyserBuffer);
        return this.analyserBuffer;
    }

    getAnalyserBufferLength() {
        return this.analyserBufferLength;
    }
}

class WgmWorkletProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
    }

    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {

    }
}

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

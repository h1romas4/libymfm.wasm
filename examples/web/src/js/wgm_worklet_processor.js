class WgmWorkletProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
    }

    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {

    }
}

console.log("aaaaa");

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

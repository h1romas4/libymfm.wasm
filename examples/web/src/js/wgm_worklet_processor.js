// license:BSD-3-Clause
// copyright-holders:Hiromasa Tanaka

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
        this.feedOutCount = 0;
        // shared memory
        this.ring1 = new Float32Array(options.ring1);
        this.ring2 = new Float32Array(options.ring2);
        this.status = new Int32Array(options.status);
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
        if(!this.play) return true;
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
                this.play = true;
                break;
            }
        }
    }
}

registerProcessor('wgm-worklet-processor', WgmWorkletProcessor);

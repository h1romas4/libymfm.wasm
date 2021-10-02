class SoundGenerator extends AudioWorkletProcessor {
    // eslint-disable-next-line no-unused-vars
    process(inputs, outputs, parameters) {
        return true;
    }
}
registerProcessor("sound-generator", SoundGenerator);
export default SoundGenerator;

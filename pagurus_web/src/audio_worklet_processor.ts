const AUDIO_WORKLET_PROCESSOR_CODE: string = `
class PagurusAudioWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.inputBuffer = [];
    this.port.onmessage = (e) => {
      this.inputBuffer.push(...e.data);
    };
  }

  process(inputs, outputs, parameters) {
    const outputChannel = outputs[0][0];
    for (let i = 0; i < outputChannel.length; i++) {
      const x = this.inputBuffer.shift();
      if (x === undefined) {
        outputChannel[i] = 0;
      } else {
        outputChannel[i] = x;
      }
    }
    return true;
  }
}

registerProcessor("pagurus-audio-worklet-processor", PagurusAudioWorkletProcessor);
`;

const AUDIO_WORKLET_PROCESSOR_NAME: string = "pagurus-audio-worklet-processor";

export { AUDIO_WORKLET_PROCESSOR_CODE, AUDIO_WORKLET_PROCESSOR_NAME };

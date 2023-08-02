const AUDIO_WORKLET_PROCESSOR_CODE = `
class PagurusAudioWorkletProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.inputBuffer = [];
    this.offset = 0;
    this.port.onmessage = (e) => {
      this.inputBuffer.push(e.data);
    };
  }

  process(inputs, outputs, parameters) {
    const outputChannel = outputs[0][0];
    for (let i = 0; i < outputChannel.length; i++) {
      const audioData = this.inputBuffer[0];
      if (audioData === undefined) {
        outputChannel[i] = 0;
      } else {
        outputChannel[i] = audioData[this.offset];
        this.offset++;
        if (this.offset == audioData.length) {
          this.inputBuffer.shift();
          this.offset = 0;
        }
      }
    }
    return true;
  }
}

registerProcessor("pagurus-audio-worklet-processor", PagurusAudioWorkletProcessor);
`;

const AUDIO_WORKLET_PROCESSOR_NAME = "pagurus-audio-worklet-processor";

export { AUDIO_WORKLET_PROCESSOR_CODE, AUDIO_WORKLET_PROCESSOR_NAME };

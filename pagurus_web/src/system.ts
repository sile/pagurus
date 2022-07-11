//
class System {
  private startTime: number;
  private nextActionId: number;
  private wasmMemory: WebAssembly.Memory;

  constructor(wasmMemory: WebAssembly.Memory) {
    this.startTime = performance.now();
    this.nextActionId = 0;
    this.wasmMemory = wasmMemory;
  }

  videoRender(videoFrameOffset: number, videoFrameLen: number, width: number) {}

  audioEnqueue(audioDataOffset: number, audioDataLen: number): number {
    return 0;
  }

  consoleLog(messageOffset: number, messageLen: number) {
    const message = this.getWasmString(messageOffset, messageLen);
    console.log(message);
  }

  clockGameTime(): number {
    return (performance.now() - this.startTime) / 1000;
  }

  clockUnixTime(): number {
    return new Date().getTime() / 1000;
  }

  clockSetTimeout(timeout: number): number {
    let actionId = this.nextActionId;
    this.nextActionId += 1;
    return actionId;
  }

  stateSave(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): number {
    let actionId = this.nextActionId;
    this.nextActionId += 1;
    return actionId;
  }

  stateLoad(nameOffset: number, nameLen: number): number {
    let actionId = this.nextActionId;
    this.nextActionId += 1;
    return actionId;
  }

  stateDelete(nameOffset: number, nameLen: number): number {
    let actionId = this.nextActionId;
    this.nextActionId += 1;
    return actionId;
  }

  private getWasmString(offset: number, len: number): string {
    const buffer = new Uint8Array(this.wasmMemory.buffer, offset, len);
    return new TextDecoder("utf-8").decode(buffer);
  }
}

export { System };

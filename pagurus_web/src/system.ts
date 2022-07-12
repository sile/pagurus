import { ActionId, Event } from "./event";

class System {
  private wasmMemory: WebAssembly.Memory;
  private canvas: HTMLCanvasElement;
  private canvasCtx: CanvasRenderingContext2D;
  private startTime: number;
  private nextActionId: ActionId;
  private eventQueue: Event[];
  private resolveNextEvent?: Function;

  constructor(wasmMemory: WebAssembly.Memory, canvas: HTMLCanvasElement) {
    this.wasmMemory = wasmMemory;

    this.canvas = canvas;
    const canvasCtx = this.canvas.getContext("2d");
    if (!canvasCtx) {
      throw Error("failed to get canvas 2D context");
    }
    this.canvasCtx = canvasCtx;

    this.startTime = performance.now();
    this.nextActionId = 0n;

    this.eventQueue = [{ window: { redrawNeeded: { size: { width: canvas.width, height: canvas.height } } } }];
  }

  nextEvent(): Promise<Event> {
    const event = this.eventQueue.shift();
    if (event !== undefined) {
      return Promise.resolve(event);
    } else {
      return new Promise((resolve) => {
        this.resolveNextEvent = resolve;
      });
    }
  }

  private enqueueEvent(event: Event) {
    if (this.resolveNextEvent !== undefined) {
      this.resolveNextEvent(event);
      this.resolveNextEvent = undefined;
    } else {
      this.eventQueue.push(event);
    }
  }

  videoDraw(videoFrameOffset: number, videoFrameLen: number, width: number) {
    const canvasHeight = videoFrameLen / 3 / width;
    const image = this.canvasCtx.createImageData(width, canvasHeight);
    const videoFrame = new Uint8ClampedArray(this.wasmMemory.buffer, videoFrameOffset, videoFrameLen);
    for (let i = 0; i < videoFrameLen / 3; i++) {
      image.data.set(videoFrame.subarray(i * 3, i * 3 + 3), i * 4);
      image.data[i * 4 + 3] = 255;
    }
    createImageBitmap(image).then((bitmap) => {
      this.canvasCtx.drawImage(bitmap, 0, 0, this.canvas.width, this.canvas.height);
    });
  }

  audioEnqueue(audioDataOffset: number, audioDataLen: number): number {
    return audioDataLen / 2;
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

  clockSetTimeout(timeout: number): ActionId {
    let actionId = this.getNextActionId();
    setTimeout(() => {
      this.enqueueEvent({ timeout: { id: actionId } });
    }, timeout * 1000);
    return actionId;
  }

  stateSave(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): ActionId {
    let actionId = this.getNextActionId();
    return actionId;
  }

  stateLoad(nameOffset: number, nameLen: number): ActionId {
    let actionId = this.getNextActionId();

    // TODO: use index db

    // const name = this.getWasmString(nameOffset, nameLen);
    // let failed;
    // let data;
    // try {
    //   const item = localStorage.getItem(name);
    //   if (item !== null) {
    //     data = new TextEncoder().encode(item);
    //   }
    // } catch (e) {
    //   failed = { reason: "${e}" };
    // }
    // this.eventQueue.push({ state: { loaded: { id: actionId, data, failed } } });

    return actionId;
  }

  stateDelete(nameOffset: number, nameLen: number): ActionId {
    let actionId = this.getNextActionId();

    // TODO: use indexedDB
    // const name = this.getWasmString(nameOffset, nameLen);
    // let failed;
    // try {
    //   localStorage.removeItem(name);
    // } catch (e) {
    //   failed = { reason: "${e}" };
    // }
    // this.eventQueue.push({ state: { deleted: { id: actionId, failed } } });

    return actionId;
  }

  private getWasmString(offset: number, len: number): string {
    const buffer = new Uint8Array(this.wasmMemory.buffer, offset, len);
    return new TextDecoder("utf-8").decode(buffer);
  }

  private getNextActionId(): ActionId {
    let actionId = this.nextActionId;
    this.nextActionId = this.nextActionId + 1n;
    return actionId;
  }
}

export { System };

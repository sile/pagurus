import { ActionId, Event } from "./event";

class System {
  private wasmMemory: WebAssembly.Memory;
  private db: IDBDatabase;
  private canvas: HTMLCanvasElement;
  private canvasCtx: CanvasRenderingContext2D;
  private startTime: number;
  private nextActionId: ActionId;
  private eventQueue: Event[];
  private resolveNextEvent?: Function;

  static async create(
    wasmMemory: WebAssembly.Memory,
    canvas: HTMLCanvasElement,
    databaseName: string = "PAGURUS_DB_TEMP0" // TODO
  ): Promise<System> {
    const openRequest = indexedDB.open(databaseName);
    return new Promise((resolve, reject) => {
      openRequest.onupgradeneeded = (event) => {
        // @ts-ignore
        const db: IDBDatabase = event.target.result;
        db.createObjectStore("states", { keyPath: "name" });
      };
      openRequest.onsuccess = (event) => {
        // @ts-ignore
        const db: IDBDatabase = event.target.result;
        resolve(new System(wasmMemory, canvas, db));
      };
      openRequest.onerror = (event) => {
        reject(new Error(`failed to open database: event=${event}`));
      };
    });
  }

  private constructor(wasmMemory: WebAssembly.Memory, canvas: HTMLCanvasElement, db: IDBDatabase) {
    this.wasmMemory = wasmMemory;
    this.db = db;

    this.canvas = canvas;
    const canvasCtx = this.canvas.getContext("2d");
    if (!canvasCtx) {
      throw Error("failed to get canvas 2D context");
    }
    this.canvasCtx = canvasCtx;

    this.startTime = performance.now();
    this.nextActionId = 0n;

    const initialEvent = { window: { redrawNeeded: { size: { width: canvas.width, height: canvas.height } } } };
    this.eventQueue = [initialEvent];
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
    const actionId = this.getNextActionId();
    setTimeout(() => {
      this.enqueueEvent({ timeout: { id: actionId } });
    }, timeout * 1000);
    return actionId;
  }

  stateSave(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);
    const data = new Uint8Array(this.wasmMemory.buffer, dataOffset, dataLen).slice();
    const system = this;

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.add({ name, data });
    request.onsuccess = function (event) {
      system.enqueueEvent({ state: { saved: { id: actionId } } });
    };

    return actionId;
  }

  stateLoad(nameOffset: number, nameLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);
    const system = this;

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.get(name);
    request.onsuccess = function (event) {
      // @ts-ignore
      if (event.target.result === undefined) {
        system.enqueueEvent({ state: { loaded: { id: actionId } } });
      } else {
        // @ts-ignore
        const data = event.target.result.data;
        system.enqueueEvent({ state: { loaded: { id: actionId, data } } });
      }
    };

    return actionId;
  }

  stateDelete(nameOffset: number, nameLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);
    const system = this;

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.delete(name);
    request.onsuccess = function (event) {
      system.enqueueEvent({ state: { deleted: { id: actionId } } });
    };

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

import { AUDIO_WORKLET_PROCESSOR_CODE, AUDIO_WORKLET_PROCESSOR_NAME } from "./audio_worklet_processor";
import { ActionId, Event, toPagurusKey, toPagurusMouseButton } from "./event";
import { Position, Size } from "./spatial";

class System {
  private wasmMemory: WebAssembly.Memory;
  private db: IDBDatabase;
  private canvas: HTMLCanvasElement;
  private canvasCtx: CanvasRenderingContext2D;
  private canvasSize: Size;
  private audioContext?: AudioContext;
  private audioInputNode?: AudioWorkletNode;
  private startTime: number;
  private nextActionId: ActionId;
  private eventQueue: Event[];
  private resolveNextEvent?: (event: Event) => void;

  static async create(
    wasmMemory: WebAssembly.Memory,
    canvas: HTMLCanvasElement,
    databaseName = "PAGURUS_STATE_DB"
  ): Promise<System> {
    const openRequest = indexedDB.open(databaseName);
    return new Promise((resolve, reject) => {
      openRequest.onupgradeneeded = (event) => {
        // @ts-ignore
        const db: IDBDatabase = event.target.result as IDBDatabase;
        db.createObjectStore("states", { keyPath: "name" });
      };
      openRequest.onsuccess = (event) => {
        // @ts-ignore
        const db: IDBDatabase = event.target.result as IDBDatabase;
        resolve(new System(wasmMemory, canvas, db));
      };
      openRequest.onerror = () => {
        reject(new Error(`failed to open database (indexedDB)`));
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
    this.canvasSize = { width: canvas.width, height: canvas.height };

    this.startTime = performance.now();
    this.nextActionId = 0;

    document.addEventListener("keyup", (event) => {
      if (this.handleKeyup(event)) {
        event.stopPropagation();
        event.preventDefault();
      }
    });
    document.addEventListener("keydown", (event) => {
      if (this.handleKeydown(event)) {
        event.stopPropagation();
        event.preventDefault();
      }
    });

    this.canvas.addEventListener("mousemove", (event) => {
      this.handleMousemove(event);
    });
    this.canvas.addEventListener("mousedown", (event) => {
      this.handleMousedown(event);
    });
    this.canvas.addEventListener("mouseup", (event) => {
      this.handleMouseup(event);
    });

    this.canvas.addEventListener("touchmove", (event) => {
      this.handleTouchmove(event);
      event.stopPropagation();
      event.preventDefault();
    });
    this.canvas.addEventListener("touchstart", (event) => {
      this.handleTouchstart(event);
      event.stopPropagation();
      event.preventDefault();
    });
    this.canvas.addEventListener("touchend", (event) => {
      this.handleTouchend(event);
      event.stopPropagation();
      event.preventDefault();
    });

    const initialEvent = { window: { redrawNeeded: { size: this.canvasSize } } };
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

  private handleKeyup(event: KeyboardEvent): boolean {
    const key = toPagurusKey(event.key);
    if (key !== undefined) {
      this.enqueueEvent({ key: { up: { key } } });
    }
    return key !== undefined;
  }

  private handleKeydown(event: KeyboardEvent): boolean {
    const key = toPagurusKey(event.key);
    if (key !== undefined) {
      this.enqueueEvent({ key: { down: { key } } });
    }
    return key !== undefined;
  }

  private touchPosition(touch: Touch): Position {
    const rect = this.canvas.getBoundingClientRect();
    return { x: Math.round(touch.clientX - rect.left), y: Math.round(touch.clientY - rect.top) };
  }

  private handleTouchmove(event: TouchEvent) {
    const touches = event.changedTouches;
    for (let i = 0; i < touches.length; i++) {
      const touch = touches[i];
      const position = this.touchPosition(touch);
      this.enqueueEvent({ mouse: { move: { position } } });
      break;
    }
  }

  private handleTouchstart(event: TouchEvent) {
    const touches = event.changedTouches;
    for (let i = 0; i < touches.length; i++) {
      const touch = touches[i];
      const button = "left";
      const position = this.touchPosition(touch);
      this.enqueueEvent({ mouse: { down: { position, button } } });
      break;
    }
  }

  private handleTouchend(event: TouchEvent) {
    const touches = event.changedTouches;
    for (let i = 0; i < touches.length; i++) {
      const touch = touches[i];
      const button = "left";
      const position = this.touchPosition(touch);
      this.enqueueEvent({ mouse: { up: { position, button } } });
      break;
    }
  }

  private handleMousemove(event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    this.enqueueEvent({ mouse: { move: { position: { x, y } } } });
  }

  private handleMousedown(event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    const button = toPagurusMouseButton(event.button);
    if (button !== undefined) {
      this.enqueueEvent({ mouse: { down: { position: { x, y }, button } } });
    }
  }

  private handleMouseup(event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    const button = toPagurusMouseButton(event.button);
    if (button !== undefined) {
      this.enqueueEvent({ mouse: { up: { position: { x, y }, button } } });
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

  notifyRedrawNeeded() {
    this.canvasSize = { width: this.canvas.width, height: this.canvas.height };
    this.enqueueEvent({ window: { redrawNeeded: { size: this.canvasSize } } });
  }

  videoInit(width: number, _height: number, pixelFormatPtr: number, stridePtr: number) {
    new DataView(this.wasmMemory.buffer).setUint8(pixelFormatPtr, 1); // 1=RGB32
    new DataView(this.wasmMemory.buffer).setUint32(stridePtr, width, true);
  }

  videoDraw(videoFrameOffset: number, videoFrameLen: number, width: number, stride: number, format: number) {
    if (format != 1) {
      throw new Error(`expected RGB32(3) format, but got ${format}`);
    }
    if (width != stride) {
      throw new Error(`width ${width} differs from stride ${stride}`);
    }

    if (this.canvasSize.width != this.canvas.width || this.canvasSize.height != this.canvas.height) {
      this.canvasSize = { width: this.canvas.width, height: this.canvas.height };
      this.enqueueEvent({ window: { redrawNeeded: { size: this.canvasSize } } });
      return;
    }

    if (width === 0 || videoFrameLen === 0) {
      return;
    }

    const height = videoFrameLen / 4 / width;
    const videoFrame = new Uint8ClampedArray(this.wasmMemory.buffer, videoFrameOffset, videoFrameLen);
    if (width == this.canvas.width && height == this.canvas.height) {
      const image = new ImageData(videoFrame, width, height);
      this.canvasCtx.putImageData(image, 0, 0);
    } else {
      const image = new ImageData(videoFrame.slice(), width, height);
      createImageBitmap(image)
        .then((bitmap) => {
          this.canvasCtx.drawImage(bitmap, 0, 0, this.canvas.width, this.canvas.height);
        })
        .catch((error) => {
          throw error;
        });
    }
  }

  audioInit(sampleRate: number, _dataSamples: number, sampleFormatPtr: number) {
    var littleEndian = (function () {
      var buffer = new ArrayBuffer(2);
      new DataView(buffer).setInt16(0, 256, true);
      return new Int16Array(buffer)[0] === 256;
    })();
    if (littleEndian) {
      new DataView(this.wasmMemory.buffer).setUint8(sampleFormatPtr, 4); // 4=F32Le
    } else {
      new DataView(this.wasmMemory.buffer).setUint8(sampleFormatPtr, 3); // 3=F32Be
    }

    // TODO: Postpone this creation until the user do any action on the page.
    const audioContext = new AudioContext({ sampleRate });
    this.audioContext = audioContext;
    this.audioContext.audioWorklet
      .addModule("data:text/javascript," + encodeURI(AUDIO_WORKLET_PROCESSOR_CODE))
      .then(() => {
        this.audioInputNode = new AudioWorkletNode(audioContext, AUDIO_WORKLET_PROCESSOR_NAME);
        this.audioInputNode.connect(audioContext.destination);
      })
      .catch((error) => {
        throw error;
      });
  }

  audioEnqueue(audioDataOffset: number, audioDataLen: number) {
    if (this.audioInputNode === undefined) {
      return;
    }
    const data = new Float32Array(this.wasmMemory.buffer, audioDataOffset, audioDataLen).slice();
    this.audioInputNode.port.postMessage(data, [data.buffer]);
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

  clockSetTimeout(tag: number, timeout: number): ActionId {
    const actionId = this.getNextActionId();
    setTimeout(() => {
      this.enqueueEvent({ timeout: { id: actionId, tag } });
    }, timeout * 1000);
    return actionId;
  }

  stateSave(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);
    const data = new Uint8Array(this.wasmMemory.buffer, dataOffset, dataLen).slice();

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.put({ name, data });
    request.onsuccess = () => {
      this.enqueueEvent({ state: { saved: { id: actionId } } });
    };
    request.onerror = () => {
      this.enqueueEvent({ state: { saved: { id: actionId, failed: { message: "PUT_FAILURE" } } } });
    };

    return actionId;
  }

  stateLoad(nameOffset: number, nameLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.get(name);
    request.onsuccess = (event) => {
      // @ts-ignore
      if (event.target.result === undefined) {
        this.enqueueEvent({ state: { loaded: { id: actionId } } });
      } else {
        // @ts-ignore
        // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access,@typescript-eslint/no-unsafe-assignment
        const data = event.target.result.data;

        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        this.enqueueEvent({ state: { loaded: { id: actionId, data } } });
      }
    };
    request.onerror = () => {
      this.enqueueEvent({ state: { loaded: { id: actionId, failed: { message: "GET_FAILURE" } } } });
    };

    return actionId;
  }

  stateDelete(nameOffset: number, nameLen: number): ActionId {
    const actionId = this.getNextActionId();
    const name = this.getWasmString(nameOffset, nameLen);

    const transaction = this.db.transaction(["states"], "readwrite");
    const objectStore = transaction.objectStore("states");
    const request = objectStore.delete(name);
    request.onsuccess = () => {
      this.enqueueEvent({ state: { deleted: { id: actionId } } });
    };
    request.onerror = () => {
      this.enqueueEvent({ state: { deleted: { id: actionId, failed: { message: "DELETE_FAILURE" } } } });
    };

    return actionId;
  }

  private getWasmString(offset: number, len: number): string {
    const buffer = new Uint8Array(this.wasmMemory.buffer, offset, len);
    return new TextDecoder("utf-8").decode(buffer);
  }

  private getNextActionId(): ActionId {
    const actionId = this.nextActionId;
    this.nextActionId = this.nextActionId + 1;
    return actionId;
  }
}

export { System };

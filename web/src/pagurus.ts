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

type Size = { width: number; height: number };
type Position = { x: number; y: number };

type Event =
  | { timeout: TimeoutTag }
  | { key: PagurusKeyEvent }
  | { mouse: PagurusMouseEvent }
  | { windowResized: Size };

type TimeoutTag = number;

type PagurusKeyEvent = { key: Key; ctrl: boolean; alt: boolean };

type PagurusMouseEvent =
  | { move: { position: Position } }
  | { down: { position: Position } }
  | { up: { position: Position } };

type Key =
  | { char: string }
  | "left"
  | "right"
  | "down"
  | "up"
  | "return"
  | "backspace"
  | "delete"
  | "shift"
  | "ctrl"
  | "alt"
  | "tab"
  | "backTab"
  | "esc";

function toPagurusKey(key: string): Key | undefined {
  switch (key) {
    case "ArrowUp":
      return "up";
    case "ArrowDown":
      return "down";
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    case "Enter":
      return "return";
    case "Backspace":
      return "backspace";
    case "Delete":
      return "delete";
    case "Tab":
      return "tab";
    case "Escape":
      return "esc";
    default:
      // TODO: Consider surrogate pairs
      if (key.length === 1) {
        return { char: key };
      } else {
        return;
      }
  }
}

interface SystemOptions {
  canvas?: HTMLCanvasElement;
  propagateControlKey?: boolean;
  disableTouchEvents?: boolean;
  disableKeyEvents?: boolean;
}

class System {
  private wasmMemory: WebAssembly.Memory;
  private canvas?: HTMLCanvasElement;
  private audioContext?: AudioContext;
  private audioInputNode?: AudioWorkletNode;
  private audioSampleRate?: number;
  private startTime: number;
  private eventQueue: Event[];
  private resolveNextEvent?: (event: Event) => void;
  private propagateControlKey: boolean;

  static create(wasmMemory: WebAssembly.Memory, options: SystemOptions = {}): System {
    return new System(wasmMemory, options.canvas, options);
  }

  private constructor(wasmMemory: WebAssembly.Memory, canvas: HTMLCanvasElement | undefined, options: SystemOptions) {
    this.wasmMemory = wasmMemory;
    this.propagateControlKey = !(options.propagateControlKey === false);

    let canvasSize = { width: 0, height: 0 };
    this.canvas = canvas;
    if (this.canvas !== undefined) {
      canvasSize = { width: this.canvas.width, height: this.canvas.height };
      this.canvas.style.width = `${canvasSize.width}px`;
      this.canvas.style.height = `${canvasSize.height}px`;
    }

    this.startTime = performance.now();

    if (this.canvas !== undefined) {
      if (!(options.disableKeyEvents === true)) {
        document.addEventListener("keyup", (event) => {
          this.handleKeyup(event);
          this.preventKeyEventDefaultIfNeed(event);
        });
        document.addEventListener("keydown", (event) => {
          this.preventKeyEventDefaultIfNeed(event);
        });
      }

      this.canvas.addEventListener("mousemove", (event) => {
        this.handleMousemove(event);
      });
      this.canvas.addEventListener("mousedown", (event) => {
        this.handleMousedown(event);
      });
      this.canvas.addEventListener("mouseup", (event) => {
        this.handleMouseup(event);
      });

      if (!(options.disableTouchEvents === true)) {
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
      }
    }

    const initialEvent = { windowResized: canvasSize };
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

  private preventKeyEventDefaultIfNeed(event: KeyboardEvent): void {
    if (this.propagateControlKey) {
      if (event.ctrlKey || event.key == "Control") {
        return;
      }
    }

    event.stopPropagation();
    event.preventDefault();
  }

  private handleKeyup(event: KeyboardEvent) {
    const key = toPagurusKey(event.key);
    if (key !== undefined) {
      const ctrl = event.ctrlKey;
      const alt = event.altKey;
      if (key == "tab" && event.shiftKey) {
        this.enqueueEvent({ key: { key: "backTab", ctrl, alt } });
      } else {
        this.enqueueEvent({ key: { key, ctrl, alt } });
      }
    }
  }

  private touchPosition(touch: Touch): Position {
    if (this.canvas === undefined) {
      throw new Error("bug");
    }
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
      const position = this.touchPosition(touch);
      this.enqueueEvent({ mouse: { down: { position } } });
      break;
    }
  }

  private handleTouchend(event: TouchEvent) {
    const touches = event.changedTouches;
    for (let i = 0; i < touches.length; i++) {
      const touch = touches[i];
      const position = this.touchPosition(touch);
      this.enqueueEvent({ mouse: { up: { position } } });
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
    if (event.button === 0) {
      this.enqueueEvent({ mouse: { down: { position: { x, y } } } });
    }
  }

  private handleMouseup(event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    if (event.button === 0) {
      this.enqueueEvent({ mouse: { up: { position: { x, y } } } });
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

  requestRedraw() {
    if (this.canvas === undefined) {
      return;
    }
    const canvasSize = { width: this.canvas.width, height: this.canvas.height };
    this.canvas.style.width = `${canvasSize.width}px`;
    this.canvas.style.height = `${canvasSize.height}px`;
    this.enqueueEvent({ windowResized: canvasSize });
  }

  videoInit(width: number, _height: number, pixelFormatPtr: number, stridePtr: number) {
    new DataView(this.wasmMemory.buffer).setUint8(pixelFormatPtr, 1); // 1=RGB32
    new DataView(this.wasmMemory.buffer).setUint32(stridePtr, width, true);
  }

  videoDraw(videoFrameOffset: number, videoFrameLen: number, width: number, stride: number, format: number) {
    if (this.canvas === undefined) {
      return;
    }
    if (format != 1) {
      throw new Error(`expected RGB32(3) format, but got ${format}`);
    }
    if (width != stride) {
      throw new Error(`width ${width} differs from stride ${stride}`);
    }

    if (width === 0 || videoFrameLen === 0) {
      return;
    }

    const canvasCtx = this.canvas.getContext("2d");
    if (!canvasCtx) {
      throw Error("failed to get canvas 2D context");
    }

    const height = videoFrameLen / 4 / width;
    const videoFrame = new Uint8ClampedArray(this.wasmMemory.buffer, videoFrameOffset, videoFrameLen);
    if (width != this.canvas.width || height != this.canvas.height) {
      const xScale = width / this.canvas.width;
      const yScale = height / this.canvas.height;
      this.canvas.width = width;
      this.canvas.height = height;
      canvasCtx.scale(xScale, yScale);
    }

    const image = new ImageData(videoFrame, width, height);
    canvasCtx.putImageData(image, 0, 0);
  }

  audioInit(sampleRate: number, _dataSamples: number, sampleFormatPtr: number) {
    this.audioSampleRate = sampleRate;
    const littleEndian = (function () {
      const buffer = new ArrayBuffer(2);
      new DataView(buffer).setInt16(0, 256, true);
      return new Int16Array(buffer)[0] === 256;
    })();
    if (littleEndian) {
      new DataView(this.wasmMemory.buffer).setUint8(sampleFormatPtr, 3); // 3=F32Le
    } else {
      new DataView(this.wasmMemory.buffer).setUint8(sampleFormatPtr, 2); // 2=F32Be
    }
  }

  audioEnqueue(audioDataOffset: number, audioDataLen: number) {
    if (this.audioSampleRate === undefined) {
      console.warn("audioInit() has not been called yet");
      return;
    }

    const data = new Float32Array(this.wasmMemory.buffer, audioDataOffset, audioDataLen / 4).slice();
    if (this.audioContext === undefined) {
      const blob = new Blob([AUDIO_WORKLET_PROCESSOR_CODE], { type: "application/javascript" });
      const audioContext = new AudioContext({ sampleRate: this.audioSampleRate });
      this.audioContext = audioContext;
      this.audioContext.audioWorklet
        .addModule(URL.createObjectURL(blob))
        .then(() => {
          this.audioInputNode = new AudioWorkletNode(audioContext, AUDIO_WORKLET_PROCESSOR_NAME);
          this.audioInputNode.connect(audioContext.destination);
          this.audioInputNode.port.postMessage(data, [data.buffer]);
        })
        .catch((error) => {
          throw error;
        });
    } else if (this.audioInputNode !== undefined) {
      this.audioInputNode.port.postMessage(data, [data.buffer]);
    }
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

  clockSetTimeout(tag: TimeoutTag, timeout: number) {
    setTimeout(() => {
      this.enqueueEvent({ timeout: tag });
    }, timeout * 1000);
  }

  private getWasmString(offset: number, len: number): string {
    const buffer = new Uint8Array(this.wasmMemory.buffer, offset, len);
    return new TextDecoder("utf-8").decode(buffer);
  }
}

class Game {
  private wasmInstance: WebAssembly.Instance;
  private gameInstance: number;
  private systemRef: SystemRef;
  readonly memory: WebAssembly.Memory;

  private constructor(wasmInstance: WebAssembly.Instance, systemRef: SystemRef) {
    this.wasmInstance = wasmInstance;
    this.gameInstance = (wasmInstance.exports.gameNew as CallableFunction)() as number;
    this.memory = wasmInstance.exports.memory as WebAssembly.Memory;
    this.systemRef = systemRef;
  }

  static async load(gameWasmPath: string): Promise<Game> {
    const systemRef = new SystemRef();
    const importObject = {
      env: {
        consoleLog(messageOffset: number, messageLen: number) {
          systemRef.getSystem().consoleLog(messageOffset, messageLen);
        },
        systemVideoInit(width: number, height: number, pixelFormatPtr: number, stridePtr: number) {
          systemRef.getSystem().videoInit(width, height, pixelFormatPtr, stridePtr);
        },
        systemVideoDraw(
          videoFrameOffset: number,
          videoFrameLen: number,
          width: number,
          stride: number,
          format: number
        ) {
          systemRef.getSystem().videoDraw(videoFrameOffset, videoFrameLen, width, stride, format);
        },
        systemAudioInit(sampleRate: number, dataSamples: number, sampleFormatPtr: number) {
          systemRef.getSystem().audioInit(sampleRate, dataSamples, sampleFormatPtr);
        },
        systemAudioEnqueue(dataOffset: number, dataLen: number) {
          systemRef.getSystem().audioEnqueue(dataOffset, dataLen);
        },
        systemClockGameTime(): number {
          return systemRef.getSystem().clockGameTime();
        },
        systemClockUnixTime(): number {
          return systemRef.getSystem().clockUnixTime();
        },
        systemClockSetTimeout(tag: number, timeout: number) {
          systemRef.getSystem().clockSetTimeout(tag, timeout);
        },
      },
    };
    const results = await WebAssembly.instantiateStreaming(fetch(gameWasmPath), importObject);
    const wasmInstance = results.instance;

    return new Game(wasmInstance, systemRef);
  }

  initialize(system: System) {
    this.systemRef.setSystem(system);
    try {
      const error = (this.wasmInstance.exports.gameInitialize as CallableFunction)(this.gameInstance) as number;
      if (error !== 0) {
        throw new Error(this.getWasmString(error));
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  handleEvent(system: System, event: Event): boolean {
    this.systemRef.setSystem(system);

    try {
      const eventBytesPtr = this.createWasmBytes(new TextEncoder().encode(JSON.stringify(event)));
      const result = (this.wasmInstance.exports.gameHandleEvent as CallableFunction)(
        this.gameInstance,
        eventBytesPtr
      ) as number;
      if (result === 0) {
        return true;
      }

      const error = this.getWasmString(result);
      if (JSON.parse(error) === null) {
        return false;
      } else {
        throw new Error(error);
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  query(system: System, name: string): Uint8Array {
    this.systemRef.setSystem(system);

    try {
      const nameBytesPtr = this.createWasmBytes(new TextEncoder().encode(name));
      const result = (this.wasmInstance.exports.gameQuery as CallableFunction)(
        this.gameInstance,
        nameBytesPtr
      ) as number;
      const bytes = this.getWasmBytes(result);
      if (bytes[bytes.length - 1] === 0) {
        return bytes.subarray(0, bytes.length - 1);
      } else {
        const error = new TextDecoder("utf-8").decode(bytes.subarray(0, bytes.length - 1));
        throw new Error(error);
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  command(system: System, name: string, data: Uint8Array) {
    this.systemRef.setSystem(system);

    try {
      const nameBytesPtr = this.createWasmBytes(new TextEncoder().encode(name));
      const dataBytesPtr = this.createWasmBytes(data);
      const result = (this.wasmInstance.exports.gameCommand as CallableFunction)(
        this.gameInstance,
        nameBytesPtr,
        dataBytesPtr
      ) as number;
      if (result !== 0) {
        const error = this.getWasmString(result);
        throw new Error(error);
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  private createWasmBytes(bytes: Uint8Array): number {
    const wasmBytesPtr = (this.wasmInstance.exports.memoryAllocateBytes as CallableFunction)(bytes.length) as number;
    const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(wasmBytesPtr) as number;
    const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(wasmBytesPtr) as number;
    new Uint8Array(this.memory.buffer, offset, len).set(bytes);
    return wasmBytesPtr;
  }

  private getWasmString(wasmBytesPtr: number): string {
    try {
      const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(wasmBytesPtr) as number;
      const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(wasmBytesPtr) as number;
      const bytes = new Uint8Array(this.memory.buffer, offset, len);
      return new TextDecoder("utf-8").decode(bytes);
    } finally {
      (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(wasmBytesPtr);
    }
  }

  private getWasmBytes(wasmBytesPtr: number): Uint8Array {
    try {
      const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(wasmBytesPtr) as number;
      const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(wasmBytesPtr) as number;
      return new Uint8Array(this.memory.buffer, offset, len).slice();
    } finally {
      (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(wasmBytesPtr);
    }
  }
}

class SystemRef {
  private system?: System;

  getSystem(): System {
    if (this.system === undefined) {
      throw Error("SystemRef.system is undefined");
    }
    return this.system;
  }

  setSystem(system: System) {
    this.system = system;
  }

  clearSystem() {
    this.system = undefined;
  }
}

export { System, Game };

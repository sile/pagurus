class System {
  canvas: HTMLCanvasElement;
  audioContext?: AudioContext;
  wasmMemory: WebAssembly.Memory;
  nextActionId: number;
  eventQueue: Event[];

  constructor(canvas: HTMLCanvasElement, wasmMemory: WebAssembly.Memory) {
    this.canvas = canvas;
    this.wasmMemory = wasmMemory;
    this.nextActionId = 0;
    this.eventQueue = [];
  }

  run(game: Game) {
    game.initialize(this);

    document.addEventListener("keyup", (event) => {
      this.handleKeyup(game, event);
    });
    document.addEventListener("keydown", (event) => {
      this.handleKeydown(game, event);
    });
    this.canvas.addEventListener("mousemove", (event) => {
      this.handleMousemove(game, event);
    });
    this.canvas.addEventListener("mousedown", (event) => {
      this.handleMousedown(game, event);
    });
    this.canvas.addEventListener("mouseup", (event) => {
      // TODO: handle mouseleave
      this.handleMouseup(game, event);
    });

    this.tick(game);
  }

  private callHandleEvent(game: Game, event: Event) {
    this.eventQueue.push(event);
    while (true) {
      const event = this.eventQueue.shift();
      if (event === undefined) {
        break;
      }

      try {
        game.handleEvent(this, event);
      } catch (e) {
        // TODO
        game.handleEvent(this, "terminated");
        throw e;
      }
    }
  }

  private handleMousemove(game: Game, event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    this.callHandleEvent(game, { mouse: { move: { position: { x, y } } } });
  }

  private handleMousedown(game: Game, event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    let button: "left" | "middle" | "right";
    switch (event.button) {
      case 0:
        button = "left";
        break;
      case 1:
        button = "middle";
        break;
      case 2:
        button = "right";
        break;
      default:
        return;
    }
    this.callHandleEvent(game, { mouse: { down: { position: { x, y }, button } } });
  }

  private handleMouseup(game: Game, event: MouseEvent) {
    const x = event.offsetX;
    const y = event.offsetY;
    let button: "left" | "middle" | "right";
    switch (event.button) {
      case 0:
        button = "left";
        break;
      case 1:
        button = "middle";
        break;
      case 2:
        button = "right";
        break;
      default:
        return;
    }
    this.callHandleEvent(game, { mouse: { up: { position: { x, y }, button } } });
  }

  private handleKeyup(game: Game, event: KeyboardEvent) {
    let key: "up" | "down" | "left" | "right" | "return";
    switch (event.key) {
      case "Enter":
        key = "return";
        break;
      case "ArrowUp":
        key = "up";
        break;
      case "ArrowDown":
        key = "down";
        break;
      case "ArrowLeft":
        key = "left";
        break;
      case "ArrowRight":
        key = "right";
        break;
      default:
        return;
    }
    this.callHandleEvent(game, { key: { up: { key } } });
  }

  private handleKeydown(game: Game, event: KeyboardEvent) {
    let key: "up" | "down" | "left" | "right" | "return";
    switch (event.key) {
      case "Enter":
        key = "return";
        break;
      case "ArrowUp":
        key = "up";
        break;
      case "ArrowDown":
        key = "down";
        break;
      case "ArrowLeft":
        key = "left";
        break;
      case "ArrowRight":
        key = "right";
        break;
      default:
        return;
    }
    this.callHandleEvent(game, { key: { down: { key } } });
  }

  private tick(game: Game) {
    if (game.isFinished()) {
      return;
    }

    this.callHandleEvent(game, "tick");
    setTimeout(() => {
      this.tick(game);
    }, 1000 / 30); // TODO: change fps
  }

  consoleLog(msgOffset: number, msgLen: number) {
    const msg = this.getWasmString(msgOffset, msgLen);
    console.log(msg);
  }

  audioEnqueue(dataOffset: number, dataLen: number) {
    if (this.audioContext === undefined) {
      this.audioContext = new AudioContext();
    }

    const data = new Uint8ClampedArray(this.wasmMemory.buffer, dataOffset, dataLen);
    const buffer = this.audioContext.createBuffer(1, dataLen / 2, 48000);
    const tmpBuffer = new Float32Array(dataLen / 2);

    for (let i = 0; i < dataLen; i += 2) {
      var n = (data[i] << 8) | data[i + 1];
      if (n > 0x7fff) {
        n -= 0x10000;
      }
      tmpBuffer[i / 2] = n / 0x7fff;
    }

    buffer.copyToChannel(tmpBuffer, 0);
    const source = this.audioContext.createBufferSource();
    source.buffer = buffer;
    source.connect(this.audioContext.destination);
    source.start();
  }

  imageRender(pixelsOffset: number, pixelsLen: number, canvasWidth: number) {
    const canvasCtx = this.canvas.getContext("2d");
    if (!canvasCtx) {
      throw Error("TODO");
    }

    const canvasHeight = pixelsLen / 3 / canvasWidth;
    const image = canvasCtx.createImageData(canvasWidth, canvasHeight);
    const pixels = new Uint8ClampedArray(this.wasmMemory.buffer, pixelsOffset, pixelsLen);
    for (let i = 0; i < pixelsLen / 3; i++) {
      image.data.set(pixels.subarray(i * 3, i * 3 + 3), i * 4);
      image.data[i * 4 + 3] = 255;
    }

    canvasCtx.putImageData(image, 0, 0);
  }

  resourceGet(nameOffset: number, nameLen: number): BigInt {
    const actionId = this.nextActionId;
    this.nextActionId += 1;

    const name = this.getWasmString(nameOffset, nameLen);
    let event: Event = { resource: { get: { action: actionId, succeeded: false } } };
    if (name.startsWith("grn:json:")) {
      const key = name.substring("grn:json:".length);
      const item = localStorage.getItem(key);
      let data;
      if (item !== null) {
        data = new TextEncoder().encode(item);
      }
      event = { resource: { get: { action: actionId, data, succeeded: true } } };
    }
    this.eventQueue.push(event);
    return BigInt(actionId);
  }

  resourcePut(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): BigInt {
    const actionId = this.nextActionId;
    this.nextActionId += 1;

    const name = this.getWasmString(nameOffset, nameLen);
    let event = { resource: { put: { action: actionId, succeeded: false } } };
    if (name.startsWith("grn:json:")) {
      const key = name.substring("grn:json:".length);
      const data = this.getWasmString(dataOffset, dataLen);

      try {
        localStorage.setItem(key, data);
        event = { resource: { put: { action: actionId, succeeded: true } } };
      } catch (e) {
        console.log(`[ERROR] Failed to localStorage.setItem(${key}, ...): reason=${e}`);
      }
    }
    this.eventQueue.push(event);
    return BigInt(actionId);
  }

  private getWasmString(offset: number, len: number): string {
    const buffer = new Uint8Array(this.wasmMemory.buffer, offset, len);
    return new TextDecoder("utf-8").decode(buffer);
  }

  // private getWasmBytes(offset: number, len: number): Uint8Array {
  //   return new Uint8Array(this.wasmMemory.buffer, offset, len);
  // }
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

class Game {
  private wasmInstance: WebAssembly.Instance;
  private gameInstance: number;
  private systemRef: SystemRef;
  readonly memory: WebAssembly.Memory;

  private constructor(wasmInstance: WebAssembly.Instance, systemRef: SystemRef) {
    this.wasmInstance = wasmInstance;
    this.gameInstance = (wasmInstance.exports.gameNew as CallableFunction)();
    this.memory = wasmInstance.exports.memory as WebAssembly.Memory;
    this.systemRef = systemRef;
  }

  static async load(assetsPath: string): Promise<Game> {
    const systemRef = new SystemRef();

    const importObject = {
      env: {
        systemImageRender: (pixelsOffset: number, pixelsLen: number, canvasWidth: number) => {
          systemRef.getSystem().imageRender(pixelsOffset, pixelsLen, canvasWidth);
        },
        systemAudioEnqueue: (dataOffset: number, dataLen: number) => {
          systemRef.getSystem().audioEnqueue(dataOffset, dataLen);
        },
        systemAudioCancel: () => {
          throw Error("not implemented");
        },
        systemClockNow: () => {
          // TODO: use system
          return performance.now() / 1000.0;
        },
        systemConsoleLog: (msgOffset: number, msgLen: number) => {
          systemRef.getSystem().consoleLog(msgOffset, msgLen);
        },
        systemResourcePut: (nameOffset: number, nameLen: number, dataOffset: number, dataLen: number) => {
          return systemRef.getSystem().resourcePut(nameOffset, nameLen, dataOffset, dataLen);
        },
        systemResourceGet: (nameOffset: number, nameLen: number) => {
          return systemRef.getSystem().resourceGet(nameOffset, nameLen);
        },
        systemResourceDelete: () => {
          throw Error("not implemented");
        },
      },
    };

    const results = await WebAssembly.instantiateStreaming(fetch(assetsPath + "/game.wasm"), importObject);
    // TODO: cache module https://developer.mozilla.org/ja/docs/WebAssembly/Caching_modules

    const wasmInstance = results.instance;
    return new Game(wasmInstance, systemRef);
  }

  getRequirements(): GameRequirements {
    const bytes = (this.wasmInstance.exports.gameRequirements as CallableFunction)();
    const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(bytes);
    const size = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(bytes);

    try {
      const buffer = new Uint8Array((this.wasmInstance.exports.memory as WebAssembly.Memory).buffer, offset, size);
      return JSON.parse(new TextDecoder("utf-8").decode(buffer));
    } finally {
      (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(bytes);
    }
  }

  initialize(system: System) {
    this.systemRef.setSystem(system);

    try {
      (this.wasmInstance.exports.gameInitialize as CallableFunction)(this.gameInstance);
    } finally {
      this.systemRef.clearSystem();
    }
  }

  handleEvent(system: System, event: Event) {
    this.systemRef.setSystem(system);

    let data;
    try {
      // @ts-ignore
      data = event.resource.get.data;

      // @ts-ignore
      event.resource.get.data = undefined;
    } catch (e) {}

    const encoded = new TextEncoder().encode(JSON.stringify(event));
    const bytes = (this.wasmInstance.exports.memoryAllocateBytes as CallableFunction)(encoded.length);
    const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(bytes);
    new Uint8Array(this.memory.buffer, offset, encoded.length).set(encoded);

    let dataBytes;
    let dataOffset = 0;
    let dataLength = 0;
    if (data !== undefined) {
      dataBytes = (this.wasmInstance.exports.memoryAllocateBytes as CallableFunction)(data.length);
      dataOffset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(dataBytes);
      dataLength = data.length;
      new Uint8Array(this.memory.buffer, dataOffset, dataLength).set(data);
    }

    try {
      (this.wasmInstance.exports.gameHandleEvent as CallableFunction)(
        this.gameInstance,
        offset,
        encoded.length,
        dataOffset,
        dataLength
      );
    } finally {
      (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(bytes);
      if (data !== undefined) {
        (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(dataBytes);
      }
      this.systemRef.clearSystem();
    }
  }

  isFinished(): boolean {
    return (this.wasmInstance.exports.gameIsFinished as CallableFunction)(this.gameInstance) == 1;
  }
}

type Event =
  | "tick"
  | "terminated"
  | { mouse: { move: { position: { x: number; y: number } } } }
  | { mouse: { down: { position: { x: number; y: number }; button: "right" | "middle" | "left" } } }
  | { mouse: { up: { position: { x: number; y: number }; button: "right" | "middle" | "left" } } }
  | { key: { down: { key: "right" | "left" | "down" | "up" | "return" } } }
  | { key: { up: { key: "right" | "left" | "down" | "up" | "return" } } }
  | { resource: { put: { action: number; succeeded: boolean } } }
  | { resource: { get: { action: number; succeeded: boolean; data?: Uint8Array } } };

interface Size {
  width: number;
  height: number;
}

interface GameRequirements {
  idealWindowSize: Size;
  iealFps?: number;
  fixedAspectRatio: boolean;
  minMemoryBytes?: number;
}

export { System, Game, GameRequirements, Size, Event };

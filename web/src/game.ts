import { Event } from "./event";
import { System } from "./system";

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
        systemClockSetTimeout(tag: number, timeout: number): bigint {
          return BigInt(systemRef.getSystem().clockSetTimeout(tag, timeout));
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

export { Game };

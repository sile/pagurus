import { Event } from "./event";
import { System } from "./system";

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

  static async load(gameWasmPath: string): Promise<Game> {
    const systemRef = new SystemRef();
    const importObject = {
      env: {
        systemVideoRender(videoFrameOffset: number, videoFrameLen: number, width: number) {
          systemRef.getSystem().videoRender(videoFrameOffset, videoFrameLen, width);
        },
        systemAudioEnqueue(dataOffset: number, dataLen: number): number {
          return systemRef.getSystem().audioEnqueue(dataOffset, dataLen);
        },
        systemConsoleLog(messageOffset: number, messageLen: number) {
          systemRef.getSystem().consoleLog(messageOffset, messageLen);
        },
        systemClockGameTime(): number {
          return systemRef.getSystem().clockGameTime();
        },
        systemClockUnixTime(): number {
          return systemRef.getSystem().clockUnixTime();
        },
        systemClockSetTimeout(timeout: number): number {
          return systemRef.getSystem().clockSetTimeout(timeout);
        },
        systemStateSave(nameOffset: number, nameLen: number, dataOffset: number, dataLen: number): number {
          return systemRef.getSystem().stateSave(nameOffset, nameLen, dataOffset, dataLen);
        },
        systemStateLoad(nameOffset: number, nameLen: number): number {
          return systemRef.getSystem().stateLoad(nameOffset, nameLen);
        },
        systemStateDelete(nameOffset: number, nameLen: number): number {
          return systemRef.getSystem().stateDelete(nameOffset, nameLen);
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
      const error = (this.wasmInstance.exports.gameInitialize as CallableFunction)(this.gameInstance);
      if (error !== 0) {
        throw new Error(this.getWasmString(error));
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  private getWasmString(bytesPtr: number): string {
    try {
      const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(bytesPtr);
      const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(bytesPtr);
      const bytes = new Uint8Array(this.memory.buffer, offset, len);
      return new TextDecoder("utf-8").decode(bytes);
    } finally {
      (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(bytesPtr);
    }
  }

  //   handleEvent(system: System, event: Event): bool {
  //     this.systemRef.setSystem(system);
  //     let data;
  //     try {
  //       // @ts-ignore
  //       data = event.resource.get.data;
  //       // @ts-ignore
  //       event.resource.get.data = undefined;
  //     } catch (e) {}
  //     const encoded = new TextEncoder().encode(JSON.stringify(event));
  //     const bytes = (this.wasmInstance.exports.memoryAllocateBytes as CallableFunction)(encoded.length);
  //     const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(bytes);
  //     new Uint8Array(this.memory.buffer, offset, encoded.length).set(encoded);
  //     let dataBytes;
  //     let dataOffset = 0;
  //     let dataLength = 0;
  //     if (data !== undefined) {
  //       dataBytes = (this.wasmInstance.exports.memoryAllocateBytes as CallableFunction)(data.length);
  //       dataOffset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(dataBytes);
  //       dataLength = data.length;
  //       new Uint8Array(this.memory.buffer, dataOffset, dataLength).set(data);
  //     }
  //     try {
  //       (this.wasmInstance.exports.gameHandleEvent as CallableFunction)(
  //         this.gameInstance,
  //         offset,
  //         encoded.length,
  //         dataOffset,
  //         dataLength
  //       );
  //     } finally {
  //       (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(bytes);
  //       if (data !== undefined) {
  //         (this.wasmInstance.exports.memoryFreeBytes as CallableFunction)(dataBytes);
  //       }
  //       this.systemRef.clearSystem();
  //     }
  //   }
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

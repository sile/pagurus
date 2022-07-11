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
        // systemImageRender: (pixelsOffset: number, pixelsLen: number, canvasWidth: number) => {
        //   systemRef.getSystem().imageRender(pixelsOffset, pixelsLen, canvasWidth);
        // },
        // systemAudioEnqueue: (dataOffset: number, dataLen: number) => {
        //   systemRef.getSystem().audioEnqueue(dataOffset, dataLen);
        // },
        // systemAudioCancel: () => {
        //   throw Error("not implemented");
        // },
        // systemClockNow: () => {
        //   // TODO: use system
        //   return performance.now() / 1000.0;
        // },
        // systemConsoleLog: (msgOffset: number, msgLen: number) => {
        //   systemRef.getSystem().consoleLog(msgOffset, msgLen);
        // },
        // systemResourcePut: (nameOffset: number, nameLen: number, dataOffset: number, dataLen: number) => {
        //   return systemRef.getSystem().resourcePut(nameOffset, nameLen, dataOffset, dataLen);
        // },
        // systemResourceGet: (nameOffset: number, nameLen: number) => {
        //   return systemRef.getSystem().resourceGet(nameOffset, nameLen);
        // },
        // systemResourceDelete: () => {
        //   throw Error("not implemented");
        // },
      },
    };
    const results = await WebAssembly.instantiateStreaming(fetch(gameWasmPath), importObject);
    const wasmInstance = results.instance;

    return new Game(wasmInstance, systemRef);
  }

  //   initialize(system: System) {
  //     this.systemRef.setSystem(system);
  //     try {
  //       (this.wasmInstance.exports.gameInitialize as CallableFunction)(this.gameInstance);
  //     } finally {
  //       this.systemRef.clearSystem();
  //     }
  //   }

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

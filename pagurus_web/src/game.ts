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

  handleEvent(system: System, event: Event): boolean {
    this.systemRef.setSystem(system);

    let data;
    try {
      if (event instanceof Object && "state" in event && "loaded" in event.state) {
        data = event.state.loaded.data;
        event.state.loaded.data = undefined;
      }

      const eventBytesPtr = this.createWasmBytes(new TextEncoder().encode(JSON.stringify(event)));
      let dataBytesPtr = 0;
      if (data !== undefined) {
        dataBytesPtr = this.createWasmBytes(data);
      }

      const result = (this.wasmInstance.exports.gameHandleEvent as CallableFunction)(
        this.gameInstance,
        eventBytesPtr,
        dataBytesPtr
      );
      if (result === 0) {
        return true;
      }

      let error = this.getWasmString(result);
      if (JSON.parse(error) === undefined) {
        return false;
      } else {
        throw new Error(error);
      }
    } finally {
      this.systemRef.clearSystem();
    }
  }

  private createWasmBytes(bytes: Uint8Array): number {
    const wasmBytesPtr = (this.wasmInstance.exports.allocateFreeBytes as CallableFunction)(bytes.length);
    const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(wasmBytesPtr);
    const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(wasmBytesPtr);
    new Uint8Array(this.memory.buffer, offset, len).set(bytes);
    return wasmBytesPtr;
  }

  private getWasmString(wasmBytesPtr: number): string {
    try {
      const offset = (this.wasmInstance.exports.memoryBytesOffset as CallableFunction)(wasmBytesPtr);
      const len = (this.wasmInstance.exports.memoryBytesLen as CallableFunction)(wasmBytesPtr);
      const bytes = new Uint8Array(this.memory.buffer, offset, len);
      return new TextDecoder("utf-8").decode(bytes);
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

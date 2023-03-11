import { Failure } from "./failure";
import { Size, Position } from "./spatial";

type Event =
  | "terminating"
  | { timeout: TimeoutEvent }
  | { key: KeyEvent }
  | { mouse: MouseEvent }
  | { window: WindowEvent }
  | { state: StateEvent };

type TimeoutEvent = { id: ActionId; tag: number };

type KeyEvent = { down: { key: Key } } | { up: { key: Key } };

type MouseEvent =
  | { move: { position: Position } }
  | { down: { position: Position; button: MouseButton } }
  | { up: { position: Position; button: MouseButton } };

type WindowEvent = { redrawNeeded: { size: Size } };

type StateEvent =
  | { loaded: { id: ActionId; data?: Uint8Array; failed?: Failure } }
  | { saved: { id: ActionId; failed?: Failure } }
  | { deleted: { id: ActionId; failed?: Failure } };

type ActionId = number; // TODO: Add TimeoutId

type MouseButton = "left" | "middle" | "right";

type Key =
  | "a"
  | "b"
  | "c"
  | "d"
  | "e"
  | "f"
  | "g"
  | "h"
  | "i"
  | "j"
  | "k"
  | "l"
  | "m"
  | "n"
  | "o"
  | "p"
  | "q"
  | "r"
  | "s"
  | "t"
  | "u"
  | "v"
  | "w"
  | "x"
  | "y"
  | "z"
  | "num0"
  | "num1"
  | "num2"
  | "num3"
  | "num4"
  | "num5"
  | "num6"
  | "num7"
  | "num8"
  | "num9"
  | "left"
  | "right"
  | "down"
  | "up"
  | "space"
  | "return"
  | "backspace"
  | "delete"
  | "shift"
  | "ctrl"
  | "alt"
  | "tab"
  | "esc";

function toPagurusKey(key: string): Key | undefined {
  switch (key) {
    case "KeyA":
      return "a";
    case "KeyB":
      return "b";
    case "KeyC":
      return "c";
    case "KeyD":
      return "d";
    case "KeyE":
      return "e";
    case "KeyF":
      return "f";
    case "KeyG":
      return "g";
    case "KeyH":
      return "h";
    case "KeyI":
      return "i";
    case "KeyJ":
      return "j";
    case "KeyK":
      return "k";
    case "KeyL":
      return "l";
    case "KeyM":
      return "m";
    case "KeyN":
      return "n";
    case "KeyO":
      return "o";
    case "KeyP":
      return "p";
    case "KeyQ":
      return "q";
    case "KeyR":
      return "r";
    case "KeyS":
      return "s";
    case "KeyT":
      return "t";
    case "KeyU":
      return "u";
    case "KeyV":
      return "v";
    case "KeyW":
      return "w";
    case "KeyX":
      return "x";
    case "KeyY":
      return "y";
    case "KeyZ":
      return "z";
    case "Digit0":
      return "num0";
    case "Digit1":
      return "num1";
    case "Digit2":
      return "num2";
    case "Digit3":
      return "num3";
    case "Digit4":
      return "num4";
    case "Digit5":
      return "num5";
    case "Digit6":
      return "num6";
    case "Digit7":
      return "num7";
    case "Digit8":
      return "num8";
    case "Digit9":
      return "num9";
    case "ArrowUp":
      return "up";
    case "ArrowDown":
      return "down";
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    case "Space":
      return "space";
    case "Enter":
      return "return";
    case "Backspace":
      return "backspace";
    case "Delete":
      return "delete";
    case "ShiftLeft":
    case "ShiftRight":
      return "shift";
    case "ControlLeft":
    case "ControlRight":
      return "ctrl";
    case "AltLeft":
    case "AltRight":
      return "alt";
    case "Tab":
      return "tab";
    case "Escape":
      return "escape";
    default:
      return;
  }
}

function toPagurusMouseButton(button: number): MouseButton | undefined {
  switch (button) {
    case 0:
      return "left";
    case 1:
      return "middle";
    case 2:
      return "right";
    default:
      return;
  }
}

export {
  Event,
  TimeoutEvent,
  KeyEvent,
  MouseEvent,
  WindowEvent,
  StateEvent,
  ActionId,
  MouseButton,
  Key,
  toPagurusKey,
  toPagurusMouseButton,
};

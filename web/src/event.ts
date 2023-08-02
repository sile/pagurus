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
  | "escape";

function toPagurusKey(key: string): Key | undefined {
  switch (key) {
    case "a":
      return "a";
    case "b":
      return "b";
    case "c":
      return "c";
    case "d":
      return "d";
    case "e":
      return "e";
    case "f":
      return "f";
    case "g":
      return "g";
    case "h":
      return "h";
    case "i":
      return "i";
    case "j":
      return "j";
    case "k":
      return "k";
    case "l":
      return "l";
    case "m":
      return "m";
    case "n":
      return "n";
    case "o":
      return "o";
    case "p":
      return "p";
    case "q":
      return "q";
    case "r":
      return "r";
    case "s":
      return "s";
    case "t":
      return "t";
    case "u":
      return "u";
    case "v":
      return "v";
    case "w":
      return "w";
    case "x":
      return "x";
    case "y":
      return "y";
    case "z":
      return "z";
    case "0":
      return "num0";
    case "1":
      return "num1";
    case "2":
      return "num2";
    case "3":
      return "num3";
    case "4":
      return "num4";
    case "5":
      return "num5";
    case "6":
      return "num6";
    case "7":
      return "num7";
    case "8":
      return "num8";
    case "9":
      return "num9";
    case "ArrowUp":
      return "up";
    case "ArrowDown":
      return "down";
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    case " ":
      return "space";
    case "Enter":
      return "return";
    case "Backspace":
      return "backspace";
    case "Delete":
      return "delete";
    case "Shift":
      return "shift";
    case "Control":
      return "ctrl";
    case "Alt":
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

import { Failure } from "./failure";
import { Size, Position } from "./spatial";

type Event =
  | { timeout: TimeoutEvent }
  | { key: KeyEvent }
  | { mouse: MouseEvent }
  | { window: WindowEvent };

type TimeoutEvent = { id: TimeoutId; tag: number };

type KeyEvent = { down: { key: Key } } | { up: { key: Key } };

type MouseEvent =
  | { move: { position: Position } }
  | { down: { position: Position; button: MouseButton } }
  | { up: { position: Position; button: MouseButton } };

type WindowEvent = { redrawNeeded: { size: Size } };

type TimeoutId = number;

type MouseButton = "left" | "middle" | "right";

type Key =
  | { char: string }
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
      return "esc";
    default:
      return {'char': key};
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
  TimeoutId,
  MouseButton,
  Key,
  toPagurusKey,
  toPagurusMouseButton,
};

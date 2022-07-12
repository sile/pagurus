import { Failure } from "./failure";
import { Size, Position } from "./spatial";

type Event =
  | "terminating"
  | { timeout: TimeoutEvent }
  | { key: KeyEvent }
  | { mouse: MouseEvent }
  | { window: WindowEvent }
  | { state: StateEvent };

type TimeoutEvent = { id: ActionId };

type KeyEvent = { down: { key: Key } } | { up: { key: Key } };

type MouseEvent =
  | { move: { position: Position } }
  | { down: { position: Position; button: MouseButton } }
  | { up: { position: Position; button: MouseButton } };

type WindowEvent = "focusGained" | "focusLost" | { redrawNeeded: { size: Size } };

type StateEvent =
  | { loaded: { id: ActionId; data?: Uint8Array; failed?: Failure } }
  | { saved: { id: ActionId; failed?: Failure } }
  | { deleted: { id: ActionId; failed?: Failure } };

type ActionId = number;

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
  | "alt";

function toPagurusKey(key: string): Key | undefined {
  switch (key) {
    case "Enter":
      return "return";
    case "ArrowUp":
      return "up";
    case "ArrowDown":
      return "down";
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    default:
      return;
  }
}

export { Event, TimeoutEvent, KeyEvent, MouseEvent, WindowEvent, StateEvent, ActionId, MouseButton, Key, toPagurusKey };

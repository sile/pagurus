import { Size, Position } from "./spatial";

type Event = { timeout: TimeoutTag } | { key: KeyEvent } | { mouse: MouseEvent } | { windowResized: Size };

type TimeoutTag = number;

type KeyEvent = { key: Key; ctrl: boolean; alt: boolean };

type MouseEvent =
  | { move: { position: Position } }
  | { down: { position: Position; button: MouseButton } }
  | { up: { position: Position; button: MouseButton } };

type MouseButton = "left" | "middle" | "right";

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

export { Event, TimeoutTag, KeyEvent, MouseEvent, MouseButton, Key, toPagurusKey, toPagurusMouseButton };

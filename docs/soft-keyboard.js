// A canvas cannot bring up a phone's on-screen keyboard, so a hidden textarea
// takes focus instead. Typed characters are recovered by diffing its value,
// which also works when the keyboard composes whole words (Android) and
// reports keydown as "Unidentified". The value always starts with SENTINEL
// so Backspace has something to delete.
const SENTINEL = "~~";

// Soft keyboards send key events without a physical key code. Those are left
// to the textarea's input diff; physical keyboards always report event.code.
export function isSoftKeyEvent(event) {
  return event.isComposing || event.keyCode === 229 || event.code === "";
}

// iOS shrinks only the visual viewport when the on-screen keyboard opens, so
// while it is open pages size themselves from --view-height to keep the TVC
// screen above it. Only then: resizing the canvas re-creates its GPU buffer,
// and Safari's toolbar can change the visual viewport at other times.
function trackViewHeight(input) {
  const viewport = globalThis.visualViewport;
  if (!viewport) {
    return;
  }
  const root = document.documentElement.style;
  let applied = 0;
  const update = () => {
    if (document.activeElement !== input) {
      if (applied) {
        root.removeProperty("--view-height");
        applied = 0;
      }
      return;
    }
    const height = Math.round(viewport.height);
    if (Math.abs(height - applied) > 1) {
      root.setProperty("--view-height", `${height}px`);
      applied = height;
    }
  };
  viewport.addEventListener("resize", update);
  input.addEventListener("focus", update);
  input.addEventListener("blur", update);
}

export function installSoftKeyboard({ typeText, bottom = "0px" }) {
  const input = document.createElement("textarea");
  input.className = "rtvc-soft-input";
  input.setAttribute("aria-label", "TVC keyboard input");
  input.setAttribute("autocapitalize", "off");
  input.setAttribute("autocomplete", "off");
  input.setAttribute("autocorrect", "off");
  input.setAttribute("enterkeyhint", "enter");
  input.spellcheck = false;

  const button = document.createElement("button");
  button.type = "button";
  button.className = "rtvc-soft-button";
  button.tabIndex = -1;
  button.textContent = "\u2328";
  button.setAttribute("aria-label", "Show keyboard");
  button.style.setProperty("--rtvc-soft-bottom", bottom);

  const style = document.createElement("style");
  style.textContent = `
    .rtvc-soft-input {
      position: fixed; left: 0; top: 0; width: 1px; height: 1px;
      padding: 0; border: 0; opacity: 0; resize: none; overflow: hidden;
      pointer-events: none; font-size: 16px; /* below 16px iOS zooms on focus */
    }
    .rtvc-soft-button {
      position: fixed; z-index: 10; display: none;
      right: calc(12px + env(safe-area-inset-right));
      /* Follows the visible height so it stays above the on-screen keyboard. */
      top: calc(var(--view-height, 100dvh) - 60px - var(--rtvc-soft-bottom)
        - env(safe-area-inset-bottom));
      width: 48px; height: 48px; border-radius: 50%;
      border: 1px solid #fff4; background: #222c; color: #eee;
      font: 24px/1 system-ui, sans-serif; touch-action: manipulation;
      -webkit-tap-highlight-color: transparent;
    }
    .rtvc-soft-button[aria-pressed="true"] { background: #468c; }
    @media (pointer: coarse) { .rtvc-soft-button { display: block; } }
  `;
  document.head.append(style);
  document.body.append(input, button);
  trackViewHeight(input);

  // `?keylog` shows the last keyboard events on screen, for debugging phone
  // keyboards where no developer console is at hand.
  const keyLog = new URLSearchParams(location.search).has("keylog")
    ? createKeyLog(input)
    : null;

  let last = SENTINEL;
  let composing = false;

  function reset() {
    input.value = SENTINEL;
    input.setSelectionRange(SENTINEL.length, SENTINEL.length);
    last = SENTINEL;
  }

  function flush() {
    const value = input.value;
    let common = 0;
    while (common < last.length && common < value.length && last[common] === value[common]) {
      common += 1;
    }
    const text =
      "\b".repeat(last.length - common) + value.slice(common).replace(/\r?\n/g, "\r");
    last = value;
    if (text) {
      typeText(text);
    }
  }

  input.addEventListener("focus", () => {
    reset();
    button.setAttribute("aria-pressed", "true");
  });
  input.addEventListener("blur", () => {
    composing = false;
    button.setAttribute("aria-pressed", "false");
  });
  input.addEventListener("compositionstart", () => {
    composing = true;
  });
  input.addEventListener("compositionend", () => {
    composing = false;
    flush();
    reset();
  });
  input.addEventListener("input", (event) => {
    keyLog?.(event);
    flush();
    if (!composing) {
      reset();
    }
  });

  // Phone keyboards may send keydown and keyup back to back, before the
  // emulator runs a frame, and iOS fills in event.code, so these keys must not
  // reach the app's physical-key handlers: a key pressed and released between
  // two frames is never seen by the TVC. Type them through the timed queue
  // instead. Registered before the app's listeners, so it runs first.
  const typedKey = (event) => {
    if (event.isComposing || event.keyCode === 229 || event.ctrlKey || event.metaKey) {
      return "";
    }
    if (event.key === "Enter") {
      return "\r";
    }
    if (event.key === "Backspace") {
      return "\b";
    }
    return event.key.length === 1 ? event.key : "";
  };
  input.addEventListener("keydown", (event) => {
    keyLog?.(event);
    const text = typedKey(event);
    if (text) {
      event.preventDefault();
      event.stopImmediatePropagation();
      typeText(text);
    }
  });
  input.addEventListener("keyup", (event) => {
    keyLog?.(event);
    if (typedKey(event)) {
      event.stopImmediatePropagation();
    }
  });
  for (const type of ["compositionstart", "compositionend", "beforeinput"]) {
    input.addEventListener(type, (event) => keyLog?.(event));
  }

  // Keep focus where it is when the button is pressed; toggle on click, which
  // iOS requires before focus() may open the keyboard.
  button.addEventListener("pointerdown", (event) => event.preventDefault());
  button.addEventListener("click", () => {
    if (document.activeElement === input) {
      input.blur();
    } else {
      input.focus({ preventScroll: true });
    }
  });

  return input;
}

function createKeyLog(input) {
  const panel = document.createElement("pre");
  panel.style.cssText =
    "position:fixed;left:0;top:0;z-index:20;margin:0;padding:4px;max-width:100vw;" +
    "font:11px/1.3 ui-monospace,monospace;color:#0f0;background:#000c;" +
    "pointer-events:none;white-space:pre-wrap";
  document.body.append(panel);
  const lines = [];
  return (event) => {
    const parts = [event.type];
    if ("key" in event) {
      parts.push(`key=${JSON.stringify(event.key)} code=${JSON.stringify(event.code)}`,
        `kc=${event.keyCode}`);
    }
    if ("inputType" in event) {
      parts.push(`it=${event.inputType} data=${JSON.stringify(event.data)}`);
    }
    if (event.isComposing) {
      parts.push("composing");
    }
    parts.push(`val=${JSON.stringify(input.value)}`);
    lines.push(parts.join(" "));
    lines.splice(0, Math.max(0, lines.length - 12));
    panel.textContent = lines.join("\n");
  };
}

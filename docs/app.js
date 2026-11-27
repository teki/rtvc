import init, { WebHandle } from "./rtvc.js?v=6b090e7a";
import { installSoftKeyboard, isSoftKeyEvent } from "./soft-keyboard.js?v=6b090e7a";

const DB_NAME = "rtvc";
const DB_VERSION = 1;
const MEDIA_STORE = "recent-media";
const MAX_RECENT_PER_KIND = 10;
const keyboardEvents = [];
const activeKeyCodes = new Map();
let audioSink;
let audioSampleRate;
let audioInitPromise;

globalThis.rtvcStartupAudioError = null;
globalThis.rtvcStartupStorageError = null;
globalThis.rtvcStartupRecentMedia = [];
globalThis.rtvcGetStartupAudioError = () => globalThis.rtvcStartupAudioError;
globalThis.rtvcGetStartupStorageError = () => globalThis.rtvcStartupStorageError;
globalThis.rtvcGetStartupRecentMedia = () => globalThis.rtvcStartupRecentMedia;
globalThis.rtvcStartupBundledMedia = [];
globalThis.rtvcGetStartupBundledMedia = () => globalThis.rtvcStartupBundledMedia;
globalThis.rtvcStartupLaunch = null;
globalThis.rtvcGetStartupLaunch = () => globalThis.rtvcStartupLaunch;

globalThis.rtvcAudioInit = async (sampleRate) => {
  const AudioContext = globalThis.AudioContext || globalThis.webkitAudioContext;
  if (!AudioContext || !("AudioWorkletNode" in globalThis)) {
    return "This browser does not support AudioWorklet";
  }
  audioSampleRate = sampleRate;
  return null;
};

globalThis.rtvcAudioResume = () => {
  void ensureAudioSink().then((sink) => sink?.resume());
};

globalThis.rtvcAudioPush = (samples) => {
  audioSink?.push(samples);
};

function ensureAudioSink() {
  if (audioSink) {
    return Promise.resolve(audioSink);
  }
  if (!audioSampleRate) {
    return Promise.resolve(null);
  }
  if (!audioInitPromise) {
    audioInitPromise = createAudioSink(audioSampleRate)
      .then((sink) => {
        audioSink = sink;
        return sink;
      })
      .catch((error) => {
        console.warn("AudioWorklet initialization failed", error);
        return null;
      })
      .finally(() => {
        audioInitPromise = null;
      });
  }
  return audioInitPromise;
}

globalThis.rtvcTakeKeyboardEvents = () => keyboardEvents.splice(0);

globalThis.rtvcFetchText = async (url) => {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} while fetching ${url}`);
  }
  return response.text();
};

globalThis.rtvcFetchBytes = async (url) => {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP ${response.status} while fetching ${url}`);
  }
  return new Uint8Array(await response.arrayBuffer());
};

globalThis.rtvcLoadRecentMedia = async () => {
  const db = await openDatabase();
  const records = await requestResult(
    db.transaction(MEDIA_STORE, "readonly").objectStore(MEDIA_STORE).getAll()
  );
  return records
    .sort((a, b) => b.usedAt - a.usedAt)
    .map((record) => ({
      kind: record.kind,
      name: record.name,
      bytes: new Uint8Array(record.bytes),
    }));
};

globalThis.rtvcStoreRecentMedia = async (kind, name, bytes) => {
  const db = await openDatabase();
  const transaction = db.transaction(MEDIA_STORE, "readwrite");
  const store = transaction.objectStore(MEDIA_STORE);
  store.put({
    id: `${kind}:${name}`,
    kind,
    name,
    bytes: Uint8Array.from(bytes).buffer,
    usedAt: Date.now(),
  });
  await transactionDone(transaction);
  if (kind === "hostfile") {
    return;
  }

  const all = await requestResult(
    db.transaction(MEDIA_STORE, "readonly").objectStore(MEDIA_STORE).getAll()
  );
  const stale = all
    .filter((record) => record.kind === kind)
    .sort((a, b) => b.usedAt - a.usedAt)
    .slice(MAX_RECENT_PER_KIND);
  if (stale.length > 0) {
    const cleanup = db.transaction(MEDIA_STORE, "readwrite");
    const cleanupStore = cleanup.objectStore(MEDIA_STORE);
    stale.forEach((record) => cleanupStore.delete(record.id));
    await transactionDone(cleanup);
  }
};

globalThis.rtvcClearRecentMedia = async (kind) => {
  const db = await openDatabase();
  const records = await requestResult(
    db.transaction(MEDIA_STORE, "readonly").objectStore(MEDIA_STORE).getAll()
  );
  const transaction = db.transaction(MEDIA_STORE, "readwrite");
  const store = transaction.objectStore(MEDIA_STORE);
  records
    .filter((record) => record.kind === kind)
    .forEach((record) => store.delete(record.id));
  await transactionDone(transaction);
};

async function createAudioSink(sampleRate) {
  const AudioContext = globalThis.AudioContext || globalThis.webkitAudioContext;
  let context;
  try {
    context = new AudioContext({ sampleRate });
  } catch {
    context = new AudioContext();
  }
  try {
    await context.audioWorklet.addModule("./audio-worklet.js?v=6b090e7a");
  } catch (error) {
    void context.close();
    throw error;
  }
  const node = new AudioWorkletNode(context, "rtvc-audio", {
    numberOfInputs: 0,
    numberOfOutputs: 1,
    outputChannelCount: [1],
  });
  node.connect(context.destination);
  const ratio = context.sampleRate / sampleRate;
  let resamplePhase = 0;

  return {
    push(samples) {
      if (samples.length === 0) {
        return;
      }
      const input = Float32Array.from(samples);
      const output = ratio === 1 ? input : resample(input);
      node.port.postMessage(output, [output.buffer]);
    },
    resume() {
      if (context.state !== "running") {
        void context.resume();
      }
    },
  };

  function resample(samples) {
    const converted = [];
    for (const sample of samples) {
      resamplePhase += ratio;
      while (resamplePhase >= 1) {
        converted.push(sample);
        resamplePhase -= 1;
      }
    }
    return new Float32Array(converted);
  }
}

function openDatabase() {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);
    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(MEDIA_STORE)) {
        db.createObjectStore(MEDIA_STORE, { keyPath: "id" });
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("Unable to open IndexedDB"));
  });
}

function requestResult(request) {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error ?? new Error("IndexedDB request failed"));
  });
}

function transactionDone(transaction) {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () =>
      reject(transaction.error ?? new Error("IndexedDB transaction failed"));
    transaction.onabort = () =>
      reject(transaction.error ?? new Error("IndexedDB transaction was aborted"));
  });
}

function installKeyboard(canvas) {
  canvas.tabIndex = 0;
  // Sits above the egui status bar.
  const softInput = installSoftKeyboard({
    bottom: "28px",
    typeText(text) {
      keyboardEvents.push({ type: "text", text });
      globalThis.rtvcAudioResume();
    },
  });
  canvas.addEventListener("pointerdown", () => {
    if (document.activeElement !== softInput) {
      canvas.focus();
    }
    globalThis.rtvcAudioResume();
  });
  for (const target of [canvas, softInput]) {
    target.addEventListener("keydown", onKeyDown);
    target.addEventListener("keyup", onKeyUp);
  }
  // Moving focus between the canvas and the soft keyboard releases held keys
  // but keeps Developer-mode screen capture, so typing still reaches the TVC.
  const onBlur = (event) => {
    const target = event.relatedTarget;
    if (target === canvas || target === softInput) {
      releaseHeldKeys();
    } else {
      resetKeyboard();
    }
  };
  canvas.addEventListener("blur", onBlur);
  softInput.addEventListener("blur", onBlur);
  window.addEventListener("blur", resetKeyboard);
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      resetKeyboard();
    }
  });
}

function onKeyDown(event) {
  if (isSoftKeyEvent(event)) {
    return;
  }
  const code = legacyKeyCode(event);
  const text =
    !event.ctrlKey && !event.metaKey && event.key.length === 1 ? event.key : "";
  if (event.repeat || activeKeyCodes.has(event.code)) {
    if (code !== 0 || text) {
      event.preventDefault();
    }
    return;
  }
  if (code !== 0) {
    if (code === 225) {
      keyboardEvents.push({ type: "up", code: 17 });
    }
    activeKeyCodes.set(event.code, code);
    keyboardEvents.push({ type: "down", code, text });
    globalThis.rtvcAudioResume();
    event.preventDefault();
  } else if (text) {
    keyboardEvents.push({ type: "text", text });
    event.preventDefault();
  }
}

function onKeyUp(event) {
  if (isSoftKeyEvent(event)) {
    return;
  }
  const code = activeKeyCodes.get(event.code) ?? legacyKeyCode(event);
  activeKeyCodes.delete(event.code);
  if (code !== 0) {
    keyboardEvents.push({ type: "up", code });
    event.preventDefault();
  }
}

function releaseHeldKeys() {
  for (const code of activeKeyCodes.values()) {
    keyboardEvents.push({ type: "up", code });
  }
  activeKeyCodes.clear();
}

function resetKeyboard() {
  activeKeyCodes.clear();
  keyboardEvents.push({ type: "blur" });
}

function legacyKeyCode(event) {
  if (event.code === "AltRight" && event.getModifierState("AltGraph")) {
    return 225;
  }
  if (event.code.startsWith("Key") && event.code.length === 4) {
    return event.code.charCodeAt(3);
  }
  if (event.code.startsWith("Digit") && event.code.length === 6) {
    return event.code.charCodeAt(5);
  }
  return {
    Backspace: 8,
    Tab: 9,
    Enter: 13,
    NumpadEnter: 13,
    ShiftLeft: 16,
    ShiftRight: 16,
    ControlLeft: 17,
    ControlRight: 17,
    AltLeft: 18,
    AltRight: 18,
    CapsLock: 20,
    Escape: 27,
    Space: 32,
    PageUp: 33,
    PageDown: 34,
    End: 35,
    Home: 36,
    ArrowLeft: 37,
    ArrowUp: 38,
    ArrowRight: 39,
    ArrowDown: 40,
    Insert: 45,
    Delete: 46,
    F9: 120,
    Semicolon: 186,
    Equal: 187,
    Comma: 188,
    Minus: 189,
    Period: 190,
    Slash: 191,
    Backquote: 192,
    BracketLeft: 219,
    Backslash: 220,
    BracketRight: 221,
    Quote: 222,
  }[event.code] ?? 0;
}

async function loadBundledMedia() {
  const response = await fetch("./bundled/index.json", { cache: "no-cache" });
  if (!response.ok) {
    return [];
  }
  const names = await response.json();
  const files = await Promise.all(
    names.map(async (name) => {
      try {
        const bytes = await globalThis.rtvcFetchBytes(
          `./bundled/${encodeURIComponent(name)}`
        );
        return { name, bytes };
      } catch (error) {
        console.warn(`Failed to load bundled media ${name}`, error);
        return null;
      }
    })
  );
  return files.filter((file) => file !== null);
}

// `?run=<file>` opens a game at startup as if it were dropped on the page.
// <file> is a bundled media name, a path relative to this page, or a URL
// (other origins must allow CORS).
async function loadStartupLaunch(bundledMedia) {
  const target = new URLSearchParams(location.search).get("run");
  if (!target) {
    return null;
  }
  const bundled = (await bundledMedia).find((file) => file.name === target);
  if (bundled) {
    return bundled;
  }
  try {
    const url = new URL(target, location.href);
    const name = decodeURIComponent(url.pathname.split("/").pop() || "") || target;
    return { name, bytes: await globalThis.rtvcFetchBytes(url.href) };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { error: `Could not open ${target}: ${message}` };
  }
}

async function run() {
  const bundledMedia = loadBundledMedia().catch((error) => {
    console.warn("Failed to load bundled media", error);
    return [];
  });
  const startupLaunch = loadStartupLaunch(bundledMedia);
  await init({ module_or_path: "./rtvc_bg.wasm?v=6b090e7a" });
  const canvas = document.getElementById("tvc_canvas");
  globalThis.rtvcStartupAudioError = await globalThis.rtvcAudioInit(44100);
  try {
    globalThis.rtvcStartupRecentMedia = await globalThis.rtvcLoadRecentMedia();
    globalThis.rtvcStartupStorageError = null;
  } catch (error) {
    globalThis.rtvcStartupRecentMedia = [];
    globalThis.rtvcStartupStorageError =
      error instanceof Error ? error.message : String(error);
  }
  globalThis.rtvcStartupBundledMedia = await bundledMedia;
  globalThis.rtvcStartupLaunch = await startupLaunch;
  installKeyboard(canvas);
  const handle = new WebHandle();
  globalThis.rtvcHandle = handle;
  handle.start("tvc_canvas");
}

run().catch((err) => {
  console.error("Failed to start rtvc: ", err);
});

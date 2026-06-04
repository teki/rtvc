import init, { WasmTvc } from "./rtvc.js";

const canvas = document.getElementById("screen");
const status = document.getElementById("status");

const wasm = await init();
const emu = new WasmTvc(true);
let snapshot;
try {
  snapshot = await loadSnapshot();
} catch (error) {
  status.textContent = `${error.message}. Copy snapshot.rtvcsnap.zip beside index.html and reload.`;
  throw error;
}
emu.loadSnapshot(snapshot);

const width = emu.screenWidth();
const height = emu.screenHeight();
canvas.width = width;
canvas.height = height;
canvas.focus();

const ctx = canvas.getContext("2d", { alpha: false });
const image = ctx.createImageData(width, height);
const audio = await createAudioSink(emu.audioSampleRate());

async function loadSnapshot() {
  let response = await fetch("./snapshot.rtvcsnap.zip");
  if (response.ok) {
    return unzipSnapshot(new Uint8Array(await response.arrayBuffer()));
  }

  response = await fetch("./snapshot.rtvcsnap");
  if (!response.ok) {
    throw new Error(`Failed to load snapshot: ${response.status}`);
  }
  return new Uint8Array(await response.arrayBuffer());
}

async function unzipSnapshot(zipBytes) {
  const entry = findFirstZipEntry(zipBytes);
  if (entry.method === 0) {
    return zipBytes.slice(entry.dataStart, entry.dataEnd);
  }
  if (entry.method !== 8) {
    throw new Error(`Unsupported zip compression method: ${entry.method}`);
  }
  if (!("DecompressionStream" in globalThis)) {
    throw new Error("This browser cannot decompress zipped snapshots");
  }

  const stream = new Blob([zipBytes.slice(entry.dataStart, entry.dataEnd)])
    .stream()
    .pipeThrough(new DecompressionStream("deflate-raw"));
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

function findFirstZipEntry(bytes) {
  let pos = 0;
  while (pos + 30 <= bytes.length) {
    if (u32(bytes, pos) !== 0x04034b50) {
      break;
    }
    const method = u16(bytes, pos + 8);
    const compressedSize = u32(bytes, pos + 18);
    const fileNameLen = u16(bytes, pos + 26);
    const extraLen = u16(bytes, pos + 28);
    const nameStart = pos + 30;
    const nameEnd = nameStart + fileNameLen;
    const dataStart = nameEnd + extraLen;
    const dataEnd = dataStart + compressedSize;
    const name = new TextDecoder().decode(bytes.slice(nameStart, nameEnd)).toLowerCase();
    if (name.endsWith(".rtvcsnap")) {
      return { method, dataStart, dataEnd };
    }
    pos = dataEnd;
  }
  throw new Error("Snapshot zip does not contain a .rtvcsnap entry");
}

function u16(bytes, offset) {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

function u32(bytes, offset) {
  return (
    bytes[offset] |
    (bytes[offset + 1] << 8) |
    (bytes[offset + 2] << 16) |
    (bytes[offset + 3] << 24)
  ) >>> 0;
}

function draw() {
  const ptr = emu.framebufferPtr();
  const len = emu.framebufferLen();
  image.data.set(new Uint8ClampedArray(wasm.memory.buffer, ptr, len));
  ctx.putImageData(image, 0, 0);
}

async function createAudioSink(sampleRate) {
  const AudioContext = globalThis.AudioContext || globalThis.webkitAudioContext;
  if (!AudioContext || !("AudioWorkletNode" in globalThis)) {
    return { push() {}, resume() {} };
  }

  try {
    let context;
    try {
      context = new AudioContext({ sampleRate });
    } catch {
      context = new AudioContext();
    }
    if (!context.audioWorklet) {
      throw new Error("audioWorklet is undefined");
    }
    await context.audioWorklet.addModule("./audio-worklet.js");
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
        if (samples.length > 0) {
          const output = ratio === 1 ? samples : resample(samples, ratio);
          node.port.postMessage(output, [output.buffer]);
        }
      },
      resume() {
        if (context.state !== "running") {
          context.resume();
        }
      },
    };
  } catch (error) {
    console.warn("Audio unavailable", error);
    return { push() {}, resume() {} };
  }

  function resample(samples, ratio) {
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

function keyCode(event) {
  if (event.key && event.key.length === 1) {
    return event.key.toUpperCase().charCodeAt(0);
  }
  return {
    Backspace: 8,
    Tab: 9,
    Enter: 13,
    Shift: 16,
    Control: 17,
    Alt: 18,
    Escape: 27,
    " ": 32,
    ArrowLeft: 37,
    ArrowUp: 38,
    ArrowRight: 39,
    ArrowDown: 40,
    Delete: 46,
  }[event.key] ?? 0;
}

canvas.addEventListener("keydown", (event) => {
  const code = keyCode(event);
  if (code !== 0) {
    event.preventDefault();
    audio.resume();
    emu.keyDown(code);
  }
});

canvas.addEventListener("pointerdown", () => {
  audio.resume();
  canvas.focus();
});

canvas.addEventListener("keyup", (event) => {
  const code = keyCode(event);
  if (code !== 0) {
    event.preventDefault();
    emu.keyUp(code);
  }
});

canvas.addEventListener("input", (event) => {
  if (event.data) {
    emu.keyPressText(event.data);
  }
});

function frame() {
  emu.runFrame();
  audio.push(emu.takeAudioSamples());
  if (emu.takeFrameComplete()) {
    draw();
  }
  requestAnimationFrame(frame);
}

draw();
requestAnimationFrame(frame);

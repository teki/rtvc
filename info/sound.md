# TVC Sound Generator

This document describes the TVC sound model implemented in [src/sound.rs](../src/sound.rs) and wired through [src/tvc.rs](../src/tvc.rs).

## Hardware Model

The TVC has a single programmable sound/timer channel derived from the 3.125 MHz system clock. The CPU writes a 12-bit divisor through ports `0x04` and `0x05`; hardware then combines a programmable divider with a fixed counter stage.

The TVC hardware documentation gives the audible oscillator frequency as:

```text
195312.5 / (4096 - n) Hz
```

where `n` is the 12-bit value written to the frequency register. This is equivalent to `3_125_000 / 16 / (4096 - n)`. A divisor of `0xFFF` stops the oscillator.

Port `0x05` controls the channel:

- Bits 0-3: high nibble of the 12-bit frequency divisor.
- Bit 4: route the timer-derived oscillator to the amplitude control.
- Bit 5: enable the sound timer interrupt.
- Bits 6-7: cassette motor control outputs.

Port `0x06` is shared with video mode control. Bits 2-5 are the 4-bit sound amplitude register, giving 16 output levels.

When port `0x05` bit 4 is set, the programmable-divider carry clocks the D10 sound counter stage. The emulator models this as a `4096 - n` cycle programmable countdown followed by a 4-bit counter. The audible signal is bit 3 of that counter, so it is low for eight divider carries and high for eight divider carries, giving frequency `195312.5 / (4096 - n)` Hz. Writing a valid divisor starts the counter chain; divisor `0xFFF` stops it. Reading port `0x5B` or `0x5F` clears the D10 counter and reloads the programmable divider for phase-accurate timing, so the next generated square wave starts low and rises halfway through the first full sound period. When port `0x05` bit 4 is clear, the amplitude register is emitted directly as a 4-bit DAC level with zero amplitude producing silence; software can create stepped waveforms by changing port `0x06` over time.

The hardware amplitude control is a NOR-gate/resistor ladder that produces a unipolar `SOUND` voltage. The TV/UHF modulator path then AC-couples that signal through a capacitor, so static DAC levels do not produce continuous speaker deflection. The emulator applies a small high-pass/DC-blocking filter to the generated PCM to model that analog coupling.

Reading port `0x5B` or `0x5F` restarts the sound oscillator/timer counter from the programmed divisor. If sound interrupt generation is enabled, the timer requests the shared cursor/sound interrupt once per full divided sound period. This is the cadence the cassette ROM uses by programming divisor `0x100`, which produces a roughly 20 ms timer.

## Emulator Output

The core renders mono `f32` PCM samples at 44.1 kHz while CPU cycles advance. Samples are produced inside `SoundTimer::advance`, so port changes between CPU instructions affect the generated waveform timing.

The public core API is:

- `Tvc::sound_sample_rate()` returns the PCM sample rate.
- `Tvc::take_audio_samples()` drains pending mono samples.
- `WasmTvc::audioSampleRate()` and `WasmTvc::takeAudioSamples()` expose the same data to browser code.

The native egui frontend drains samples after each emulated frame and feeds them to [src/audio.rs](../src/audio.rs), a small `cpal` output sink. The sink opens the default host output device, prefers the TVC's 44.1 kHz stream rate, duplicates mono samples to all host channels, and keeps a bounded one-second queue. If a host device only accepts another sample rate, the sink does a lightweight queue-side resample.

The web bundles drain samples through JavaScript and feed an `AudioWorklet` processor generated as `audio-worklet.js`. The full web app initializes the audio context and worklet from a user gesture. It requests a 44.1 kHz browser audio context and falls back to the browser default with a lightweight JavaScript resample when needed. The browser queue is bounded to one second so a suspended audio context cannot grow memory indefinitely.

## Logging

The TVC I/O log records oscillator path changes from CPU writes to ports `0x04` and `0x05`:

- `sound on`: the oscillator route became audible, including the current frequency and divisor.
- `sound off`: the oscillator route was disabled or stopped by divisor `0xFFF`.
- `sound freq`: the programmed oscillator divisor changed while the oscillator route stayed audible.
- `sound volume`: the 4-bit amplitude level changed from a CPU write to port `0x06`.

The `sound on` and `sound freq` entries include the port write that caused the log entry. TVC ROM routines commonly write the high/control byte at port `0x05` before the low byte at port `0x04`, so an immediate `port 0x05` entry can show an intermediate divisor before the following `port 0x04` entry settles the complete PITCH value.

Cursor/video interrupt timing is intentionally not logged.

## Snapshot State

Snapshots store the sound frequency register, control register, programmable divider countdown, running flag, amplitude register, D10 sound counter state, fractional sample scheduler state, and analog output filter state. Pending PCM samples are not serialized; loading a snapshot resumes deterministic generator state and starts with an empty frontend sample buffer.

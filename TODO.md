# TODO

Open issues and planned improvements for the TVC emulator.

## Gamebase

- [ ] Improve Gamebase game loading and user guidance.
  - Prepare the machine for the selected media type before loading the game.
  - CAS injection does not require VT-DOS, but the machine must be in a state
    where direct cassette injection can succeed.
  - DSK games require a VT-DOS machine configuration. The machine currently
    needs a full reset and must finish booting before the disk can be used,
    which takes a long time.
  - Avoid resetting and waiting for a full VT-DOS boot when loading a CAS game.
  - Show a clear confirmation after the media has loaded.
  - Tell the user exactly how to start the loaded game, including any commands,
    keys, reset, or boot steps required for that media and title.
  - Report when the machine is still preparing or booting instead of presenting
    the game as immediately ready.

## Machine Startup

- [x] Complete the optional fast-boot setting.
  - [x] Skip the RAM test in the known TVC 1.2 and 2.2 system ROMs.
  - [x] Skip drawing the TVC 1.2 boot screen.
  - [x] Skip drawing the TVC 2.2 boot screen.
  - Alternatively, restore a prepared boot snapshot. This would require a
    separate snapshot for every machine type, which is less elegant and adds a
    maintenance burden.

## Developer Workspace

- [ ] Add persistent BASIC and assembly editor panes after the debugger
  interfaces have stabilized.
  - Keep the default user experience as a simple emulator screen.
  - Reuse the existing dock workspace and persistence model.
  - Support both native and full-web builds without adding the egui workspace
    to lightweight WASM builds.
  - Ensure open editor panes do not prevent real-time 50 Hz emulation.
  - Start with keyboard-based BASIC transfer into the emulated machine.
  - Start with sequential assembly into mapped writable memory.

## Debugger

- [ ] Improve breakpoint management.
  - Allow individual breakpoints to be enabled and disabled without deleting
    them.
  - Persist breakpoints between sessions.
  - Allow breakpoints to have user-defined labels.
  - Show active breakpoints as red dots in the disassembly view.
  - Make the breakpoint dot beside each disassembled instruction toggleable so
    breakpoints can be added and removed directly from disassembly.

- [ ] Add layered visual debugging overlays to the TVC screen.
  - Render overlays above the TVC framebuffer with configurable translucency,
    initially around 0.5 alpha.
  - Make each information layer independently toggleable.
  - Clear transient overlay state at the start of every new TVC frame.
  - Begin with scanline granularity.
  - Highlight the current CRTC scanline in blue.
  - Highlight scanlines traversed while the CPU is halted in red.
  - Add a memory-write layer that highlights written video-memory locations in
    white and the location at which the write occurred in green.
  - Initially approximate memory-write visualization at scanline granularity,
    then increase its spatial and timing precision later.

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

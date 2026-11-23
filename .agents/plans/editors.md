# Integrated Editors (IDE A: and IDE B:)

Backlog entries: [TODO.md — Developer Workspace](../../TODO.md#developer-workspace)
(BASIC and assembly editor panes) and
[Storage And Expansion](../../TODO.md#storage-and-expansion) (custom media
directory).

Status: **design revised 2026-11-23** (see [Decisions](#decisions)). The
project/cart model of the first draft is dropped; the file system is the unit
of work.

## Goal

Give rtvc a simple, PICO-8-like way to write and run TVC programs: open a
file, edit it, press one key, see it run on the real (emulated) machine. The
UI stays small and direct, not a traditional IDE. Users who only play games
never see it.

What we take from PICO-8 and TIC-80:

- One key to run, one key to come back, no build configuration.
- Few, simple tools: a code editor, later an image editor and a sound editor,
  each sized to the machine's real limits.
- Files you can load and save from the machine itself.

Where we differ:

- No project or cart file. A **drive folder** (A: or B:) is the workspace,
  and the TVC sees the same files through the host file card.
- No built-in shell: the TVC's BASIC prompt is the machine, but the IDE is
  egui panes beside it, not drawn inside the TVC screen.
- Run uses the real machine: tokenize or assemble, load into RAM, `RUN`.

## Scope

Round one:

1. **One rtvc root folder** for all user files, the same layout on native and
   web, with `A/` and `B/` drive folders and Gamebase downloads under it.
2. **Host file card with two drives**: A: and B: backed by folders, selected
   from the Disk menu.
3. **File manager**: a two-panel, Norton Commander-style manager over the
   root, which opens DSK images as folders. Essential on web, useful natively.
4. **DSK library**: create DSK images, copy files in and out (shared by the
   file manager and `rtvc-dsk`).
5. **Dev boot snapshot**: an embedded 64K+ BASIC 1.2 machine with the host
   file card, the starting point of every Run.
6. **IDE A: and IDE B: panes**: a tool row with a file selector, and a plain
   text editor for `.bas` and `.asm` with Run and Build.

Later rounds, in order:

1. Image editor for `.gfx` (with the assembler's `INCGFX`).
2. Sound editor.
3. Source-level debugger (breakpoints in the gutter, current line).
4. C80 as a third code language.
5. A richer code editor component, after the web bundle size is reduced
   (see [Text editor](#text-editor)).

Not planned: projects or a single cart file, a map editor, a built-in shell,
lightweight WASM support, folder-backed VT-DOS disks (see
[Decisions](#decisions)).

## UI/UX

### Overview

```
+-----------------------------+-----------------------------+
| IDE A:                      | Screen                      |
| [game.asm v] New Save | Run |                             |
|-----------------------------|   (TVC 4:3)                 |
|   1  ORG 1B00H              |                             |
|   2  START: LD A,1          |                             |
|   3  ...                    +-----------------------------+
|                             | IDE B:                      |
|                             | [title.gfx v] New Save      |
|-----------------------------|   (image editor, later)     |
| game.asm:40 unknown symbol X|                             |
+-----------------------------+-----------------------------+
```

- **IDE A:** and **IDE B:** are dock panes of one kind,
  `WorkspaceTab::Ide(Drive)` with `Drive::A`/`Drive::B`, one per host file
  drive; all pane code is shared and only the drive differs. Each opens files from its
  drive's current folder. They can be closed, docked, and moved like the
  debugger panes. View > **IDE Layout** (next to Debugger Layout) arranges IDE
  A:, Screen, and IDE B:.
- Each pane shows **one file at a time**. The file selector switches files;
  there are no tabs. Two panes give two files side by side, which is enough
  for a program and its include or its graphics.
- Keys go to the pane with focus, as today (`accepts_machine_input`): the TVC
  gets keys only while the Screen has focus. A thin outline marks the active
  pane.

### Tool row

The top line of each pane:

```
[game.asm v]  New  Save  |  Run  Build  |            12:4  ASM  A:/games/x
```

- **File selector** (first): a dropdown listing the drive folder's files,
  with subfolders shown as entries that open them, plus `..`. The selected
  file decides the pane's editor:

  | Extension | Editor |
  | --- | --- |
  | `.bas` | text editor, BASIC |
  | `.asm`, `.inc` | text editor, ASM |
  | `.txt`, `.toml`, other text | text editor, plain |
  | `.gfx` | image editor (later; text editor until then) |
  | `.sfx` | sound editor (later; text editor until then) |
  | `.cas`, `.dsk`, binaries | info view: size, CAS header, BASIC listing when detokenizable, "Open in File Manager" for DSK |

- **New**: asks for a name; the extension picks the editor. **Save**
  (`Ctrl+S`), with `*` in the selector for unsaved changes.
- The rest of the row depends on the editor. The text editor adds **Run** and
  **Build**; the image editor will add its tools (pen, fill, mode) to the same
  row, as PICO-8's editors do.
- Right side: cursor position, language, and the drive and folder.
- Switching files with unsaved changes asks to save or discard.
- Both panes may show the same file; they share one buffer.

### Text editor

- egui's `TextEdit` with a custom highlighting layouter, a normal monospace
  font, line numbers, and syntax colours (BASIC keywords and line numbers;
  Z80 mnemonics, registers, directives, labels, comments).
- Undo/redo, find (`Ctrl+F`), go to line (`Ctrl+G`), comment toggle, tab
  indent. No completion, no multi-cursor.
- An error line under the editor (`game.asm:40: unknown symbol 'X'`); clicking
  it moves the cursor there, and the gutter marks the line.
- BASIC: **Renumber** (in the row's `...` menu) renumbers and fixes `GOTO` and
  `GOSUB` targets. Hungarian letters are typed normally and mapped to the TVC
  character set on Run; characters the TVC cannot show are errors.
- No egui upgrade for this. The editor crates (`egui_code_editor` and others)
  need egui 0.34 or later, and the web bundle size must be reduced before
  rtvc upgrades egui.

### Run and Build

| Action | Key | BASIC | ASM |
| --- | --- | --- | --- |
| **Run** | `Ctrl+R` (web: `Ctrl+Enter`) | restore dev boot snapshot, tokenize, inject, type `RUN` | restore dev boot snapshot, assemble, inject as CAS with its copy stub, type `RUN` |
| **Run without reset** | `Ctrl+Shift+R` | as Run, into the current machine | as Run, into the current machine |
| **Build** | `Ctrl+B` | write `<stem>.cas` beside the source | write `<stem>.cas` and `<stem>.toml` (symbols) beside the source |

- Run always starts from the [dev boot snapshot](#dev-boot-snapshot): a known
  64K+ BASIC 1.2 machine at the BASIC prompt, with the host file card mounted
  on the current A: and B: folders. This is PICO-8's "clean run": instant,
  deterministic, and safe when a program is still running.
- Run switches the machine to the dev setup, as a Gamebase launch does.
  **Run without reset** keeps the current machine (for BASIC 2.2, VT-DOS, or
  state such as POKEs), and refuses with "machine busy" if the machine is not
  at the BASIC prompt after an ESC break.
- Errors never touch the machine; the error line shows them.
- Run gives the Screen keyboard focus. Clicking the pane or pressing
  ``Ctrl+` `` returns to the editor with the cursor where it was.
- Because Build writes into the drive folder, `LOAD "NAME"` at the TVC prompt
  loads the build, and `SAVE` from the TVC appears in the file selector.

### Simple mode

Simple mode keeps only the screen. ``Ctrl+` `` (physical key left of `1`)
flips the whole window between the screen and **IDE A:** full window, PICO-8
style; a toolbar button does the same for touch and mouse users. The pane's
file selector reaches B: through `..` and the drive switch in the selector.
Esc stays the TVC ESC key.

### Disk menu with the host file card

When the host file card is fitted, the Disk menu's A: and B: entries select
**folders** instead of DSK images:

```
Disk
  A: games/laser          >   Mount folder...    (folder picker under the root)
                              Recent folders
                              Reset to A
                              Open in File Manager
  B: B                    >   ...
  Default drive: A:
  ---
  VT-DOS disks            >   (image entries, only when an HBF card is also fitted)
```

- The default mounts are `<root>/A` and `<root>/B`. **Mount folder** picks any
  folder under the root; this is how the user changes the drive's folder for
  file management (for example, one folder per program). The mounts persist.
- With an HBF card fitted as well, its image entries stay in a VT-DOS
  submenu. The status bar shows which device the TVC's `LOAD` uses (the higher
  slot claims channel 5).
- Without the host file card, the Disk menu is unchanged.

### File manager

A two-panel manager, opened from File > **File Manager** (`F2` outside the
TVC screen) as a dock pane or, in Simple mode, a full-window view:

```
+- <root>/A ---------------------+- <root>/gamebase/laser ---------+
| ..                             | ..                              |
| game.asm          3 210  11-23 | LASER.DSK     737 280  11-20    |
| game.cas          1 024  11-23 |                                 |
| title.gfx           890  11-22 |                                 |
+--------------------------------+---------------------------------+
 F3 View  F4 Edit  F5 Copy  F6 Move  F7 Mkdir  F8 Delete  Tab  New DSK
```

- Norton Commander keys: Tab switches panels, Enter opens a folder, Backspace
  goes up, Insert or Space marks files, F3 views, F4 opens the file in IDE A:,
  F5 copies to the other panel, F6 moves or renames, F7 makes a folder, F8
  deletes. Every key has a button in the bottom row for mouse and touch.
- **DSK images open like folders** (as archives do in Norton Commander).
  F5 copies files into or out of a DSK; names are converted to 8.3 upper case
  and conflicts ask. **New DSK** creates an empty 720K VT-DOS disk.
- Both panels start at the root on web; native starts at `<root>/A` and the
  other panel at the root.
- Web: **Upload** (and drag and drop onto the panel) imports files from the
  computer, **Download** saves the marked files (a zip when several), and the
  info line shows the browser storage used and whether it is persistent.
- Delete and overwrite always confirm. Deleting a mounted folder or DSK is
  refused until it is unmounted.

### Web differences

- The same UI in the full web build; the root lives in IndexedDB.
- `Ctrl+R` and `Ctrl+W` belong to the browser, so the web UI shows
  `Ctrl+Enter` for Run. `Ctrl+S`, `Ctrl+B`, `Ctrl+F`, `Ctrl+G`, and
  ``Ctrl+` `` are intercepted while an IDE pane has focus.
- Browser data can be cleared: the file manager's Download is the backup, and
  rtvc requests persistent storage on the first save.

## Storage

### One root folder

```
<root>/
  A/          default folder of host file drive A:
  B/          default folder of host file drive B:
  gamebase/   Gamebase downloads and saves (<game>/...)
  ...         anything else: subfolders, DSK and CAS files
```

- **Native**: `root_dir` in `rtvc.toml`, chosen in Options (a folder picker),
  default `~/Documents/rtvc`. It replaces `host_file_dir` and the fixed
  `rtvc-media/` beside the config. On first start with the new version, rtvc
  offers once to move `rtvc-media/` into `<root>/gamebase/` and keeps reading
  an existing `host_file_dir` as the A: mount until the user changes it.
- **Web**: one IndexedDB store keyed by path (`A/GAME.CAS`). The existing
  `hostfile` records move to `A/` and cached `tape`/`disk` media to
  `gamebase/`, once, on upgrade.
- `progs/` (bundled media) stays read-only and outside the root; the Tape and
  Disk menus keep listing it.
- Recent lists keep paths; entries under the root are stored relative to it.
- Snapshots stay where they are in round one.

### Folder model

A small path-based store shared by everything above:

```rust
pub trait FileStore {
    fn list(&self, dir: &str) -> io::Result<Vec<Entry>>;   // name, is_dir, size, modified
    fn read(&self, path: &str) -> io::Result<Vec<u8>>;
    fn write(&self, path: &str, bytes: &[u8]) -> io::Result<()>;
    fn delete(&self, path: &str) -> io::Result<()>;          // file or empty folder
    fn rename(&self, from: &str, to: &str) -> io::Result<()>;
    fn mkdir(&self, path: &str) -> io::Result<()>;
}
```

- Paths are `/`-separated and relative to the root; `..` and absolute paths
  are rejected, so nothing can escape the root.
- Native: `RootDir` over `std::fs`. Web: `MemoryRoot`, the whole root loaded
  into memory at start (TVC files are small), every change mirrored to
  IndexedDB through the pending-write queue the host file card uses today.
- The host file card gets a `FileStore` view per drive (root plus mount
  path), replacing `HostFileStore`, `DirStore`, and `MemoryStore`.

## Technical implementation

### Module layout

| Module | Builds | Responsibility |
| --- | --- | --- |
| `src/files/` (new) | all except lightweight WASM | `FileStore`, `RootDir`, `MemoryRoot`, path rules, migration |
| `src/emulator/dsk.rs` (new) | all | FAT12 DSK create, list, read, write, delete, moved from [rtvc_dsk.rs](../../src/bin/rtvc_dsk.rs) and [emu.rs](../../src/emulator/emu.rs) |
| [host_file.rs](../../src/emulator/host_file.rs) | all | two drives, `A:`/`B:` name prefixes, default drive |
| `src/studio/` (new) | all except lightweight WASM | build to CAS (`convert_source` moved from [rtvc_tocas.rs](../../src/bin/rtvc_tocas.rs)), Run plan, renumber, BASIC character table use |
| `src/ui/ide/` (new) | `native`, `wasm-full` | IDE pane: tool row, file selector, text editor, info view, error line |
| `src/ui/file_manager.rs` (new) | `native`, `wasm-full` | two-panel manager over `FileStore` and DSK images |
| [workspace.rs](../../src/ui/workspace.rs), [ui.rs](../../src/ui/ui.rs) | as today | `Ide(Drive)` and `Files` tabs, IDE Layout, Simple flip, Disk menu folders, shortcuts |
| [xtask](../../xtask/src/main.rs) | tool | path-keyed IndexedDB store, migration, persist request, dev snapshot task |

`fatfs` is already a non-optional dependency, so DSK code builds for every
target.

### Host file card with two drives

- The card holds two drive views and a default drive (A: unless
  `host_file_default` says otherwise).
- Name handling in `host_name()`: a leading `A:` or `B:` (either case) selects
  the drive and is removed before the existing mapping; names without a
  prefix use the default drive. Names stay single-component inside the drive
  folder, as today.
- The ROM ([RTVC-HOSTFILE.asm](../../roms/RTVC-HOSTFILE.asm)) is unchanged if
  the system ROM passes the full name, colon included, to the `FILE` device.
  **Verify first**: `SAVE "B:TEST"` and `LOAD "B:TEST"` on BASIC 1.2 and 2.2
  with a trace of the bytes the card receives. If the OS strips or rejects
  the prefix, add drive selection to the card ROM (for example a `DRIVE`
  command or a port the ROM sets), and record what the OS does in
  [info/tvc.md](../../info/tvc.md).
- Snapshots store the mount paths and default drive with the card state. On
  restore, the card is re-attached to the current mounts, not the saved ones,
  so a snapshot never points at a folder that no longer exists.
- Limitation, stated in the Disk menu help: the drives serve `LOAD`, `SAVE`,
  and `OPEN` through channel 5. VT-DOS commands (`DIR` and others) and
  programs that call VT-DOS directly need an HBF card with a DSK image.

### DSK library

- `src/emulator/dsk.rs`: `DskImage::new_720k()`, `open(bytes)`,
  `list()`, `read(name)`, `write(name, bytes)`, `delete(name)`,
  `into_bytes()`, on top of `fatfs` as `rtvc-dsk` does now. 8.3 name mapping
  and its errors live here.
- `rtvc-dsk` and the Disk menu's directory listing switch to it.
- The file manager edits a DSK in memory and writes the whole image back on
  each change. A DSK mounted in an HBF drive is read-only in the file manager
  (VT-DOS caches the directory); the manager offers to eject it.
- VT-DOS subfolder support is unverified; round one treats DSK images as flat
  and shows any subfolder read-only.

### Dev boot snapshot

- `snapshots/dev12hostfile.rtvcsnap.zip`: machine `64k+tvc12+ex0:hostfile`,
  booted to the BASIC 1.2 prompt, embedded with `include_bytes!` like
  `GAMEBASE_BOOT_SNAPSHOT`.
- `cargo xtask dev-snapshot` regenerates it: boot headless with warp for a
  fixed number of frames, check the BASIC prompt on screen, save.
- A test compares the card ROM inside the snapshot with the current
  `RTVC-HOSTFILE` build and fails with "run `cargo xtask dev-snapshot`" when
  they differ.
- Run restores it with the same path the Gamebase launch uses, then attaches
  the host file card to the current A:/B: mounts.

### Run pipeline

1. **Build** the pane's file to CAS bytes:
   - BASIC: `tokenize_program` → `encode_tvc_cas(payload, BASIC, 0x00, 0)`.
   - ASM: `assemble_program_with(source, 0, name, &drive_files)` →
     `tvc_cas_payload(program, entry)` → `encode_tvc_cas`. `drive_files`
     resolves `INCLUDE`/`INCBIN` (later `INCGFX`) in the file's folder.
2. **Reset** (Run only): restore the dev boot snapshot.
3. **Inject** with `Emu::inject_tape_bytes`, **type** `RUN` and Return with
   `Emu::type_text`, and focus the Screen.

Build writes the same bytes to `<stem>.cas` beside the source; ASM also
writes `<stem>.toml` (`rtvc-asm-v1`, symbols and source lines for the later
debugger). Round-one limits are reported as errors: one contiguous ASM
segment, `TVC_CAS_MAX_PAYLOAD`, and only the assembler's first error.

### BASIC language work

- A TVC character table for Hungarian letters in
  [basic.rs](../../src/emulator/basic.rs), used by tokenize and detokenize;
  verify against the ROM character generator and document it in
  [info/tvc.md](../../info/tvc.md).
- `renumber(source, start, step)` on tokens, updating `GOTO`, `GOSUB`,
  `THEN`/`ELSE` targets, `RESTORE`, and `ON ... GOTO/GOSUB` lists; round-trip
  tests.
- Highlighting uses the keyword table (`keyword_for_token`).

### Text editor

- `TextEdit::multiline` with a layouter that highlights per line, cached by
  line hash; a gutter column for line numbers and error marks (later
  breakpoints).
- Find, go to line, comment toggle, and indent work on the text and the
  `TextEditState` cursor.
- Measure early with a large file (the BASIC ROM listing). If `TextEdit` is
  too slow, limit the highlighter first; a custom widget is a last resort.
- ASM highlighting uses name lists exported from
  [asm.rs](../../src/emulator/asm.rs).

### Web storage

In the [xtask web template](../../xtask/src/main.rs):

- `DB_VERSION` 2 adds a `files` store `{ path, bytes, modified }`; the
  upgrade moves `hostfile` records to `A/<name>` and `tape`/`disk` records to
  `gamebase/<name>`, then keeps `recent-media` for recent paths only.
- Functions: `rtvcFilesLoadAll()`, `rtvcFilesPut(path, bytes)`,
  `rtvcFilesDelete(path)`, `rtvcStoragePersist()`, `rtvcStorageEstimate()`,
  and upload/download helpers for the file manager.
- Writes are batched per frame, as host file writes are today.

### Persistence

- `rtvc.toml`: `root_dir`, `host_file_mounts = ["A", "B"]`, recent folders,
  and each IDE pane's folder and file. Cursor positions are kept for the
  session only.
- The dock layout gains `Ide(Drive)` (serialized as `{"ide": "a"}`) and
  `Files`; older builds fall back to
  the default layout when they see them.

### Implementation order

Each step is usable on its own and keeps native and `wasm-full` building.

1. **Spikes**: does the TVC OS pass `B:NAME` to the `FILE` device; `TextEdit`
   speed on a large file; eframe text input in a dock pane on web.
2. **Root and `FileStore`**: native `root_dir`, web `files` store and
   migration, host file card on `FileStore`.
3. **Two-drive host file card** and the Disk menu folder entries.
4. **DSK library** moved out of `rtvc-dsk`, with tests.
5. **File manager**, native and web, including DSK browsing, upload and
   download.
6. **Dev boot snapshot** and its xtask task and test.
7. **IDE panes** with the text editor, Run, Build, and the Simple-mode flip.
8. **BASIC extras**: character table and Renumber.
9. Later rounds: `INCGFX` and the image editor, sound editor, debugger, C80.

### Validation

- Unit tests: path rules (no escape from the root), `RootDir` and
  `MemoryRoot` behaving the same, drive prefixes, DSK round trips against
  `rtvc-dsk` output, builds matching `rtvc-tocas` bytes, renumber and
  character table round trips.
- Emulator tests: `SAVE "B:X"` / `LOAD "B:X"` through the card; Run of a small
  BASIC and ASM program from the dev snapshot, checking screen memory after a
  fixed number of frames.
- Manual: native and web (Chrome, Firefox, Safari); web migration from an
  existing IndexedDB; file manager keys and touch buttons; 50 Hz with IDE
  panes open.
- Build checks: native, `cli-tools`, lightweight WASM (size unchanged),
  `wasm-full` (record the size change).

### Documentation to update when implemented

[info/rtvc.md](../../info/rtvc.md) (root folder, host file drives, file
manager, IDE panes, web storage), [info/tvc.md](../../info/tvc.md) (device
name handling, character set), [info/assembler.md](../../info/assembler.md)
(`INCGFX`, later), [README.md](../../README.md), the
[development skill](../skills/development/SKILL.md) (`dev-snapshot` task, new
tests), and an `editors-progress.md` record once work starts.

## Later editors

### Image editor

- Edits `.gfx` files: canvas in a TVC mode (2 colours 512×240, 4 colours
  256×240, 16 colours 128×240) with the TVC palette and pixel aspect, pen,
  line, rectangle, fill, select and move, grid, zoom. Tools sit in the pane's
  tool row.
- Plain-text source read by the assembler's `INCGFX` directive, which packs
  pixels into the TVC byte layout and emits labels and size constants:

  ```
  ; ship.gfx
  mode 4                 ; 2, 4 or 16 colours
  palette 0 1 4 5        ; optional: colours for indexes 0-3
  sprite SHIP 8x8
  ..1111..
  .122221.
  ...
  ```

  ```asm
          INCGFX "ship.gfx"    ; SHIP, SHIP_W, SHIP_H, ...
  ```

- Pixels are `.` or one hex digit. Widths that do not fill whole bytes and
  out-of-range pixels are assembler errors with the `.gfx` line.
- Grab from screen: capture TVC video memory into a new `.gfx`.

### Sound editor

The TVC has one tone generator and a 4-bit amplitude
([Sound and Timer](../../info/tvc.md#sound-and-timer)), so this is a PICO-8
SFX-style editor: an effect is a row of steps (pitch, volume, length) played
through the emulated hardware; a tune is a sequence of effects. Source is a
plain-text `.sfx` file; export as BASIC `SOUND` lines or a data table plus a
small player. The format is decided in that round.

### Debugger

Gutter clicks set breakpoints shared with the Breakpoints pane; a stop in
code built from a drive folder shows the current line. Needs the source-line
data in [rtvc-asm Workflow Improvements](rtvc-asm-workflow-improvements.md)
and bank-aware breakpoints ([TODO.md](../../TODO.md#debugger)).

## Decisions

From the 2026-11-22 review, still valid:

1. **Flip key**: ``Ctrl+` ``, matched by physical key. Esc is the TVC ESC key.
2. **Run resets**: Run starts from a clean snapshot; `Ctrl+Shift+R` runs into
   the current state.
3. **Editor font**: normal monospace.
4. **Image source**: plain-text `.gfx` read by `INCGFX`.

From the 2026-11-23 review:

5. **No projects or cart file.** IDE A: and IDE B: panes work on the host
   file card's drive folders; the file decides the editor.
6. **One root folder** with `A/`, `B/`, and `gamebase/`, the same on native
   and web; the web file manager starts at the root.
7. **Drives come from the host file card**, extended to A: and B: folders
   selected in the Disk menu. Folder-backed VT-DOS disks (a FAT12 image built
   from a folder) were rejected for now: more compatible, but much more
   complex, limited to 720K, and unsafe while VT-DOS caches the directory.
8. **DSK images** are made and filled through the file manager, which opens
   them like folders.
9. **Dev boot snapshot**: an embedded 64K+ BASIC 1.2 machine with the host
   file card replaces BASIC key-wait detection.
10. **Plain `TextEdit`**, no egui upgrade until the web bundle size is
    reduced.
11. **Simple UI like PICO-8**, but egui panes beside the TVC screen, not
    editors drawn inside it.

Proposed defaults, to confirm when implementation starts:

- Simple mode flips to IDE A: full window.
- Build writes `<stem>.cas` beside the source.
- Native default root `~/Documents/rtvc`.

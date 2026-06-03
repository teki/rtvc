#[cfg(all(feature = "web-vid-simple", feature = "web-vid-realistic"))]
compile_error!("features `web-vid-simple` and `web-vid-realistic` are mutually exclusive");

pub mod bus;
pub mod cas;
pub mod disasm;
pub mod expansion;
pub mod fd1793;
pub mod hbf;
pub mod key;
pub mod log;
pub mod mmu;
pub mod snapshot;
pub mod sound;
pub mod tape;
pub mod tvc;
pub mod tvc_snapshot;
pub mod vid;
pub mod z80;

#[cfg(feature = "native")]
pub mod audio;

#[cfg(feature = "native")]
pub mod emu;

#[cfg(feature = "native")]
pub mod ui;

#[cfg(feature = "wasm")]
pub mod wasm;

#[cfg(all(feature = "web-vid-simple", feature = "web-vid-realistic"))]
compile_error!("features `web-vid-simple` and `web-vid-realistic` are mutually exclusive");

pub mod asm;
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

#[cfg(any(feature = "native", feature = "wasm-full"))]
pub mod app_state;

#[cfg(any(feature = "native", feature = "wasm-full"))]
pub mod audio;

#[cfg(any(feature = "native", feature = "wasm-full"))]
pub mod emu;

#[cfg(any(feature = "native", feature = "wasm-full"))]
pub mod ui;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod debugger;

#[cfg(any(feature = "wasm", feature = "wasm-full"))]
pub mod wasm;

use crate::emu::Emu;
use eframe::egui;
use serde::Deserialize;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const FPS_WINDOW: Duration = Duration::from_secs(5);

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
enum DebuggerCommand {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "stats")]
    Stats,
    #[serde(rename = "close_app")]
    CloseApp,
    #[serde(rename = "step")]
    Step { count: Option<u32> },
    #[serde(rename = "continue")]
    Continue,
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "reset")]
    Reset,
    #[serde(rename = "breakpoint_add")]
    BreakpointAdd { addr: u16 },
    #[serde(rename = "breakpoint_remove")]
    BreakpointRemove { addr: u16 },
    #[serde(rename = "breakpoint_list")]
    BreakpointList,
    #[serde(rename = "read_memory")]
    ReadMemory {
        addr: u16,
        len: usize,
        bank: Option<String>,
    },
    #[serde(rename = "write_memory")]
    WriteMemory {
        addr: u16,
        data: Vec<u8>,
        bank: Option<String>,
    },
    #[serde(rename = "set_register")]
    SetRegister { name: String, value: u16 },
    #[serde(rename = "write_port")]
    WritePort { port: u8, value: u8 },
    #[serde(rename = "run_to_interrupt")]
    RunToInterrupt,
    /// Runs `count` frames as fast as the host allows, then replies.
    #[serde(rename = "run_frames")]
    RunFrames { count: u32 },
    #[serde(rename = "disassemble")]
    Disassemble { addr: u16, len: usize },
    #[serde(rename = "assemble")]
    Assemble { addr: u16, source: String },
    #[serde(rename = "save_snapshot")]
    SaveSnapshot { path: String },
    #[serde(rename = "load_snapshot")]
    LoadSnapshot { path: String },
    /// TVC CRTC registers and television-receiver sync lock.
    #[serde(rename = "video_state")]
    VideoState,
    #[serde(rename = "save_screenshot")]
    SaveScreenshot { path: String },
    #[serde(rename = "key")]
    Key {
        action: String, // "down", "up", "press"
        code: Option<u32>,
        #[serde(rename = "char")]
        character: Option<String>,
    },
    #[serde(rename = "key_press")]
    KeyPress { key: u32, duration: u32 },
    /// Sets (when `enabled` is given) and reports warp while loading.
    #[serde(rename = "warp")]
    Warp { enabled: Option<bool> },
    #[serde(rename = "instruction_trace_start")]
    InstructionTraceStart {
        capacity: Option<usize>,
        /// Also record the TVC CRTC beam and selected register (off by default).
        beam: Option<bool>,
    },
    #[serde(rename = "instruction_trace_stop")]
    InstructionTraceStop,
    #[serde(rename = "instruction_trace_clear")]
    InstructionTraceClear,
    #[serde(rename = "instruction_trace_status")]
    InstructionTraceStatus,
    #[serde(rename = "instruction_trace_list")]
    InstructionTraceList {
        limit: Option<usize>,
        /// Page forward: the entries after this sequence number, oldest first.
        after_sequence: Option<u64>,
        /// Only entries that wrote an I/O port, searched over the whole trace.
        port_writes_only: Option<bool>,
    },
}

pub struct DebuggerMessage {
    pub cmd_line: String,
    pub reply_tx: Sender<String>,
}

pub enum DebuggerEvent {
    BreakpointHit { pc: u16 },
}

pub struct DebuggerInterface {
    pub cmd_rx: Receiver<DebuggerMessage>,
    pub event_tx: Sender<DebuggerEvent>,
    ctx: Arc<Mutex<Option<egui::Context>>>,
    frame_stats: Mutex<FrameStats>,
    close_requested: AtomicBool,
}

impl DebuggerInterface {
    pub fn set_context(&self, ctx: egui::Context) {
        if let Ok(mut guard) = self.ctx.lock() {
            if guard.is_none() {
                *guard = Some(ctx);
            }
        }
    }

    pub fn record_frame(&self) {
        if let Ok(mut stats) = self.frame_stats.lock() {
            stats.record_at(Instant::now());
        }
    }

    pub fn handle_command(&self, emu: &mut Emu, line: &str) -> String {
        let close_requested = matches!(serde_json::from_str(line), Ok(DebuggerCommand::CloseApp));
        let stats = self
            .frame_stats
            .lock()
            .ok()
            .map(|mut stats| stats.snapshot_at(Instant::now()))
            .unwrap_or_default();
        let response = handle_command(emu, line, stats);
        if close_requested {
            self.close_requested.store(true, Ordering::Release);
        }
        response
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested.load(Ordering::Acquire)
    }
}

struct FrameStats {
    started_at: Instant,
    frames: VecDeque<Instant>,
}

#[derive(Clone, Copy, Default)]
struct FrameStatsSnapshot {
    average_fps: f64,
    window_seconds: f64,
    frames: usize,
}

impl FrameStats {
    fn new() -> Self {
        Self::new_at(Instant::now())
    }

    fn new_at(now: Instant) -> Self {
        Self {
            started_at: now,
            frames: VecDeque::new(),
        }
    }

    fn record_at(&mut self, now: Instant) {
        self.prune(now);
        self.frames.push_back(now);
    }

    fn snapshot_at(&mut self, now: Instant) -> FrameStatsSnapshot {
        self.prune(now);
        let window = now.duration_since(self.started_at).min(FPS_WINDOW);
        let window_seconds = window.as_secs_f64();
        FrameStatsSnapshot {
            average_fps: if window_seconds > 0.0 {
                self.frames.len() as f64 / window_seconds
            } else {
                0.0
            },
            window_seconds,
            frames: self.frames.len(),
        }
    }

    fn prune(&mut self, now: Instant) {
        let window_start = now.checked_sub(FPS_WINDOW).unwrap_or(self.started_at);
        while self
            .frames
            .front()
            .is_some_and(|frame| *frame < window_start)
        {
            self.frames.pop_front();
        }
    }
}

pub fn start_debugger_server(port: u16) -> std::io::Result<DebuggerInterface> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    start_debugger_on_listener(listener)
}

/// Binds the first free port at or above `port`, so several emulators can run
/// side by side. Returns the interface and the port actually bound.
pub fn start_debugger_server_from(port: u16) -> std::io::Result<(DebuggerInterface, u16)> {
    let mut candidate = port;
    loop {
        match TcpListener::bind(("127.0.0.1", candidate)) {
            Ok(listener) => return Ok((start_debugger_on_listener(listener)?, candidate)),
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse && candidate < u16::MAX => {
                candidate += 1;
            }
            Err(err) => return Err(err),
        }
    }
}

/// Writes one reply line in full. The stream is non-blocking for reads, so a
/// large reply (an instruction trace) would stop at `WouldBlock` once the
/// send buffer fills; block for the write instead.
fn write_line(stream: &mut TcpStream, line: &str) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    let result = stream
        .write_all(line.as_bytes())
        .and_then(|()| stream.write_all(b"\n"));
    stream.set_nonblocking(true)?;
    result
}

fn start_debugger_on_listener(listener: TcpListener) -> std::io::Result<DebuggerInterface> {
    let port = listener.local_addr()?.port();
    listener.set_nonblocking(true)?;
    let (cmd_tx, cmd_rx) = channel::<DebuggerMessage>();
    let (event_tx, event_rx) = channel::<DebuggerEvent>();
    let ctx = Arc::new(Mutex::new(None::<egui::Context>));
    let ctx_clone = Arc::clone(&ctx);

    thread::spawn(move || {
        println!("Debugger server listening on 127.0.0.1:{}", port);

        let mut stream: Option<TcpStream> = None;
        let mut command_buffer = String::new();

        loop {
            // 1. Accept new connection
            if stream.is_none() {
                if let Ok((new_stream, _addr)) = listener.accept() {
                    new_stream.set_nonblocking(true).unwrap();
                    stream = Some(new_stream);
                    command_buffer.clear();
                }
            }

            // 2. Read from TCP stream
            let mut disconnect = false;
            if let Some(ref mut s) = stream {
                let mut buf = [0u8; 4096];
                match s.read(&mut buf) {
                    Ok(0) => {
                        // Connection closed by client
                        disconnect = true;
                    }
                    Ok(n) => {
                        if let Ok(text) = std::str::from_utf8(&buf[..n]) {
                            command_buffer.push_str(text);
                            while let Some(pos) = command_buffer.find('\n') {
                                let line = command_buffer[..pos].trim().to_string();
                                command_buffer = command_buffer[pos + 1..].to_string();
                                if !line.is_empty() {
                                    let (reply_tx, reply_rx) = channel();
                                    if cmd_tx
                                        .send(DebuggerMessage {
                                            cmd_line: line,
                                            reply_tx,
                                        })
                                        .is_ok()
                                    {
                                        if let Ok(guard) = ctx_clone.lock() {
                                            if let Some(ref c) = *guard {
                                                c.request_repaint();
                                            }
                                        }
                                        // Wait for response from main thread
                                        if let Ok(response) = reply_rx.recv() {
                                            if write_line(s, &response).is_err() {
                                                disconnect = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => {
                        disconnect = true;
                    }
                }

                if !disconnect {
                    // 3. Process events from main thread (e.g. breakpoints)
                    while let Ok(event) = event_rx.try_recv() {
                        match event {
                            DebuggerEvent::BreakpointHit { pc } => {
                                let msg = serde_json::json!({
                                    "event": "breakpoint",
                                    "pc": pc
                                });
                                if write_line(s, &msg.to_string()).is_err() {
                                    disconnect = true;
                                }
                            }
                        }
                    }
                }
            }
            if disconnect {
                stream = None;
            }

            thread::sleep(Duration::from_millis(5));
        }
    });

    Ok(DebuggerInterface {
        cmd_rx,
        event_tx,
        ctx,
        frame_stats: Mutex::new(FrameStats::new()),
        close_requested: AtomicBool::new(false),
    })
}

/// Host time spent running frames between command checks while warping.
const HEADLESS_WARP_BUDGET: Duration = Duration::from_millis(15);
/// One emulated 50 Hz frame of real time.
const FRAME_PERIOD: Duration = Duration::from_millis(20);
/// Frames headless mode may run back to back to catch up after a late wake.
const HEADLESS_MAX_CATCH_UP: u32 = 5;
/// Upper bound for one `run_frames` command (ten emulated minutes).
const MAX_RUN_FRAMES: u32 = 30_000;

pub fn run_headless(mut emu: Emu, port: u16) -> std::io::Result<()> {
    let (debugger, _port) = start_debugger_server_from(port)?;
    // Frames are due on a fixed 20 ms grid, so a late wake from sleep is made
    // up by the next frames instead of slowing the emulation down.
    let mut next_frame = Instant::now();

    loop {
        // 1. Process debugger commands
        while let Ok(msg) = debugger.cmd_rx.try_recv() {
            let response = debugger.handle_command(&mut emu, &msg.cmd_line);
            let _ = msg.reply_tx.send(response);
        }
        if debugger.close_requested() {
            thread::sleep(Duration::from_millis(10));
            return Ok(());
        }

        // 2. Emulate frame if running. While warping, run frames back to back
        // for a short budget, then return to serve commands.
        if emu.warp_active() {
            let start = Instant::now();
            while emu.warp_active() && start.elapsed() < HEADLESS_WARP_BUDGET {
                let hit_breakpoint = emu.tick();
                debugger.record_frame();
                let _ = emu.take_audio_samples();
                if hit_breakpoint {
                    emu.running = false;
                    let pc = emu.z80_state().pc;
                    println!("Hit breakpoint at PC = 0x{:04X}", pc);
                    let _ = debugger.event_tx.send(DebuggerEvent::BreakpointHit { pc });
                }
            }
            next_frame = Instant::now();
        } else if emu.running {
            let now = Instant::now();
            if now >= next_frame {
                let hit_breakpoint = emu.tick();
                debugger.record_frame();
                let _ = emu.take_audio_samples();
                next_frame += FRAME_PERIOD;
                if now.saturating_duration_since(next_frame) > FRAME_PERIOD * HEADLESS_MAX_CATCH_UP
                {
                    // Too far behind (a long command, a stalled host): resync
                    // rather than running a burst of frames.
                    next_frame = now + FRAME_PERIOD;
                }
                if hit_breakpoint {
                    emu.running = false;
                    let pc = emu.z80_state().pc;
                    println!("Hit breakpoint at PC = 0x{:04X}", pc);
                    let _ = debugger.event_tx.send(DebuggerEvent::BreakpointHit { pc });
                }
            } else {
                std::thread::sleep(next_frame - now);
            }
        } else {
            next_frame = Instant::now();
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

fn video_stats_json(stats: &crate::tvc_vid::VideoStats) -> serde_json::Value {
    serde_json::json!({
        "synced_frames": stats.synced_frames,
        "unsynced_host_frames": stats.unsynced_host_frames,
        "host_frames": stats.host_frames,
        "h_lock_losses": stats.h_lock_losses,
        "v_lock_losses": stats.v_lock_losses,
        "bad_v_periods": stats.bad_v_periods,
        "early_vsyncs": stats.early_vsyncs,
        "vsync_timeouts": stats.vsync_timeouts,
    })
}

fn handle_command(emu: &mut Emu, line: &str, stats: FrameStatsSnapshot) -> String {
    let parsed: Result<DebuggerCommand, serde_json::Error> = serde_json::from_str(line);
    let response_val = match parsed {
        Ok(cmd) => match cmd {
            DebuggerCommand::Status => {
                let state = emu.z80_state();
                serde_json::json!({
                    "status": "ok",
                    "system": emu.system_label(),
                    "running": emu.running,
                    "pc": state.pc,
                    "sp": state.sp,
                    "af": state.get_reg16(0),
                    "bc": state.get_reg16(1),
                    "de": state.get_reg16(2),
                    "hl": state.get_reg16(3),
                    "ix": state.get_reg16(4),
                    "iy": state.get_reg16(5),
                    "halted": state.halted != 0,
                    "cycles": emu.clock(),
                    "tape_loading": emu.tape_loading(),
                    "tape_progress": emu.tape_progress_percent(),
                    "warp": emu.warp_loading(),
                    "warp_active": emu.warp_active(),
                })
            }
            DebuggerCommand::VideoState => match emu.tvc() {
                Some(tvc) => {
                    let v = tvc.bus.vid.debug_state();
                    serde_json::json!({
                        "status": "ok",
                        "crtc": v.registers.to_vec(),
                        "crtc_selected": v.selected_register,
                        "row": v.position.0,
                        "raster": v.position.1,
                        "h_count": v.position.2,
                        "h_locked": v.horizontal_locked,
                        "v_locked": v.vertical_locked,
                        "h_period": v.measured_h_period,
                        "v_period": v.measured_v_period,
                        "last_h_period": v.last_h_period,
                        "last_v_period": v.last_v_period,
                        "lines_since_vsync": v.lines_since_vsync,
                        "vsync_intervals": v.vsync_intervals,
                        "timeout_raw_lines": v.timeout_raw_lines,
                        "video_stats": video_stats_json(&v.stats),
                    })
                }
                None => {
                    serde_json::json!({"status": "error", "message": "video_state needs a TVC"})
                }
            },
            DebuggerCommand::Stats => {
                serde_json::json!({
                    "status": "ok",
                    "running": emu.running,
                    "average_fps": stats.average_fps,
                    "window_seconds": stats.window_seconds,
                    "frames": stats.frames,
                    "video": emu.tvc().map(|tvc| video_stats_json(&tvc.bus.vid.stats())),
                })
            }
            DebuggerCommand::CloseApp => {
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::Step { count } => {
                let step_count = count.unwrap_or(1);
                emu.debug_step(step_count);
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::RunToInterrupt => {
                let result = emu.debug_run_to_interrupt();
                serde_json::json!({
                    "status": "ok",
                    "elapsed_cycles": result.elapsed_cycles,
                    "interrupt_accepted": result.interrupt_accepted
                })
            }
            DebuggerCommand::RunFrames { count } => {
                if count > MAX_RUN_FRAMES {
                    serde_json::json!({
                        "status": "error",
                        "message": format!("run_frames count is limited to {MAX_RUN_FRAMES}"),
                    })
                } else {
                    let was_running = emu.running;
                    let start_cycles = emu.clock();
                    let start = Instant::now();
                    emu.running = true;
                    let mut frames = 0;
                    let mut breakpoint = None;
                    while frames < count {
                        let hit_breakpoint = emu.tick();
                        let _ = emu.take_audio_samples();
                        frames += 1;
                        if hit_breakpoint {
                            breakpoint = Some(emu.z80_state().pc);
                            break;
                        }
                    }
                    emu.running = was_running && breakpoint.is_none();
                    serde_json::json!({
                        "status": "ok",
                        "frames": frames,
                        "elapsed_cycles": emu.clock() - start_cycles,
                        "breakpoint": breakpoint,
                        "host_ms": start.elapsed().as_secs_f64() * 1000.0,
                    })
                }
            }
            DebuggerCommand::Continue => {
                emu.running = true;
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::Pause => {
                emu.running = false;
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::Reset => {
                emu.reset();
                emu.running = false;
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::BreakpointAdd { addr } => {
                emu.set_breakpoint(addr);
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::BreakpointRemove { addr } => {
                emu.clear_breakpoint(addr);
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::BreakpointList => {
                let list = emu.get_breakpoints();
                serde_json::json!({
                    "status": "ok",
                    "breakpoints": list
                })
            }
            DebuggerCommand::ReadMemory { addr, len, bank } => {
                if let Some(b) = bank {
                    match emu.read_raw_bank(&b, addr as usize, len) {
                        Some(data) => serde_json::json!({ "status": "ok", "data": data }),
                        None => {
                            serde_json::json!({ "status": "error", "message": format!("Unknown or uninitialized memory bank: {}", b) })
                        }
                    }
                } else {
                    let data = emu.read_mapped_memory(addr, len);
                    serde_json::json!({ "status": "ok", "data": data })
                }
            }
            DebuggerCommand::WriteMemory { addr, data, bank } => {
                if let Some(bank) = bank {
                    match emu.write_raw_bank(&bank, addr as usize, &data) {
                        Some(len) => serde_json::json!({
                            "status": "ok",
                            "bank": bank,
                            "addr": addr,
                            "len": len
                        }),
                        None => serde_json::json!({
                            "status": "error",
                            "message": format!("Unknown, read-only, or unavailable memory bank: {}", bank)
                        }),
                    }
                } else {
                    emu.write_mapped_memory(addr, &data);
                    serde_json::json!({
                        "status": "ok",
                        "addr": addr,
                        "len": data.len()
                    })
                }
            }
            DebuggerCommand::SetRegister { name, value } => match normalize_register_name(&name) {
                Some(name) => {
                    emu.set_z80_register(name, value);
                    serde_json::json!({
                        "status": "ok",
                        "name": name,
                        "value": value
                    })
                }
                None => serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown register: {}", name)
                }),
            },
            DebuggerCommand::WritePort { port, value } => match emu.write_port(port, value) {
                Ok(()) => serde_json::json!({
                    "status": "ok",
                    "port": port,
                    "value": value
                }),
                Err(err) => serde_json::json!({
                    "status": "error",
                    "message": err
                }),
            },
            DebuggerCommand::Disassemble { addr, len } => {
                let insts = emu.disassemble(addr, len);
                let mapped: Vec<_> = insts
                    .iter()
                    .map(|inst| {
                        serde_json::json!({
                            "addr": inst.addr,
                            "len": inst.len,
                            "bytes": inst.bytes,
                            "text": inst.text
                        })
                    })
                    .collect();
                serde_json::json!({
                    "status": "ok",
                    "instructions": mapped
                })
            }
            DebuggerCommand::Assemble { addr, source } => {
                match crate::asm::assemble_program(&source, addr) {
                    Ok(program) => {
                        let segments: Vec<_> = program
                            .segments
                            .iter()
                            .map(|segment| {
                                serde_json::json!({
                                    "addr": segment.addr,
                                    "len": segment.bytes.len(),
                                    "bytes": segment.bytes
                                })
                            })
                            .collect();
                        let lines: Vec<_> = program
                            .lines
                            .iter()
                            .map(|line| {
                                serde_json::json!({
                                    "line": line.line,
                                    "addr": line.addr,
                                    "run_addr": line.run_addr,
                                    "len": line.len,
                                    "source": line.source
                                })
                            })
                            .collect();
                        let mappings: Vec<_> = program
                            .mappings
                            .iter()
                            .map(|mapping| {
                                serde_json::json!({
                                    "name": mapping.name,
                                    "source_base": mapping.source_base,
                                    "mapped_base": mapping.mapped_base
                                })
                            })
                            .collect();
                        let breakpoints: Vec<_> = program
                            .breakpoints
                            .iter()
                            .map(|breakpoint| {
                                serde_json::json!({
                                    "addr": breakpoint.addr,
                                    "line": breakpoint.line
                                })
                            })
                            .collect();
                        serde_json::json!({
                            "status": "ok",
                            "addr": program.origin,
                            "len": program.bytes.len(),
                            "bytes": program.bytes,
                            "next_addr": program.next_addr,
                            "breakpoints": breakpoints,
                            "segments": segments,
                            "symbols": program.symbols,
                            "mappings": mappings,
                            "lines": lines
                        })
                    }
                    Err(err) => {
                        serde_json::json!({ "status": "error", "message": err.to_string() })
                    }
                }
            }
            DebuggerCommand::SaveSnapshot { path } => {
                match emu.save_snapshot_file(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => {
                        serde_json::json!({ "status": "error", "message": format!("Failed to save snapshot: {}", err) })
                    }
                }
            }
            DebuggerCommand::LoadSnapshot { path } => {
                match emu.load_snapshot_file(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => {
                        serde_json::json!({ "status": "error", "message": format!("Failed to load snapshot: {}", err) })
                    }
                }
            }
            DebuggerCommand::SaveScreenshot { path } => {
                match emu.save_screenshot(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => {
                        serde_json::json!({ "status": "error", "message": format!("Failed to save screenshot: {}", err) })
                    }
                }
            }
            DebuggerCommand::Key {
                action,
                code,
                character,
            } => match action.as_str() {
                "down" => {
                    if let Some(c) = code {
                        emu.key_down(c);
                        serde_json::json!({ "status": "ok" })
                    } else {
                        serde_json::json!({ "status": "error", "message": "Missing key code for key_down" })
                    }
                }
                "up" => {
                    if let Some(c) = code {
                        emu.key_up(c);
                        serde_json::json!({ "status": "ok" })
                    } else {
                        serde_json::json!({ "status": "error", "message": "Missing key code for key_up" })
                    }
                }
                "press" => {
                    if let Some(ch_str) = character {
                        emu.type_text(&ch_str);
                        serde_json::json!({ "status": "ok" })
                    } else {
                        serde_json::json!({ "status": "error", "message": "Missing char for key_press" })
                    }
                }
                _ => serde_json::json!({ "status": "error", "message": "Unknown key action" }),
            },
            DebuggerCommand::Warp { enabled } => {
                if let Some(enabled) = enabled {
                    emu.set_warp_loading(enabled);
                }
                serde_json::json!({
                    "status": "ok",
                    "warp": emu.warp_loading(),
                    "tape_loading": emu.tape_loading(),
                    "warp_active": emu.warp_active(),
                    "tape_progress": emu.tape_progress_percent(),
                })
            }
            DebuggerCommand::KeyPress { key, duration } => {
                match emu.key_press_frames(key, duration) {
                    Ok(()) => serde_json::json!({
                        "status": "ok",
                        "key": key,
                        "duration": duration
                    }),
                    Err(error) => serde_json::json!({
                        "status": "error",
                        "message": error
                    }),
                }
            }
            DebuggerCommand::InstructionTraceStart { capacity, beam } => {
                let capacity = capacity.unwrap_or(crate::instruction_trace::DEFAULT_TRACE_CAPACITY);
                emu.start_instruction_trace(capacity);
                emu.set_instruction_trace_beam(beam.unwrap_or(false));
                let trace = emu.instruction_trace();
                serde_json::json!({
                    "status": "ok",
                    "recording": trace.is_recording(),
                    "beam": trace.records_beam(),
                    "capacity": trace.capacity(),
                    "entries": trace.len()
                })
            }
            DebuggerCommand::InstructionTraceStop => {
                emu.stop_instruction_trace();
                let trace = emu.instruction_trace();
                serde_json::json!({
                    "status": "ok",
                    "recording": trace.is_recording(),
                    "beam": trace.records_beam(),
                    "capacity": trace.capacity(),
                    "entries": trace.len()
                })
            }
            DebuggerCommand::InstructionTraceClear => {
                emu.clear_instruction_trace();
                let trace = emu.instruction_trace();
                serde_json::json!({
                    "status": "ok",
                    "recording": trace.is_recording(),
                    "beam": trace.records_beam(),
                    "capacity": trace.capacity(),
                    "entries": trace.len()
                })
            }
            DebuggerCommand::InstructionTraceStatus => {
                let trace = emu.instruction_trace();
                serde_json::json!({
                    "status": "ok",
                    "recording": trace.is_recording(),
                    "beam": trace.records_beam(),
                    "capacity": trace.capacity(),
                    "entries": trace.len()
                })
            }
            DebuggerCommand::InstructionTraceList {
                limit,
                after_sequence,
                port_writes_only,
            } => {
                let limit = limit
                    .unwrap_or(100)
                    .clamp(1, crate::instruction_trace::MAX_TRACE_CAPACITY);
                let entries: Vec<_> = emu
                    .query_instruction_trace(
                        limit,
                        after_sequence,
                        port_writes_only.unwrap_or(false),
                    )
                    .into_iter()
                    .map(|entry| {
                        let instruction =
                            crate::disasm::disassemble_captured(entry.pc(), &entry.opcode);
                        serde_json::json!({
                            "sequence": entry.sequence,
                            "clock": entry.clock,
                            "pc": entry.pc(),
                            "opcode": entry.opcode,
                            "instruction": instruction.text,
                            "registers": entry.registers,
                            "main_map": entry.main_map,
                            "video_map": entry.video_map,
                            "beam": entry.beam,
                            "elapsed_cycles": entry.elapsed_cycles,
                            "interrupt_accepted": entry.interrupt_accepted,
                            "memory_writes": entry.effects.memory_writes,
                            "port_writes": entry.effects.port_writes
                        })
                    })
                    .collect();
                serde_json::json!({
                    "status": "ok",
                    "entries": entries
                })
            }
        },
        Err(err) => {
            serde_json::json!({
                "status": "error",
                "message": format!("Invalid JSON command: {}", err)
            })
        }
    };

    response_val.to_string()
}

fn normalize_register_name(name: &str) -> Option<&'static str> {
    match name.to_ascii_uppercase().as_str() {
        "AF" => Some("AF"),
        "BC" => Some("BC"),
        "DE" => Some("DE"),
        "HL" => Some("HL"),
        "AF'" | "AFA" | "AF_ALT" => Some("AFa"),
        "BC'" | "BCA" | "BC_ALT" => Some("BCa"),
        "DE'" | "DEA" | "DE_ALT" => Some("DEa"),
        "HL'" | "HLA" | "HL_ALT" => Some("HLa"),
        "IX" => Some("IX"),
        "IY" => Some("IY"),
        "SP" => Some("SP"),
        "PC" => Some("PC"),
        "A" => Some("A"),
        "F" => Some("F"),
        "B" => Some("B"),
        "C" => Some("C"),
        "D" => Some("D"),
        "E" => Some("E"),
        "H" => Some("H"),
        "L" => Some("L"),
        "I" => Some("I"),
        "R" => Some("R"),
        "IFF1" => Some("IFF1"),
        "IFF2" => Some("IFF2"),
        "IM" => Some("im"),
        "HALTED" => Some("halted"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "debugger_tests.rs"]
mod tests;

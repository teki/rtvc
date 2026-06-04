use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::time::{Instant, Duration};
use std::path::Path;
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use serde::Deserialize;
use crate::emu::Emu;
use crate::bus::CpuBus;
use eframe::egui;

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
enum DebuggerCommand {
    #[serde(rename = "status")]
    Status,
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
    #[serde(rename = "disassemble")]
    Disassemble { addr: u16, len: usize },
    #[serde(rename = "save_snapshot")]
    SaveSnapshot { path: String },
    #[serde(rename = "load_snapshot")]
    LoadSnapshot { path: String },
    #[serde(rename = "save_screenshot")]
    SaveScreenshot { path: String },
    #[serde(rename = "key")]
    Key {
        action: String, // "down", "up", "press"
        code: Option<u32>,
        #[serde(rename = "char")]
        character: Option<String>,
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
}

impl DebuggerInterface {
    pub fn set_context(&self, ctx: egui::Context) {
        if let Ok(mut guard) = self.ctx.lock() {
            if guard.is_none() {
                *guard = Some(ctx);
            }
        }
    }
}

pub fn start_debugger_server(port: u16) -> DebuggerInterface {
    let (cmd_tx, cmd_rx) = channel::<DebuggerMessage>();
    let (event_tx, event_rx) = channel::<DebuggerEvent>();
    let ctx = Arc::new(Mutex::new(None::<egui::Context>));
    let ctx_clone = Arc::clone(&ctx);

    thread::spawn(move || {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
            .expect("Failed to bind TCP port");
        listener.set_nonblocking(true).unwrap();
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
                                    if cmd_tx.send(DebuggerMessage { cmd_line: line, reply_tx }).is_ok() {
                                        if let Ok(guard) = ctx_clone.lock() {
                                            if let Some(ref c) = *guard {
                                                c.request_repaint();
                                            }
                                        }
                                        // Wait for response from main thread
                                        if let Ok(response) = reply_rx.recv() {
                                            let _ = s.write_all(format!("{}\n", response).as_bytes());
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
                                let _ = s.write_all(format!("{}\n", msg.to_string()).as_bytes());
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

    DebuggerInterface { cmd_rx, event_tx, ctx }
}

pub fn run_headless(mut emu: Emu, port: u16) {
    let debugger = start_debugger_server(port);
    let mut last_frame_time = Instant::now();

    loop {
        // 1. Process debugger commands
        while let Ok(msg) = debugger.cmd_rx.try_recv() {
            let response = handle_command(&mut emu, &msg.cmd_line);
            let _ = msg.reply_tx.send(response);
        }

        // 2. Emulate frame if running
        if emu.running {
            let now = Instant::now();
            let elapsed = now.duration_since(last_frame_time);
            if elapsed >= Duration::from_millis(20) {
                let hit_breakpoint = emu.tvc.run_for_a_frame();
                last_frame_time = now;

                if hit_breakpoint {
                    emu.running = false;
                    let pc = emu.tvc.z80.state.r16[11];
                    println!("Hit breakpoint at PC = 0x{:04X}", pc);
                    let _ = debugger.event_tx.send(DebuggerEvent::BreakpointHit { pc });
                }
            } else {
                let sleep_time = Duration::from_millis(20).saturating_sub(elapsed);
                std::thread::sleep(sleep_time);
            }
        } else {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

pub fn handle_command(emu: &mut Emu, line: &str) -> String {
    let parsed: Result<DebuggerCommand, serde_json::Error> = serde_json::from_str(line);
    let response_val = match parsed {
        Ok(cmd) => match cmd {
            DebuggerCommand::Status => {
                let z80 = &emu.tvc.z80;
                serde_json::json!({
                    "status": "ok",
                    "running": emu.running,
                    "pc": z80.state.r16[11],
                    "sp": z80.state.r16[10],
                    "af": z80.state.get_reg16(0),
                    "bc": z80.state.get_reg16(1),
                    "de": z80.state.get_reg16(2),
                    "hl": z80.state.get_reg16(3),
                    "ix": z80.state.get_reg16(8),
                    "iy": z80.state.get_reg16(9),
                    "halted": z80.state.halted != 0,
                    "cycles": emu.tvc.clock,
                })
            }
            DebuggerCommand::Step { count } => {
                let step_count = count.unwrap_or(1);
                emu.running = false;
                for _ in 0..step_count {
                    emu.tvc.bus.trace_pc = emu.tvc.z80.state.r16[11];
                    let cpu_time = emu.tvc.z80.step(&mut emu.tvc.bus, 0);
                    emu.tvc.clock += cpu_time as u64;
                    emu.tvc.bus.advance_tape(cpu_time as u64);
                    emu.tvc.bus.advance_sound_timer(cpu_time as u64);
                }
                serde_json::json!({ "status": "ok" })
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
                emu.tvc.set_breakpoint(addr);
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::BreakpointRemove { addr } => {
                emu.tvc.clear_breakpoint(addr);
                serde_json::json!({ "status": "ok" })
            }
            DebuggerCommand::BreakpointList => {
                let list = emu.tvc.get_breakpoints();
                serde_json::json!({
                    "status": "ok",
                    "breakpoints": list
                })
            }
            DebuggerCommand::ReadMemory { addr, len, bank } => {
                if let Some(b) = bank {
                    match emu.read_raw_bank(&b, addr as usize, len) {
                        Some(data) => serde_json::json!({ "status": "ok", "data": data }),
                        None => serde_json::json!({ "status": "error", "message": format!("Unknown or uninitialized memory bank: {}", b) })
                    }
                } else {
                    let mut data = Vec::with_capacity(len);
                    for offset in 0..len {
                        let target_addr = addr.wrapping_add(offset as u16);
                        data.push(emu.tvc.bus.r8(target_addr));
                    }
                    serde_json::json!({ "status": "ok", "data": data })
                }
            }
            DebuggerCommand::Disassemble { addr, len } => {
                let insts = crate::disasm::disassemble_block(&mut emu.tvc.bus, addr, len);
                let mapped: Vec<_> = insts.iter().map(|inst| {
                    serde_json::json!({
                        "addr": inst.addr,
                        "len": inst.len,
                        "bytes": inst.bytes,
                        "text": inst.text
                    })
                }).collect();
                serde_json::json!({
                    "status": "ok",
                    "instructions": mapped
                })
            }
            DebuggerCommand::SaveSnapshot { path } => {
                match emu.save_snapshot_file(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => serde_json::json!({ "status": "error", "message": format!("Failed to save snapshot: {}", err) })
                }
            }
            DebuggerCommand::LoadSnapshot { path } => {
                match emu.load_snapshot_file(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => serde_json::json!({ "status": "error", "message": format!("Failed to load snapshot: {}", err) })
                }
            }
            DebuggerCommand::SaveScreenshot { path } => {
                match emu.save_screenshot(Path::new(&path)) {
                    Ok(()) => serde_json::json!({ "status": "ok" }),
                    Err(err) => serde_json::json!({ "status": "error", "message": format!("Failed to save screenshot: {}", err) })
                }
            }
            DebuggerCommand::Key { action, code, character } => {
                match action.as_str() {
                    "down" => {
                        if let Some(c) = code {
                            emu.tvc.key_down(c);
                            serde_json::json!({ "status": "ok" })
                        } else {
                            serde_json::json!({ "status": "error", "message": "Missing key code for key_down" })
                        }
                    }
                    "up" => {
                        if let Some(c) = code {
                            emu.tvc.key_up(c);
                            serde_json::json!({ "status": "ok" })
                        } else {
                            serde_json::json!({ "status": "error", "message": "Missing key code for key_up" })
                        }
                    }
                    "press" => {
                        if let Some(ch_str) = character {
                            for ch in ch_str.chars() {
                                emu.tvc.key_press(ch);
                            }
                            serde_json::json!({ "status": "ok" })
                        } else {
                            serde_json::json!({ "status": "error", "message": "Missing char for key_press" })
                        }
                    }
                    _ => serde_json::json!({ "status": "error", "message": "Unknown key action" }),
                }
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

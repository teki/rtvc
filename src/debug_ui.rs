use std::collections::VecDeque;
use std::sync::OnceLock;

use crate::bus::CpuBus;
use crate::disasm::disassemble_block;
use crate::emu::{Emu, RomVersion};
use crate::mmu::RomBank;
use eframe::egui;
use serde::Deserialize;

const EVENT_LIMIT: usize = 200;
const DISASSEMBLY_BYTES: usize = 96;
const MEMORY_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemorySource {
    Mapped,
    U0,
    U1,
    U2,
    U3,
    Vid0,
    Vid1,
    Vid2,
    Vid3,
    Sys,
    Cart,
    Exth,
}

impl MemorySource {
    const ALL: [Self; 12] = [
        Self::Mapped,
        Self::U0,
        Self::U1,
        Self::U2,
        Self::U3,
        Self::Vid0,
        Self::Vid1,
        Self::Vid2,
        Self::Vid3,
        Self::Sys,
        Self::Cart,
        Self::Exth,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Mapped => "Mapped CPU",
            Self::U0 => "U0 RAM",
            Self::U1 => "U1 RAM",
            Self::U2 => "U2 RAM",
            Self::U3 => "U3 RAM",
            Self::Vid0 => "VID0",
            Self::Vid1 => "VID1",
            Self::Vid2 => "VID2",
            Self::Vid3 => "VID3",
            Self::Sys => "SYS ROM",
            Self::Cart => "CART ROM",
            Self::Exth => "EXTH ROM",
        }
    }

    fn bank_name(self) -> Option<&'static str> {
        match self {
            Self::Mapped => None,
            Self::U0 => Some("u0"),
            Self::U1 => Some("u1"),
            Self::U2 => Some("u2"),
            Self::U3 => Some("u3"),
            Self::Vid0 => Some("vid0"),
            Self::Vid1 => Some("vid1"),
            Self::Vid2 => Some("vid2"),
            Self::Vid3 => Some("vid3"),
            Self::Sys => Some("sys"),
            Self::Cart => Some("cart"),
            Self::Exth => Some("exth"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DebugEventKind {
    Control,
    Breakpoint,
    RomTrace,
}

impl DebugEventKind {
    fn label(self) -> &'static str {
        match self {
            Self::Control => "control",
            Self::Breakpoint => "breakpoint",
            Self::RomTrace => "rom",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugEvent {
    pub sequence: u64,
    pub kind: DebugEventKind,
    pub address: Option<u16>,
    pub bank: Option<RomBank>,
    pub summary: String,
}

pub struct DebuggerUi {
    disassembly_address: u16,
    disassembly_address_text: String,
    follow_pc: bool,
    memory_address: u16,
    memory_address_text: String,
    memory_source: MemorySource,
    breakpoint_address_text: String,
    symbol_search: String,
    trace_rom_landmarks: bool,
    events: VecDeque<DebugEvent>,
    next_event_sequence: u64,
}

impl Default for DebuggerUi {
    fn default() -> Self {
        Self {
            disassembly_address: 0,
            disassembly_address_text: "0000".to_string(),
            follow_pc: true,
            memory_address: 0,
            memory_address_text: "0000".to_string(),
            memory_source: MemorySource::Mapped,
            breakpoint_address_text: String::new(),
            symbol_search: String::new(),
            trace_rom_landmarks: false,
            events: VecDeque::new(),
            next_event_sequence: 1,
        }
    }
}

impl DebuggerUi {
    pub fn draw_cpu(&mut self, ui: &mut egui::Ui, emu: &mut Emu) {
        ui.horizontal(|ui| {
            if ui
                .button(if emu.running { "Pause" } else { "Run" })
                .clicked()
            {
                emu.running = !emu.running;
                self.record_control(if emu.running { "Continued" } else { "Paused" });
            }
            if ui.button("Step").clicked() {
                emu.debug_step(1);
                self.record_control("Stepped 1 instruction");
            }
            if ui.button("Step 10").clicked() {
                emu.debug_step(10);
                self.record_control("Stepped 10 instructions");
            }
            if ui
                .button("Run to IRQ")
                .on_hover_text("Run until the Z80 accepts an interrupt")
                .clicked()
            {
                let result = emu.debug_run_to_interrupt();
                if result.interrupt_accepted {
                    self.record_control(&format!(
                        "Ran {} cycles to interrupt",
                        result.elapsed_cycles
                    ));
                } else {
                    self.record_control(&format!(
                        "No interrupt accepted within {} cycles",
                        result.elapsed_cycles
                    ));
                }
            }
            if ui.button("Reset").clicked() {
                emu.reset();
                emu.running = false;
                self.record_control("Reset and paused");
            }
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let state = &emu.tvc.z80.state;
            let registers = [
                ("AF", state.get_reg16(0)),
                ("BC", state.get_reg16(1)),
                ("DE", state.get_reg16(2)),
                ("HL", state.get_reg16(3)),
                ("IX", state.get_reg16(4)),
                ("IY", state.get_reg16(5)),
                ("SP", state.get_reg16(10)),
                ("PC", state.get_reg16(11)),
                ("AF'", state.get_reg16(6)),
                ("BC'", state.get_reg16(7)),
                ("DE'", state.get_reg16(8)),
                ("HL'", state.get_reg16(9)),
            ];
            egui::Grid::new("debug_cpu_registers")
                .num_columns(4)
                .striped(true)
                .show(ui, |ui| {
                    for chunk in registers.chunks(2) {
                        for (name, value) in chunk {
                            ui.strong(*name);
                            ui.monospace(format!("{value:04X}"));
                        }
                        ui.end_row();
                    }
                });

            let flags = state.get_reg16(0) as u8;
            ui.horizontal_wrapped(|ui| {
                ui.strong("Flags");
                for (mask, name) in [
                    (0x80, "S"),
                    (0x40, "Z"),
                    (0x10, "H"),
                    (0x04, "P/V"),
                    (0x02, "N"),
                    (0x01, "C"),
                ] {
                    ui.label(format!("{name}:{}", u8::from(flags & mask != 0)));
                }
            });
            ui.label(format!(
                "IM {}  IFF1 {}  IFF2 {}  HALT {}",
                state.im,
                state.iff1,
                state.iff2,
                u8::from(state.halted != 0)
            ));
            let map = emu.tvc.bus.mmu.map_labels();
            ui.monospace(format!(
                "Clock {}  MMU {},{},{},{}",
                emu.tvc.clock, map[0], map[1], map[2], map[3]
            ))
            .on_hover_text(format!(
                "Paging register {:02X}\n0000-3FFF, 4000-7FFF, 8000-BFFF, C000-FFFF",
                emu.tvc.bus.mmu.get_map_val()
            ));
            if let Some(symbol) = current_symbol(emu) {
                ui.separator();
                ui.strong(format!("{} ({})", symbol.name, symbol.bank.name()));
                ui.label(&symbol.summary);
            }
        });
    }

    pub fn draw_disassembly(&mut self, ui: &mut egui::Ui, emu: &mut Emu) {
        let pc = emu.tvc.z80.state.get_reg16(11);
        if self.follow_pc {
            self.set_disassembly_address(pc);
        }
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.follow_pc, "Follow PC");
            ui.label("Address");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.disassembly_address_text)
                    .desired_width(70.0)
                    .font(egui::TextStyle::Monospace),
            );
            if response.lost_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                && let Some(address) = parse_address(&self.disassembly_address_text)
            {
                self.disassembly_address = address;
                self.follow_pc = false;
                self.disassembly_address_text = format!("{address:04X}");
            }
            if ui.button("PC").clicked() {
                self.follow_pc = true;
                self.set_disassembly_address(pc);
            }
        });

        let instructions = disassemble_block(
            &mut emu.tvc.bus,
            self.disassembly_address,
            DISASSEMBLY_BYTES,
        );
        let breakpoints = emu.tvc.get_breakpoints();
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for instruction in instructions {
                    let at_pc = instruction.addr == pc;
                    let has_breakpoint = breakpoints.binary_search(&instruction.addr).is_ok();
                    let symbol = symbol_at_mapped_address(emu, instruction.addr);
                    let background = at_pc.then_some(egui::Color32::from_rgb(35, 70, 105));
                    egui::Frame::NONE
                        .fill(background.unwrap_or(egui::Color32::TRANSPARENT))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let marker = if has_breakpoint { "●" } else { "○" };
                                if ui
                                    .small_button(marker)
                                    .on_hover_text("Toggle breakpoint")
                                    .clicked()
                                {
                                    if has_breakpoint {
                                        emu.tvc.clear_breakpoint(instruction.addr);
                                    } else {
                                        emu.tvc.set_breakpoint(instruction.addr);
                                    }
                                }
                                ui.monospace(format!("{:04X}", instruction.addr));
                                ui.monospace(format_bytes(&instruction.bytes, 4));
                                let response = ui.monospace(&instruction.text);
                                let mut details = Vec::new();
                                if let Some(t_states) = instruction.t_states {
                                    details.push(format!("{t_states} T"));
                                }
                                if let Some(flags) = instruction.flags {
                                    details.push(format!("flags {flags}"));
                                }
                                if let Some(effect) = instruction.effect {
                                    details.push(effect.to_string());
                                }
                                if !details.is_empty() {
                                    response.on_hover_text(details.join("\n"));
                                }
                            });
                            if let Some(symbol) = symbol {
                                ui.horizontal(|ui| {
                                    ui.add_space(28.0);
                                    ui.colored_label(
                                        egui::Color32::from_rgb(130, 190, 255),
                                        format!("{}: {}", symbol.name, symbol.summary),
                                    );
                                });
                            }
                        });
                }
            });
    }

    pub fn draw_memory(&mut self, ui: &mut egui::Ui, emu: &mut Emu) {
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("debug_memory_source")
                .selected_text(self.memory_source.label())
                .show_ui(ui, |ui| {
                    for source in MemorySource::ALL {
                        ui.selectable_value(&mut self.memory_source, source, source.label());
                    }
                });
            ui.label("Address");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.memory_address_text)
                    .desired_width(70.0)
                    .font(egui::TextStyle::Monospace),
            );
            if response.lost_focus()
                && ui.input(|input| input.key_pressed(egui::Key::Enter))
                && let Some(address) = parse_address(&self.memory_address_text)
            {
                self.set_memory_address(address);
            }
            if ui.button("PC").clicked() {
                self.set_memory_address(emu.tvc.z80.state.get_reg16(11));
                self.memory_source = MemorySource::Mapped;
            }
            if ui.button("SP").clicked() {
                self.set_memory_address(emu.tvc.z80.state.get_reg16(10));
                self.memory_source = MemorySource::Mapped;
            }
        });

        let bytes = if let Some(bank) = self.memory_source.bank_name() {
            emu.tvc
                .bus
                .mmu
                .read_raw_bank(bank, self.memory_address as usize, MEMORY_BYTES)
        } else {
            Some(
                (0..MEMORY_BYTES)
                    .map(|offset| {
                        emu.tvc
                            .bus
                            .r8(self.memory_address.wrapping_add(offset as u16))
                    })
                    .collect(),
            )
        };

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| match bytes {
                Some(bytes) if !bytes.is_empty() => {
                    ui.set_min_width(650.0);
                    for (row, chunk) in bytes.chunks(16).enumerate() {
                        let address = self.memory_address.wrapping_add((row * 16) as u16);
                        let hex = chunk
                            .iter()
                            .map(|byte| format!("{byte:02X}"))
                            .collect::<Vec<_>>()
                            .join(" ");
                        let ascii: String = chunk
                            .iter()
                            .map(|byte| {
                                if byte.is_ascii_graphic() {
                                    *byte as char
                                } else {
                                    '.'
                                }
                            })
                            .collect();
                        ui.monospace(format!("{address:04X}  {hex:<47}  {ascii}"));
                    }
                }
                Some(_) => {
                    ui.label("Address is outside this bank.");
                }
                None => {
                    ui.label("This bank is not available on the selected machine.");
                }
            });
    }

    pub fn draw_breakpoints(&mut self, ui: &mut egui::Ui, emu: &mut Emu) {
        ui.horizontal(|ui| {
            ui.label("Address");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.breakpoint_address_text)
                    .desired_width(80.0)
                    .font(egui::TextStyle::Monospace),
            );
            let submit =
                response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
            if (ui.button("Add").clicked() || submit)
                && let Some(address) = parse_address(&self.breakpoint_address_text)
            {
                emu.tvc.set_breakpoint(address);
                self.breakpoint_address_text = format!("{address:04X}");
            }
            if ui.button("Clear All").clicked() {
                emu.tvc.clear_all_breakpoints();
            }
        });
        ui.separator();

        let breakpoints = emu.tvc.get_breakpoints();
        if breakpoints.is_empty() {
            ui.label("No execution breakpoints.");
            return;
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for address in breakpoints {
                ui.horizontal(|ui| {
                    if ui.small_button("×").clicked() {
                        emu.tvc.clear_breakpoint(address);
                    }
                    if ui
                        .link(format!("{address:04X}"))
                        .on_hover_text("Show in disassembly")
                        .clicked()
                    {
                        self.follow_pc = false;
                        self.set_disassembly_address(address);
                    }
                    if let Some(symbol) = symbol_at_mapped_address(emu, address) {
                        ui.label(&symbol.name);
                    }
                });
            }
        });
    }

    pub fn draw_rom_symbols(&mut self, ui: &mut egui::Ui, emu: &mut Emu) {
        if emu.machine_type.rom_version != RomVersion::V1_2 {
            ui.label("A ROM symbol database for BASIC 2.2 is not available yet.");
            return;
        }
        ui.add(
            egui::TextEdit::singleline(&mut self.symbol_search)
                .hint_text("Search name, summary, tag, address...")
                .desired_width(f32::INFINITY),
        );
        ui.separator();
        let search = self.symbol_search.trim().to_ascii_lowercase();
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for symbol in rom_symbols()
                    .iter()
                    .filter(|symbol| symbol.matches(&search))
                {
                    ui.horizontal(|ui| {
                        if ui.small_button("Disasm").clicked() {
                            self.follow_pc = false;
                            self.set_disassembly_address(symbol.address);
                        }
                        if ui.small_button("Memory").clicked() {
                            self.memory_source = match symbol.bank {
                                RomBank::Sys => MemorySource::Sys,
                                RomBank::Exth => MemorySource::Exth,
                            };
                            self.set_memory_address(symbol.offset);
                        }
                        if ui.small_button("BP").clicked() {
                            emu.tvc.set_breakpoint(symbol.address);
                        }
                        ui.monospace(format!("{}:{:04X}", symbol.bank.name(), symbol.address));
                        ui.strong(&symbol.name);
                    });
                    ui.label(&symbol.summary);
                    if !symbol.input.is_empty() || !symbol.output.is_empty() {
                        ui.horizontal_wrapped(|ui| {
                            if !symbol.input.is_empty() {
                                ui.label(format!("In: {}", symbol.input));
                            }
                            if !symbol.output.is_empty() {
                                ui.label(format!("Out: {}", symbol.output));
                            }
                        });
                    }
                    ui.separator();
                }
            });
    }

    pub fn draw_events(&mut self, ui: &mut egui::Ui, emu: &Emu) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.trace_rom_landmarks, "Trace ROM landmarks");
            if ui.small_button("Clear").clicked() {
                self.events.clear();
            }
        });
        if emu.machine_type.rom_version != RomVersion::V1_2 {
            ui.label("ROM tracing requires the BASIC 1.2 symbol database.");
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for event in &self.events {
                    ui.horizontal_wrapped(|ui| {
                        ui.monospace(format!("#{:04}", event.sequence));
                        ui.strong(event.kind.label());
                        if let Some(address) = event.address {
                            if let Some(bank) = event.bank {
                                ui.monospace(format!("{}:{address:04X}", bank.name()));
                            } else {
                                ui.monospace(format!("{address:04X}"));
                            }
                        }
                        ui.label(&event.summary);
                    });
                }
            });
    }

    pub fn record_breakpoint_hit(&mut self, pc: u16, emu: &Emu) {
        let symbol = symbol_at_mapped_address(emu, pc);
        let bank = emu.tvc.bus.mmu.mapped_rom_bank(pc);
        let summary = symbol
            .map(|symbol| format!("Hit {}", symbol.name))
            .unwrap_or_else(|| "Execution breakpoint hit".to_string());
        self.push_event(DebugEventKind::Breakpoint, Some(pc), bank, summary);
        self.follow_pc = true;
    }

    pub fn drain_trace_events(&mut self, emu: &mut Emu) {
        for event in emu.tvc.take_trace_events() {
            let symbol = symbol_at(event.bank, event.pc);
            let summary = symbol
                .map(|symbol| format!("{}: {}", symbol.name, symbol.summary))
                .unwrap_or_else(|| "ROM tracepoint".to_string());
            self.push_event(
                DebugEventKind::RomTrace,
                Some(event.pc),
                Some(event.bank),
                summary,
            );
        }
    }

    pub fn update_tracing(&mut self, emu: &mut Emu, events_visible: bool) {
        let should_trace = events_visible
            && self.trace_rom_landmarks
            && emu.machine_type.rom_version == RomVersion::V1_2;
        if should_trace == emu.tvc.tracepoints_enabled() {
            return;
        }
        if should_trace {
            let tracepoints: Vec<_> = rom_symbols()
                .iter()
                .filter(|symbol| symbol.usage.iter().any(|usage| usage == "trace"))
                .flat_map(|symbol| {
                    std::iter::once((symbol.bank, symbol.address)).chain(
                        symbol
                            .aliases
                            .iter()
                            .copied()
                            .map(|address| (symbol.bank, address)),
                    )
                })
                .collect();
            emu.tvc.set_tracepoints(&tracepoints);
        } else {
            emu.tvc.set_tracepoints(&[]);
        }
    }

    fn record_control(&mut self, summary: &str) {
        self.push_event(DebugEventKind::Control, None, None, summary.to_string());
    }

    fn push_event(
        &mut self,
        kind: DebugEventKind,
        address: Option<u16>,
        bank: Option<RomBank>,
        summary: String,
    ) {
        self.events.push_back(DebugEvent {
            sequence: self.next_event_sequence,
            kind,
            address,
            bank,
            summary,
        });
        self.next_event_sequence += 1;
        while self.events.len() > EVENT_LIMIT {
            self.events.pop_front();
        }
    }

    fn set_disassembly_address(&mut self, address: u16) {
        self.disassembly_address = address;
        self.disassembly_address_text = format!("{address:04X}");
    }

    fn set_memory_address(&mut self, address: u16) {
        self.memory_address = address;
        self.memory_address_text = format!("{address:04X}");
    }
}

#[derive(Deserialize)]
struct RomSymbolDocument {
    symbols: Vec<RomSymbolRaw>,
}

#[derive(Deserialize)]
struct RomSymbolRaw {
    bank: String,
    address: String,
    offset: String,
    #[serde(default)]
    aliases: Vec<String>,
    name: String,
    #[serde(default)]
    usage: Vec<String>,
    summary: String,
    #[serde(default)]
    input: String,
    #[serde(default)]
    output: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug)]
struct RomSymbol {
    bank: RomBank,
    address: u16,
    offset: u16,
    aliases: Vec<u16>,
    name: String,
    usage: Vec<String>,
    summary: String,
    input: String,
    output: String,
    tags: Vec<String>,
}

impl RomSymbol {
    fn matches(&self, search: &str) -> bool {
        search.is_empty()
            || self.name.to_ascii_lowercase().contains(search)
            || self.summary.to_ascii_lowercase().contains(search)
            || self
                .tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains(search))
            || format!("{:04x}", self.address).contains(search)
            || self.bank.name().contains(search)
    }
}

fn rom_symbols() -> &'static [RomSymbol] {
    static SYMBOLS: OnceLock<Vec<RomSymbol>> = OnceLock::new();
    SYMBOLS.get_or_init(|| {
        let document: RomSymbolDocument =
            serde_json::from_str(include_str!("../data/rom_symbols_1_2.json"))
                .expect("embedded ROM symbol database must be valid");
        document
            .symbols
            .into_iter()
            .filter_map(|symbol| {
                let bank = match symbol.bank.as_str() {
                    "sys" => RomBank::Sys,
                    "exth" => RomBank::Exth,
                    _ => return None,
                };
                Some(RomSymbol {
                    bank,
                    address: parse_address(&symbol.address)?,
                    offset: parse_address(&symbol.offset)?,
                    aliases: symbol
                        .aliases
                        .iter()
                        .filter_map(|alias| parse_address(alias))
                        .collect(),
                    name: symbol.name,
                    usage: symbol.usage,
                    summary: symbol.summary,
                    input: symbol.input,
                    output: symbol.output,
                    tags: symbol.tags,
                })
            })
            .collect()
    })
}

fn current_symbol(emu: &Emu) -> Option<&'static RomSymbol> {
    symbol_at_mapped_address(emu, emu.tvc.z80.state.get_reg16(11))
}

fn symbol_at_mapped_address(emu: &Emu, address: u16) -> Option<&'static RomSymbol> {
    if emu.machine_type.rom_version != RomVersion::V1_2 {
        return None;
    }
    let bank = emu.tvc.bus.mmu.mapped_rom_bank(address)?;
    symbol_at(bank, address)
}

fn symbol_at(bank: RomBank, address: u16) -> Option<&'static RomSymbol> {
    rom_symbols().iter().find(|symbol| {
        symbol.bank == bank && (symbol.address == address || symbol.aliases.contains(&address))
    })
}

fn parse_address(text: &str) -> Option<u16> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (digits, radix) = if let Some(value) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        (value, 16)
    } else if let Some(value) = trimmed.strip_prefix('$') {
        (value, 16)
    } else if let Some(value) = trimmed
        .strip_suffix('H')
        .or_else(|| trimmed.strip_suffix('h'))
    {
        (value, 16)
    } else {
        (trimmed, 16)
    };
    u16::from_str_radix(digits, radix).ok()
}

fn format_bytes(bytes: &[u8], width: usize) -> String {
    let mut text = bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ");
    while text.len() < width * 3 - 1 {
        text.push(' ');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_debugger_addresses_as_hex() {
        assert_eq!(parse_address("C229"), Some(0xC229));
        assert_eq!(parse_address("0xc229"), Some(0xC229));
        assert_eq!(parse_address("$C229"), Some(0xC229));
        assert_eq!(parse_address("C229H"), Some(0xC229));
        assert_eq!(parse_address("not-an-address"), None);
    }

    #[test]
    fn embedded_rom_symbols_load_and_resolve_aliases() {
        let symbols = rom_symbols();
        assert!(!symbols.is_empty());
        assert_eq!(
            symbol_at(RomBank::Sys, 0xC229).map(|symbol| symbol.name.as_str()),
            Some("BASIC_COLD_START")
        );
        assert_eq!(
            symbol_at(RomBank::Sys, 0x0229).map(|symbol| symbol.name.as_str()),
            Some("BASIC_COLD_START")
        );
    }

    #[test]
    fn event_history_is_capped() {
        let mut debugger = DebuggerUi::default();
        for index in 0..EVENT_LIMIT + 20 {
            debugger.record_control(&format!("event {index}"));
        }

        assert_eq!(debugger.events.len(), EVENT_LIMIT);
        assert_eq!(debugger.events.front().unwrap().sequence, 21);
    }
}

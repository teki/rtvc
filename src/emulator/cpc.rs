#![allow(dead_code)]

//! Minimal Amstrad CPC 464: Z80, Gate Array, HD6845S CRTC (type 0), 8255 PPI,
//! and just enough of the AY-3-8912 register latch to scan the keyboard.
//! Cassette input plays TZX/CDT images; there is no audio, cassette output,
//! disk, or expansion ROM support.

use std::collections::{HashMap, HashSet};

use crate::bus::CpuBus;
use crate::cpc_vid::{BORDER_PEN, CpcVid};
use crate::instruction_trace::{
    InstructionEffects, InstructionTrace, InstructionTraceEntry, TraceRegisters,
};
use crate::machine::VidModel;
use crate::tzx::{Tape, TapePlayer};
use crate::z80::Z80;

pub const CPU_CLOCK_HZ: u64 = 4_000_000;
/// 312 lines of 64 µs; one CRTC character clock is 4 CPU T-states.
pub const FRAME_CLOCKS: u64 = 312 * 64 * 4;
/// How long the motor must run before a load counts as started (0.25 s);
/// games may pulse the relay briefly after loading.
const MOTOR_SETTLE_CLOCKS: u64 = CPU_CLOCK_HZ / 4;

const ROM_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0x10000;
const KEYBOARD_ROWS: usize = 10;
/// PPI port B: printer not busy, /EXP high, 50 Hz, distributor id 7
/// (Amstrad). Bit 0 is VSYNC and bit 7 the cassette input.
const PPI_PORT_B: u8 = 0x5E;
/// PPI port C bit 4 switches the cassette motor relay.
const PPI_MOTOR: u8 = 0x10;

pub struct CpcBus {
    os_rom: [u8; ROM_SIZE],
    basic_rom: [u8; ROM_SIZE],
    ram: Box<[u8; RAM_SIZE]>,
    pub vid: CpcVid,
    // Gate Array
    selected_pen: u8,
    lower_rom_enabled: bool,
    upper_rom_enabled: bool,
    interrupt_counter: u8,
    vsync_irq_delay: u8,
    irq_pending: bool,
    // 8255 PPI and AY register latch
    ppi_port_a: u8,
    ppi_port_c: u8,
    ppi_control: u8,
    psg_selected: u8,
    psg_regs: [u8; 16],
    keyboard: [u8; KEYBOARD_ROWS],
    tape: Option<TapePlayer>,
    instruction_effects: Option<InstructionEffects>,
}

impl CpcBus {
    pub fn new() -> Self {
        let mut bus = Self {
            os_rom: [0; ROM_SIZE],
            basic_rom: [0; ROM_SIZE],
            ram: Box::new([0; RAM_SIZE]),
            vid: CpcVid::new(),
            selected_pen: 0,
            lower_rom_enabled: true,
            upper_rom_enabled: true,
            interrupt_counter: 0,
            vsync_irq_delay: 0,
            irq_pending: false,
            ppi_port_a: 0,
            ppi_port_c: 0,
            ppi_control: 0x9B,
            psg_selected: 0,
            psg_regs: [0; 16],
            keyboard: [0xFF; KEYBOARD_ROWS],
            tape: None,
            instruction_effects: None,
        };
        bus.reset();
        bus
    }

    pub fn reset(&mut self) {
        self.vid.reset();
        self.selected_pen = 0;
        self.lower_rom_enabled = true;
        self.upper_rom_enabled = true;
        self.interrupt_counter = 0;
        self.vsync_irq_delay = 0;
        self.irq_pending = false;
        self.ppi_port_a = 0;
        self.ppi_port_c = 0;
        self.ppi_control = 0x9B;
        self.psg_selected = 0;
        self.psg_regs = [0; 16];
        self.keyboard = [0xFF; KEYBOARD_ROWS];
        self.instruction_effects = None;
    }

    pub fn clear_ram(&mut self) {
        self.ram.fill(0);
    }

    pub fn load_roms(&mut self, os: &[u8], basic: &[u8]) -> Result<(), String> {
        for (name, data) in [("OS", os), ("BASIC", basic)] {
            if data.len() != ROM_SIZE {
                return Err(format!(
                    "CPC {name} ROM must be {ROM_SIZE} bytes, got {}",
                    data.len()
                ));
            }
        }
        self.os_rom.copy_from_slice(os);
        self.basic_rom.copy_from_slice(basic);
        Ok(())
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram[..]
    }

    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram[..]
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    pub fn set_key(&mut self, row: usize, bit: usize, pressed: bool) {
        if row >= KEYBOARD_ROWS || bit >= 8 {
            return;
        }
        let mask = 1u8 << bit;
        if pressed {
            self.keyboard[row] &= !mask;
        } else {
            self.keyboard[row] |= mask;
        }
    }

    pub fn motor_on(&self) -> bool {
        self.ppi_port_c & PPI_MOTOR != 0
    }

    pub fn tape(&self) -> Option<&TapePlayer> {
        self.tape.as_ref()
    }

    pub fn tape_mut(&mut self) -> Option<&mut TapePlayer> {
        self.tape.as_mut()
    }

    /// Inserts a tape with PLAY pressed; the motor relay starts it.
    pub fn insert_tape(&mut self, tape: Tape) {
        let mut player = TapePlayer::new(tape, CPU_CLOCK_HZ, false);
        player.play();
        self.tape = Some(player);
    }

    pub fn eject_tape(&mut self) {
        self.tape = None;
    }

    fn advance_tape(&mut self, cycles: u32) {
        if self.motor_on()
            && let Some(tape) = &mut self.tape
        {
            tape.advance(cycles);
        }
    }

    fn acknowledge_interrupt(&mut self) {
        self.irq_pending = false;
        self.interrupt_counter &= 0x1F;
    }

    fn gate_array_write(&mut self, val: u8) {
        match val >> 6 {
            0 => self.selected_pen = val & 0x1F,
            1 => {
                let pen = if self.selected_pen & 0x10 != 0 {
                    BORDER_PEN
                } else {
                    (self.selected_pen & 0x0F) as usize
                };
                self.vid.set_pen(pen, val);
            }
            2 => {
                self.vid.set_mode(val);
                self.lower_rom_enabled = val & 0x04 == 0;
                self.upper_rom_enabled = val & 0x08 == 0;
                if val & 0x10 != 0 {
                    self.interrupt_counter = 0;
                    self.irq_pending = false;
                }
            }
            // RAM banking exists only on 128K machines.
            _ => {}
        }
    }

    fn ppi_port_a_is_input(&self) -> bool {
        self.ppi_control & 0x10 != 0
    }

    fn ppi_write(&mut self, port: u8, val: u8) {
        match port {
            0 => self.ppi_port_a = val,
            1 => {}
            2 => self.ppi_port_c = val,
            _ => {
                if val & 0x80 != 0 {
                    self.ppi_control = val;
                    self.ppi_port_a = 0;
                    self.ppi_port_c = 0;
                } else {
                    let mask = 1u8 << ((val >> 1) & 0x07);
                    if val & 1 != 0 {
                        self.ppi_port_c |= mask;
                    } else {
                        self.ppi_port_c &= !mask;
                    }
                }
            }
        }
        self.update_psg();
    }

    /// Applies the AY bus function selected by PPI port C bits 7..6.
    fn update_psg(&mut self) {
        match self.ppi_port_c >> 6 {
            3 => self.psg_selected = self.ppi_port_a & 0x0F,
            2 if !self.ppi_port_a_is_input() => {
                self.psg_regs[self.psg_selected as usize] = self.ppi_port_a;
            }
            _ => {}
        }
    }

    fn psg_read(&self) -> u8 {
        if self.psg_selected == 14 {
            let row = (self.ppi_port_c & 0x0F) as usize;
            self.keyboard.get(row).copied().unwrap_or(0xFF)
        } else {
            self.psg_regs[self.psg_selected as usize]
        }
    }

    fn ppi_read(&self, port: u8) -> u8 {
        match port {
            0 => {
                if self.ppi_port_a_is_input() {
                    if self.ppi_port_c >> 6 == 1 {
                        self.psg_read()
                    } else {
                        0xFF
                    }
                } else {
                    self.ppi_port_a
                }
            }
            1 => {
                let cassette = self.tape.as_ref().is_some_and(TapePlayer::level);
                PPI_PORT_B | ((cassette as u8) << 7) | self.vid.vsync() as u8
            }
            2 => self.ppi_port_c,
            _ => 0xFF,
        }
    }

    /// Advances the video and the Gate Array interrupt counter by `chars`
    /// 1 MHz clocks.
    fn run_chars(&mut self, chars: u32) {
        for _ in 0..chars {
            let events = self.vid.tick(&self.ram[..]);
            if events.hsync_end {
                self.count_hsync();
            }
            if events.vsync_start {
                self.vsync_irq_delay = 2;
            }
        }
    }

    fn count_hsync(&mut self) {
        self.interrupt_counter += 1;
        if self.interrupt_counter >= 52 {
            self.interrupt_counter = 0;
            self.irq_pending = true;
        }
        if self.vsync_irq_delay != 0 {
            self.vsync_irq_delay -= 1;
            if self.vsync_irq_delay == 0 {
                if self.interrupt_counter >= 32 {
                    self.irq_pending = true;
                }
                self.interrupt_counter = 0;
            }
        }
    }

    fn begin_instruction_effects(&mut self) {
        self.instruction_effects = Some(InstructionEffects::default());
    }

    fn take_instruction_effects(&mut self) -> InstructionEffects {
        self.instruction_effects.take().unwrap_or_default()
    }
}

impl Default for CpcBus {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuBus for CpcBus {
    fn r8(&mut self, addr: u16) -> u8 {
        if addr < 0x4000 && self.lower_rom_enabled {
            self.os_rom[addr as usize]
        } else if addr >= 0xC000 && self.upper_rom_enabled {
            self.basic_rom[addr as usize - 0xC000]
        } else {
            self.ram[addr as usize]
        }
    }

    fn w8(&mut self, addr: u16, val: u8) {
        if let Some(effects) = &mut self.instruction_effects {
            effects.record_memory_write(addr, val);
        }
        self.ram[addr as usize] = val;
    }

    fn out8(&mut self, port: u8, val: u8, high_addr: u8) {
        if let Some(effects) = &mut self.instruction_effects {
            effects.record_port_write(u16::from_be_bytes([high_addr, port]), val);
        }
        // Devices decode individual address lines, so several may respond.
        if high_addr & 0xC0 == 0x40 {
            self.gate_array_write(val);
        }
        if high_addr & 0x40 == 0 {
            match high_addr & 0x03 {
                0 => self.vid.crtc_mut().select(val),
                1 => self.vid.crtc_mut().write(val),
                _ => {}
            }
        }
        // Upper ROM select (A13 low) is ignored: the 464 has only BASIC.
        if high_addr & 0x08 == 0 {
            self.ppi_write(high_addr & 0x03, val);
        }
    }

    fn in8(&mut self, _port: u8, high_addr: u8) -> u8 {
        let mut value = 0xFF;
        if high_addr & 0x40 == 0 && high_addr & 0x03 == 3 {
            value &= self.vid.crtc().read();
        }
        if high_addr & 0x08 == 0 {
            value &= self.ppi_read(high_addr & 0x03);
        }
        value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct MatrixKey {
    row: usize,
    bit: usize,
}

pub struct Cpc {
    pub bus: CpcBus,
    pub z80: Z80,
    pub frame_complete: bool,
    vid_model: VidModel,
    clock: u64,
    next_frame_clock: u64,
    frame_counter: u64,
    last_frame_interrupt_accepted: bool,
    breakpoints: HashSet<u16>,
    pressed_bindings: HashMap<u32, Vec<MatrixKey>>,
    matrix_key_counts: [[u8; 8]; KEYBOARD_ROWS],
    pending_key_release: Vec<MatrixKey>,
    instruction_trace: InstructionTrace,
    motor_on_since: Option<u64>,
}

impl Cpc {
    pub fn new() -> Self {
        Self::new_with_vid_model(VidModel::FastFrame)
    }

    pub fn new_with_vid_model(vid_model: VidModel) -> Self {
        let mut machine = Self {
            bus: CpcBus::new(),
            z80: Z80::new(),
            frame_complete: false,
            vid_model,
            clock: 0,
            next_frame_clock: FRAME_CLOCKS,
            frame_counter: 0,
            last_frame_interrupt_accepted: false,
            breakpoints: HashSet::new(),
            pressed_bindings: HashMap::new(),
            matrix_key_counts: [[0; 8]; KEYBOARD_ROWS],
            pending_key_release: Vec::new(),
            instruction_trace: InstructionTrace::default(),
            motor_on_since: None,
        };
        machine.reset();
        machine
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
        self.next_frame_clock = FRAME_CLOCKS;
        self.frame_counter = 0;
        self.last_frame_interrupt_accepted = false;
        self.release_all_keys();
        self.instruction_trace.clear();
        self.frame_complete = true;
    }

    pub fn hard_reset(&mut self) {
        self.bus.clear_ram();
        self.reset();
    }

    pub fn load_roms(&mut self, os: &[u8], basic: &[u8]) -> Result<(), String> {
        self.bus.load_roms(os, basic)
    }

    pub fn framebuffer(&self) -> &[u32] {
        self.bus.vid.framebuffer()
    }

    /// The (incomplete) frame currently being drawn, for single-stepping.
    pub fn back_buffer(&self) -> &[u32] {
        self.bus.vid.back_buffer()
    }

    pub fn vid_model(&self) -> VidModel {
        self.vid_model
    }

    /// The CRTC is always emulated per character; the model is only recorded.
    pub fn set_vid_model(&mut self, vid_model: VidModel) {
        self.vid_model = vid_model;
    }

    pub fn clock(&self) -> u64 {
        self.clock
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    pub fn last_frame_interrupt_accepted(&self) -> bool {
        self.last_frame_interrupt_accepted
    }

    pub fn instruction_trace(&self) -> &InstructionTrace {
        &self.instruction_trace
    }

    pub fn instruction_trace_mut(&mut self) -> &mut InstructionTrace {
        &mut self.instruction_trace
    }

    pub fn set_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    pub fn clear_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }

    pub fn clear_all_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub fn get_breakpoints(&self) -> Vec<u16> {
        let mut list: Vec<_> = self.breakpoints.iter().copied().collect();
        list.sort_unstable();
        list
    }

    pub fn debug_step_instruction(&mut self) -> u32 {
        let elapsed = self.step_instruction();
        self.release_pending_keys();
        self.frame_complete = true;
        elapsed
    }

    pub fn debug_run_to_interrupt(&mut self, max_cycles: u32) -> (u32, bool) {
        let mut elapsed = 0;
        self.last_frame_interrupt_accepted = false;
        while elapsed < max_cycles && !self.last_frame_interrupt_accepted {
            elapsed = elapsed.saturating_add(self.step_instruction());
        }
        self.release_pending_keys();
        self.frame_complete = true;
        (elapsed, self.last_frame_interrupt_accepted)
    }

    pub fn run_for_a_frame(&mut self) -> bool {
        let frame_target = self.next_frame_clock;
        let mut hit_breakpoint = false;
        while self.clock < frame_target {
            self.step_instruction();
            if self.breakpoints.contains(&self.z80.state.pc) {
                hit_breakpoint = true;
                break;
            }
        }
        if !hit_breakpoint {
            self.next_frame_clock += FRAME_CLOCKS;
            self.frame_counter += 1;
        }
        self.release_pending_keys();
        let frame_ready = self.bus.vid.take_frame_ready();
        if frame_ready || hit_breakpoint {
            self.frame_complete = true;
        }
        hit_breakpoint
    }

    fn step_instruction(&mut self) -> u32 {
        let trace_entry = self.instruction_trace.is_recording().then(|| {
            let pc = self.z80.state.pc;
            let opcode = std::array::from_fn(|offset| self.bus.r8(pc.wrapping_add(offset as u16)));
            self.bus.begin_instruction_effects();
            InstructionTraceEntry {
                sequence: 0,
                clock: self.clock,
                registers: TraceRegisters::from(&self.z80.state),
                opcode,
                main_map: None,
                video_map: None,
                elapsed_cycles: 0,
                interrupt_accepted: false,
                effects: InstructionEffects::default(),
            }
        });
        // The Gate Array stretches every memory access to a 1 µs slot, which
        // is approximated by rounding each instruction up to 4 T-states.
        let cpu_cycles = round_to_slot(self.z80.step(&mut self.bus, 0));
        self.advance(cpu_cycles);

        let mut irq_cycles = 0;
        if self.bus.irq_pending() {
            let accepted = self.z80.irq(&mut self.bus);
            if accepted != 0 {
                self.bus.acknowledge_interrupt();
                self.last_frame_interrupt_accepted = true;
                irq_cycles = round_to_slot(accepted);
                self.advance(irq_cycles);
            }
        }

        if let Some(mut entry) = trace_entry {
            entry.elapsed_cycles = cpu_cycles + irq_cycles;
            entry.interrupt_accepted = irq_cycles != 0;
            entry.effects = self.bus.take_instruction_effects();
            self.instruction_trace.record(entry);
        }
        cpu_cycles + irq_cycles
    }

    fn advance(&mut self, cycles: u32) {
        self.clock += cycles as u64;
        self.bus.run_chars(cycles / 4);
        self.bus.advance_tape(cycles);
        self.motor_on_since = if self.bus.motor_on() {
            self.motor_on_since.or(Some(self.clock))
        } else {
            None
        };
    }

    /// Whether the motor has been running a playing tape for a while.
    pub fn tape_loading(&self) -> bool {
        self.bus
            .tape()
            .is_some_and(|tape| tape.is_playing() && !tape.is_finished())
            && self
                .motor_on_since
                .is_some_and(|since| self.clock - since >= MOTOR_SETTLE_CLOCKS)
    }

    pub fn key_down(&mut self, code: u32) -> bool {
        if self.pressed_bindings.contains_key(&code) {
            return true;
        }
        let Some(binding) = binding_for_code(code) else {
            return false;
        };
        for key in &binding {
            self.press_matrix_key(*key);
        }
        self.pressed_bindings.insert(code, binding);
        true
    }

    pub fn key_up(&mut self, code: u32) {
        if let Some(binding) = self.pressed_bindings.remove(&code) {
            for key in binding {
                self.release_matrix_key(key);
            }
        }
    }

    /// Presses the keys for a typed character until the end of the frame.
    /// A symbol's own shift keys replace whatever host keys produced it (for
    /// example host Shift and `2` for `"`), which would otherwise combine
    /// with them on the matrix.
    pub fn key_press(&mut self, ch: char) {
        self.release_pending_keys();
        if let Some(binding) = binding_for_char(ch) {
            if !ch.is_ascii_alphanumeric() && !matches!(ch, ' ' | '\r') {
                self.release_host_keys();
            }
            for key in &binding {
                self.press_matrix_key(*key);
            }
            self.pending_key_release = binding;
        }
    }

    pub fn focus_change(&mut self, has_focus: bool) {
        if !has_focus {
            self.release_all_keys();
        }
    }

    fn press_matrix_key(&mut self, key: MatrixKey) {
        let count = &mut self.matrix_key_counts[key.row][key.bit];
        *count = count.saturating_add(1);
        self.bus.set_key(key.row, key.bit, true);
    }

    fn release_matrix_key(&mut self, key: MatrixKey) {
        let count = &mut self.matrix_key_counts[key.row][key.bit];
        *count = count.saturating_sub(1);
        if *count == 0 {
            self.bus.set_key(key.row, key.bit, false);
        }
    }

    fn release_host_keys(&mut self) {
        let bindings: Vec<_> = self.pressed_bindings.drain().collect();
        for (_, binding) in bindings {
            for key in binding {
                self.release_matrix_key(key);
            }
        }
    }

    fn release_pending_keys(&mut self) {
        for key in std::mem::take(&mut self.pending_key_release) {
            self.release_matrix_key(key);
        }
    }

    fn release_all_keys(&mut self) {
        self.pressed_bindings.clear();
        self.pending_key_release.clear();
        self.matrix_key_counts = [[0; 8]; KEYBOARD_ROWS];
        self.bus.keyboard = [0xFF; KEYBOARD_ROWS];
    }
}

impl Default for Cpc {
    fn default() -> Self {
        Self::new()
    }
}

fn round_to_slot(cycles: u32) -> u32 {
    (cycles + 3) & !3
}

const fn matrix_key(row: usize, bit: usize) -> MatrixKey {
    MatrixKey { row, bit }
}

const SHIFT: MatrixKey = matrix_key(2, 5);
const CONTROL: MatrixKey = matrix_key(2, 7);
const COPY: MatrixKey = matrix_key(1, 1);

/// Host key codes (browser `keyCode` values) mapped by key position.
fn binding_for_code(code: u32) -> Option<Vec<MatrixKey>> {
    let key = match code {
        16 => SHIFT,
        17 => CONTROL,
        18 | 35 => COPY,
        8 => matrix_key(9, 7),       // DEL
        46 | 36 => matrix_key(2, 0), // CLR
        9 => matrix_key(8, 4),       // TAB
        13 => matrix_key(2, 2),      // RETURN
        20 => matrix_key(8, 6),      // CAPS LOCK
        27 => matrix_key(8, 2),      // ESC
        32 => matrix_key(5, 7),
        37 => matrix_key(1, 0),
        38 => matrix_key(0, 0),
        39 => matrix_key(0, 1),
        40 => matrix_key(0, 2),
        48 => matrix_key(4, 0),
        49 => matrix_key(8, 0),
        50 => matrix_key(8, 1),
        51 => matrix_key(7, 1),
        52 => matrix_key(7, 0),
        53 => matrix_key(6, 1),
        54 => matrix_key(6, 0),
        55 => matrix_key(5, 1),
        56 => matrix_key(5, 0),
        57 => matrix_key(4, 1),
        65 => matrix_key(8, 5),
        66 => matrix_key(6, 6),
        67 => matrix_key(7, 6),
        68 => matrix_key(7, 5),
        69 => matrix_key(7, 2),
        70 => matrix_key(6, 5),
        71 => matrix_key(6, 4),
        72 => matrix_key(5, 4),
        73 => matrix_key(4, 3),
        74 => matrix_key(5, 5),
        75 => matrix_key(4, 5),
        76 => matrix_key(4, 4),
        77 => matrix_key(4, 6),
        78 => matrix_key(5, 6),
        79 => matrix_key(4, 2),
        80 => matrix_key(3, 3),
        81 => matrix_key(8, 3),
        82 => matrix_key(6, 2),
        83 => matrix_key(7, 4),
        84 => matrix_key(6, 3),
        85 => matrix_key(5, 2),
        86 => matrix_key(6, 7),
        87 => matrix_key(7, 3),
        88 => matrix_key(7, 7),
        89 => matrix_key(5, 3),
        90 => matrix_key(8, 7),
        _ => return None,
    };
    Some(vec![key])
}

/// Typed characters mapped to the CPC's own key legends.
fn binding_for_char(ch: char) -> Option<Vec<MatrixKey>> {
    if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '\r' | ' ') {
        return binding_for_code(ch.to_ascii_uppercase() as u32);
    }
    if ch.is_ascii_uppercase() {
        return binding_for_code(ch as u32).map(|mut keys| {
            keys.insert(0, SHIFT);
            keys
        });
    }
    let (key, shifted) = match ch {
        '!' => (matrix_key(8, 0), true),
        '"' => (matrix_key(8, 1), true),
        '#' => (matrix_key(7, 1), true),
        '$' => (matrix_key(7, 0), true),
        '%' => (matrix_key(6, 1), true),
        '&' => (matrix_key(6, 0), true),
        '\'' => (matrix_key(5, 1), true),
        '(' => (matrix_key(5, 0), true),
        ')' => (matrix_key(4, 1), true),
        '_' => (matrix_key(4, 0), true),
        '-' => (matrix_key(3, 1), false),
        '=' => (matrix_key(3, 1), true),
        '^' => (matrix_key(3, 0), false),
        '£' => (matrix_key(3, 0), true),
        '@' => (matrix_key(3, 2), false),
        '|' => (matrix_key(3, 2), true),
        '[' => (matrix_key(2, 1), false),
        '{' => (matrix_key(2, 1), true),
        ']' => (matrix_key(2, 3), false),
        '}' => (matrix_key(2, 3), true),
        ';' => (matrix_key(3, 4), false),
        '+' => (matrix_key(3, 4), true),
        ':' => (matrix_key(3, 5), false),
        '*' => (matrix_key(3, 5), true),
        ',' => (matrix_key(4, 7), false),
        '<' => (matrix_key(4, 7), true),
        '.' => (matrix_key(3, 7), false),
        '>' => (matrix_key(3, 7), true),
        '/' => (matrix_key(3, 6), false),
        '?' => (matrix_key(3, 6), true),
        '\\' => (matrix_key(2, 6), false),
        '`' => (matrix_key(2, 6), true),
        _ => return None,
    };
    Some(if shifted { vec![SHIFT, key] } else { vec![key] })
}

#[cfg(test)]
#[path = "cpc_tests.rs"]
mod tests;

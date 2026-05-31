#![allow(dead_code)]

use std::collections::HashSet;

use crate::cas::TapeBitstreamGenerator;
use crate::hbf::HBF;
use crate::key::Key;
use crate::log::{Log, Logger};
use crate::mmu::{CpuBus, TvcMmu};
use crate::snapshot::{self, Reader, SnapshotError, Writer};
use crate::vid::{Vid, VidModel};
use crate::z80::Z80;

const CPU_CLOCK_HZ: u64 = 3_125_000;
const FRAME_RATE_HZ: u64 = 50;
const FRAME_CLOCKS: u64 = CPU_CLOCK_HZ / FRAME_RATE_HZ;
const SCREEN_LINES: u32 = 312;
const LINE_CLOCKS: u32 = (FRAME_CLOCKS as u32) / SCREEN_LINES;
const LINE_CLOCK_REMAINDER: u32 = (FRAME_CLOCKS as u32) % SCREEN_LINES;
const SYNC_TIMEOUT_HOST_FRAMES: u32 = 8;
const SHARED_CURSOR_SOUND_IT: u8 = 0x10;

struct TapeInterface {
    generator: Option<TapeBitstreamGenerator>,
    play_active: bool,
    start_cycle: u64,
    cycles: u64,
    motor_on: bool,
    output_flip_flop: bool,
}

impl TapeInterface {
    fn new() -> Self {
        Self {
            generator: None,
            play_active: false,
            start_cycle: 0,
            cycles: 0,
            motor_on: false,
            output_flip_flop: false,
        }
    }

    fn reset(&mut self) {
        self.play_active = false;
        self.cycles = 0;
        self.start_cycle = 0;
        self.motor_on = false;
        self.output_flip_flop = false;
    }

    fn set_cycles(&mut self, cycles: u64) {
        self.cycles = cycles;
    }

    fn advance(&mut self, cycles: u64) {
        self.cycles += cycles;
    }

    fn set_motor_from_port5(&mut self, val: u8) {
        self.motor_on = (val & 0xC0) != 0;
    }

    fn toggle_output(&mut self) {
        self.output_flip_flop = !self.output_flip_flop;
    }

    fn play(&mut self, generator: TapeBitstreamGenerator) {
        self.generator = Some(generator);
        self.play_active = true;
        self.start_cycle = self.cycles;
    }

    fn stop(&mut self) {
        self.play_active = false;
    }

    fn is_active(&self) -> bool {
        self.play_active
    }

    fn motor_on(&self) -> bool {
        self.motor_on
    }

    fn cycles(&self) -> u64 {
        self.cycles
    }

    fn state(&self) -> (u64, f32, u8) {
        let elapsed = self.cycles - self.start_cycle;
        let level = if self.play_active {
            self.generator
                .as_ref()
                .map(|generator| generator.get_signal_at_cycle(elapsed))
                .unwrap_or(0.5)
        } else {
            0.5
        };
        let bit = if self.motor_on && self.play_active && level > 0.5 {
            1
        } else {
            0
        };
        (elapsed, level, bit)
    }

    fn input_bit(&mut self) -> u8 {
        let (_, _, mut tape_bit) = self.state();
        if !self.motor_on || !self.play_active {
            tape_bit = 0;
        } else if let Some(ref generator) = self.generator {
            let elapsed = self.cycles - self.start_cycle;
            if elapsed >= generator.total_cycles {
                self.play_active = false;
                tape_bit = 0;
            }
        }
        tape_bit
    }

    fn current_level(&self) -> f32 {
        self.state().1
    }
}

struct SoundTimer {
    freq_low: u8,
    ctrl: u8,
    period_cycles: Option<u64>,
    counter: u64,
    running: bool,
}

impl SoundTimer {
    fn new() -> Self {
        Self {
            freq_low: 0,
            ctrl: 0,
            period_cycles: Some(0x1000 * 16),
            counter: 0,
            running: false,
        }
    }

    fn reset(&mut self) {
        self.freq_low = 0;
        self.ctrl = 0;
        self.period_cycles = Some(0x1000 * 16);
        self.counter = 0;
        self.running = false;
    }

    fn write_low(&mut self, val: u8) {
        self.freq_low = val;
        self.update_period_cycles();
    }

    fn write_control(&mut self, val: u8) {
        self.ctrl = val;
        self.update_period_cycles();
    }

    fn divisor(&self) -> u16 {
        ((self.ctrl as u16 & 0x0F) << 8) | self.freq_low as u16
    }

    fn update_period_cycles(&mut self) {
        let divisor = self.divisor();
        self.period_cycles = if divisor == 0x0FFF {
            None
        } else {
            Some((0x1000u64 - divisor as u64) * 16)
        };
    }

    fn interrupt_enabled(&self) -> bool {
        (self.ctrl & 0x20) != 0
    }

    fn period_cycles(&self) -> Option<u64> {
        self.period_cycles
    }

    fn counter(&self) -> u64 {
        self.counter
    }

    fn running(&self) -> bool {
        self.running
    }

    fn restart(&mut self) {
        self.counter = self.period_cycles.unwrap_or(0);
        self.running = self.counter != 0;
    }

    fn advance(&mut self, cycles: u64) -> bool {
        if !self.running {
            return false;
        }

        let Some(period) = self.period_cycles else {
            self.running = false;
            self.counter = 0;
            return false;
        };

        let mut remaining = cycles;
        let mut fired = false;
        while remaining >= self.counter {
            remaining -= self.counter;
            self.counter = period;
            fired |= self.interrupt_enabled();
        }
        self.counter -= remaining;
        fired
    }
}

struct ExpansionSlots {
    slots: [Option<HBF>; 4],
    // Two-bit extension type identifiers exposed by the 0x5A/0x5E status port.
    type_status: u8,
    // Selected extension memory mapping from port 0x03 bits 6-7.
    selected_mapping: u8,
}

impl ExpansionSlots {
    fn new() -> Self {
        Self {
            slots: [None, None, None, None],
            type_status: 0xFF,
            selected_mapping: 0,
        }
    }

    fn reset(&mut self) {
        self.selected_mapping = 0;
        self.recompute_type_status();
    }

    fn attach_hbf(&mut self, slot: usize, hbf: HBF) {
        if slot >= self.slots.len() {
            return;
        }
        self.slots[slot] = Some(hbf);
        self.recompute_type_status();
    }

    fn recompute_type_status(&mut self) {
        self.type_status = 0xFF;
        for (slot, ext) in self.slots.iter().enumerate() {
            let Some(ext) = ext else {
                continue;
            };
            let hbf_type = ext.get_type();
            self.type_status &= !(3 << (slot * 2));
            self.type_status |= hbf_type << (slot * 2);
        }
    }

    fn selected_mapping(&self) -> u8 {
        self.selected_mapping
    }

    fn set_selected_mapping(&mut self, mapping: u8) {
        self.selected_mapping = mapping & 0x03;
    }

    fn type_status(&self) -> u8 {
        self.type_status
    }

    fn active_mem_slot_mut(&mut self) -> Option<&mut HBF> {
        self.slots
            .get_mut(self.selected_mapping as usize)
            .and_then(Option::as_mut)
    }

    fn read_mem(&mut self, offset: u16) -> u8 {
        self.active_mem_slot_mut()
            .map(|slot| slot.r8(offset))
            .unwrap_or(0xFF)
    }

    fn write_mem(&mut self, offset: u16, val: u8) {
        if let Some(slot) = self.active_mem_slot_mut() {
            slot.w8(offset, val);
        }
    }

    fn port_slot(port: u8) -> Option<usize> {
        match port {
            0x10..=0x1F => Some(0),
            0x20..=0x2F => Some(1),
            0x30..=0x3F => Some(2),
            0x40..=0x4F => Some(3),
            _ => None,
        }
    }

    fn read_port(&mut self, port: u8) -> Option<u8> {
        let slot = Self::port_slot(port)?;
        Some(
            self.slots[slot]
                .as_mut()
                .map(|ext| ext.read_port(port & 0x0F))
                .unwrap_or(0xFF),
        )
    }

    fn write_port(&mut self, port: u8, val: u8) -> bool {
        let Some(slot) = Self::port_slot(port) else {
            return false;
        };
        if let Some(ext) = self.slots[slot].as_mut() {
            ext.write_port(port & 0x0F, val);
        }
        true
    }

    fn slot0(&self) -> Option<&HBF> {
        self.slots[0].as_ref()
    }

    fn slot0_mut(&mut self) -> Option<&mut HBF> {
        self.slots[0].as_mut()
    }

    fn replace_slot0(&mut self, hbf: HBF) {
        self.attach_hbf(0, hbf);
    }
}

pub struct TvcBus {
    pub mmu: TvcMmu,
    pub vid: Vid,
    pub key: Key,
    pub log: Log,
    // Active-low interrupt status bits exposed by the 0x59/0x5D status port.
    pend_it: u8,
    extensions: ExpansionSlots,
    tape: TapeInterface,
    sound: SoundTimer,
    // Last Z80 PC before an I/O instruction, used only for debug logging.
    pub trace_pc: u16,
}

impl TvcBus {
    pub fn new(is_plus: bool) -> Self {
        TvcBus {
            mmu: TvcMmu::new(is_plus),
            vid: Vid::new(),
            key: Key::new(),
            log: Log::new(),
            pend_it: 0x1F,
            extensions: ExpansionSlots::new(),
            tape: TapeInterface::new(),
            sound: SoundTimer::new(),
            trace_pc: 0,
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
        self.vid.reset();
        self.key.reset();
        self.pend_it = 0x1F;
        self.extensions.reset();
        self.tape.reset();
        self.trace_pc = 0;
        self.sound.reset();
    }

    pub fn extension_attach(&mut self, port: u8, ext: HBF) {
        self.extensions.attach_hbf(port as usize, ext);
    }

    fn write_port(&mut self, addr: u8, val: u8) {
        let (tape_elapsed, tape_level, tape_bit) = self.tape.state();
        println!(
            "PW {:02X} <- {:02X} (pc: {:04X}, active: {}, motor: {}, cycles: {}, tape_elapsed: {}, tape_level: {:.1}, tape_bit: {})",
            addr,
            val,
            self.trace_pc,
            self.tape.is_active(),
            self.tape.motor_on(),
            self.tape.cycles(),
            tape_elapsed,
            tape_level,
            tape_bit
        );
        match addr {
            0x00 => self.vid.set_border(val),

            0x02 => self.mmu.set_map(val),

            0x03 => {
                self.key.select_row(val & 0x0F);
                self.extensions.set_selected_mapping(val >> 6);
            }

            0x04 => {
                self.sound.write_low(val);
                self.log_sound_timer_setup("low");
            }

            0x05 => {
                self.sound.write_control(val);
                self.tape.set_motor_from_port5(val);
                self.log_sound_timer_setup("ctrl");
            }

            0x06 => {
                self.vid.set_mode(val & 0x03);
            }

            0x07 => self.pend_it |= SHARED_CURSOR_SOUND_IT,

            0x0C..=0x0F => self.mmu.set_vid_map(val),

            0x60..=0x63 => self.vid.set_palette(addr - 0x60, val),

            0x70 => self.vid.set_reg_idx(val),
            0x71 => self.vid.set_reg(val),

            0x58 => {}
            0x59 => {}
            0x5A => {}
            0x50..=0x57 => {
                self.tape.toggle_output();
            }
            0x5B => {}

            _ => {
                self.extensions.write_port(addr, val);
            }
        }
    }

    fn shared_it_pending(&self) -> bool {
        (self.pend_it & SHARED_CURSOR_SOUND_IT) == 0
    }

    fn request_shared_irq(&mut self) {
        self.pend_it &= !SHARED_CURSOR_SOUND_IT;
    }

    fn advance_tape(&mut self, cycles: u64) {
        self.tape.advance(cycles);
    }

    fn set_tape_cycles(&mut self, cycles: u64) {
        self.tape.set_cycles(cycles);
    }

    fn restart_sound_timer(&mut self) {
        self.sound.restart();
        println!(
            "SOUND timer restart (pc: {:04X}, cycles: {}, divisor: {:03X}, period: {:?}, counter: {}, it_enabled: {}, running: {})",
            self.trace_pc,
            self.tape.cycles(),
            self.sound.divisor(),
            self.sound.period_cycles(),
            self.sound.counter(),
            self.sound.interrupt_enabled(),
            self.sound.running()
        );
    }

    fn advance_sound_timer(&mut self, cycles: u64) {
        let fired = self.sound.advance(cycles);
        if fired {
            println!(
                "SOUND timer fired (pc: {:04X}, cycles: {}, advanced: {}, divisor: {:03X}, period: {:?}, next_counter: {}, it_enabled: {}, pend_it_before: {:02X})",
                self.trace_pc,
                self.tape.cycles(),
                cycles,
                self.sound.divisor(),
                self.sound.period_cycles(),
                self.sound.counter(),
                self.sound.interrupt_enabled(),
                self.pend_it
            );
            self.request_shared_irq();
        }
    }

    fn log_sound_timer_setup(&self, source: &str) {
        println!(
            "SOUND timer setup/{source} (pc: {:04X}, cycles: {}, low: {:02X}, ctrl: {:02X}, divisor: {:03X}, period: {:?}, counter: {}, it_enabled: {}, running: {})",
            self.trace_pc,
            self.tape.cycles(),
            self.sound.freq_low,
            self.sound.ctrl,
            self.sound.divisor(),
            self.sound.period_cycles(),
            self.sound.counter(),
            self.sound.interrupt_enabled(),
            self.sound.running()
        );
    }

    fn tape_input_bit(&mut self) -> u8 {
        self.tape.input_bit()
    }

    pub fn play_tape(&mut self, generator: TapeBitstreamGenerator) {
        self.tape.play(generator);
    }

    pub fn stop_tape(&mut self) {
        self.tape.stop();
    }

    pub fn tape_play_active(&self) -> bool {
        self.tape.is_active()
    }

    pub fn current_tape_level(&self) -> f32 {
        self.tape.current_level()
    }

    fn read_port(&mut self, addr: u8) -> u8 {
        let val = match addr {
            0x58 => self.key.read_row(),
            0x59 | 0x5D => (self.tape_input_bit() << 5) | 0x40 | self.pend_it,
            0x50..=0x57 => {
                self.tape.toggle_output();
                0xFF
            }
            0x5B | 0x5F => {
                self.restart_sound_timer();
                0xFF
            }
            0x5A | 0x5E => self.extensions.type_status(),
            _ => self.extensions.read_port(addr).unwrap_or(0xFF),
        };
        let (tape_elapsed, tape_level, tape_bit) = self.tape.state();
        println!(
            "PR {:02X} -> {:02X} (pc: {:04X}, active: {}, motor: {}, cycles: {}, tape_elapsed: {}, tape_level: {:.1}, tape_bit: {})",
            addr,
            val,
            self.trace_pc,
            self.tape.is_active(),
            self.tape.motor_on(),
            self.tape.cycles(),
            tape_elapsed,
            tape_level,
            tape_bit
        );
        val
    }
}

impl CpuBus for TvcBus {
    fn r8(&mut self, addr: u16) -> u8 {
        if let Some(offset) = self.mmu.ext_card_offset(addr) {
            return self.extensions.read_mem(offset);
        }
        self.mmu.r8(addr)
    }

    fn w8(&mut self, addr: u16, val: u8) {
        if let Some(offset) = self.mmu.ext_card_offset(addr) {
            self.extensions.write_mem(offset, val);
            return;
        }
        self.mmu.w8(addr, val);
    }

    fn out8(&mut self, port: u8, val: u8, _expected_val: u8) {
        self.write_port(port, val);
    }

    fn in8(&mut self, port: u8, _val: u8) -> u8 {
        self.read_port(port)
    }
}

pub struct Tvc {
    pub bus: TvcBus,
    pub z80: Z80,
    pub framebuffer: Vec<u32>,
    pub frame_complete: bool,
    vid_model: VidModel,
    clock: u64,
    sync_timeout_frames: u32,
    line_cycle_debt: u32,
    last_cursor_it_clock: Option<u64>,
    last_blocked_irq_log_clock: Option<u64>,
    breakpoints: HashSet<u16>,
}

impl Tvc {
    pub fn new(is_plus: bool) -> Self {
        Self::new_with_vid_model(is_plus, VidModel::FastFrame)
    }

    pub fn new_with_vid_model(is_plus: bool, vid_model: VidModel) -> Self {
        let mut tvc = Tvc {
            bus: TvcBus::new(is_plus),
            z80: Z80::new(),
            framebuffer: vec![0xFF000000; 608 * 288],
            frame_complete: false,
            vid_model,
            clock: 0,
            sync_timeout_frames: 0,
            line_cycle_debt: 0,
            last_cursor_it_clock: None,
            last_blocked_irq_log_clock: None,
            breakpoints: HashSet::new(),
        };
        tvc.reset();
        tvc
    }

    pub fn vid_model(&self) -> VidModel {
        self.vid_model
    }

    pub fn set_vid_model(&mut self, vid_model: VidModel) {
        self.vid_model = vid_model;
    }

    pub fn save_snapshot(&self) -> Vec<u8> {
        let mut chunks = Vec::new();

        let mut meta = Writer::new();
        meta.u8(self.bus.mmu.is_plus() as u8);
        meta.u8(match self.vid_model {
            VidModel::FastFrame => 0,
            VidModel::Interleaved => 1,
            VidModel::Line => 2,
        });
        meta.u64(self.clock);
        meta.u8(self.frame_complete as u8);
        chunks.push((*b"META", meta.into_inner()));

        let mut cpu = Writer::new();
        cpu.raw_bytes(&self.z80.state.r8);
        for reg in self.z80.state.r16 {
            cpu.u16(reg);
        }
        cpu.u8(self.z80.state.halted);
        cpu.u8(self.z80.state.im);
        cpu.u8(self.z80.state.iff1);
        cpu.u8(self.z80.state.iff2);
        chunks.push((*b"CPUZ", cpu.into_inner()));

        let mut mmu = Writer::new();
        self.bus.mmu.write_snapshot(&mut mmu);
        chunks.push((*b"MMU ", mmu.into_inner()));

        let mut vid = Writer::new();
        self.bus.vid.write_snapshot(&mut vid);
        chunks.push((*b"VID ", vid.into_inner()));

        if let Some(ext) = self.bus.extensions.slot0() {
            let mut hbf = Writer::new();
            ext.write_snapshot(&mut hbf);
            chunks.push((*b"HBF ", hbf.into_inner()));
        }

        let mut bus = Writer::new();
        bus.u8(self.bus.pend_it);
        bus.u8(self.bus.extensions.type_status());
        bus.u8(self.bus.extensions.selected_mapping());
        chunks.push((*b"BUS ", bus.into_inner()));

        snapshot::write_file(&chunks)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> snapshot::Result<()> {
        let chunks = snapshot::read_file(data)?;
        let meta = chunks
            .iter()
            .find(|chunk| chunk.id == *b"META")
            .ok_or(SnapshotError::InvalidChunk("META"))?;
        let mut meta = Reader::new(meta.data);
        let is_plus = meta.u8()? != 0;
        let vid_model = match meta.u8()? {
            0 => VidModel::FastFrame,
            1 => VidModel::Interleaved,
            2 => VidModel::Line,
            _ => {
                return Err(SnapshotError::InvalidData(
                    "unknown video model".to_string(),
                ));
            }
        };
        let clock = meta.u64()?;
        let frame_complete = meta.u8()? != 0;

        *self = Tvc::new_with_vid_model(is_plus, vid_model);
        self.clock = clock;
        self.bus.set_tape_cycles(clock);
        self.frame_complete = frame_complete;

        for chunk in chunks {
            let mut reader = Reader::new(chunk.data);
            match &chunk.id {
                b"META" => {}
                b"CPUZ" => {
                    self.z80.state.r8.copy_from_slice(reader.raw_bytes(22)?);
                    for reg in &mut self.z80.state.r16 {
                        *reg = reader.u16()?;
                    }
                    self.z80.state.halted = reader.u8()?;
                    self.z80.state.im = reader.u8()?;
                    self.z80.state.iff1 = reader.u8()?;
                    self.z80.state.iff2 = reader.u8()?;
                }
                b"MMU " => self.bus.mmu.read_snapshot(&mut reader)?,
                b"VID " => self.bus.vid.read_snapshot(&mut reader)?,
                b"HBF " => {
                    self.bus
                        .extensions
                        .replace_slot0(HBF::read_snapshot(&mut reader)?);
                }
                b"BUS " => {
                    self.bus.pend_it = reader.u8()?;
                    self.bus.extensions.type_status = reader.u8()?;
                    self.bus.extensions.set_selected_mapping(reader.u8()?);
                }
                _ => {}
            }
        }
        self.bus.key.reset();
        self.bus.log.clear();
        Ok(())
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
        self.sync_timeout_frames = 0;
        self.line_cycle_debt = 0;
        self.last_cursor_it_clock = None;
        self.last_blocked_irq_log_clock = None;
    }

    pub fn load_cas(&mut self, data: &[u8]) -> bool {
        if data.len() < 144 || data[0] != 0x11 {
            return false;
        }
        let savemap = self.bus.mmu.get_map_val();
        self.bus.mmu.set_map(0xB0);
        for i in 144..data.len() {
            let addr = (6639 + i - 144) as u16;
            self.bus.w8(addr, data[i]);
        }
        self.bus.mmu.set_map(savemap);
        true
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        if name.contains("DOS") {
            let hbf = HBF::new(data);
            self.bus.extension_attach(0, hbf);
        } else {
            self.bus.mmu.add_rom(name, data);
        }
    }

    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.bus.mmu.load_cart_rom(data);
    }

    pub fn load_disk(&mut self, name: &str, data: &[u8]) {
        if let Some(ext) = self.bus.extensions.slot0_mut() {
            ext.load_disk(name, data);
        }
    }

    pub fn key_down(&mut self, code: u32) -> bool {
        self.bus.key.key_down(code)
    }

    pub fn key_up(&mut self, code: u32) {
        self.bus.key.key_up(code);
    }

    pub fn key_press(&mut self, ch: char) {
        self.bus.key.key_press(ch);
    }

    pub fn log_entries(&self) -> &[String] {
        &self.bus.log.entries
    }

    pub fn clear_log(&mut self) {
        self.bus.log.clear();
    }

    pub fn focus_change(&mut self, has_focus: bool) {
        if !has_focus {
            self.bus.key.reset();
        }
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

    fn draw_sync_timeout(&mut self) {
        let phase = (self.sync_timeout_frames as usize * 11) % 96;
        for (idx, pixel) in self.framebuffer.iter_mut().enumerate() {
            let x = idx % 608;
            let y = idx / 608;
            let stripe = (x + y * 2 + phase) % 96 < 8;
            *pixel = if stripe { 0xFFFFFFFF } else { 0xFF000000 };
        }
    }

    fn advance_video_for(&mut self, cycles: u32) -> bool {
        let cursor_it = self.bus.vid.stream_some(self.bus.mmu.get_vid_mem(), cycles);
        if cursor_it {
            self.request_cursor_irq();
        }
        self.bus.vid.render_stream(&mut self.framebuffer, 608)
    }

    fn service_pending_shared_irq(&mut self) -> u32 {
        if self.bus.shared_it_pending() {
            self.service_shared_irq()
        } else {
            0
        }
    }

    fn run_cpu_budget(&mut self, budget: u32, sync_video: bool) -> (bool, bool, u32) {
        let mut do_break = false;
        let mut frame_complete = false;
        let mut elapsed = 0u32;

        while !do_break && elapsed < budget {
            self.bus.trace_pc = self.z80.state.r16[11];
            let cpu_time = self.z80.step(&mut self.bus, 0);

            if self.breakpoints.contains(&self.z80.state.r16[11]) {
                do_break = true;
            }

            self.clock += cpu_time as u64;
            self.bus.advance_tape(cpu_time as u64);
            self.bus.advance_sound_timer(cpu_time as u64);
            elapsed += cpu_time;

            if sync_video {
                frame_complete |= self.advance_video_for(cpu_time);
            }

            elapsed += self.service_pending_shared_irq();
        }

        (do_break, frame_complete, elapsed)
    }

    pub fn run_for_a_frame(&mut self) -> bool {
        let (do_break, frame_complete) = match self.vid_model {
            VidModel::FastFrame => {
                let (do_break, _, _) = self.run_cpu_budget(FRAME_CLOCKS as u32, false);
                if self.bus.vid.is_initialized() && self.bus.vid.cursor_enabled() {
                    // sync_video=false skips video generation during CPU execution, so request the video IRQ here.
                    self.request_cursor_irq();
                    self.service_shared_irq();
                }
                let vidmem = self.bus.mmu.get_vid_mem();
                self.bus.vid.draw_frame(vidmem, &mut self.framebuffer);
                (do_break, true)
            }
            VidModel::Line => {
                let mut do_break = false;
                let mut frame_complete = false;
                let mut line_error = 0u32;
                for _ in 0..SCREEN_LINES {
                    let mut line_clocks = LINE_CLOCKS;
                    line_error += LINE_CLOCK_REMAINDER;
                    if line_error >= SCREEN_LINES {
                        line_clocks += 1;
                        line_error -= SCREEN_LINES;
                    }

                    let cpu_budget = if self.line_cycle_debt > line_clocks {
                        0
                    } else {
                        line_clocks - self.line_cycle_debt
                    };
                    let (line_break, line_complete, elapsed) =
                        self.run_cpu_budget(cpu_budget, false);
                    let mut elapsed = elapsed;
                    do_break |= line_break;
                    frame_complete |= self.advance_video_for(line_clocks);
                    elapsed += self.service_pending_shared_irq();
                    let line_cpu_time = self.line_cycle_debt + elapsed;
                    self.line_cycle_debt = if line_cpu_time > line_clocks {
                        line_cpu_time - line_clocks
                    } else {
                        0
                    };
                    if do_break {
                        break;
                    }
                    frame_complete |= line_complete;
                }
                (do_break, frame_complete)
            }
            VidModel::Interleaved => {
                let (do_break, frame_complete, _) = self.run_cpu_budget(FRAME_CLOCKS as u32, true);
                (do_break, frame_complete)
            }
        };

        if frame_complete {
            self.sync_timeout_frames = 0;
            self.frame_complete = true;
        } else {
            self.sync_timeout_frames += 1;
            if self.sync_timeout_frames >= SYNC_TIMEOUT_HOST_FRAMES {
                self.draw_sync_timeout();
            }
            self.frame_complete = true;
        }

        do_break
    }

    fn request_cursor_irq(&mut self) {
        if let Some(last_clock) = self.last_cursor_it_clock {
            self.bus.log.log(&format!(
                "cursor_it at {} (+{} cycles)",
                self.clock,
                self.clock - last_clock
            ));
        } else {
            self.bus
                .log
                .log(&format!("cursor_it at {} (first)", self.clock));
        }
        self.last_cursor_it_clock = Some(self.clock);
        self.bus.request_shared_irq();
    }

    fn service_shared_irq(&mut self) -> u32 {
        let pc_before = self.z80.state.r16[11];
        let iff1_before = self.z80.state.iff1;
        let iff2_before = self.z80.state.iff2;
        let pend_before = self.bus.pend_it;
        let irq_duration = self.z80.irq(&mut self.bus);
        if irq_duration > 0 {
            println!(
                "IRQ accepted (pc: {:04X}, cycles: {}, duration: {}, pend_it: {:02X}, iff1: {}, iff2: {}, im: {}, new_pc: {:04X})",
                pc_before,
                self.clock,
                irq_duration,
                pend_before,
                iff1_before,
                iff2_before,
                self.z80.state.im,
                self.z80.state.r16[11]
            );
            self.clock += irq_duration as u64;
            let irq_duration = irq_duration as u64;
            self.bus.advance_tape(irq_duration);
            self.bus.advance_sound_timer(irq_duration);
            self.bus.vid.stream_some(
                self.bus.mmu.get_vid_mem(),
                irq_duration.try_into().unwrap_or(u32::MAX),
            );
            self.bus.vid.render_stream(&mut self.framebuffer, 608);
        } else {
            let should_log = self
                .last_blocked_irq_log_clock
                .map(|last_clock| self.clock.saturating_sub(last_clock) >= 1024)
                .unwrap_or(true);
            if should_log {
                println!(
                    "IRQ pending but not accepted (pc: {:04X}, cycles: {}, pend_it: {:02X}, iff1: {}, iff2: {}, im: {}, halted: {})",
                    pc_before,
                    self.clock,
                    pend_before,
                    iff1_before,
                    iff2_before,
                    self.z80.state.im,
                    self.z80.state.halted
                );
                self.last_blocked_irq_log_clock = Some(self.clock);
            }
        }
        irq_duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trips_core_state() {
        let mut tvc = Tvc::new_with_vid_model(true, VidModel::Interleaved);
        tvc.z80.state.r16[11] = 0x1234;
        tvc.bus.mmu.w8(0x4000, 0xAB);
        tvc.bus.vid.set_border(0x55);
        tvc.bus.pend_it = 0x0F;

        let snapshot = tvc.save_snapshot();
        let mut restored = Tvc::new(false);
        restored.load_snapshot(&snapshot).unwrap();

        assert!(restored.bus.mmu.is_plus());
        assert_eq!(restored.vid_model(), VidModel::Interleaved);
        assert_eq!(restored.z80.state.r16[11], 0x1234);
        assert_eq!(restored.bus.mmu.r8(0x4000), 0xAB);
        assert_eq!(restored.bus.pend_it, 0x0F);
    }

    #[test]
    fn sound_timer_sets_shared_interrupt_when_enabled() {
        let mut bus = TvcBus::new(true);
        bus.write_port(0x04, 0xFE);
        bus.write_port(0x05, 0x2F);

        bus.read_port(0x5B);
        assert_eq!(bus.pend_it & SHARED_CURSOR_SOUND_IT, SHARED_CURSOR_SOUND_IT);

        bus.advance_sound_timer(31);
        assert_eq!(bus.pend_it & SHARED_CURSOR_SOUND_IT, SHARED_CURSOR_SOUND_IT);

        bus.advance_sound_timer(1);
        assert_eq!(bus.pend_it & SHARED_CURSOR_SOUND_IT, 0);

        bus.write_port(0x07, 0xFF);
        assert_eq!(bus.pend_it & SHARED_CURSOR_SOUND_IT, SHARED_CURSOR_SOUND_IT);
    }

    #[test]
    fn sound_timer_without_interrupt_enable_does_not_set_shared_interrupt() {
        let mut bus = TvcBus::new(true);
        bus.write_port(0x04, 0xFE);
        bus.write_port(0x05, 0x0F);

        bus.read_port(0x5B);
        bus.advance_sound_timer(32);

        assert_eq!(bus.pend_it & SHARED_CURSOR_SOUND_IT, SHARED_CURSOR_SOUND_IT);
    }
}

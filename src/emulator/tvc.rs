#![allow(dead_code)]

use std::collections::HashSet;

use crate::bus::CpuBus;
use crate::cas::TapeBitstreamGenerator;
use crate::expansion::ExpansionSlots;
use crate::hbf::HBF;
use crate::key::Key;
use crate::log::{Log, Logger};
use crate::mmu::{RomBank, TvcMmu};
use crate::snapshot;
use crate::sound::SoundTimer;
use crate::tape::TapeInterface;
use crate::vid::{Vid, VidModel};
use crate::z80::Z80;

const CPU_CLOCK_HZ: u64 = 3_125_000;
const FRAME_RATE_HZ: u64 = 50;
const FRAME_CLOCKS: u64 = CPU_CLOCK_HZ / FRAME_RATE_HZ;
const SYNC_TIMEOUT_HOST_FRAMES: u32 = 8;
const SHARED_CURSOR_SOUND_IT: u8 = 0x10;
pub const DEBUG_RUN_TO_IRQ_MAX_CYCLES: u32 = FRAME_CLOCKS as u32 * 2;

pub struct TvcBus {
    pub mmu: TvcMmu,
    pub vid: Vid,
    pub key: Key,
    pub log: Log,
    // Active-low interrupt status bits exposed by the 0x59/0x5D status port.
    pub(crate) pend_it: u8,
    pub(crate) extensions: ExpansionSlots,
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
        match addr {
            0x00 => self.vid.set_border(val),

            0x02 => self.mmu.set_map(val),

            0x03 => {
                self.key.select_row(val & 0x0F);
                self.extensions.set_selected_mapping(val >> 6);
            }

            0x04 => {
                let was_on = self.sound.audible_oscillator_enabled();
                let old_divisor = self.sound.divisor();
                self.sound.write_low(val);
                self.log_sound_change(was_on, old_divisor, addr);
            }

            0x05 => {
                let was_on = self.sound.audible_oscillator_enabled();
                let old_divisor = self.sound.divisor();
                self.sound.write_control(val);
                self.log_sound_change(was_on, old_divisor, addr);
                self.tape.set_motor_from_port5(val);
            }

            0x06 => {
                self.vid.set_mode(val & 0x03);
                let old_amplitude = self.sound.amplitude();
                self.sound.write_amplitude(val);
                self.log_sound_volume_change(old_amplitude);
            }

            0x07 => self.pend_it |= SHARED_CURSOR_SOUND_IT,

            0x0C..=0x0F => self.mmu.set_vid_map(val),

            0x60..=0x63 => self.vid.set_palette(addr - 0x60, val),

            0x70..=0x7F => self.vid.write_crtc_port(addr, val),

            0x58 => {}
            0x59 => {}
            0x5A => {}
            0x50..=0x57 => {
                self.tape.toggle_output();
            }
            0x5B => {}

            _ => {
                if addr >= 0x10 && addr <= 0x14 {
                    match addr & 0x0F {
                        0x00 => {
                            let cmd_name = match val >> 4 {
                                0x00 => "Restore",
                                0x01 => "Seek",
                                0x02 | 0x03 => "Step In",
                                0x04 | 0x05 => "Step Out",
                                0x06 | 0x07 => "Step",
                                0x08 | 0x09 => "Read Sector",
                                0x0A | 0x0B => "Write Sector",
                                0x0C => "Read Address",
                                0x0D => "Force Interrupt",
                                0x0E => "Read Track",
                                0x0F => "Write Track",
                                _ => "Unknown",
                            };
                            self.log.log(&format!(
                                "FDC command: 0x{:02X} [{}] (pc 0x{:04X})",
                                val, cmd_name, self.trace_pc
                            ));
                        }
                        0x01 => self
                            .log
                            .log(&format!("FDC track: {} (pc 0x{:04X})", val, self.trace_pc)),
                        0x02 => self
                            .log
                            .log(&format!("FDC sector: {} (pc 0x{:04X})", val, self.trace_pc)),
                        0x04 => {
                            let drive = if (val & 1) != 0 {
                                "A:"
                            } else if (val & 2) != 0 {
                                "B:"
                            } else {
                                "none"
                            };
                            let side = (val & 0x80) >> 7;
                            self.log.log(&format!(
                                "FDC select: drive {}, side {} (pc 0x{:04X})",
                                drive, side, self.trace_pc
                            ));
                        }
                        _ => {}
                    }
                }
                self.extensions.write_port(addr, val);
            }
        }
    }

    fn log_sound_change(&mut self, was_on: bool, old_divisor: u16, port: u8) {
        let is_on = self.sound.audible_oscillator_enabled();
        let divisor = self.sound.divisor();
        if was_on != is_on {
            if is_on {
                self.log.log(&format!(
                    "sound on: {} (divisor 0x{divisor:03X}, port 0x{port:02X}, pc 0x{:04X})",
                    self.sound_frequency_label(),
                    self.trace_pc
                ));
            } else {
                self.log
                    .log(&format!("sound off (pc 0x{:04X})", self.trace_pc));
            }
        } else if is_on && old_divisor != divisor {
            self.log.log(&format!(
                "sound freq: {} (divisor 0x{divisor:03X}, port 0x{port:02X}, pc 0x{:04X})",
                self.sound_frequency_label(),
                self.trace_pc
            ));
        }
    }

    fn log_sound_volume_change(&mut self, old_amplitude: u8) {
        let amplitude = self.sound.amplitude();
        if old_amplitude != amplitude {
            self.log.log(&format!(
                "sound volume: {amplitude}/15 (pc 0x{:04X})",
                self.trace_pc
            ));
        }
    }

    fn sound_frequency_label(&self) -> String {
        match self.sound.frequency_hz() {
            Some(freq) => format!("{freq:.2} Hz"),
            None => "stopped".to_string(),
        }
    }

    fn shared_it_pending(&self) -> bool {
        (self.pend_it & SHARED_CURSOR_SOUND_IT) == 0
    }

    fn request_shared_irq(&mut self) {
        self.pend_it &= !SHARED_CURSOR_SOUND_IT;
    }

    pub(crate) fn advance_tape(&mut self, cycles: u64) {
        self.tape.advance(cycles);
    }

    pub(crate) fn set_tape_cycles(&mut self, cycles: u64) {
        self.tape.set_cycles(cycles);
    }

    fn restart_sound_timer(&mut self) {
        self.sound.restart();
    }

    pub(crate) fn advance_sound_timer(&mut self, cycles: u64) {
        if self.sound.advance(cycles) {
            self.request_shared_irq();
        }
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

    pub fn tape_progress_percent(&self) -> Option<u8> {
        self.tape.progress_percent()
    }

    pub(crate) fn tape_motor_on(&self) -> bool {
        self.tape.motor_on()
    }

    pub(crate) fn tape_elapsed_cycles(&self) -> u64 {
        self.tape.state().0
    }

    pub(crate) fn write_tape_snapshot(&self, w: &mut snapshot::Writer) {
        self.tape.write_snapshot(w);
    }

    pub(crate) fn read_tape_snapshot(
        &mut self,
        r: &mut snapshot::Reader<'_>,
    ) -> snapshot::Result<()> {
        self.tape.read_snapshot(r)
    }

    pub(crate) fn write_sound_snapshot(&self, w: &mut snapshot::Writer) {
        self.sound.write_snapshot(w);
    }

    pub(crate) fn read_sound_snapshot(
        &mut self,
        r: &mut snapshot::Reader<'_>,
    ) -> snapshot::Result<()> {
        self.sound.read_snapshot(r)
    }

    pub fn sound_sample_rate(&self) -> u32 {
        self.sound.sample_rate()
    }

    pub fn take_audio_samples(&mut self) -> Vec<f32> {
        self.sound.take_samples()
    }

    pub(crate) fn restore_tape_motor_from_rom_shadow(&mut self) {
        let port5_shadow = self.mmu.r8(0x0B12);
        self.tape.set_motor_from_port5(port5_shadow);
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
            0x70..=0x7F => self.vid.read_crtc_port(addr),
            _ => self.extensions.read_port(addr).unwrap_or(0xFF),
        };
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
    pub(crate) vid_model: VidModel,
    pub(crate) clock: u64,
    sync_timeout_frames: u32,
    breakpoints: HashSet<u16>,
    tracepoints: HashSet<(RomBank, u16)>,
    trace_events: Vec<ExecutionTrace>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionTrace {
    pub bank: RomBank,
    pub pc: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebugRunToIrqResult {
    pub elapsed_cycles: u32,
    pub interrupt_accepted: bool,
}

impl Tvc {
    pub fn new(is_plus: bool) -> Self {
        Self::new_with_vid_model(is_plus, VidModel::Interleaved)
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
            breakpoints: HashSet::new(),
            tracepoints: HashSet::new(),
            trace_events: Vec::new(),
        };
        tvc.reset();
        tvc
    }

    pub fn vid_model(&self) -> VidModel {
        self.vid_model
    }

    pub fn is_plus(&self) -> bool {
        self.bus.mmu.is_plus()
    }

    pub fn has_hbf(&self) -> bool {
        self.bus.extensions.slot0().is_some()
    }

    pub fn set_vid_model(&mut self, vid_model: VidModel) {
        self.vid_model = vid_model;
    }

    pub fn fast_boot(&self) -> bool {
        self.bus.mmu.fast_boot()
    }

    pub fn set_fast_boot(&mut self, enabled: bool) {
        self.bus.mmu.set_fast_boot(enabled);
    }

    pub fn save_snapshot(&self) -> Vec<u8> {
        crate::tvc_snapshot::save(self)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> snapshot::Result<()> {
        crate::tvc_snapshot::load(self, data)
    }

    pub(crate) fn prepare_snapshot_load(
        &mut self,
        vid_model: VidModel,
        clock: u64,
        frame_complete: bool,
    ) {
        self.vid_model = vid_model;
        self.clock = clock;
        self.bus.set_tape_cycles(clock);
        self.frame_complete = frame_complete;
        self.sync_timeout_frames = 0;
    }

    pub fn sound_sample_rate(&self) -> u32 {
        self.bus.sound_sample_rate()
    }

    pub fn take_audio_samples(&mut self) -> Vec<f32> {
        self.bus.take_audio_samples()
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
        self.sync_timeout_frames = 0;
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

    pub fn load_disk(&mut self, drive: usize, name: &str, data: &[u8]) {
        if let Some(ext) = self.bus.extensions.slot0_mut() {
            ext.load_disk(drive, name, data);
        }
    }

    pub fn disk_dirty(&self, drive: usize) -> bool {
        self.bus
            .extensions
            .slot0()
            .is_some_and(|ext| ext.disk_dirty(drive))
    }

    pub fn clear_disk_dirty(&mut self, drive: usize) {
        if let Some(ext) = self.bus.extensions.slot0_mut() {
            ext.clear_disk_dirty(drive);
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

    pub fn get_breakpoints(&self) -> Vec<u16> {
        let mut list: Vec<u16> = self.breakpoints.iter().copied().collect();
        list.sort();
        list
    }

    pub fn set_tracepoints(&mut self, tracepoints: &[(RomBank, u16)]) {
        self.tracepoints.clear();
        self.tracepoints.extend(tracepoints.iter().copied());
        if self.tracepoints.is_empty() {
            self.trace_events.clear();
        }
    }

    pub fn tracepoints_enabled(&self) -> bool {
        !self.tracepoints.is_empty()
    }

    pub fn take_trace_events(&mut self) -> Vec<ExecutionTrace> {
        std::mem::take(&mut self.trace_events)
    }

    pub fn debug_step_instruction(&mut self) -> u32 {
        let sync_video = self.vid_model == VidModel::Interleaved;
        let (elapsed, frame_complete, _) = self.step_instruction(sync_video);
        self.finish_debug_video(frame_complete);
        elapsed
    }

    pub fn debug_run_to_interrupt(&mut self, max_cycles: u32) -> DebugRunToIrqResult {
        let mut elapsed_cycles = 0u32;
        let mut frame_complete = false;
        let mut interrupt_accepted = false;

        while elapsed_cycles < max_cycles && !interrupt_accepted {
            // Even FastFrame needs CRTC timing here so a video IRQ can end the run.
            let (elapsed, completed, accepted) = self.step_instruction(true);
            elapsed_cycles = elapsed_cycles.saturating_add(elapsed);
            frame_complete |= completed;
            interrupt_accepted = accepted;
        }

        self.finish_debug_video(frame_complete);
        DebugRunToIrqResult {
            elapsed_cycles,
            interrupt_accepted,
        }
    }

    fn finish_debug_video(&mut self, mut frame_complete: bool) {
        if self.vid_model == VidModel::FastFrame {
            let vidmem = self.bus.mmu.get_vid_mem();
            self.bus.vid.draw_frame(vidmem, &mut self.framebuffer);
            frame_complete = true;
        }
        if frame_complete {
            self.sync_timeout_frames = 0;
            self.frame_complete = true;
        }
    }

    fn step_instruction(&mut self, sync_video: bool) -> (u32, bool, bool) {
        self.bus.trace_pc = self.z80.state.r16[11];
        let cpu_time = self.z80.step(&mut self.bus, 0);
        self.record_tracepoint();

        self.clock += cpu_time as u64;
        self.bus.advance_tape(cpu_time as u64);
        self.bus.advance_sound_timer(cpu_time as u64);

        let mut frame_complete = false;
        if sync_video {
            frame_complete = self.advance_video_for(cpu_time);
        }

        let (irq_time, irq_frame_complete) = self.service_pending_shared_irq();
        (
            cpu_time + irq_time,
            frame_complete || irq_frame_complete,
            irq_time > 0,
        )
    }

    fn record_tracepoint(&mut self) {
        if self.tracepoints.is_empty() || self.trace_events.len() >= 256 {
            return;
        }
        let pc = self.z80.state.r16[11];
        let Some(bank) = self.bus.mmu.mapped_rom_bank(pc) else {
            return;
        };
        if self.tracepoints.contains(&(bank, pc))
            && self
                .trace_events
                .last()
                .is_none_or(|event| event.bank != bank || event.pc != pc)
        {
            self.trace_events.push(ExecutionTrace { bank, pc });
        }
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

    fn service_pending_shared_irq(&mut self) -> (u32, bool) {
        if self.bus.shared_it_pending() {
            self.service_shared_irq()
        } else {
            (0, false)
        }
    }

    fn run_cpu_budget(&mut self, budget: u32, sync_video: bool) -> (bool, bool, u32) {
        let mut do_break = false;
        let mut frame_complete = false;
        let mut elapsed = 0u32;

        while !do_break && elapsed < budget {
            let (instruction_time, instruction_frame_complete, _) =
                self.step_instruction(sync_video);

            if self.breakpoints.contains(&self.z80.state.r16[11]) {
                do_break = true;
            }

            elapsed += instruction_time;
            frame_complete |= instruction_frame_complete;
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
        self.bus.request_shared_irq();
    }

    fn service_shared_irq(&mut self) -> (u32, bool) {
        let irq_duration = self.z80.irq(&mut self.bus);
        let mut frame_complete = false;
        if irq_duration > 0 {
            self.clock += irq_duration as u64;
            let irq_duration = irq_duration as u64;
            self.bus.advance_tape(irq_duration);
            self.bus.advance_sound_timer(irq_duration);
            self.bus.vid.stream_some(
                self.bus.mmu.get_vid_mem(),
                irq_duration.try_into().unwrap_or(u32::MAX),
            );
            frame_complete = self.bus.vid.render_stream(&mut self.framebuffer, 608);
        }
        (irq_duration, frame_complete)
    }
}

#[cfg(test)]
#[path = "tvc_tests.rs"]
mod tests;

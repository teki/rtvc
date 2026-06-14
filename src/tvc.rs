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
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trips_core_state() {
        let mut tvc = Tvc::new_with_vid_model(true, VidModel::Interleaved);
        tvc.z80.state.r16[11] = 0x1234;
        tvc.bus.mmu.w8(0x4000, 0xAB);
        tvc.bus.vid.set_border(0x55);
        tvc.bus.pend_it = 0x0F;
        tvc.bus.write_port(0x04, 0xDC);
        tvc.bus.write_port(0x05, 0x6F);
        tvc.bus.write_port(0x06, 0x3C);
        tvc.bus.read_port(0x5B);
        tvc.bus.advance_sound_timer(11);
        let sound_counter = tvc.bus.sound.counter();

        let snapshot = tvc.save_snapshot();
        let mut restored = Tvc::new(true);
        restored.load_snapshot(&snapshot).unwrap();

        assert!(restored.bus.mmu.is_plus());
        assert_eq!(restored.vid_model(), VidModel::Interleaved);
        assert_eq!(restored.z80.state.r16[11], 0x1234);
        assert_eq!(restored.bus.mmu.r8(0x4000), 0xAB);
        assert_eq!(restored.bus.pend_it, 0x0F);
        assert!(restored.bus.tape_motor_on());
        assert_eq!(restored.bus.sound.freq_low, 0xDC);
        assert_eq!(restored.bus.sound.ctrl, 0x6F);
        assert_eq!(restored.bus.sound.amplitude(), 0x0F);
        assert_eq!(restored.bus.sound.counter(), sound_counter);
        assert!(restored.bus.sound.running());
        assert_eq!(
            restored.bus.sound.filter_state_bits(),
            tvc.bus.sound.filter_state_bits()
        );
    }

    #[test]
    fn snapshot_stores_only_model_ram_and_mutable_hbf_state() {
        let tvc = Tvc::new(false);
        let snapshot = tvc.save_snapshot();
        let chunks = crate::snapshot::read_file(&snapshot).unwrap();
        assert_eq!(
            chunks
                .iter()
                .find(|chunk| chunk.id == *b"MMU ")
                .unwrap()
                .data
                .len(),
            3 + 5 * 0x4000
        );
        assert!(chunks.iter().all(|chunk| chunk.id != *b"HBF "));

        let mut plus = Tvc::new(true);
        plus.add_rom("D_TVCDOS.128", &[0xA5; 0x4000]);
        plus.load_disk("large.dsk", &[0xE5; 368_640]);
        let snapshot = plus.save_snapshot();
        let chunks = crate::snapshot::read_file(&snapshot).unwrap();
        assert_eq!(
            chunks
                .iter()
                .find(|chunk| chunk.id == *b"MMU ")
                .unwrap()
                .data
                .len(),
            3 + 8 * 0x4000
        );
        assert!(
            chunks
                .iter()
                .find(|chunk| chunk.id == *b"HBF ")
                .unwrap()
                .data
                .len()
                < 0x1200
        );
    }

    #[test]
    fn snapshot_load_keeps_runtime_roms() {
        let mut source = Tvc::new(false);
        source.add_rom("TVC12_D4.64K", &[0x11; 0x4000]);
        let snapshot = source.save_snapshot();

        let mut restored = Tvc::new(false);
        restored.add_rom("TVC12_D4.64K", &[0x22; 0x4000]);
        restored.load_snapshot(&snapshot).unwrap();

        assert_eq!(
            restored.bus.mmu.read_raw_bank("sys", 0, 1).unwrap(),
            vec![0x22]
        );
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

    #[test]
    fn sound_amplitude_is_taken_from_port_0x06_bits_2_to_5() {
        let mut bus = TvcBus::new(true);

        bus.write_port(0x06, 0b0011_1101);

        assert_eq!(bus.sound.amplitude(), 0x0F);
    }

    #[test]
    fn sound_dac_mode_emits_amplitude_samples_without_oscillator_enable() {
        let mut tvc = Tvc::new(true);
        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;
        tvc.bus.write_port(0x06, 0x00);
        tvc.bus.advance_sound_timer(sample_cycles);
        let low = tvc.take_audio_samples();

        tvc.bus.write_port(0x06, 0x3C);
        tvc.bus.advance_sound_timer(sample_cycles);
        let high = tvc.take_audio_samples();

        assert!(!low.is_empty());
        assert!(!high.is_empty());
        assert_eq!(low[0], 0.0);
        assert!(low[0] < high[0]);
    }

    #[test]
    fn sound_dac_constant_level_is_ac_coupled() {
        let mut tvc = Tvc::new(true);
        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;

        tvc.bus.write_port(0x06, 0x3C);
        tvc.bus
            .advance_sound_timer(sample_cycles * tvc.sound_sample_rate() as u64);
        let samples = tvc.take_audio_samples();

        assert!(!samples.is_empty());
        assert!(samples[0] > 0.0);
        assert!(samples.last().copied().unwrap().abs() < 0.001);
    }

    #[test]
    fn sound_reset_state_emits_silence() {
        let mut tvc = Tvc::new(true);
        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;

        tvc.bus.advance_sound_timer(sample_cycles * 4);
        let samples = tvc.take_audio_samples();

        assert!(!samples.is_empty());
        assert!(samples.iter().all(|sample| *sample == 0.0));
    }

    #[test]
    fn execution_tracepoints_are_opt_in_and_bank_aware() {
        let mut tvc = Tvc::new(false);

        tvc.debug_step_instruction();
        assert!(tvc.take_trace_events().is_empty());

        tvc.reset();
        tvc.set_tracepoints(&[(RomBank::Sys, 0x0001)]);
        tvc.debug_step_instruction();

        assert_eq!(
            tvc.take_trace_events(),
            vec![ExecutionTrace {
                bank: RomBank::Sys,
                pc: 0x0001,
            }]
        );
    }

    #[test]
    fn debug_step_advances_interleaved_video() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::Interleaved);
        for (reg, value) in [(0, 99), (1, 76), (4, 38), (6, 36), (9, 7), (10, 0x20)] {
            tvc.bus.vid.set_reg_idx(reg);
            tvc.bus.vid.set_reg(value);
        }
        let before = tvc.bus.vid.stream_position();

        let elapsed = tvc.debug_step_instruction();

        assert!(elapsed > 0);
        assert_ne!(tvc.bus.vid.stream_position(), before);
        assert_eq!(tvc.clock, elapsed as u64);
    }

    #[test]
    fn debug_step_redraws_fast_frame_video() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::FastFrame);
        tvc.frame_complete = false;

        tvc.debug_step_instruction();

        assert!(tvc.frame_complete);
    }

    #[test]
    fn debug_run_to_interrupt_wakes_a_halted_cpu() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::Interleaved);
        tvc.z80.state.halted = 1;
        tvc.z80.state.iff1 = 1;
        tvc.z80.state.iff2 = 1;
        tvc.z80.state.set_reg16(10, 0x8000);
        tvc.request_cursor_irq();

        let result = tvc.debug_run_to_interrupt(100);

        assert_eq!(
            result,
            DebugRunToIrqResult {
                elapsed_cycles: 17,
                interrupt_accepted: true,
            }
        );
        assert_eq!(tvc.z80.state.halted, 0);
        assert_eq!(tvc.z80.state.get_reg16(11), 0x0038);
    }

    #[test]
    fn debug_run_to_interrupt_is_bounded_when_interrupts_are_disabled() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::Interleaved);
        tvc.z80.state.halted = 1;
        tvc.z80.state.iff1 = 0;
        tvc.request_cursor_irq();

        let result = tvc.debug_run_to_interrupt(16);

        assert_eq!(
            result,
            DebugRunToIrqResult {
                elapsed_cycles: 16,
                interrupt_accepted: false,
            }
        );
        assert_eq!(tvc.z80.state.halted, 1);
    }

    #[test]
    fn debug_run_to_interrupt_advances_fast_frame_crtc() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::FastFrame);
        for (reg, value) in [(0, 99), (1, 76), (4, 38), (6, 36), (9, 7), (10, 0)] {
            tvc.bus.vid.set_reg_idx(reg);
            tvc.bus.vid.set_reg(value);
        }
        tvc.z80.state.halted = 1;
        tvc.z80.state.iff1 = 1;
        tvc.z80.state.iff2 = 1;
        tvc.z80.state.set_reg16(10, 0x8000);

        let result = tvc.debug_run_to_interrupt(100);

        assert!(result.interrupt_accepted);
        assert_eq!(tvc.z80.state.halted, 0);
        assert_eq!(tvc.z80.state.get_reg16(11), 0x0038);
        assert!(tvc.frame_complete);
    }

    #[test]
    fn sound_oscillator_outputs_square_wave_when_enabled() {
        let mut tvc = Tvc::new(true);
        tvc.bus.write_port(0x04, 0xFE);
        tvc.bus.write_port(0x05, 0x1F);
        tvc.bus.write_port(0x06, 0x3C);
        tvc.bus.read_port(0x5B);

        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;
        tvc.bus.advance_sound_timer(10 * sample_cycles);
        let samples = tvc.take_audio_samples();

        assert!(samples.iter().any(|sample| *sample > 0.0));
        assert!(samples.iter().any(|sample| *sample < 0.0));
    }

    #[test]
    fn sound_oscillator_starts_after_programming_without_instart() {
        let mut tvc = Tvc::new(true);
        tvc.bus.write_port(0x04, 0xFE);
        tvc.bus.write_port(0x05, 0x1F);
        tvc.bus.write_port(0x06, 0x3C);

        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;
        tvc.bus.advance_sound_timer(10 * sample_cycles);
        let samples = tvc.take_audio_samples();

        assert!(samples.iter().any(|sample| *sample > 0.0));
        assert!(samples.iter().any(|sample| *sample < 0.0));
    }

    #[test]
    fn sound_oscillator_starts_low_after_restart() {
        let mut tvc = Tvc::new(true);
        let sample_cycles = CPU_CLOCK_HZ / tvc.sound_sample_rate() as u64 + 1;
        tvc.bus.write_port(0x04, 0x00);
        tvc.bus.write_port(0x05, 0x10);
        tvc.bus.write_port(0x06, 0x3C);
        tvc.bus.read_port(0x5B);

        tvc.bus.advance_sound_timer(sample_cycles);
        let first_samples = tvc.take_audio_samples();
        assert!(!first_samples.is_empty());
        assert!(first_samples.iter().all(|sample| *sample == 0.0));

        tvc.bus.advance_sound_timer(32_768);
        let high_samples = tvc.take_audio_samples();
        assert!(high_samples.iter().any(|sample| *sample > 0.0));
    }

    #[test]
    fn cursor_interrupt_does_not_log() {
        let mut tvc = Tvc::new_with_vid_model(false, VidModel::FastFrame);

        tvc.request_cursor_irq();

        assert!(tvc.bus.log.entries.is_empty());
        assert!(tvc.bus.shared_it_pending());
    }

    #[test]
    fn sound_port_writes_log_on_off_and_frequency_changes() {
        let mut bus = TvcBus::new(false);
        bus.trace_pc = 0x1234;

        bus.write_port(0x04, 0xFE);
        assert!(bus.log.entries.is_empty());

        bus.write_port(0x05, 0x1F);
        assert_eq!(bus.log.entries.len(), 1);
        assert!(bus.log.entries[0].starts_with("sound on: "));
        assert!(bus.log.entries[0].contains("divisor 0xFFE"));
        assert!(bus.log.entries[0].contains("port 0x05"));
        assert!(bus.log.entries[0].contains("pc 0x1234"));

        bus.trace_pc = 0x2345;
        bus.write_port(0x04, 0xFD);
        assert_eq!(bus.log.entries.len(), 2);
        assert!(bus.log.entries[1].starts_with("sound freq: "));
        assert!(bus.log.entries[1].contains("divisor 0xFFD"));
        assert!(bus.log.entries[1].contains("port 0x04"));
        assert!(bus.log.entries[1].contains("pc 0x2345"));

        bus.trace_pc = 0x3456;
        bus.write_port(0x05, 0x0F);
        assert_eq!(bus.log.entries.len(), 3);
        assert_eq!(bus.log.entries[2], "sound off (pc 0x3456)");
    }

    #[test]
    fn sound_volume_writes_log_only_when_amplitude_changes() {
        let mut bus = TvcBus::new(false);
        bus.trace_pc = 0x4567;

        bus.write_port(0x06, 0x00);
        assert!(bus.log.entries.is_empty());

        bus.write_port(0x06, 0x14);
        assert_eq!(bus.log.entries.len(), 1);
        assert_eq!(bus.log.entries[0], "sound volume: 5/15 (pc 0x4567)");

        bus.write_port(0x06, 0x15);
        assert_eq!(bus.log.entries.len(), 1);

        bus.trace_pc = 0x5678;
        bus.write_port(0x06, 0x3C);
        assert_eq!(bus.log.entries.len(), 2);
        assert_eq!(bus.log.entries[1], "sound volume: 15/15 (pc 0x5678)");
    }

    #[test]
    fn crtc_ports_are_mirrored_across_0x70_to_0x7f() {
        let mut bus = TvcBus::new(true);

        bus.write_port(0x72, 12);
        bus.write_port(0x73, 0x12);
        bus.write_port(0x7E, 13);
        bus.write_port(0x7F, 0x34);

        bus.write_port(0x70, 12);
        assert_eq!(bus.read_port(0x71), 0x12);
        bus.write_port(0x78, 13);
        assert_eq!(bus.read_port(0x79), 0x34);
    }

    #[test]
    fn crtc_reads_follow_register_access_permissions() {
        let mut bus = TvcBus::new(true);

        bus.write_port(0x70, 0);
        bus.write_port(0x71, 0x63);
        assert_eq!(bus.read_port(0x71), 0xFF);

        bus.write_port(0x70, 12);
        bus.write_port(0x71, 0xC1);
        assert_eq!(bus.read_port(0x71), 0x01);

        bus.write_port(0x70, 14);
        bus.write_port(0x71, 0xFE);
        assert_eq!(bus.read_port(0x71), 0x3E);

        bus.write_port(0x70, 16);
        bus.write_port(0x71, 0x5A);
        assert_eq!(bus.read_port(0x71), 0x00);
    }

    #[test]
    fn crtc_address_register_reads_as_unavailable() {
        let mut bus = TvcBus::new(true);

        bus.write_port(0x70, 12);

        assert_eq!(bus.read_port(0x70), 0xFF);
        assert_eq!(bus.read_port(0x72), 0xFF);
    }

    #[test]
    fn invalid_crtc_register_index_selects_no_data_register() {
        let mut bus = TvcBus::new(true);

        bus.write_port(0x70, 12);
        bus.write_port(0x71, 0x12);
        bus.write_port(0x70, 18);
        bus.write_port(0x71, 0x34);
        assert_eq!(bus.read_port(0x71), 0xFF);

        bus.write_port(0x70, 12);
        assert_eq!(bus.read_port(0x71), 0x12);
    }
}

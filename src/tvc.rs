#![allow(dead_code)]

use std::collections::HashSet;

use crate::bus::CpuBus;
use crate::cas::TapeBitstreamGenerator;
use crate::expansion::ExpansionSlots;
use crate::hbf::HBF;
use crate::key::Key;
use crate::log::{Log, Logger};
use crate::mmu::TvcMmu;
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
                self.sound.write_low(val);
            }

            0x05 => {
                self.sound.write_control(val);
                self.tape.set_motor_from_port5(val);
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

    pub(crate) fn set_tape_cycles(&mut self, cycles: u64) {
        self.tape.set_cycles(cycles);
    }

    fn restart_sound_timer(&mut self) {
        self.sound.restart();
    }

    fn advance_sound_timer(&mut self, cycles: u64) {
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
    last_cursor_it_clock: Option<u64>,
    breakpoints: HashSet<u16>,
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
            last_cursor_it_clock: None,
            breakpoints: HashSet::new(),
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

    pub fn save_snapshot(&self) -> Vec<u8> {
        crate::tvc_snapshot::save(self)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> snapshot::Result<()> {
        crate::tvc_snapshot::load(self, data)
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
        self.sync_timeout_frames = 0;
        self.last_cursor_it_clock = None;
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
        let irq_duration = self.z80.irq(&mut self.bus);
        if irq_duration > 0 {
            self.clock += irq_duration as u64;
            let irq_duration = irq_duration as u64;
            self.bus.advance_tape(irq_duration);
            self.bus.advance_sound_timer(irq_duration);
            self.bus.vid.stream_some(
                self.bus.mmu.get_vid_mem(),
                irq_duration.try_into().unwrap_or(u32::MAX),
            );
            self.bus.vid.render_stream(&mut self.framebuffer, 608);
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
        tvc.bus.write_port(0x04, 0xDC);
        tvc.bus.write_port(0x05, 0x6F);
        tvc.bus.read_port(0x5B);
        tvc.bus.advance_sound_timer(11);
        let sound_counter = tvc.bus.sound.counter();

        let snapshot = tvc.save_snapshot();
        let mut restored = Tvc::new(false);
        restored.load_snapshot(&snapshot).unwrap();

        assert!(restored.bus.mmu.is_plus());
        assert_eq!(restored.vid_model(), VidModel::Interleaved);
        assert_eq!(restored.z80.state.r16[11], 0x1234);
        assert_eq!(restored.bus.mmu.r8(0x4000), 0xAB);
        assert_eq!(restored.bus.pend_it, 0x0F);
        assert!(restored.bus.tape_motor_on());
        assert_eq!(restored.bus.sound.freq_low, 0xDC);
        assert_eq!(restored.bus.sound.ctrl, 0x6F);
        assert_eq!(restored.bus.sound.counter(), sound_counter);
        assert!(restored.bus.sound.running());
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

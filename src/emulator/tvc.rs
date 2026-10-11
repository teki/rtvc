#![allow(dead_code)]

use std::collections::HashSet;

use crate::bus::CpuBus;
use crate::cas::{TVC_CAS_HEADER_LEN, TVC_CAS_LOAD_ADDR, TapeBitstreamGenerator};
use crate::expansion::{Card, ExpansionSlots};
use crate::hbf::HBF;
use crate::host_file::HostFileCard;
use crate::instruction_trace::{
    InstructionEffects, InstructionTrace, InstructionTraceEntry, TraceRegisters,
};
use crate::key::Key;
use crate::log::{Log, LogCategory, LogEntry, Logger};
use crate::machine_type::ExpansionCard;
use crate::mmu::{RomBank, TvcMmu};
use crate::snapshot;
use crate::sound::SoundTimer;
use crate::tape::TapeInterface;
use crate::vid::{Vid, VidModel};
use crate::z80::Z80;
use crate::z80_bus_cycles::BusCycles;

const CPU_CLOCK_HZ: u64 = 3_125_000;
const FRAME_RATE_HZ: u64 = 50;
const FRAME_CLOCKS: u64 = CPU_CLOCK_HZ / FRAME_RATE_HZ;
const SYNC_TIMEOUT_HOST_FRAMES: u32 = 8;
const SHARED_CURSOR_SOUND_IT: u8 = 0x10;
// Devices latch or drive the data bus this many T-states into an I/O cycle.
const IO_ACCESS_TSTATE: u32 = 3;
const MAX_TIMED_ACCESSES: usize = 8;
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
    instruction_effects: Option<InstructionEffects>,
    // Device time already advanced inside the current instruction.
    instruction_synced_cycles: u32,
    stream_video: bool,
    // Memory cycles of the current instruction, for video RAM wait states.
    timing_active: bool,
    access_count: usize,
    access_values: [u8; MAX_TIMED_ACCESSES],
    video_access_mask: u32,
    // Video RAM wait states of the current instruction, in half T-states,
    // resolved for its first `waits_resolved` memory cycles.
    wait_half_tstates: u32,
    waits_resolved: usize,
    // CPU time in half T-states modulo the 4-half-T-state video RAM slot,
    // which is aligned with the CRTC character clock.
    half_tstate_phase: u32,
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
            instruction_effects: None,
            instruction_synced_cycles: 0,
            stream_video: false,
            timing_active: false,
            access_count: 0,
            access_values: [0; MAX_TIMED_ACCESSES],
            video_access_mask: 0,
            wait_half_tstates: 0,
            waits_resolved: 0,
            half_tstate_phase: 0,
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
        self.vid.reset();
        self.reset_video_slot_phase();
        self.key.reset();
        self.pend_it = 0x1F;
        self.extensions.reset();
        self.tape.reset();
        self.trace_pc = 0;
        self.sound.reset();
        self.instruction_effects = None;
    }

    fn begin_instruction_effects(&mut self) {
        self.instruction_effects = Some(InstructionEffects::default());
    }

    fn take_instruction_effects(&mut self) -> InstructionEffects {
        self.instruction_effects.take().unwrap_or_default()
    }

    pub fn write_port(&mut self, addr: u8, val: u8) {
        self.write_port16(addr as u16, val);
    }

    /// Port write with the full 16-bit address; TVC devices decode the low byte.
    fn write_port16(&mut self, port: u16, val: u8) {
        let addr = port as u8;
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
                let was_motor_on = self.tape.motor_on();
                self.sound.write_control(val);
                self.log_sound_change(was_on, old_divisor, addr);
                self.tape.set_motor_from_port5(val);
                self.log_tape_motor_change(was_motor_on, addr);
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

            0x70..=0x7F => self.write_crtc_port(addr, val),

            0x58..=0x5B => self
                .extensions
                .write_irq_control((addr - 0x58) as usize, val),
            0x50..=0x57 => {
                let level = self.tape.toggle_output();
                self.log.log_with_category(
                    LogCategory::Tape,
                    &format!(
                        "tape output {} via port 0x{addr:02X} (pc 0x{:04X})",
                        if level { "high" } else { "low" },
                        self.trace_pc
                    ),
                );
            }
            _ => {
                let hbf_slot = ExpansionSlots::port_slot(addr)
                    .filter(|slot| self.extensions.hbf_at(*slot).is_some());
                if let Some(slot) = hbf_slot {
                    self.log_fdc_write(slot, addr & 0x0F, val);
                }
                self.extensions.write_port(port, val);
            }
        }
    }

    fn log_fdc_write(&mut self, slot: usize, reg: u8, val: u8) {
        let prefix = if slot == 0 {
            "FDC".to_string()
        } else {
            format!("FDC slot {slot}")
        };
        let message = match reg {
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
                format!(
                    "{prefix} command: 0x{:02X} [{}] (pc 0x{:04X})",
                    val, cmd_name, self.trace_pc
                )
            }
            0x01 => format!("{prefix} track: {} (pc 0x{:04X})", val, self.trace_pc),
            0x02 => format!("{prefix} sector: {} (pc 0x{:04X})", val, self.trace_pc),
            0x04 => {
                let drive = if (val & 1) != 0 {
                    "A:"
                } else if (val & 2) != 0 {
                    "B:"
                } else {
                    "none"
                };
                let side = (val & 0x80) >> 7;
                format!(
                    "{prefix} select: drive {}, side {} (pc 0x{:04X})",
                    drive, side, self.trace_pc
                )
            }
            _ => return,
        };
        self.log.log_with_category(LogCategory::Disk, &message);
    }

    fn log_sound_change(&mut self, was_on: bool, old_divisor: u16, port: u8) {
        let is_on = self.sound.audible_oscillator_enabled();
        let divisor = self.sound.divisor();
        if was_on != is_on {
            if is_on {
                self.log.log_with_category(
                    LogCategory::Sound,
                    &format!(
                        "sound on: {} (divisor 0x{divisor:03X}, port 0x{port:02X}, pc 0x{:04X})",
                        self.sound_frequency_label(),
                        self.trace_pc
                    ),
                );
            } else {
                self.log.log_with_category(
                    LogCategory::Sound,
                    &format!("sound off (pc 0x{:04X})", self.trace_pc),
                );
            }
        } else if is_on && old_divisor != divisor {
            self.log.log_with_category(
                LogCategory::Sound,
                &format!(
                    "sound freq: {} (divisor 0x{divisor:03X}, port 0x{port:02X}, pc 0x{:04X})",
                    self.sound_frequency_label(),
                    self.trace_pc
                ),
            );
        }
    }

    fn log_sound_volume_change(&mut self, old_amplitude: u8) {
        let amplitude = self.sound.amplitude();
        if old_amplitude != amplitude {
            self.log.log_with_category(
                LogCategory::Sound,
                &format!("sound volume: {amplitude}/15 (pc 0x{:04X})", self.trace_pc),
            );
        }
    }

    fn log_tape_motor_change(&mut self, was_motor_on: bool, port: u8) {
        let motor_on = self.tape.motor_on();
        if was_motor_on != motor_on {
            self.log.log_with_category(
                LogCategory::Tape,
                &format!(
                    "tape motor {} via port 0x{port:02X} (pc 0x{:04X})",
                    if motor_on { "on" } else { "off" },
                    self.trace_pc
                ),
            );
        }
    }

    fn write_crtc_port(&mut self, addr: u8, val: u8) {
        if addr & 1 == 0 {
            self.vid.write_crtc_port(addr, val);
            let idx = self.vid.get_reg_idx();
            self.log.log_with_category(
                LogCategory::Video,
                &format!(
                    "CRTC select R{idx} {} via port 0x{addr:02X} (pc 0x{:04X})",
                    crtc_register_name(idx),
                    self.trace_pc
                ),
            );
            return;
        }

        let idx = self.vid.get_reg_idx();
        if idx >= 16 {
            self.vid.write_crtc_port(addr, val);
            self.log.log_with_category(LogCategory::Video, &format!(
                "CRTC write ignored: R{idx} {} is not writable (value 0x{val:02X}, port 0x{addr:02X}, pc 0x{:04X})",
                crtc_register_name(idx),
                self.trace_pc
            ));
            return;
        }

        let old_value = self.vid.raw_reg(idx);
        self.vid.write_crtc_port(addr, val);

        let Some(new_value) = self.vid.raw_reg(idx) else {
            self.log.log_with_category(LogCategory::Video, &format!(
                "CRTC write ignored: R{idx} {} is not writable (value 0x{val:02X}, port 0x{addr:02X}, pc 0x{:04X})",
                crtc_register_name(idx),
                self.trace_pc
            ));
            return;
        };

        let change = match old_value {
            Some(old) if old != new_value => format!("0x{old:02X}->0x{new_value:02X}"),
            Some(_) => format!("0x{new_value:02X}"),
            None => format!("0x{new_value:02X}"),
        };
        self.log.log_with_category(
            LogCategory::Video,
            &format!(
                "CRTC R{idx} {} = {change} via port 0x{addr:02X}: {} (pc 0x{:04X})",
                crtc_register_name(idx),
                self.crtc_write_effect(idx),
                self.trace_pc
            ),
        );
    }

    fn crtc_write_effect(&self, idx: u8) -> String {
        let reg = |idx| self.vid.raw_reg(idx).unwrap_or(0);
        match idx {
            0 => format!(
                "horizontal total {} character clocks",
                reg(0).wrapping_add(1)
            ),
            1 => format!("horizontal displayed {} characters", reg(1)),
            2 => format!("horizontal sync starts at character {}", reg(2)),
            3 => format!(
                "horizontal sync width {}, vertical sync width field {}",
                reg(3) & 0x0F,
                (reg(3) >> 4) & 0x0F
            ),
            4 => format!("vertical total {} character rows", (reg(4) & 0x7F) + 1),
            5 => format!("vertical total adjust {} scanlines", reg(5) & 0x1F),
            6 => format!("vertical displayed {} character rows", reg(6) & 0x7F),
            7 => format!("vertical sync starts at character row {}", reg(7) & 0x7F),
            8 => format!(
                "interlace mode {}, skew bits {}",
                reg(8) & 0x03,
                (reg(8) >> 6) & 0x03
            ),
            9 => format!("{} scanlines per character row", (reg(9) & 0x1F) + 1),
            10 => format!(
                "cursor starts on raster {}, mode {}",
                reg(10) & 0x1F,
                cursor_mode_label((reg(10) >> 5) & 0x03)
            ),
            11 => format!("cursor ends on raster {}", reg(11) & 0x1F),
            12 | 13 => format!(
                "display start address 0x{:04X}",
                crtc_address(reg(12), reg(13))
            ),
            14 | 15 => format!(
                "cursor address 0x{:04X}, cursor raster {}",
                crtc_address(reg(14), reg(15)),
                reg(10) & 0x1F
            ),
            16 | 17 => "light pen register is read-only".to_string(),
            _ => "invalid CRTC register index".to_string(),
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

    fn irq_pending(&self) -> bool {
        self.shared_it_pending() || self.extensions.irq_requested()
    }

    /// Port `0x59` low five bits: shared cursor/sound request in bit 4 and
    /// active-low slot requests in bits 0-3.
    fn interrupt_status(&self) -> u8 {
        (self.pend_it & !0x0F) | self.extensions.irq_status()
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

    fn advance_devices(&mut self, cycles: u32) {
        if cycles == 0 {
            return;
        }
        self.tape.advance(cycles as u64);
        self.advance_sound_timer(cycles as u64);
        if self.stream_video {
            let mut cursor_it = self.vid.stream_some(self.mmu.get_vid_mem(), cycles);
            while cursor_it {
                self.request_shared_irq();
                cursor_it = self.vid.stream_some(self.mmu.get_vid_mem(), 0);
            }
        }
    }

    /// Realigns the CPU clock with a video circuit that restarted its
    /// character clock.
    pub(crate) fn reset_video_slot_phase(&mut self) {
        self.half_tstate_phase = 0;
    }

    fn begin_instruction_timing(&mut self, stream_video: bool) {
        self.instruction_synced_cycles = 0;
        self.stream_video = stream_video;
        self.timing_active = true;
        self.access_count = 0;
        self.video_access_mask = 0;
        self.wait_half_tstates = 0;
        self.waits_resolved = 0;
    }

    /// Advances devices through the rest of an instruction that took
    /// `tstates` T-states without wait states, and returns its length
    /// including video RAM wait states.
    fn finish_instruction_timing(&mut self, tstates: u32) -> u32 {
        self.timing_active = false;
        self.resolve_video_waits(Some(tstates));
        let half_tstates = 2 * tstates + self.wait_half_tstates;
        let elapsed = self.elapsed_cycles(half_tstates);
        self.half_tstate_phase = (self.half_tstate_phase + half_tstates) & 3;
        let remaining = elapsed.saturating_sub(self.instruction_synced_cycles);
        self.instruction_synced_cycles = 0;
        self.advance_devices(remaining);
        elapsed
    }

    /// Whole device cycles elapsed `half_tstates` into the instruction.
    fn elapsed_cycles(&self, half_tstates: u32) -> u32 {
        ((self.half_tstate_phase + half_tstates) >> 1) - (self.half_tstate_phase >> 1)
    }

    fn sync_devices_to(&mut self, cycles: u32) {
        if cycles > self.instruction_synced_cycles {
            let delta = cycles - self.instruction_synced_cycles;
            self.instruction_synced_cycles = cycles;
            self.advance_devices(delta);
        }
    }

    #[inline]
    fn record_memory_cycle(&mut self, addr: u16, val: u8) {
        if !self.timing_active {
            return;
        }
        let index = self.access_count;
        if index < MAX_TIMED_ACCESSES {
            self.access_values[index] = val;
            if self.mmu.is_video(addr) {
                self.video_access_mask |= 1 << index;
            }
        }
        self.access_count += 1;
    }

    /// Accumulates the video RAM wait states of the memory cycles performed
    /// so far. The CPU is stretched until the video circuit's access slot.
    fn resolve_video_waits(&mut self, tstates: Option<u32>) {
        let count = self.access_count.min(MAX_TIMED_ACCESSES);
        let pending = self.video_access_mask >> self.waits_resolved;
        let resolved = std::mem::replace(&mut self.waits_resolved, count);
        if pending == 0 {
            return;
        }
        let Some(cycles) =
            BusCycles::lookup(&self.access_values[..count], tstates, self.access_count)
        else {
            return;
        };
        for index in resolved..count {
            if self.video_access_mask & (1 << index) == 0 {
                continue;
            }
            let Some(cycle) = cycles.get(index) else {
                break;
            };
            let start = self.half_tstate_phase + 2 * cycle.tstate + self.wait_half_tstates;
            let slot_offset = if cycle.m1 { start + 3 } else { start };
            self.wait_half_tstates += 4 - (slot_offset & 3);
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

    fn read_port(&mut self, port: u16) -> u8 {
        let addr = port as u8;
        let val = match addr {
            0x58 => self.key.read_row(),
            0x59 | 0x5D => (self.tape_input_bit() << 5) | 0x40 | self.interrupt_status(),
            0x50..=0x57 => {
                let level = self.tape.toggle_output();
                self.log.log_with_category(
                    LogCategory::Tape,
                    &format!(
                        "tape output {} via input port 0x{addr:02X} (pc 0x{:04X})",
                        if level { "high" } else { "low" },
                        self.trace_pc
                    ),
                );
                0xFF
            }
            0x5B | 0x5F => {
                self.restart_sound_timer();
                0xFF
            }
            0x5A | 0x5E => self.extensions.type_status(),
            0x70..=0x7F => self.vid.read_crtc_port(addr),
            _ => self.extensions.read_port(port).unwrap_or(0xFF),
        };
        val
    }
}

fn crtc_register_name(idx: u8) -> &'static str {
    match idx {
        0 => "Horizontal Total",
        1 => "Horizontal Displayed",
        2 => "Horizontal Sync Position",
        3 => "Sync Width",
        4 => "Vertical Total",
        5 => "Vertical Total Adjust",
        6 => "Vertical Displayed",
        7 => "Vertical Sync Position",
        8 => "Interlace Mode and Skew",
        9 => "Maximum Raster Address",
        10 => "Cursor Start",
        11 => "Cursor End",
        12 => "Start Address (H)",
        13 => "Start Address (L)",
        14 => "Cursor Address (H)",
        15 => "Cursor Address (L)",
        16 => "Light Pen (H)",
        17 => "Light Pen (L)",
        _ => "Invalid Register",
    }
}

fn crtc_address(hi: u8, lo: u8) -> u16 {
    (((hi & 0x3F) as u16) << 8) | lo as u16
}

fn cursor_mode_label(mode: u8) -> &'static str {
    match mode {
        0 => "visible",
        1 => "hidden",
        2 => "blink 16 frames",
        3 => "blink 32 frames",
        _ => "unknown",
    }
}

impl CpuBus for TvcBus {
    fn r8(&mut self, addr: u16) -> u8 {
        let val = if let Some(offset) = self.mmu.ext_card_offset(addr) {
            self.extensions.read_mem(offset)
        } else {
            self.mmu.r8(addr)
        };
        self.record_memory_cycle(addr, val);
        val
    }

    fn w8(&mut self, addr: u16, val: u8) {
        self.record_memory_cycle(addr, val);
        if let Some(effects) = &mut self.instruction_effects {
            effects.record_memory_write(addr, val);
        }
        if let Some(offset) = self.mmu.ext_card_offset(addr) {
            self.extensions.write_mem(offset, val);
            return;
        }
        self.mmu.w8(addr, val);
    }

    fn io_cycle_start(&mut self, tstates: u32) {
        self.resolve_video_waits(None);
        let access = 2 * (tstates + IO_ACCESS_TSTATE) + self.wait_half_tstates;
        self.sync_devices_to(self.elapsed_cycles(access));
    }

    fn out8(&mut self, port: u8, val: u8, high_addr: u8) {
        if let Some(effects) = &mut self.instruction_effects {
            effects.record_port_write(u16::from_be_bytes([high_addr, port]), val);
        }
        self.write_port16(u16::from_be_bytes([high_addr, port]), val);
    }

    fn in8(&mut self, port: u8, high_addr: u8) -> u8 {
        self.read_port(u16::from_be_bytes([high_addr, port]))
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
    /// Physical ROM landmarks: (bank, bank-offset) pairs. Offsets are used
    /// instead of CPU addresses so a landmark fires in any paging view.
    tracepoints: HashSet<(RomBank, u16)>,
    trace_events: Vec<ExecutionTrace>,
    instruction_trace: InstructionTrace,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionTrace {
    pub bank: RomBank,
    pub pc: u16,
    /// Bank offset of `pc`, captured through the live mapping so consumers
    /// can resolve symbols without knowing the paging view.
    pub offset: u16,
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
            instruction_trace: InstructionTrace::default(),
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

    /// Legacy snapshot check: whether slot 0 holds an HBF card.
    pub fn has_hbf(&self) -> bool {
        self.bus.extensions.hbf_at(0).is_some()
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
        let result = crate::tvc_snapshot::load(self, data);
        if result.is_ok() {
            self.instruction_trace.clear();
        }
        result
    }

    pub fn refresh_framebuffer_from_state(&mut self) {
        let vidmem = self.bus.mmu.get_vid_mem();
        self.bus.vid.draw_frame(vidmem, &mut self.framebuffer);
        self.frame_complete = true;
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
        self.instruction_trace.clear();
    }

    pub fn instruction_trace(&self) -> &InstructionTrace {
        &self.instruction_trace
    }

    pub fn instruction_trace_mut(&mut self) -> &mut InstructionTrace {
        &mut self.instruction_trace
    }

    pub fn load_cas(&mut self, data: &[u8]) -> bool {
        if data.len() < TVC_CAS_HEADER_LEN || data[0] != 0x11 {
            return false;
        }
        let savemap = self.bus.mmu.get_map_val();
        self.bus.mmu.set_map(0xB0);
        for i in TVC_CAS_HEADER_LEN..data.len() {
            let addr = TVC_CAS_LOAD_ADDR.wrapping_add((i - TVC_CAS_HEADER_LEN) as u16);
            self.bus.w8(addr, data[i]);
        }
        if let Some(len) = crate::basic::tokenized_program_len(&data[TVC_CAS_HEADER_LEN..]) {
            if len > 1 {
                let last = TVC_CAS_LOAD_ADDR.wrapping_add((len as u16) - 1);
                poke16(&mut self.bus, 0x1722, TVC_CAS_LOAD_ADDR);
                poke16(&mut self.bus, 0x1724, last);
                poke16(&mut self.bus, 0x1726, last);
            }
        }
        self.bus.mmu.set_map(savemap);
        true
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        if name.contains("CART") {
            self.load_cart_rom(data);
        } else {
            self.bus.mmu.add_rom(name, data);
        }
    }

    /// Plugs a card into `slot`. `rom` is the card ROM image for cards that
    /// need one (HBF); built-in cards ignore it.
    pub fn attach_card(&mut self, slot: usize, card: ExpansionCard, rom: &[u8]) {
        let card = match card {
            ExpansionCard::VtDos12 => Card::Hbf(HBF::new(rom)),
            ExpansionCard::HostFile => Card::HostFile(HostFileCard::new(None, true)),
        };
        self.bus.extensions.attach(slot, card);
    }

    /// Slot holding the card, if any.
    pub fn card_kind(&self, slot: usize) -> Option<ExpansionCard> {
        self.bus.extensions.kind(slot)
    }

    /// Slots holding an HBF card, in slot order.
    pub fn hbf_slots(&self) -> Vec<usize> {
        self.bus.extensions.hbf_slots()
    }

    pub fn host_file_cards_mut(&mut self) -> impl Iterator<Item = &mut HostFileCard> {
        self.bus.extensions.host_file_cards_mut()
    }

    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.bus.mmu.load_cart_rom(data);
    }

    pub fn load_disk(&mut self, slot: usize, drive: usize, name: &str, data: &[u8]) {
        if let Some(ext) = self.bus.extensions.hbf_at_mut(slot) {
            ext.load_disk(drive, name, data);
        }
    }

    pub fn disk_dirty(&self, slot: usize, drive: usize) -> bool {
        self.bus
            .extensions
            .hbf_at(slot)
            .is_some_and(|ext| ext.disk_dirty(drive))
    }

    pub fn clear_disk_dirty(&mut self, slot: usize, drive: usize) {
        if let Some(ext) = self.bus.extensions.hbf_at_mut(slot) {
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

    pub fn log_entries(&self) -> &[LogEntry] {
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

    /// Physical ROM tracepoints as (bank, bank-offset) pairs.
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
        self.bus.trace_pc = self.z80.state.pc;
        let trace_entry = self.instruction_trace.is_recording().then(|| {
            let pc = self.z80.state.pc;
            let opcode = std::array::from_fn(|offset| self.bus.r8(pc.wrapping_add(offset as u16)));
            self.bus.begin_instruction_effects();
            InstructionTraceEntry {
                sequence: 0,
                clock: self.clock,
                registers: TraceRegisters::from(&self.z80.state),
                opcode,
                main_map: Some(self.bus.mmu.get_map_val()),
                video_map: Some(self.bus.mmu.get_vid_map_val()),
                elapsed_cycles: 0,
                interrupt_accepted: false,
                effects: InstructionEffects::default(),
            }
        });
        self.bus.begin_instruction_timing(sync_video);
        let tstates = self.z80.step(&mut self.bus, 0);
        self.record_tracepoint();

        let cpu_time = self.bus.finish_instruction_timing(tstates);
        self.clock += cpu_time as u64;

        let mut frame_complete = false;
        if sync_video {
            frame_complete = self.bus.vid.render_stream(&mut self.framebuffer, 608);
        }

        let (irq_time, irq_frame_complete) = self.service_pending_shared_irq();
        if let Some(mut entry) = trace_entry {
            entry.elapsed_cycles = cpu_time + irq_time;
            entry.interrupt_accepted = irq_time > 0;
            entry.effects = self.bus.take_instruction_effects();
            self.instruction_trace.record(entry);
        }
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
        let pc = self.z80.state.pc;
        let Some((bank, offset)) = self.bus.mmu.mapped_rom_offset(pc) else {
            return;
        };
        if self.tracepoints.contains(&(bank, offset))
            && self
                .trace_events
                .last()
                .is_none_or(|event| event.bank != bank || event.pc != pc)
        {
            self.trace_events.push(ExecutionTrace { bank, pc, offset });
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

    fn service_pending_shared_irq(&mut self) -> (u32, bool) {
        if self.bus.irq_pending() {
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

            if self.breakpoints.contains(&self.z80.state.pc) {
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
            self.bus.begin_instruction_timing(true);
            self.bus.finish_instruction_timing(irq_duration);
            frame_complete = self.bus.vid.render_stream(&mut self.framebuffer, 608);
        }
        (irq_duration, frame_complete)
    }
}

fn poke16(bus: &mut TvcBus, addr: u16, value: u16) {
    let bytes = value.to_le_bytes();
    bus.w8(addr, bytes[0]);
    bus.w8(addr.wrapping_add(1), bytes[1]);
}

#[cfg(test)]
#[path = "tvc_tests.rs"]
mod tests;

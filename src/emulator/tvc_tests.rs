use super::*;

#[test]
fn instruction_trace_records_tvc_mapper_and_memory_writes() {
    let mut tvc = Tvc::new(true);
    tvc.bus.mmu.set_map(0x10);
    tvc.bus.mmu.w8(0x0000, 0x32); // LD (4000H),A
    tvc.bus.mmu.w8(0x0001, 0x00);
    tvc.bus.mmu.w8(0x0002, 0x40);
    tvc.z80.state.set_reg8(0, 0x2A);
    tvc.instruction_trace_mut()
        .start(crate::instruction_trace::MIN_TRACE_CAPACITY);

    tvc.debug_step_instruction();

    let entry = tvc.instruction_trace().entries().back().unwrap();
    assert_eq!(entry.pc(), 0x0000);
    assert_eq!(entry.registers.af >> 8, 0x2A);
    assert_eq!(entry.opcode[..3], [0x32, 0x00, 0x40]);
    assert_eq!(entry.main_map, Some(0x10));
    assert_eq!(entry.video_map, Some(0x00));
    assert_eq!(
        entry.effects.memory_writes,
        vec![crate::instruction_trace::TraceMemoryWrite {
            addr: 0x4000,
            value: 0x2A,
        }]
    );
}

#[test]
fn snapshot_round_trips_core_state() {
    let mut tvc = Tvc::new_with_vid_model(true, VidModel::Interleaved);
    tvc.z80.state.pc = 0x1234;
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
    assert_eq!(restored.z80.state.pc, 0x1234);
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
    assert!(chunks.iter().all(|chunk| &chunk.id[..3] != b"SLT"));

    let mut plus = Tvc::new(true);
    plus.attach_card(0, ExpansionCard::VtDos12, &[0xA5; 0x4000]);
    plus.load_disk(0, 0, "large.dsk", &[0xE5; 368_640]);
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
            .find(|chunk| chunk.id == *b"SLT0")
            .unwrap()
            .data
            .len()
            < 0x1200
    );
}

#[test]
fn two_hbf_cards_answer_only_on_their_own_slot() {
    let mut tvc = Tvc::new(true);
    tvc.attach_card(0, ExpansionCard::VtDos12, &[0x10; 0x4000]);
    tvc.attach_card(1, ExpansionCard::VtDos12, &[0x20; 0x4000]);
    assert_eq!(tvc.bus.read_port(0x5A), 0xFA);
    assert_eq!(tvc.hbf_slots(), vec![0, 1]);

    tvc.bus.write_port(0x11, 5);
    tvc.bus.write_port(0x21, 9);
    assert_eq!(tvc.bus.read_port(0x11), 5);
    assert_eq!(tvc.bus.read_port(0x21), 9);
    assert_eq!(tvc.bus.read_port(0x31), 0xFF);

    tvc.bus.write_port(0x03, 0x40);
    assert_eq!(tvc.bus.extensions.read_mem(0), 0x20);
    tvc.bus.write_port(0x03, 0x00);
    assert_eq!(tvc.bus.extensions.read_mem(0), 0x10);

    tvc.load_disk(1, 0, "second.dsk", &[0xE5; 368_640]);
    assert!(tvc.bus.extensions.hbf_at(0).unwrap().disk_name(0).is_none());
    assert_eq!(
        tvc.bus.extensions.hbf_at(1).unwrap().disk_name(0),
        Some("second.dsk")
    );
}

#[test]
fn slot_interrupt_writes_enable_and_clear_per_slot() {
    let mut bus = TvcBus::new(true);
    assert_eq!(bus.extensions.irq_enable(), 0);
    bus.write_port(0x59, 0x80);
    bus.write_port(0x5B, 0x80);
    assert_eq!(bus.extensions.irq_enable(), 0b1010);
    bus.write_port(0x59, 0x00);
    assert_eq!(bus.extensions.irq_enable(), 0b1000);
    // No card raises a request, so the active-low slot bits stay high.
    assert_eq!(bus.read_port(0x59) & 0x0F, 0x0F);
    assert!(!bus.irq_pending());
}

#[test]
fn per_slot_snapshot_chunks_round_trip_each_card_kind() {
    let mut tvc = Tvc::new(true);
    tvc.attach_card(1, ExpansionCard::VtDos12, &[0xA5; 0x4000]);
    tvc.attach_card(3, ExpansionCard::HostFile, &[]);
    tvc.bus.write_port(0x18, 0x20);
    tvc.bus.write_port(0x5A, 0x80);
    tvc.bus.write_port(0x40, crate::host_file::CMD_NAME_CLEAR);
    tvc.bus.write_port(0x42, b'X');
    let snapshot = tvc.save_snapshot();
    let ids: Vec<[u8; 4]> = crate::snapshot::read_file(&snapshot)
        .unwrap()
        .iter()
        .map(|chunk| chunk.id)
        .collect();
    assert!(ids.contains(b"SLT1") && ids.contains(b"SLT3"));
    assert!(!ids.contains(b"SLT0") && !ids.contains(b"HBF "));

    let mut restored = Tvc::new(true);
    restored.attach_card(1, ExpansionCard::VtDos12, &[0xA5; 0x4000]);
    restored.attach_card(3, ExpansionCard::HostFile, &[]);
    restored.load_snapshot(&snapshot).unwrap();
    assert_eq!(restored.bus.extensions.irq_enable(), 0b0100);
    assert_eq!(restored.save_snapshot(), snapshot);

    let mut mismatched = Tvc::new(true);
    mismatched.attach_card(1, ExpansionCard::HostFile, &[]);
    mismatched.attach_card(3, ExpansionCard::HostFile, &[]);
    assert!(mismatched.load_snapshot(&snapshot).is_err());
}

#[test]
fn legacy_hbf_chunk_loads_into_first_hbf_slot() {
    let mut source = Tvc::new(true);
    source.attach_card(1, ExpansionCard::VtDos12, &[0xA5; 0x4000]);
    source.bus.write_port(0x28, 0x30);
    let snapshot = source.save_snapshot();
    let legacy: Vec<([u8; 4], Vec<u8>)> = crate::snapshot::read_file(&snapshot)
        .unwrap()
        .into_iter()
        .map(|chunk| {
            if chunk.id == *b"SLT1" {
                (*b"HBF ", chunk.data[1..].to_vec())
            } else {
                (chunk.id, chunk.data.to_vec())
            }
        })
        .collect();
    let legacy = crate::snapshot::write_file(&legacy);

    let mut restored = Tvc::new(true);
    restored.attach_card(1, ExpansionCard::VtDos12, &[0xA5; 0x4000]);
    restored.load_snapshot(&legacy).unwrap();
    assert_eq!(restored.save_snapshot(), snapshot);
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
            offset: 0x0001,
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
fn ei_delays_maskable_interrupt_by_one_instruction() {
    let mut tvc = Tvc::new_with_vid_model(false, VidModel::FastFrame);
    tvc.bus.mmu.set_map(0x70);
    tvc.bus.mmu.w8(0x0100, 0xFB);
    tvc.bus.mmu.w8(0x0101, 0x00);
    tvc.z80.state.pc = 0x0100;
    tvc.z80.state.iff1 = 0;
    tvc.request_cursor_irq();

    tvc.debug_step_instruction();
    assert_eq!(tvc.z80.state.pc, 0x0101);
    assert_eq!(tvc.z80.state.iff1, 1);

    tvc.debug_step_instruction();
    assert_eq!(tvc.z80.state.pc, 0x0038);
    assert_eq!(tvc.z80.state.iff1, 0);
}

#[test]
fn interrupt_mode_2_uses_the_vector_table() {
    let mut tvc = Tvc::new_with_vid_model(true, VidModel::FastFrame);
    tvc.bus.mmu.set_map(0xB0);
    tvc.bus.mmu.w8(0x0100, 0x00);
    tvc.bus.mmu.w8(0xFEFF, 0xFD);
    tvc.bus.mmu.w8(0xFF00, 0xFD);
    tvc.bus.mmu.w8(0xFDFD, 0xC3);
    tvc.bus.mmu.w8(0xFDFE, 0x00);
    tvc.bus.mmu.w8(0xFDFF, 0x40);
    tvc.z80.state.pc = 0x0100;
    tvc.z80.state.sp = 0x8000;
    tvc.z80.state.im = 2;
    tvc.z80.state.iff1 = 1;
    tvc.z80.state.iff2 = 1;
    tvc.z80.state.set_reg16(12, 0xFE00);
    tvc.request_cursor_irq();

    tvc.debug_step_instruction();

    assert_eq!(tvc.z80.state.pc, 0xFDFD);
    assert_eq!(tvc.z80.state.iff1, 0);
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
fn crtc_writes_log_selected_register_and_decoded_effect() {
    let mut bus = TvcBus::new(true);
    bus.trace_pc = 0x1111;

    bus.write_port(0x70, 14);
    assert_eq!(
        bus.log.entries[0],
        "CRTC select R14 Cursor Address (H) via port 0x70 (pc 0x1111)"
    );

    bus.trace_pc = 0x2222;
    bus.write_port(0x71, 0x0E);
    assert_eq!(
        bus.log.entries[1],
        "CRTC R14 Cursor Address (H) = 0x00->0x0E via port 0x71: cursor address 0x0E00, cursor raster 0 (pc 0x2222)"
    );

    bus.trace_pc = 0x3333;
    bus.write_port(0x72, 15);
    bus.write_port(0x73, 0xFF);
    assert_eq!(
        bus.log.entries[3],
        "CRTC R15 Cursor Address (L) = 0x00->0xFF via port 0x73: cursor address 0x0EFF, cursor raster 0 (pc 0x3333)"
    );

    bus.trace_pc = 0x4444;
    bus.write_port(0x70, 16);
    bus.write_port(0x71, 0x5A);
    assert_eq!(
        bus.log.entries[5],
        "CRTC write ignored: R16 Light Pen (H) is not writable (value 0x5A, port 0x71, pc 0x4444)"
    );
}

#[test]
fn video_setup_reports_start_address_and_irq_raster_line() {
    let mut bus = TvcBus::new(false);

    bus.vid.set_reg_idx(12);
    bus.vid.set_reg(0x40);
    bus.vid.set_reg_idx(13);
    bus.vid.set_reg(0x00);
    bus.vid.set_reg_idx(1);
    bus.vid.set_reg(64);
    bus.vid.set_reg_idx(6);
    bus.vid.set_reg(60);
    bus.vid.set_reg_idx(9);
    bus.vid.set_reg(3);
    bus.vid.set_reg_idx(14);
    bus.vid.set_reg(0x0A);
    bus.vid.set_reg_idx(15);
    bus.vid.set_reg(0xFF);
    bus.vid.set_reg_idx(10);
    bus.vid.set_reg(3);

    assert_eq!(bus.vid.display_start_address(), 0x0000);
    assert_eq!(bus.vid.cursor_interrupt_setup(), (0x0AFF, Some(175)));
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

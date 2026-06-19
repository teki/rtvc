use super::{FPS_WINDOW, FrameStats, FrameStatsSnapshot, handle_command};
use crate::emu::{Emu, MachineType, RomVersion};
use std::time::{Duration, Instant};

#[test]
fn frame_stats_report_average_over_five_second_window() {
    let start = Instant::now();
    let mut stats = FrameStats::new_at(start);
    for frame in 1..=250 {
        stats.record_at(start + Duration::from_millis(frame * 20));
    }

    let snapshot = stats.snapshot_at(start + FPS_WINDOW);

    assert_eq!(snapshot.frames, 250);
    assert_eq!(snapshot.window_seconds, 5.0);
    assert!((snapshot.average_fps - 50.0).abs() < f64::EPSILON);
}

#[test]
fn frame_stats_drop_frames_outside_the_window() {
    let start = Instant::now();
    let mut stats = FrameStats::new_at(start);
    stats.record_at(start + Duration::from_secs(1));
    stats.record_at(start + Duration::from_secs(6));

    let snapshot = stats.snapshot_at(start + Duration::from_secs(7));

    assert_eq!(snapshot.frames, 1);
    assert!((snapshot.average_fps - 0.2).abs() < f64::EPSILON);
}

#[test]
fn tcp_debugger_reads_and_writes_active_zx82_memory() {
    let mut snapshot = vec![0; 30 + 0xC000];
    snapshot[6..8].copy_from_slice(&0x4000u16.to_le_bytes());
    let mut emu = Emu::new(MachineType {
        is_plus: false,
        rom_version: RomVersion::V1_2,
        has_dos: false,
    });
    emu.load_z80_bytes(&snapshot).unwrap();

    let response = handle_command(
        &mut emu,
        r#"{"cmd":"write_memory","addr":32768,"data":[62,42]}"#,
        FrameStatsSnapshot::default(),
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&response).unwrap()["status"],
        "ok"
    );

    let response = handle_command(
        &mut emu,
        r#"{"cmd":"read_memory","addr":32768,"len":2}"#,
        FrameStatsSnapshot::default(),
    );
    let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    assert_eq!(response["data"], serde_json::json!([62, 42]));

    let response = handle_command(
        &mut emu,
        r#"{"cmd":"disassemble","addr":32768,"len":2}"#,
        FrameStatsSnapshot::default(),
    );
    let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    assert_eq!(response["instructions"][0]["text"], "LD A,2AH");
}

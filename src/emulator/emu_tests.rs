use super::*;
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions};
use std::io::{Cursor, Read};

fn formatted_disk(geometry: DiskGeometry) -> Vec<u8> {
    let mut disk = vec![0u8; geometry.bytes];
    let mut cursor = Cursor::new(&mut disk);
    let options = FormatVolumeOptions::new()
        .bytes_per_sector(512)
        .bytes_per_cluster(1024)
        .fats(2)
        .max_root_dir_entries(112)
        .total_sectors(geometry.total_sectors)
        .media(geometry.media)
        .sectors_per_track(9)
        .heads(geometry.heads);
    fatfs::format_volume(&mut cursor, options).unwrap();
    disk
}

fn root_dir_sector_with_file() -> [u8; 512] {
    let mut sector = [0u8; 512];
    sector[0..11].copy_from_slice(b"FFF     CAS");
    sector[11] = 0x20;
    sector[26..28].copy_from_slice(&2u16.to_le_bytes());
    sector[28..32].copy_from_slice(&4u32.to_le_bytes());
    sector
}

#[test]
fn zipped_snapshot_round_trips() {
    let mut emu = Emu::new(MachineType::standard(true, RomVersion::V2_2, true));
    emu.load_roms();
    emu.tvc_mut().unwrap().z80.state.pc = 0xBEEF;
    let zipped = zip_snapshot(&emu.save_snapshot()).unwrap();
    assert!(zipped.len() < emu.save_snapshot().len());

    let raw = unzip_snapshot(&zipped).unwrap();
    let mut restored = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));
    restored.load_snapshot(&raw).unwrap();
    assert_eq!(restored.machine_type, emu.machine_type);
    assert_eq!(restored.tvc().unwrap().z80.state.pc, 0xBEEF);
}

#[test]
fn snapshot_restores_selected_program() {
    let mut emu = Emu::new(MachineType::standard(true, RomVersion::V1_2, false));
    emu.progs = vec![
        ProgEntry {
            name: "First".to_string(),
            file_name: "first.cas".to_string(),
            is_cas: true,
            is_disk: false,
        },
        ProgEntry {
            name: "Second".to_string(),
            file_name: "second.cas".to_string(),
            is_cas: true,
            is_disk: false,
        },
    ];
    emu.selected_prog = 1;

    let snapshot = emu.save_snapshot();
    let mut restored = Emu::new(emu.machine_type);
    restored.progs = emu.progs.clone();
    restored.selected_prog = 0;
    restored.load_snapshot(&snapshot).unwrap();

    assert_eq!(restored.selected_prog, 1);
}

#[test]
fn debug_snapshot_restores_in_place_and_clears_typed_input() {
    let machine_type = MachineType::standard(false, RomVersion::V1_2, false);
    let mut emu = Emu::new(machine_type);
    emu.tvc_mut().unwrap().bus.mmu.set_map(0x10);
    emu.set_z80_register("PC", 0x4567);
    assert_eq!(emu.write_raw_bank("u0", 0x1234, &[0xA5]), Some(1));
    let snapshot = emu.capture_debug_snapshot().unwrap();

    emu.set_z80_register("PC", 0x9999);
    assert_eq!(emu.write_raw_bank("u0", 0x1234, &[0x5A]), Some(1));
    emu.queue_typed_text("RUN\r");
    emu.running = true;

    emu.restore_debug_snapshot(&snapshot).unwrap();

    assert!(!emu.running);
    assert_eq!(emu.z80_state().pc, 0x4567);
    assert_eq!(emu.read_mapped_memory(0x1234, 1), vec![0xA5]);
    assert!(emu.typed_text.is_empty());
    assert!(emu.typed_key.is_none());
    assert!(emu.frame_complete());
}

fn any_tvc_key_pressed(emu: &mut Emu) -> bool {
    let key = &mut emu.tvc_mut().unwrap().bus.key;
    (0..11).any(|row| {
        key.select_row(row);
        key.read_row() != 0xFF
    })
}

#[test]
fn timed_key_press_releases_after_requested_frames() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));

    emu.key_press_frames(49, 2).unwrap();
    assert!(any_tvc_key_pressed(&mut emu));
    assert_eq!(emu.timed_keys.get(&49), Some(&2));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));
    assert_eq!(emu.timed_keys.get(&49), Some(&1));

    emu.tick();
    assert!(!any_tvc_key_pressed(&mut emu));
    assert!(!emu.timed_keys.contains_key(&49));
}

#[test]
fn timed_key_press_rejects_zero_duration_and_unknown_codes() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));

    assert!(emu.key_press_frames(49, 0).is_err());
    assert!(emu.key_press_frames(999, 1).is_err());
    assert!(!any_tvc_key_pressed(&mut emu));
}

#[test]
fn typed_text_presses_and_releases_each_tvc_key() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));

    emu.type_text("a\r");
    assert!(!any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(!any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(!any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(any_tvc_key_pressed(&mut emu));

    emu.tick();
    assert!(!any_tvc_key_pressed(&mut emu));
}

#[test]
fn game_archive_uses_exact_requested_media() {
    let archive = game_archive(&[
        ("OTHER.CAS", b"wrong"),
        ("folder/TARGET.CAS", b"correct"),
        ("disk.dsk", b"disk"),
    ]);

    assert_eq!(
        extract_game_archive_member("TARGET.CAS", &archive).unwrap(),
        b"correct"
    );
    assert_eq!(
        extract_game_archive_member("folder\\target.cas", &archive).unwrap(),
        b"correct"
    );
}

#[test]
fn game_archive_reports_missing_media() {
    let archive = game_archive(&[("OTHER.CAS", b"wrong")]);
    let error = extract_game_archive_member("MISSING.CAS", &archive)
        .unwrap_err()
        .to_string();
    assert!(error.contains("MISSING.CAS"));
}

#[test]
fn recent_media_replaces_same_display_name() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));
    emu.recent_tapes = vec![
        "progs/TVBALL.CAS".to_string(),
        "old-cache/TVBALL.CAS".to_string(),
    ];

    emu.add_recent_tape("rtvc-media/tapes/tvball/TVBALL.CAS".to_string());

    assert_eq!(
        emu.recent_tapes,
        vec!["rtvc-media/tapes/tvball/TVBALL.CAS".to_string()]
    );
}

#[test]
fn gamebase_cas_starts_from_clean_boot_and_queues_run() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let mut cas = vec![0; 145];
    cas[0] = 0x11;

    emu.start_gamebase_media_bytes("TEST.CAS", &cas).unwrap();

    assert_eq!(
        emu.machine_type,
        MachineType::standard(true, RomVersion::V1_2, true)
    );
    assert!(emu.running);
    assert_eq!(emu.typed_text.iter().collect::<String>(), "RUN\r");
    assert_eq!(emu.loaded_tape.as_deref(), Some("TEST.CAS (Injected)"));
}

#[test]
fn gamebase_disk_starts_from_clean_boot_and_queues_load() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let disk = vec![0; 368_640];

    emu.start_gamebase_media_bytes("TEST.DSK", &disk).unwrap();

    assert!(emu.running);
    assert_eq!(emu.typed_text.iter().collect::<String>(), "LOAD \"*\"\r");
    assert_eq!(emu.loaded_disk[0].as_deref(), Some("TEST.DSK"));
}

#[test]
fn dirty_file_backed_disk_flushes_to_host_file() {
    let path = std::env::temp_dir().join(format!("rtvc-dirty-disk-{}.dsk", std::process::id()));
    std::fs::write(&path, formatted_disk(DiskGeometry::TVC_360K)).unwrap();

    let mut emu = Emu::new(MachineType::standard(true, RomVersion::V1_2, true));
    emu.load_roms();
    emu.insert_disk_file_path_drive(0, &path).unwrap();

    let fdc = emu
        .tvc_mut()
        .unwrap()
        .bus
        .extensions
        .hbf_mut()
        .unwrap()
        .get_fdc_mut();
    fdc.write(4, 0x01);
    fdc.write(1, 0);
    fdc.write(2, 6);
    fdc.write(0, 0xA0);
    for byte in root_dir_sector_with_file() {
        fdc.write(3, byte);
    }
    assert!(emu.disk_dirty(0));

    assert!(emu.flush_dirty_disk_files().is_empty());
    assert!(!emu.disk_dirty(0));

    let bytes = std::fs::read(&path).unwrap();
    let fs = FileSystem::new(Cursor::new(bytes), FsOptions::new()).unwrap();
    let mut file = fs.root_dir().open_file("FFF.CAS").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).unwrap();
    assert_eq!(contents.len(), 4);

    let _ = std::fs::remove_file(path);
}

#[test]
fn basic_save_writes_file_to_new_disk() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let snapshot = unzip_snapshot(GAMEBASE_BOOT_SNAPSHOT).unwrap();
    emu.load_snapshot(&snapshot).unwrap();
    emu.insert_empty_disk_drive(0, DiskGeometry::TVC_360K)
        .unwrap();
    emu.queue_typed_text("SAVE \"DW\"\r");
    emu.running = true;

    for _ in 0..1500 {
        emu.tick();
    }

    let bytes = emu.save_disk_bytes(0).unwrap();
    let fs = FileSystem::new(Cursor::new(bytes), FsOptions::new()).unwrap();
    assert!(fs.root_dir().open_file("DW.CAS").is_ok());
}

#[test]
fn new_720k_disk_uses_double_sided_geometry() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let snapshot = unzip_snapshot(GAMEBASE_BOOT_SNAPSHOT).unwrap();
    emu.load_snapshot(&snapshot).unwrap();
    emu.insert_empty_disk_drive(0, DiskGeometry::TVC_720K)
        .unwrap();

    let bytes = emu.save_disk_bytes(0).unwrap();
    assert_eq!(bytes.len(), DiskGeometry::TVC_720K.bytes);
    assert_eq!(u16::from_le_bytes([bytes[19], bytes[20]]), 1440);
    assert_eq!(bytes[21], 0xf9);
    assert_eq!(u16::from_le_bytes([bytes[24], bytes[25]]), 9);
    assert_eq!(u16::from_le_bytes([bytes[26], bytes[27]]), 2);

    let fs = FileSystem::new(Cursor::new(bytes), FsOptions::new()).unwrap();
    assert!(fs.root_dir().iter().next().is_none());
}

#[test]
fn basic_save_writes_file_to_new_720k_disk() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let snapshot = unzip_snapshot(GAMEBASE_BOOT_SNAPSHOT).unwrap();
    emu.load_snapshot(&snapshot).unwrap();
    emu.insert_empty_disk_drive(0, DiskGeometry::TVC_720K)
        .unwrap();
    emu.queue_typed_text("SAVE \"DW\"\r");
    emu.running = true;

    for _ in 0..1500 {
        emu.tick();
    }

    let bytes = emu.save_disk_bytes(0).unwrap();
    let fs = FileSystem::new(Cursor::new(bytes), FsOptions::new()).unwrap();
    assert!(fs.root_dir().open_file("DW.CAS").is_ok());
}

fn game_archive(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let cursor = std::io::Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(cursor);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, bytes) in entries {
        archive.start_file(*name, options).unwrap();
        archive.write_all(bytes).unwrap();
    }
    archive.finish().unwrap().into_inner()
}

#[test]
fn dropped_zip_launches_first_cas_or_dsk_member() {
    let zip = game_archive(&[("README.TXT", b"hi"), ("Games/Ball.CAS", &[0x11, 0x22])]);

    let Ok(DroppedMedia::Launch { name, bytes }) = classify_dropped_media("ball.zip", &zip) else {
        panic!("expected launchable media");
    };

    assert_eq!(name, "Ball.CAS");
    assert_eq!(bytes, vec![0x11, 0x22]);
}

#[test]
fn dropped_zip_prefers_snapshot_member() {
    let zip = game_archive(&[("GAME.DSK", &[0; 4]), ("state.rtvcsnap", b"SNAP")]);

    let Ok(DroppedMedia::Snapshot(bytes)) = classify_dropped_media("state.zip", &zip) else {
        panic!("expected snapshot");
    };

    assert_eq!(bytes, b"SNAP");
}

#[test]
fn dropped_files_classify_by_extension() {
    assert!(matches!(
        classify_dropped_media("GAME.dsk", &[1]),
        Ok(DroppedMedia::Launch { .. })
    ));
    assert!(matches!(
        classify_dropped_media("game.z80", &[1]),
        Ok(DroppedMedia::Z80(_))
    ));
    assert!(classify_dropped_media("notes.txt", &[1]).is_err());
    assert!(classify_dropped_media("empty.zip", &game_archive(&[("a.txt", b"")])).is_err());
}

#[test]
fn dropped_cas_bytes_boot_vtdos_and_queue_run() {
    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V2_2, false));
    let mut cas = vec![0; 145];
    cas[0] = 0x11;
    let zip = game_archive(&[("GAME.CAS", &cas)]);

    let Ok(DroppedMedia::Launch { name, bytes }) = classify_dropped_media("game.zip", &zip) else {
        panic!("expected launchable media");
    };
    emu.start_gamebase_media_bytes(&name, &bytes).unwrap();

    assert_eq!(
        emu.machine_type,
        MachineType::standard(true, RomVersion::V1_2, true)
    );
    assert_eq!(emu.typed_text.iter().collect::<String>(), "RUN\r");
}

#[test]
fn load_tape_snapshot_restores_selection_and_can_play() {
    let snapshot_path = std::path::Path::new("snapshots/load_tape.rtvcsnap.zip");
    if !snapshot_path.exists() {
        return;
    }
    let snapshot_data = unzip_snapshot(&std::fs::read(snapshot_path).unwrap()).unwrap();
    if snapshot::read_file(&snapshot_data).is_err() {
        return;
    }

    let mut emu = Emu::new(MachineType::standard(true, RomVersion::V1_2, true));
    emu.load_snapshot_file(snapshot_path).unwrap();
    assert_eq!(
        emu.progs
            .get(emu.selected_prog)
            .map(|entry| entry.file_name.as_str()),
        Some("TVBALL.CAS")
    );

    assert!(emu.tvc().unwrap().bus.tape_motor_on());
    emu.play_tape();
    assert!(emu.tvc().unwrap().bus.tape_play_active());
    let before = emu.tvc().unwrap().bus.tape_elapsed_cycles();
    emu.tvc_mut().unwrap().run_for_a_frame();
    assert!(emu.tvc().unwrap().bus.tape_elapsed_cycles() > before);
}

#[test]
fn z80_state_switches_system_and_exposes_mapped_debug_memory() {
    let mut snapshot = vec![0; 30 + 0xC000];
    snapshot[6..8].copy_from_slice(&0x1234u16.to_le_bytes());
    snapshot[30 + 0x4000] = 0xA5;

    let mut emu = Emu::new(MachineType::standard(false, RomVersion::V1_2, false));
    emu.load_z80_bytes(&snapshot).unwrap();

    assert_eq!(emu.system(), System::Zx82);
    assert_eq!(emu.z80_state().get_reg16(11), 0x1234);
    assert_eq!(emu.read_mapped_memory(0x8000, 1), vec![0xA5]);

    emu.write_mapped_memory(0x8000, &[0x5A]);
    assert_eq!(emu.read_mapped_memory(0x8000, 1), vec![0x5A]);
}

#[test]
fn zx82_switch_and_state_load_preserve_running_state() {
    let machine_type = MachineType::standard(false, RomVersion::V1_2, false);
    let mut snapshot = vec![0; 30 + 0xC000];
    snapshot[6..8].copy_from_slice(&0x1234u16.to_le_bytes());

    let mut running = Emu::new(machine_type);
    running.switch_to_zx82().unwrap();
    assert!(running.running);
    running.load_z80_bytes(&snapshot).unwrap();
    assert!(running.running);

    let mut paused = Emu::new(machine_type);
    paused.running = false;
    paused.switch_to_zx82().unwrap();
    assert!(!paused.running);
    paused.load_z80_bytes(&snapshot).unwrap();
    assert!(!paused.running);
}

#[test]
fn machine_ids_round_trip_presets_and_custom_mixes() {
    for machine in MachineType::all_types() {
        assert_eq!(MachineType::from_id(&machine.id()), Some(machine));
        assert!(machine.preset_label().is_some());
    }
    assert_eq!(
        MachineType::from_id("64k-plus-1.2-vtdos"),
        Some(MachineType::standard(true, RomVersion::V1_2, true))
    );
    let custom = MachineType::standard(false, RomVersion::V1_2, false)
        .with_slot(1, Some(ExpansionCard::VtDos12))
        .with_cart("pack_man");
    assert_eq!(custom.id(), "64ktvc12+ex1:vtdos12+cart:pack_man");
    assert_eq!(MachineType::from_id(&custom.id()), Some(custom));
    assert!(custom.preset_label().is_none());
    assert!(MachineType::from_id("64k+tvc12+ex1:vtdos12+ex1:hostfile").is_none());
    assert!(MachineType::from_id("64k+tvc12+ex1:floppy").is_none());
    assert!(MachineType::from_id("64ktvc12+ex4:vtdos12").is_none());
    assert_eq!(
        MachineType::from_id("64ktvc12+cart:vtdos12")
            .and_then(|machine| machine.cart)
            .map(|cart| cart.as_str().to_string()),
        Some("VT-DOS12-CART".to_string())
    );
}

#[test]
fn machine_ids_and_emut_round_trip_multiple_cards() {
    let machine = MachineType::standard(true, RomVersion::V1_2, true)
        .with_slot(1, Some(ExpansionCard::VtDos12))
        .with_slot(3, Some(ExpansionCard::HostFile));
    assert_eq!(
        machine.id(),
        "64k+tvc12+ex0:vtdos12+ex1:vtdos12+ex3:hostfile"
    );
    assert_eq!(MachineType::from_id(&machine.id()), Some(machine));
    assert_eq!(machine.slots_with(ExpansionCard::VtDos12), vec![0, 1]);
    assert!(machine.preset_label().is_none());
    let chunk = machine.write_snapshot_chunk();
    assert_eq!(chunk[2], 1);
    assert_eq!(&chunk[chunk.len() - 4..], &[1, 1, 0, 2]);
    assert_eq!(MachineType::from_snapshot_chunk(&chunk).unwrap(), machine);

    let host_only = MachineType::standard(true, RomVersion::V2_2, false)
        .with_slot(0, Some(ExpansionCard::HostFile));
    let chunk = host_only.write_snapshot_chunk();
    assert_eq!(chunk[2], 0);
    assert_eq!(MachineType::from_snapshot_chunk(&chunk).unwrap(), host_only);
    assert!(MachineType::from_snapshot_chunk(&[1, 0, 0, 2, 0, 3, 0, 0, 0]).is_err());
}

#[test]
fn two_hbf_cards_get_separate_disk_units() {
    let machine = MachineType::standard(true, RomVersion::V1_2, true)
        .with_slot(2, Some(ExpansionCard::VtDos12));
    let mut emu = Emu::new(machine);
    emu.load_roms();
    assert_eq!(emu.disk_unit_count(), 4);
    assert_eq!(emu.disk_unit_target(3), Some((2, 1)));
    assert_eq!(emu.disk_unit_label(2), "ex2 A:");
    let disk = formatted_disk(DiskGeometry::TVC_360K);
    emu.insert_disk_bytes_drive(2, "second.dsk", &disk).unwrap();
    assert_eq!(emu.save_disk_bytes(2).unwrap(), disk);
    assert!(emu.save_disk_bytes(0).is_none());

    let snapshot = emu.save_snapshot();
    let state = Emu::read_emu_snapshot_state(&snapshot).unwrap();
    assert_eq!(
        state.loaded_disk_file_names[2].as_deref(),
        Some("second.dsk")
    );
    assert_eq!(state.machine_type, Some(machine));
}

fn host_file_emu(name: &str, hbf_slot0: bool) -> (Box<Emu>, PathBuf) {
    let dir = std::env::temp_dir().join(format!("rtvc-hostcard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let machine = MachineType::standard(true, RomVersion::V1_2, hbf_slot0)
        .with_slot(1, Some(ExpansionCard::HostFile));
    let mut emu = Box::new(Emu::new(machine));
    emu.set_host_file_dir(dir.clone());
    emu.set_fast_boot(true);
    emu.load_roms();
    emu.running = true;
    (emu, dir)
}

fn run_frames(emu: &mut Emu, frames: usize) {
    for _ in 0..frames {
        emu.tick();
    }
}

#[test]
fn host_file_card_is_registered_as_file_device_at_boot() {
    for hbf_slot0 in [false, true] {
        let (mut emu, dir) = host_file_emu(&format!("boot{hbf_slot0}"), hbf_slot0);
        run_frames(&mut emu, if hbf_slot0 { 700 } else { 300 });
        assert_eq!(
            emu.read_raw_bank("u0", 0x0070, 5).unwrap(),
            b"\x04FILE",
            "hbf in slot 0: {hbf_slot0}"
        );
        // INIT routes channel 5 to the card in slot 1.
        assert_eq!(emu.read_raw_bank("u0", 0x0B05, 1).unwrap(), vec![0x81]);
        assert_eq!(emu.read_raw_bank("u0", 0x0B0D, 1).unwrap(), vec![0x81]);
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn basic_save_and_load_go_through_the_host_file_card() {
    let (mut emu, dir) = host_file_emu("saveload", false);
    run_frames(&mut emu, 300);
    emu.type_text("10 PRINT 42\r");
    run_frames(&mut emu, 100);
    emu.type_text("SAVE \"HELLO\"\r");
    run_frames(&mut emu, 200);

    let saved = std::fs::read(dir.join("hello.cas")).expect("SAVE created hello.cas");
    let (file_type, stream) = crate::cas::decode_cas_container(&saved).unwrap();
    assert_eq!(file_type, crate::cas::TVC_CAS_TYPE_UNBUFFERED);
    let payload = &saved[crate::cas::TVC_CAS_HEADER_LEN..];
    assert!(stream.len() > 16 && payload.contains(&0x0A));
    let generator = TapeBitstreamGenerator::new(&saved, "hello.cas");
    assert!(generator.is_ok(), "saved file is not a valid tape image");

    let program = emu.read_raw_bank("u0", 0x19EF, payload.len()).unwrap();
    assert_eq!(program, payload);

    emu.type_text("NEW\r");
    run_frames(&mut emu, 100);
    assert_ne!(
        emu.read_raw_bank("u0", 0x19EF, payload.len()).unwrap(),
        payload
    );
    emu.type_text("LOAD \"HELLO\"\r");
    run_frames(&mut emu, 200);
    assert_eq!(
        emu.read_raw_bank("u0", 0x19EF, payload.len()).unwrap(),
        payload
    );

    emu.type_text("NEW\r");
    run_frames(&mut emu, 100);
    assert!(emu.tvc_mut().unwrap().load_cas(&saved));
    assert_eq!(
        emu.read_raw_bank("u0", 0x19EF, payload.len()).unwrap(),
        payload
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn existing_cas_program_loads_through_the_host_file_card() {
    let (mut emu, dir) = host_file_emu("tvball", false);
    let cas = std::fs::read("progs/TVBALL.CAS").unwrap();
    std::fs::write(dir.join("TVBALL.CAS"), &cas).unwrap();
    run_frames(&mut emu, 300);
    emu.type_text("LOAD \"tvball\"\r");
    run_frames(&mut emu, 300);
    let (_, stream) = crate::cas::decode_cas_container(&cas).unwrap();
    // TVBALL autostarts and rewrites later lines, so compare only the first line.
    let payload = &stream[16..32];
    assert_eq!(
        emu.read_raw_bank("u0", 0x19EF, payload.len()).unwrap(),
        payload
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn vt_dos_system_disk_lists_a_directory() {
    let mut emu =
        Emu::new(MachineType::standard(true, RomVersion::V2_2, true).with_cart("VT-DOS12-CART"));
    emu.set_fast_boot(true);
    emu.load_roms();
    emu.insert_disk_file_path(std::path::Path::new("progs/VTDOS-SYSTEM.DSK"))
        .unwrap();
    emu.running = true;
    for _ in 0..700 {
        emu.tick();
    }
    emu.type_text("DIR\r");
    for _ in 0..250 {
        emu.tick();
    }
    let colored = emu
        .framebuffer()
        .pixels
        .iter()
        .filter(|px| *px & 0x00ff_ffff != 0)
        .count();
    assert!(
        colored > 2_000,
        "vt-dos did not list the system disk, pc={:04X}, colored={colored}",
        emu.z80_state().pc
    );
}

#[test]
fn pack_man_cart_autostarts_into_ram() {
    let mut emu =
        Emu::new(MachineType::standard(false, RomVersion::V1_2, false).with_cart("pack_man"));
    emu.set_fast_boot(true);
    emu.load_roms();
    assert_eq!(emu.read_raw_bank("cart", 0, 4).unwrap(), b"MOPS");

    let cart = std::fs::read("roms/pack_man.crt").unwrap();
    for _ in 0..4000 {
        emu.tvc_mut().unwrap().run_for_a_frame();
        if (0x2000..0x4000).contains(&emu.z80_state().pc) {
            // The cart copies itself to 0x2000 and then runs. One frame of that
            // code rewrites the loader, so compare a later slice of the image.
            let copied = emu.read_raw_bank("u0", 0x2100, 32).unwrap();
            assert_eq!(copied, cart[0x500..0x520]);
            return;
        }
    }
    panic!("pack_man did not reach RAM, pc={:04X}", emu.z80_state().pc);
}

#[test]
fn legacy_three_byte_machine_chunk_is_slot0_without_cart() {
    let machine = MachineType::from_snapshot_chunk(&[1, 0, 1]).unwrap();
    assert_eq!(machine, MachineType::standard(true, RomVersion::V1_2, true));
}

#[test]
fn custom_machine_snapshot_keeps_cart_and_hbf_slot() {
    let machine = MachineType::standard(false, RomVersion::V1_2, false)
        .with_slot(1, Some(ExpansionCard::VtDos12))
        .with_cart("VT-DOS12-CART");
    let mut emu = Emu::new(machine);
    emu.load_roms();
    assert!(emu.roms_loaded);
    assert!(emu.tvc().unwrap().card_kind(0).is_none());
    assert!(emu.tvc().unwrap().hbf_slots() == vec![1]);
    let cart = std::fs::read("roms/VT-DOS12-CART.crt").unwrap();
    assert_eq!(emu.read_raw_bank("cart", 0, 16).unwrap(), cart[..16]);

    let snapshot = emu.save_snapshot();
    let mut restored = Emu::new(MachineType::standard(true, RomVersion::V2_2, true));
    restored.load_snapshot(&snapshot).unwrap();
    assert_eq!(restored.machine_type, machine);
    assert!(restored.tvc().unwrap().card_kind(0).is_none());
    assert!(restored.tvc().unwrap().hbf_slots() == vec![1]);
    assert_eq!(restored.read_raw_bank("cart", 0, 16).unwrap(), cart[..16]);

    let disk = formatted_disk(DiskGeometry::TVC_360K);
    restored.insert_disk_bytes("slot1.dsk", &disk).unwrap();
    assert_eq!(restored.save_disk_bytes(0).unwrap(), disk);
}

#[test]
fn zx82_and_cpc_snapshots_reopen_on_their_machine() {
    let machine_type = MachineType::standard(false, RomVersion::V1_2, false);
    let mut source = Emu::new(machine_type);
    let mut target = Emu::new(machine_type);
    for switch in [Emu::switch_to_zx82, Emu::switch_to_cpc] {
        if switch(&mut source).is_err() {
            continue; // ROMs not present
        }
        for _ in 0..20 {
            source.tick();
        }
        source.write_mapped_memory(0x8000, &[0xA5]);
        let data = source.save_snapshot();
        target.load_snapshot(&data).unwrap();
        assert_eq!(target.system(), source.system());
        assert_eq!(target.z80_state(), source.z80_state());
        assert_eq!(target.read_mapped_memory(0x8000, 1), vec![0xA5]);
    }

    // TVC snapshots still carry no system chunk and load as TVC.
    let tvc = Emu::new(machine_type).save_snapshot();
    let chunks = crate::snapshot::read_file(&tvc).unwrap();
    assert_eq!(crate::snapshot::system_tag(&chunks), None);
    target.load_snapshot(&tvc).unwrap();
    assert_eq!(target.system(), System::Tvc);
}

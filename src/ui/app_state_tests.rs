use super::*;
use crate::emu::RomVersion;

#[test]
fn parses_minimal_app_state() {
    let state = parse_state(
        r#"
machine_type = "64k-plus-2.2-vtdos"
video_model = "fast-frame"
fast_boot = true

[tape]
selected = "TVBALL.CAS"
loaded = true
recent = ["TVBALL.CAS", "TVBALL2.CAS"]

[disk]
selected = "VT-DOS \"Games\".dsk"
loaded = false
recent = ["Games.dsk"]
"#,
    );

    assert_eq!(
        state.machine_type,
        Some(MachineType::standard(true, RomVersion::V2_2, true))
    );
    assert_eq!(state.vid_model, Some(VidModel::FastFrame));
    assert!(state.fast_boot);
    assert_eq!(state.tape_file_name.as_deref(), Some("TVBALL.CAS"));
    assert!(state.tape_loaded);
    assert_eq!(
        state.recent_tapes,
        vec!["TVBALL.CAS".to_string(), "TVBALL2.CAS".to_string()]
    );
    assert_eq!(
        state.disk_file_name.as_deref(),
        Some("VT-DOS \"Games\".dsk")
    );
    assert!(!state.disk_loaded);
    assert_eq!(state.recent_disks, vec!["Games.dsk".to_string()]);
}

#[test]
fn parses_custom_machine_id() {
    let state = parse_state("machine_type = \"64ktvc12+ex1:vtdos12+cart:vtdos12\"\n");
    assert_eq!(
        state.machine_type,
        Some(
            MachineType::standard(false, RomVersion::V1_2, false)
                .with_slot(1, Some(crate::emu::ExpansionCard::VtDos12))
                .with_cart("VT-DOS12-CART")
        )
    );
}

#[test]
fn host_file_settings_round_trip_beside_machine_id() {
    let state = AppState {
        machine_type: Some(
            MachineType::standard(true, RomVersion::V1_2, false)
                .with_slot(1, Some(crate::emu::ExpansionCard::HostFile)),
        ),
        host_file_dir: Some("C:\\TVC \"files\"".to_string()),
        host_file_default: Some(false),
        ..AppState::default()
    };
    let text = host_file_lines(&state);
    assert_eq!(
        text,
        "host_file_dir = \"C:\\\\TVC \\\"files\\\"\"\nhost_file_default = false\n"
    );
    let parsed = parse_state(&format!(
        "machine_type = \"{}\"\n{text}",
        machine_type_id(state.machine_type)
    ));
    assert_eq!(parsed.machine_type, state.machine_type);
    assert_eq!(parsed.host_file_dir, state.host_file_dir);
    assert_eq!(parsed.host_file_default, Some(false));
    assert_eq!(parse_state("").host_file_default, None);
}

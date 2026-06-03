use eframe::egui::ViewportBuilder;
use rtvc::{app_state, emu, ui};

fn main() -> eframe::Result<()> {
    let app_state_file = app_state::AppStateFile::load();
    let machine_type = app_state_file
        .state
        .machine_type
        .unwrap_or_else(|| emu::MachineType::all_types()[0]);
    let mut emu = emu::Emu::new(machine_type);
    if let Some(vid_model) = app_state_file.state.vid_model {
        emu.tvc.set_vid_model(vid_model);
    }
    emu.load_roms();
    let mut loaded_snapshot = false;
    if let Some(snapshot_path) = std::env::args_os().nth(1) {
        let snapshot_path = std::path::PathBuf::from(snapshot_path);
        if let Err(err) = emu.load_snapshot_file(&snapshot_path) {
            eprintln!("failed to load snapshot {}: {err}", snapshot_path.display());
        } else {
            loaded_snapshot = true;
        }
    }
    if !loaded_snapshot {
        if app_state_file.state.disk_loaded {
            if let Some(file_name) = &app_state_file.state.disk_file_name {
                emu.insert_disk_by_file_name(file_name);
            }
        }
        if app_state_file.state.tape_loaded {
            if let Some(file_name) = &app_state_file.state.tape_file_name {
                emu.inject_tape_by_file_name(file_name);
            }
        }
    }
    let app = ui::EmuApp::new(emu, app_state_file);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_title(format!(
                "rtvc v{} - Videoton TV Computer Emulator",
                env!("CARGO_PKG_VERSION")
            )),
        ..Default::default()
    };

    eframe::run_native("rtvc", options, Box::new(|_cc| Ok(Box::new(app))))
}

use eframe::egui::ViewportBuilder;
use rtvc::{emu, ui};

fn main() -> eframe::Result<()> {
    let machine_type = emu::MachineType::all_types()[0];
    let mut emu = emu::Emu::new(machine_type);
    emu.load_roms();
    if let Some(snapshot_path) = std::env::args_os().nth(1) {
        let snapshot_path = std::path::PathBuf::from(snapshot_path);
        if let Err(err) = emu.load_snapshot_file(&snapshot_path) {
            eprintln!("failed to load snapshot {}: {err}", snapshot_path.display());
        }
    }
    let app = ui::EmuApp::new(emu);

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_title("rtvc - Videoton TV Computer Emulator"),
        ..Default::default()
    };

    eframe::run_native("rtvc", options, Box::new(|_cc| Ok(Box::new(app))))
}

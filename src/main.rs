use eframe::egui::ViewportBuilder;
use rtvc::{emu, ui};

fn main() -> eframe::Result<()> {
    let machine_type = emu::MachineType::all_types()[0];
    let mut app = ui::EmuApp::new(emu::Emu::new(machine_type));
    app.emu.load_roms();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_title("rtvc - Videoton TV Computer Emulator"),
        ..Default::default()
    };

    eframe::run_native("rtvc", options, Box::new(|_cc| Ok(Box::new(app))))
}

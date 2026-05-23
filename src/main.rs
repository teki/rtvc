mod emu;
mod key;
mod log;
mod mmu;
mod tvc;
mod ui;
mod vid;
mod z80;
mod z80_tables;
mod dasm;
mod asm;

use eframe::egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    let mut app = ui::EmuApp::new(emu::Emu::new());
    app.emu.load_roms();

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(true)
            .with_title("rtvc - Videoton TV Computer Emulator"),
        ..Default::default()
    };

    eframe::run_native(
        "rtvc",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}

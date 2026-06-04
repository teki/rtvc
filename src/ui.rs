use std::time::{Duration, Instant};

use crate::app_state::{AppState, AppStateFile};
use crate::audio::NativeAudioSink;
use crate::emu::{Emu, MachineType, ProgEntry};
use crate::vid::VidModel;
use eframe::egui::{self, ColorImage, TextureHandle};

const TVC_REFRESH_HZ: u32 = 50;
const TVC_FRAME_DT: f64 = 1.0 / TVC_REFRESH_HZ as f64;

fn egui_key_to_js_code(key: egui::Key) -> Option<u32> {
    Some(match key {
        egui::Key::Backspace => 8,
        egui::Key::Tab => 9,
        egui::Key::Enter => 13,
        egui::Key::Space => 32,
        egui::Key::Escape => 27,
        egui::Key::Delete => 46,
        egui::Key::ArrowLeft => 37,
        egui::Key::ArrowUp => 38,
        egui::Key::ArrowRight => 39,
        egui::Key::ArrowDown => 40,
        egui::Key::Home => 36,
        egui::Key::End => 35,
        egui::Key::PageUp => 33,
        egui::Key::PageDown => 34,
        egui::Key::Insert => 45,
        egui::Key::A => 65,
        egui::Key::B => 66,
        egui::Key::C => 67,
        egui::Key::D => 68,
        egui::Key::E => 69,
        egui::Key::F => 70,
        egui::Key::G => 71,
        egui::Key::H => 72,
        egui::Key::I => 73,
        egui::Key::J => 74,
        egui::Key::K => 75,
        egui::Key::L => 76,
        egui::Key::M => 77,
        egui::Key::N => 78,
        egui::Key::O => 79,
        egui::Key::P => 80,
        egui::Key::Q => 81,
        egui::Key::R => 82,
        egui::Key::S => 83,
        egui::Key::T => 84,
        egui::Key::U => 85,
        egui::Key::V => 86,
        egui::Key::W => 87,
        egui::Key::X => 88,
        egui::Key::Y => 89,
        egui::Key::Z => 90,
        egui::Key::Num0 => 48,
        egui::Key::Num1 => 49,
        egui::Key::Num2 => 50,
        egui::Key::Num3 => 51,
        egui::Key::Num4 => 52,
        egui::Key::Num5 => 53,
        egui::Key::Num6 => 54,
        egui::Key::Num7 => 55,
        egui::Key::Num8 => 56,
        egui::Key::Num9 => 57,
        egui::Key::Minus => 189,
        egui::Key::Equals => 187,
        egui::Key::Comma => 188,
        egui::Key::Period => 190,
        egui::Key::Semicolon => 186,
        egui::Key::Quote => 222,
        egui::Key::Backslash => 220,
        egui::Key::Slash => 191,
        egui::Key::OpenBracket => 219,
        egui::Key::CloseBracket => 221,
        egui::Key::Backtick => 192,
        _ => return None,
    })
}

fn selected_media_matches<F>(emu: &Emu, filter: F) -> bool
where
    F: Fn(&ProgEntry) -> bool,
{
    emu.progs
        .get(emu.selected_prog)
        .map(|entry| filter(entry))
        .unwrap_or(false)
}

fn media_entries<F>(progs: &[ProgEntry], filter: F) -> Vec<(usize, String)>
where
    F: Fn(&ProgEntry) -> bool,
{
    progs
        .iter()
        .enumerate()
        .filter(|(_, entry)| filter(entry))
        .map(|(index, entry)| (index, entry.name.clone()))
        .collect()
}

pub struct EmuApp {
    pub emu: Emu,
    screen_texture: Option<TextureHandle>,
    last_frame_time: Instant,
    last_repaint_time: Instant,
    emu_frame_accumulator: f64,
    frame_count: u32,
    fps: u32,
    prev_shift: bool,
    prev_ctrl: bool,
    prev_alt: bool,
    show_log: bool,
    file_status: Option<String>,
    machine_types: Vec<MachineType>,
    selected_machine: usize,
    audio: Option<NativeAudioSink>,
    audio_status: Option<String>,
    app_state_file: AppStateFile,
}

impl EmuApp {
    pub fn new(mut emu: Emu, app_state_file: AppStateFile) -> Self {
        let machine_types = MachineType::all_types();
        let selected_machine = Self::selected_machine_index(&machine_types, emu.machine_type);
        let (audio, audio_status) = match NativeAudioSink::new(emu.tvc.sound_sample_rate()) {
            Ok(sink) => (Some(sink), None),
            Err(err) => (None, Some(format!("Audio unavailable: {err}"))),
        };
        emu.recent_tapes = app_state_file.state.recent_tapes.clone();
        emu.recent_disks = app_state_file.state.recent_disks.clone();
        EmuApp {
            emu,
            screen_texture: None,
            last_frame_time: Instant::now(),
            last_repaint_time: Instant::now(),
            emu_frame_accumulator: 0.0,
            frame_count: 0,
            fps: 0,
            prev_shift: false,
            prev_ctrl: false,
            prev_alt: false,
            show_log: false,
            file_status: None,
            machine_types,
            selected_machine,
            audio,
            audio_status,
            app_state_file,
        }
    }

    fn selected_machine_index(machine_types: &[MachineType], machine_type: MachineType) -> usize {
        machine_types
            .iter()
            .position(|candidate| *candidate == machine_type)
            .unwrap_or(0)
    }

    fn sync_selection_from_emu(&mut self) {
        self.selected_machine =
            Self::selected_machine_index(&self.machine_types, self.emu.machine_type);
    }

    fn update_screen_texture(&mut self, ctx: &egui::Context) {
        if !self.emu.tvc.frame_complete {
            return;
        }

        let size = [608usize, 288];
        let pixels: Vec<u8> = self
            .emu
            .tvc
            .framebuffer
            .iter()
            .copied()
            .flat_map(u32::to_ne_bytes)
            .collect();
        let image = ColorImage::from_rgba_unmultiplied(size, &pixels);
        self.screen_texture = Some(ctx.load_texture("tvc-screen", image, Default::default()));
        self.emu.tvc.frame_complete = false;
    }

    fn handle_modifier(&mut self, new_shift: bool, new_ctrl: bool, new_alt: bool) {
        if new_shift != self.prev_shift {
            if new_shift {
                self.emu.tvc.key_down(16);
            } else {
                self.emu.tvc.key_up(16);
            }
            self.prev_shift = new_shift;
        }
        if new_ctrl != self.prev_ctrl {
            if new_ctrl {
                self.emu.tvc.key_down(17);
            } else {
                self.emu.tvc.key_up(17);
            }
            self.prev_ctrl = new_ctrl;
        }
        if new_alt != self.prev_alt {
            if new_alt {
                self.emu.tvc.key_down(18);
            } else {
                self.emu.tvc.key_up(18);
            }
            self.prev_alt = new_alt;
        }
    }

    fn save_screenshot(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        const SRC_W: usize = 608;
        const SRC_H: usize = 288;
        const OUT_W: usize = 768;
        const OUT_H: usize = 576;

        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(writer, OUT_W as u32, OUT_H as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut png_writer = encoder.write_header()?;

        let mut pixels = vec![0; OUT_W * OUT_H * 4];
        for y in 0..OUT_H {
            let src_y = y * SRC_H / OUT_H;
            for x in 0..OUT_W {
                let src_x = x * SRC_W / OUT_W;
                let rgba = self.emu.tvc.framebuffer[src_y * SRC_W + src_x].to_ne_bytes();
                let offset = (y * OUT_W + x) * 4;
                pixels[offset..offset + 4].copy_from_slice(&rgba);
            }
        }

        png_writer.write_image_data(&pixels)?;
        Ok(())
    }

    fn save_snapshot_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("compressed rtvc snapshot", &["zip"])
            .add_filter("rtvc snapshot", &["rtvcsnap"])
            .set_file_name("snapshot.rtvcsnap.zip")
            .save_file()
        {
            match self.emu.save_snapshot_file(&path) {
                Ok(()) => {
                    self.file_status = Some(format!("Saved: {}", path.display()));
                }
                Err(err) => {
                    self.file_status = Some(format!("Save failed: {}", err));
                }
            }
        }
    }

    fn load_snapshot_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("rtvc snapshot", &["rtvcsnap", "zip"])
            .pick_file()
        {
            match self.emu.load_snapshot_file(&path) {
                Ok(()) => {
                    self.sync_selection_from_emu();
                    self.file_status = Some(format!("Loaded: {}", path.display()));
                }
                Err(err) => {
                    self.file_status = Some(format!("Load failed: {}", err));
                }
            }
        }
    }

    fn load_tape_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("tape image", &["cas", "zip"])
            .pick_file()
        {
            match self.emu.play_tape_file_path(&path) {
                Ok(()) => {
                    self.save_app_state();
                    self.file_status = Some(format!("Loaded tape: {}", path.display()));
                }
                Err(err) => {
                    self.file_status = Some(format!("Tape load failed: {}", err));
                }
            }
        }
    }

    fn load_disk_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("disk image", &["dsk", "zip"])
            .pick_file()
        {
            match self.emu.insert_disk_file_path(&path) {
                Ok(()) => {
                    self.save_app_state();
                    self.file_status = Some(format!("Loaded disk: {}", path.display()));
                }
                Err(err) => {
                    self.file_status = Some(format!("Disk load failed: {}", err));
                }
            }
        }
    }

    fn save_screenshot_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("png image", &["png"])
            .set_file_name("rtvc-screen.png")
            .save_file()
        {
            match self.save_screenshot(&path) {
                Ok(()) => {
                    self.file_status = Some(format!("Saved: {}", path.display()));
                }
                Err(err) => {
                    self.file_status = Some(format!("Screenshot failed: {}", err));
                }
            }
        }
    }

    fn draw_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                self.draw_file_menu(ui);
                self.draw_machine_menu(ui);
                self.draw_tape_menu(ui);
                self.draw_disk_menu(ui);
                self.draw_view_menu(ui);
            });
        });
    }

    fn draw_file_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Load Snapshot...").clicked() {
                self.load_snapshot_dialog();
                ui.close_menu();
            }
            if ui.button("Save Snapshot...").clicked() {
                self.save_snapshot_dialog();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Load Tape...").clicked() {
                self.load_tape_dialog();
                ui.close_menu();
            }
            if ui.button("Load Disk...").clicked() {
                self.load_disk_dialog();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Save Screenshot...").clicked() {
                self.save_screenshot_dialog();
                ui.close_menu();
            }
        });
    }

    fn draw_machine_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Machine", |ui| {
            if ui
                .button(if self.emu.running { "Pause" } else { "Run" })
                .clicked()
            {
                self.emu.toggle_running();
                self.last_repaint_time = Instant::now();
                self.emu_frame_accumulator = 0.0;
                self.save_app_state();
                ui.close_menu();
            }

            if ui.button("Reset").clicked() {
                self.emu.reset();
                self.emu_frame_accumulator = 0.0;
                self.save_app_state();
                ui.close_menu();
            }

            ui.separator();
            ui.label("Type");
            let machine_types = self.machine_types.clone();
            for (index, machine_type) in machine_types.into_iter().enumerate() {
                if ui
                    .selectable_label(self.selected_machine == index, machine_type.label())
                    .clicked()
                {
                    self.selected_machine = index;
                    let vid_model = self.emu.tvc.vid_model();
                    self.emu.reload(machine_type);
                    self.emu.tvc.set_vid_model(vid_model);
                    self.save_app_state();
                    ui.close_menu();
                }
            }

            ui.separator();
            ui.label("Video");
            let vid_model = self.emu.tvc.vid_model();
            if ui
                .selectable_label(vid_model == VidModel::FastFrame, "Fast frame")
                .clicked()
            {
                self.emu.tvc.set_vid_model(VidModel::FastFrame);
                self.save_app_state();
                ui.close_menu();
            }
            if ui
                .selectable_label(vid_model == VidModel::Interleaved, "Interleaved")
                .clicked()
            {
                self.emu.tvc.set_vid_model(VidModel::Interleaved);
                self.save_app_state();
                ui.close_menu();
            }
        });
    }

    fn draw_tape_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Tape", |ui| {
            if ui.button("Open Tape File...").clicked() {
                self.load_tape_dialog();
                ui.close_menu();
            }

            if !self.emu.recent_tapes.is_empty() {
                ui.separator();
                ui.label("Recent Tapes:");
                for path_str in self.emu.recent_tapes.clone() {
                    let display_name = std::path::Path::new(&path_str)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());

                    if ui.button(display_name).clicked() {
                        let path = std::path::Path::new(&path_str);
                        if let Err(err) = self.emu.play_tape_file_path(path) {
                            self.file_status = Some(format!("Tape load failed: {}", err));
                        } else {
                            self.save_app_state();
                        }
                        ui.close_menu();
                    }
                }
            }

            ui.separator();

            let entries = media_entries(&self.emu.progs, |entry| entry.is_cas);
            if entries.is_empty() {
                ui.add_enabled(false, egui::Label::new("No tape images"));
            } else {
                for (index, name) in entries {
                    if ui
                        .selectable_label(self.emu.selected_prog == index, name)
                        .clicked()
                    {
                        self.emu.selected_prog = index;
                        self.emu.play_tape();
                        self.save_app_state();
                        ui.close_menu();
                    }
                }
            }

            ui.separator();
            let tape_selected = selected_media_matches(&self.emu, |entry| entry.is_cas);
            if ui
                .add_enabled(tape_selected, egui::Button::new("Inject"))
                .clicked()
            {
                self.emu.inject_selected_tape();
                self.save_app_state();
                ui.close_menu();
            }

            if self.emu.tvc.bus.tape_play_active() {
                if ui.button("Stop").clicked() {
                    self.emu.stop_tape();
                    self.save_app_state();
                    ui.close_menu();
                }
            } else if ui
                .add_enabled(tape_selected, egui::Button::new("Play"))
                .clicked()
            {
                self.emu.play_tape();
                self.save_app_state();
                ui.close_menu();
            }
        });
    }

    fn draw_disk_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Disk", |ui| {
            if ui.button("Open Disk File...").clicked() {
                self.load_disk_dialog();
                ui.close_menu();
            }

            if !self.emu.recent_disks.is_empty() {
                ui.separator();
                ui.label("Recent Disks:");
                for path_str in self.emu.recent_disks.clone() {
                    let display_name = std::path::Path::new(&path_str)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());

                    if ui.button(display_name).clicked() {
                        let path = std::path::Path::new(&path_str);
                        if let Err(err) = self.emu.insert_disk_file_path(path) {
                            self.file_status = Some(format!("Disk load failed: {}", err));
                        } else {
                            self.save_app_state();
                        }
                        ui.close_menu();
                    }
                }
            }

            ui.separator();

            let entries = media_entries(&self.emu.progs, |entry| entry.is_disk);
            if entries.is_empty() {
                ui.add_enabled(false, egui::Label::new("No disk images"));
            } else {
                for (index, name) in entries {
                    if ui
                        .selectable_label(self.emu.selected_prog == index, name)
                        .clicked()
                    {
                        self.emu.selected_prog = index;
                        self.emu.insert_selected_disk();
                        self.save_app_state();
                        ui.close_menu();
                    }
                }
            }

            ui.separator();
            let disk_selected = selected_media_matches(&self.emu, |entry| entry.is_disk);
            if ui
                .add_enabled(disk_selected, egui::Button::new("Insert"))
                .clicked()
            {
                self.emu.insert_selected_disk();
                self.save_app_state();
                ui.close_menu();
            }
        });
    }

    fn draw_view_menu(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.show_log, "IO Log");
        });
    }

    fn draw_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar")
            .resizable(false)
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    if ui.small_button("Reset").clicked() {
                        self.emu.reset();
                        self.emu_frame_accumulator = 0.0;
                        self.save_app_state();
                    }
                    ui.separator();
                    draw_tape_led(
                        ui,
                        self.emu.tvc.bus.tape_play_active(),
                        self.emu.get_current_tape_level(),
                    );
                    ui.label(if self.emu.tvc.bus.tape_play_active() {
                        "Tape active"
                    } else {
                        "Tape idle"
                    });
                    ui.separator();
                    ui.label(format!(
                        "Tape: {}",
                        self.emu.loaded_tape.as_deref().unwrap_or("(none)")
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Disk: {}",
                        self.emu.loaded_disk.as_deref().unwrap_or("(none)")
                    ));
                    ui.separator();
                    ui.label(if self.emu.running {
                        "Running"
                    } else {
                        "Paused"
                    });
                    ui.separator();
                    ui.label(format!("FPS: {}", self.fps));
                    ui.separator();
                    ui.label(format!(
                        "Machine: {}",
                        if self.emu.roms_loaded {
                            self.emu.machine_type.label()
                        } else {
                            format!("{} (ROMs missing)", self.emu.machine_type.label())
                        }
                    ));
                    if let Some(status) = &self.audio_status {
                        ui.separator();
                        ui.label(status);
                    }
                    if let Some(status) = &self.file_status {
                        ui.separator();
                        ui.label(status);
                    }
                });
            });
    }

    fn current_app_state(&self) -> AppState {
        AppState {
            machine_type: Some(self.emu.machine_type),
            vid_model: Some(self.emu.tvc.vid_model()),
            tape_file_name: self.emu.loaded_tape_file_name.clone(),
            tape_loaded: self.emu.loaded_tape_file_name.is_some(),
            disk_file_name: self.emu.loaded_disk_file_name.clone(),
            disk_loaded: self.emu.loaded_disk_file_name.is_some(),
            recent_tapes: self.emu.recent_tapes.clone(),
            recent_disks: self.emu.recent_disks.clone(),
        }
    }

    fn save_app_state(&mut self) {
        let state = self.current_app_state();
        if let Err(err) = self.app_state_file.save(&state) {
            self.file_status = Some(format!("Config save failed: {err}"));
        }
    }
}

fn draw_tape_led(ui: &mut egui::Ui, active: bool, level: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
    let color = if active {
        if level > 0.5 {
            egui::Color32::from_rgb(255, 218, 80)
        } else {
            egui::Color32::from_rgb(48, 180, 90)
        }
    } else {
        egui::Color32::from_rgb(55, 60, 58)
    };
    ui.painter().circle_filled(rect.center(), 4.5, color);
    ui.painter().circle_stroke(
        rect.center(),
        4.5,
        egui::Stroke::new(1.0, egui::Color32::from_gray(120)),
    );
}

impl eframe::App for EmuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            let modifiers = i.modifiers;
            self.handle_modifier(modifiers.shift, modifiers.ctrl, modifiers.alt);

            for event in &i.events {
                match event {
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => {
                        if let Some(code) = egui_key_to_js_code(*key) {
                            self.emu.tvc.key_down(code);
                        }
                    }
                    egui::Event::Key {
                        key,
                        pressed: false,
                        ..
                    } => {
                        if let Some(code) = egui_key_to_js_code(*key) {
                            self.emu.tvc.key_up(code);
                        }
                    }
                    egui::Event::Text(text) => {
                        for ch in text.chars() {
                            self.emu.tvc.key_press(ch);
                        }
                    }
                    _ => {}
                }
            }
        });

        let now = Instant::now();
        let dt = now.duration_since(self.last_repaint_time).as_secs_f64();
        self.last_repaint_time = now;
        if self.emu.running {
            self.emu_frame_accumulator += dt.min(0.1);
        }

        if self.emu.running && self.emu_frame_accumulator >= TVC_FRAME_DT {
            self.emu.tick();
            self.push_audio_samples();
            self.update_screen_texture(ctx);
            self.emu_frame_accumulator %= TVC_FRAME_DT;
            self.frame_count += 1;
        }

        let elapsed = self.last_frame_time.elapsed();
        if elapsed.as_secs() >= 1 {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.last_frame_time = Instant::now();
        }

        self.draw_menu_bar(ctx);
        self.draw_status_bar(ctx);

        if self.show_log {
            egui::TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .default_height(140.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong("IO Log");
                        if ui.small_button("Clear").clicked() {
                            self.emu.tvc.clear_log();
                        }
                    });
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let entries = self.emu.tvc.log_entries();
                            for entry in entries.iter().rev() {
                                ui.label(entry);
                            }
                        });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let disp_hw = 3.0 / 4.0;
            let w = avail.x.min(avail.y / disp_hw);
            let h = w * disp_hw;
            let img_size = egui::vec2(w, h);

            let (_, rect) = ui.allocate_space(img_size);
            if let Some(texture) = &self.screen_texture {
                ui.painter().image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            } else {
                ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
            }
        });

        if self.emu.running {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_app_state();
    }
}

impl EmuApp {
    fn push_audio_samples(&mut self) {
        let samples = self.emu.tvc.take_audio_samples();
        if let Some(audio) = &mut self.audio {
            audio.push_samples(&samples);
        }
    }
}

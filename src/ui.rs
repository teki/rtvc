use std::time::{Duration, Instant};

use crate::emu::{Emu, MachineType};
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
}

impl EmuApp {
    pub fn new(emu: Emu) -> Self {
        let machine_types = MachineType::all_types();
        let selected_machine = 0;
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
        }
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
            ui.heading("rtvc - Videoton TV Computer Emulator");

            ui.horizontal(|ui| {
                ui.label("Type:");
                let prev_selected = self.selected_machine;
                egui::ComboBox::from_id_salt("machine_type")
                    .selected_text(self.machine_types[self.selected_machine].label())
                    .show_ui(ui, |ui| {
                        for (i, mt) in self.machine_types.iter().enumerate() {
                            ui.selectable_value(&mut self.selected_machine, i, mt.label());
                        }
                    });
                if self.selected_machine != prev_selected {
                    let vid_model = self.emu.tvc.vid_model();
                    self.emu.reload(self.machine_types[self.selected_machine]);
                    self.emu.tvc.set_vid_model(vid_model);
                }

                ui.separator();

                ui.label("Video:");
                let mut vid_model = self.emu.tvc.vid_model();
                egui::ComboBox::from_id_salt("vid_model")
                    .selected_text(match vid_model {
                        VidModel::FastFrame => "Fast frame",
                        VidModel::Line => "Line",
                        VidModel::Interleaved => "Interleaved",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut vid_model, VidModel::FastFrame, "Fast frame");
                        ui.selectable_value(&mut vid_model, VidModel::Line, "Line");
                        ui.selectable_value(&mut vid_model, VidModel::Interleaved, "Interleaved");
                    });
                self.emu.tvc.set_vid_model(vid_model);

                ui.separator();

                if ui
                    .selectable_label(
                        !self.emu.running,
                        if self.emu.running {
                            "Running"
                        } else {
                            "Paused"
                        },
                    )
                    .clicked()
                {
                    self.emu.toggle_running();
                    self.last_repaint_time = Instant::now();
                    self.emu_frame_accumulator = 0.0;
                }
                if ui.button("Reset").clicked() {
                    self.emu.reset();
                    self.emu_frame_accumulator = 0.0;
                }
                ui.label(format!("FPS: {}", self.fps));
                ui.label(format!(
                    "ROMs: {}",
                    if self.emu.roms_loaded {
                        "loaded"
                    } else {
                        "not found"
                    }
                ));
                if ui.button("Log").clicked() {
                    self.show_log = !self.show_log;
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Save Snapshot").clicked() {
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

                if ui.button("Load Snapshot").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("rtvc snapshot", &["rtvcsnap", "zip"])
                        .pick_file()
                    {
                        match self.emu.load_snapshot_file(&path) {
                            Ok(()) => {
                                self.file_status = Some(format!("Loaded: {}", path.display()));
                            }
                            Err(err) => {
                                self.file_status = Some(format!("Load failed: {}", err));
                            }
                        }
                    }
                }

                if ui.button("Save Screenshot").clicked() {
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

                if let Some(ref status) = self.file_status {
                    ui.label(status);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Program:");
                egui::ComboBox::from_id_salt("prog_list")
                    .selected_text(
                        self.emu
                            .progs
                            .get(self.emu.selected_prog)
                            .map(|p| p.name.as_str())
                            .unwrap_or("(none)"),
                    )
                    .show_ui(ui, |ui| {
                        for (i, prog) in self.emu.progs.iter().enumerate() {
                            ui.selectable_value(&mut self.emu.selected_prog, i, &prog.name);
                        }
                    });
                if ui.button("Load").clicked() {
                    self.emu.load_selected_prog();
                }

                // Play / Stop controls for cassette files
                let selected_is_cas = self
                    .emu
                    .progs
                    .get(self.emu.selected_prog)
                    .map(|p| p.is_cas)
                    .unwrap_or(false);

                if selected_is_cas {
                    if self.emu.tvc.bus.tape_play_active() {
                        let level = self.emu.get_current_tape_level();
                        let btn_color = if level > 0.6 {
                            egui::Color32::from_rgb(255, 235, 59) // Bright yellow for high phase
                        } else if level < 0.4 {
                            egui::Color32::from_rgb(46, 125, 50) // Solid dark green for low phase
                        } else {
                            egui::Color32::from_rgb(128, 128, 128) // Neutral gray for silence
                        };

                        let stop_btn = egui::Button::new(
                            egui::RichText::new("Stop ⏹").color(egui::Color32::BLACK),
                        )
                        .fill(btn_color);
                        if ui.add(stop_btn).clicked() {
                            self.emu.stop_tape();
                        }
                    } else {
                        if ui.button("Play ▶").clicked() {
                            self.emu.play_tape();
                        }
                    }
                }

                if let Some(ref name) = self.emu.prog_loaded {
                    ui.label(format!("Loaded: {}", name));
                }
            });

            ui.separator();

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
}

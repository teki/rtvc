#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

use crate::app_state::{AppState, AppStateFile};
use crate::audio::NativeAudioSink;
use crate::emu::{Emu, MachineType, ProgEntry};
use crate::vid::VidModel;
use eframe::egui::{self, ColorImage, TextureHandle};

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub type DebuggerType = crate::debugger::DebuggerInterface;
#[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
pub type DebuggerType = ();

pub enum PendingFile {
    Tape { name: String, bytes: Vec<u8> },
    Disk { name: String, bytes: Vec<u8> },
    Snapshot { name: String, bytes: Vec<u8> },
    StorageResult { error: Option<String> },
    RecentCleared { kind: String },
}

#[cfg(target_arch = "wasm32")]
pub fn download_file(name: &str, data: &[u8], mime_type: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(body) = document.body() {
                let uint8_array = unsafe { js_sys::Uint8Array::view(data) };
                let array = js_sys::Array::new();
                array.push(&uint8_array);
                let blob_options = web_sys::BlobPropertyBag::new();
                blob_options.set_type(mime_type);
                if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence_and_options(&array, &blob_options) {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        if let Ok(a) = document.create_element("a") {
                            if let Ok(a) = a.dyn_into::<web_sys::HtmlAnchorElement>() {
                                a.set_href(&url);
                                a.set_download(name);
                                a.style().set_property("display", "none").ok();
                                body.append_child(&a).ok();
                                a.click();
                                body.remove_child(&a).ok();
                                web_sys::Url::revoke_object_url(&url).ok();
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(
        js_namespace = globalThis,
        js_name = rtvcTakeKeyboardEvents
    )]
    fn web_take_keyboard_events() -> js_sys::Array;

    #[wasm_bindgen::prelude::wasm_bindgen(
        js_namespace = globalThis,
        js_name = rtvcStoreRecentMedia
    )]
    fn web_store_recent_media(
        kind: &str,
        name: &str,
        bytes: &js_sys::Uint8Array,
    ) -> js_sys::Promise;

    #[wasm_bindgen::prelude::wasm_bindgen(
        js_namespace = globalThis,
        js_name = rtvcClearRecentMedia
    )]
    fn web_clear_recent_media(kind: &str) -> js_sys::Promise;
}

const TVC_REFRESH_HZ: u32 = 50;
const TVC_FRAME_DT: f64 = 1.0 / TVC_REFRESH_HZ as f64;

#[cfg(not(target_arch = "wasm32"))]
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
    pub debugger: Option<DebuggerType>,
    pressed_keys: std::collections::HashSet<u32>,
    #[cfg(target_arch = "wasm32")]
    pub file_tx: std::sync::mpsc::Sender<PendingFile>,
    #[cfg(target_arch = "wasm32")]
    pub file_rx: std::sync::mpsc::Receiver<PendingFile>,
}

impl EmuApp {
    pub fn new(mut emu: Emu, app_state_file: AppStateFile, debugger: Option<DebuggerType>) -> Self {
        let machine_types = MachineType::all_types();
        let selected_machine = Self::selected_machine_index(&machine_types, emu.machine_type);
        let (audio, audio_status) = match NativeAudioSink::new(emu.tvc.sound_sample_rate()) {
            Ok(sink) => (Some(sink), None),
            Err(err) => (None, Some(format!("Audio unavailable: {err}"))),
        };
        #[cfg(target_arch = "wasm32")]
        let (file_tx, file_rx) = std::sync::mpsc::channel();

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
            debugger,
            pressed_keys: std::collections::HashSet::new(),
            #[cfg(target_arch = "wasm32")]
            file_tx,
            #[cfg(target_arch = "wasm32")]
            file_rx,
        }
    }

    fn selected_machine_index(machine_types: &[MachineType], machine_type: MachineType) -> usize {
        machine_types
            .iter()
            .position(|candidate| *candidate == machine_type)
            .unwrap_or(0)
    }

    pub fn set_audio_status(&mut self, status: String) {
        self.audio_status = Some(status);
    }

    pub fn set_file_status(&mut self, status: String) {
        self.file_status = Some(status);
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(not(target_arch = "wasm32"))]
    fn save_screenshot(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        self.emu.save_screenshot(path)
    }

    fn save_snapshot_dialog(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let snapshot = self.emu.save_snapshot();
            match crate::emu::zip_snapshot(&snapshot) {
                Ok(zipped) => {
                    download_file("snapshot.rtvcsnap.zip", &zipped, "application/zip");
                    self.file_status = Some("Snapshot download started".to_string());
                }
                Err(err) => {
                    self.file_status = Some(format!("Snapshot zip failed: {err}"));
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    }

    fn load_snapshot_dialog(&mut self, _ctx: egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            let file_tx = self.file_tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("rtvc snapshot", &["rtvcsnap", "zip"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let name = file.file_name();
                    let bytes = file.read().await;
                    let _ = file_tx.send(PendingFile::Snapshot { name, bytes });
                    _ctx.request_repaint();
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    }

    fn load_tape_dialog(&mut self, _ctx: egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            let file_tx = self.file_tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("tape image", &["cas", "zip"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let name = file.file_name();
                    let bytes = file.read().await;
                    let _ = file_tx.send(PendingFile::Tape { name, bytes });
                    _ctx.request_repaint();
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    }

    fn load_disk_dialog(&mut self, _ctx: egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            let file_tx = self.file_tx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file = rfd::AsyncFileDialog::new()
                    .add_filter("disk image", &["dsk", "zip"])
                    .pick_file()
                    .await;
                if let Some(file) = file {
                    let name = file.file_name();
                    let bytes = file.read().await;
                    let _ = file_tx.send(PendingFile::Disk { name, bytes });
                    _ctx.request_repaint();
                }
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
    }

    fn save_screenshot_dialog(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            match self.emu.get_screenshot_png() {
                Ok(png) => {
                    download_file("rtvc-screen.png", &png, "image/png");
                    self.file_status = Some("Screenshot download started".to_string());
                }
                Err(err) => {
                    self.file_status = Some(format!("Screenshot failed: {err}"));
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
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
                self.load_snapshot_dialog(ui.ctx().clone());
                ui.close_menu();
            }
            if ui.button("Save Snapshot...").clicked() {
                self.save_snapshot_dialog();
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Load Tape...").clicked() {
                self.load_tape_dialog(ui.ctx().clone());
                ui.close_menu();
            }
            if ui.button("Load Disk...").clicked() {
                self.load_disk_dialog(ui.ctx().clone());
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
                    if let Err(err) = self.emu.reload(machine_type) {
                        self.file_status = Some(err);
                    }
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
                self.load_tape_dialog(ui.ctx().clone());
                ui.close_menu();
            }

            #[cfg(target_arch = "wasm32")]
            {
                if !self.emu.recent_tapes_wasm.is_empty() {
                    ui.separator();
                    ui.label("Recent Tapes:");
                    for recent in self.emu.recent_tapes_wasm.clone() {
                        if ui.button(&recent.name).clicked() {
                            if let Err(err) = self.emu.play_tape_bytes(&recent.name, &recent.bytes) {
                                self.file_status = Some(format!("Tape load failed: {}", err));
                            } else {
                                self.persist_wasm_recent("tape", recent, ui.ctx().clone());
                            }
                            ui.close_menu();
                        }
                    }
                }
                if !self.emu.recent_tapes_wasm.is_empty() {
                    if ui.button("Clear Recent Tapes").clicked() {
                        self.clear_wasm_recents("tape", ui.ctx().clone());
                        ui.close_menu();
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
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
            let tape_injectable = self.emu.can_inject_tape();
            if ui
                .add_enabled(tape_injectable, egui::Button::new("Inject"))
                .clicked()
            {
                match self.emu.inject_tape() {
                    Ok(()) => {
                        self.file_status = self
                            .emu
                            .loaded_tape
                            .as_ref()
                            .map(|name| format!("Injected tape: {name}"));
                        self.save_app_state();
                    }
                    Err(err) => {
                        self.file_status = Some(format!("Tape injection failed: {err}"));
                    }
                }
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
                self.load_disk_dialog(ui.ctx().clone());
                ui.close_menu();
            }

            #[cfg(target_arch = "wasm32")]
            {
                if !self.emu.recent_disks_wasm.is_empty() {
                    ui.separator();
                    ui.label("Recent Disks:");
                    for recent in self.emu.recent_disks_wasm.clone() {
                        if ui.button(&recent.name).clicked() {
                            if let Err(err) = self.emu.insert_disk_bytes(&recent.name, &recent.bytes) {
                                self.file_status = Some(format!("Disk load failed: {}", err));
                            } else {
                                self.persist_wasm_recent("disk", recent, ui.ctx().clone());
                            }
                            ui.close_menu();
                        }
                    }
                }
                if !self.emu.recent_disks_wasm.is_empty() {
                    if ui.button("Clear Recent Disks").clicked() {
                        self.clear_wasm_recents("disk", ui.ctx().clone());
                        ui.close_menu();
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
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

    #[cfg(target_arch = "wasm32")]
    fn persist_wasm_recent(
        &self,
        kind: &'static str,
        recent: crate::emu::WasmRecentFile,
        ctx: egui::Context,
    ) {
        let tx = self.file_tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let bytes = js_sys::Uint8Array::new_with_length(recent.bytes.len() as u32);
            bytes.copy_from(&recent.bytes);
            let result = wasm_bindgen_futures::JsFuture::from(web_store_recent_media(
                kind,
                &recent.name,
                &bytes,
            ))
            .await;
            let error = result.err().map(js_value_string);
            let _ = tx.send(PendingFile::StorageResult { error });
            ctx.request_repaint();
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn clear_wasm_recents(&self, kind: &'static str, ctx: egui::Context) {
        let tx = self.file_tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result =
                wasm_bindgen_futures::JsFuture::from(web_clear_recent_media(kind)).await;
            let event = match result {
                Ok(_) => PendingFile::RecentCleared {
                    kind: kind.to_string(),
                },
                Err(err) => PendingFile::StorageResult {
                    error: Some(js_value_string(err)),
                },
            };
            let _ = tx.send(event);
            ctx.request_repaint();
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn handle_wasm_keyboard_events(&mut self) {
        for event in web_take_keyboard_events().iter() {
            let event_type = js_sys::Reflect::get(
                &event,
                &wasm_bindgen::JsValue::from_str("type"),
            )
            .ok()
            .and_then(|value| value.as_string())
            .unwrap_or_default();
            match event_type.as_str() {
                "down" => {
                    let code = wasm_event_code(&event);
                    let first_press = code == 0 || self.pressed_keys.insert(code);
                    if code != 0 && first_press {
                        self.emu.tvc.key_down(code);
                    }
                    if first_press {
                        for ch in wasm_event_text(&event).chars() {
                            self.emu.tvc.key_press(ch);
                        }
                    }
                }
                "up" => {
                    let code = wasm_event_code(&event);
                    if code != 0 {
                        self.pressed_keys.remove(&code);
                        self.emu.tvc.key_up(code);
                    }
                }
                "text" => {
                    for ch in wasm_event_text(&event).chars() {
                        self.emu.tvc.key_press(ch);
                    }
                }
                "blur" => {
                    self.pressed_keys.clear();
                    self.emu.tvc.focus_change(false);
                    self.prev_shift = false;
                    self.prev_ctrl = false;
                    self.prev_alt = false;
                }
                _ => {}
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_event_code(event: &wasm_bindgen::JsValue) -> u32 {
    js_sys::Reflect::get(event, &wasm_bindgen::JsValue::from_str("code"))
        .ok()
        .and_then(|value| value.as_f64())
        .unwrap_or(0.0) as u32
}

#[cfg(target_arch = "wasm32")]
fn wasm_event_text(event: &wasm_bindgen::JsValue) -> String {
    js_sys::Reflect::get(event, &wasm_bindgen::JsValue::from_str("text"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn js_value_string(value: wasm_bindgen::JsValue) -> String {
    value
        .as_string()
        .unwrap_or_else(|| "browser storage operation failed".to_string())
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
        let mut user_interacted = false;
        ctx.input(|i| {
            if !i.events.is_empty() {
                user_interacted = true;
            }
        });
        if user_interacted {
            if self.audio.is_none() {
                match NativeAudioSink::new(self.emu.tvc.sound_sample_rate()) {
                    Ok(sink) => {
                        self.audio = Some(sink);
                        self.audio_status = None;
                    }
                    Err(err) => {
                        self.audio_status = Some(format!("Audio unavailable: {err}"));
                    }
                }
            }
            if let Some(ref audio) = self.audio {
                let _ = audio.resume();
            }
        }

        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        if let Some(ref dbg) = self.debugger {
            dbg.set_context(ctx.clone());
            while let Ok(msg) = dbg.cmd_rx.try_recv() {
                let response = crate::debugger::handle_command(&mut self.emu, &msg.cmd_line);
                let _ = msg.reply_tx.send(response);
            }
        }

        #[cfg(target_arch = "wasm32")]
        while let Ok(pending) = self.file_rx.try_recv() {
            match pending {
                PendingFile::Tape { name, bytes } => {
                    match self.emu.play_tape_bytes(&name, &bytes) {
                        Ok(()) => {
                            if let Some(recent) = self.emu.recent_tapes_wasm.first().cloned() {
                                self.persist_wasm_recent("tape", recent, ctx.clone());
                            }
                            self.file_status = Some(format!("Loaded tape: {name}"));
                        }
                        Err(err) => {
                            self.file_status = Some(format!("Tape load failed: {err}"));
                        }
                    }
                }
                PendingFile::Disk { name, bytes } => {
                    match self.emu.insert_disk_bytes(&name, &bytes) {
                        Ok(()) => {
                            if let Some(recent) = self.emu.recent_disks_wasm.first().cloned() {
                                self.persist_wasm_recent("disk", recent, ctx.clone());
                            }
                            self.file_status = Some(format!("Loaded disk: {name}"));
                        }
                        Err(err) => {
                            self.file_status = Some(format!("Disk load failed: {err}"));
                        }
                    }
                }
                PendingFile::Snapshot { name, bytes } => {
                    let mut data = bytes;
                    if crate::emu::is_zip_data(&data) {
                        match crate::emu::unzip_snapshot(&data) {
                            Ok(unzipped) => data = unzipped,
                            Err(err) => {
                                self.file_status = Some(format!("Snapshot unzip failed: {err}"));
                                continue;
                            }
                        }
                    }
                    match self.emu.load_snapshot(&data) {
                        Ok(()) => {
                            self.sync_selection_from_emu();
                            self.file_status = Some(format!("Loaded snapshot: {name}"));
                        }
                        Err(err) => {
                            self.file_status = Some(format!("Snapshot load failed: {err}"));
                        }
                    }
                }
                PendingFile::StorageResult { error } => {
                    if let Some(err) = error {
                        self.file_status = Some(format!("Browser storage failed: {err}"));
                    }
                }
                PendingFile::RecentCleared { kind } => match kind.as_str() {
                    "tape" => {
                        self.emu.recent_tapes_wasm.clear();
                        self.file_status = Some("Recent tapes cleared".to_string());
                    }
                    "disk" => {
                        self.emu.recent_disks_wasm.clear();
                        self.file_status = Some("Recent disks cleared".to_string());
                    }
                    _ => {}
                },
            }
        }

        #[cfg(target_arch = "wasm32")]
        self.handle_wasm_keyboard_events();

        #[cfg(not(target_arch = "wasm32"))]
        let has_focus = ctx.input(|i| i.focused);
        #[cfg(not(target_arch = "wasm32"))]
        if !has_focus && !self.pressed_keys.is_empty() {
            self.pressed_keys.clear();
            self.emu.tvc.focus_change(false);
            self.prev_shift = false;
            self.prev_ctrl = false;
            self.prev_alt = false;
        }

        #[cfg(not(target_arch = "wasm32"))]
        ctx.input(|i| {
            let modifiers = i.modifiers;
            self.handle_modifier(modifiers.shift, modifiers.ctrl, modifiers.alt);

            for event in &i.events {
                match event {
                    egui::Event::Key {
                        key,
                        physical_key,
                        pressed: true,
                        ..
                    } => {
                        let key = physical_key.unwrap_or(*key);
                        if let Some(code) = egui_key_to_js_code(key) {
                            if self.pressed_keys.insert(code) {
                                self.emu.tvc.key_down(code);
                            }
                        }
                    }
                    egui::Event::Key {
                        key,
                        physical_key,
                        pressed: false,
                        ..
                    } => {
                        let key = physical_key.unwrap_or(*key);
                        if let Some(code) = egui_key_to_js_code(key) {
                            self.pressed_keys.remove(&code);
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
            let hit_breakpoint = self.emu.tick();
            if hit_breakpoint {
                self.emu.running = false;
                #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
                if let Some(ref dbg) = self.debugger {
                    let pc = self.emu.tvc.z80.state.r16[11];
                    let _ = dbg.event_tx.send(crate::debugger::DebuggerEvent::BreakpointHit { pc });
                }
            }
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

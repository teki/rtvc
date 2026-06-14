use std::path::Path;

use crate::debug_ui::DebuggerUi;
use crate::emu::Emu;
use eframe::egui::{self, TextureHandle};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use serde::{Deserialize, Serialize};

const WORKSPACE_VERSION: u32 = 1;
#[cfg(not(target_arch = "wasm32"))]
const WORKSPACE_FILE_NAME: &str = "rtvc-workspace.json";
#[cfg(target_arch = "wasm32")]
const WORKSPACE_STORAGE_KEY: &str = "rtvc_workspace_v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMode {
    #[default]
    Simple,
    Developer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTab {
    Screen,
    IoLog,
    Cpu,
    Disassembly,
    Memory,
    Breakpoints,
    RomSymbols,
    Events,
}

#[derive(Serialize, Deserialize)]
struct WorkspaceDocument {
    version: u32,
    mode: WorkspaceMode,
    dock_state: DockState<WorkspaceTab>,
}

pub struct Workspace {
    mode: WorkspaceMode,
    dock_state: DockState<WorkspaceTab>,
    dirty: bool,
    persisted_json: Option<String>,
    persisted_location: Option<String>,
}

impl Workspace {
    pub fn load(config_path: &Path) -> Self {
        match load_text(config_path) {
            Ok(Some(text)) => {
                let mut workspace = Self::from_persisted_text(text);
                workspace.persisted_location = Some(persistence_location(config_path));
                workspace
            }
            Ok(None) => Self::simple_default(),
            Err(_) => Self::simple_default(),
        }
    }

    pub fn mode(&self) -> WorkspaceMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: WorkspaceMode) {
        if self.mode != mode {
            self.mode = mode;
            self.dirty = true;
        }
    }

    pub fn is_developer(&self) -> bool {
        self.mode == WorkspaceMode::Developer
    }

    pub fn accepts_tvc_input(&self, screen_captured: bool) -> bool {
        self.mode == WorkspaceMode::Simple || screen_captured
    }

    pub fn has_tab(&self, tab: WorkspaceTab) -> bool {
        self.dock_state.find_tab(&tab).is_some()
    }

    pub fn close_tab(&mut self, tab: WorkspaceTab) {
        if tab == WorkspaceTab::Screen {
            return;
        }
        if let Some(location) = self.dock_state.find_tab(&tab) {
            self.dock_state.remove_tab(location);
            self.dirty = true;
        }
    }

    pub fn open_tab(&mut self, tab: WorkspaceTab) {
        if let Some(location) = self.dock_state.find_tab(&tab) {
            self.dock_state.set_active_tab(location);
            self.dock_state
                .set_focused_node_and_surface((location.0, location.1));
        } else {
            self.dock_state.push_to_focused_leaf(tab);
            if let Some(location) = self.dock_state.find_tab(&tab) {
                self.dock_state.set_active_tab(location);
                self.dock_state
                    .set_focused_node_and_surface((location.0, location.1));
            }
        }
        self.dirty = true;
    }

    pub fn reset_layout(&mut self) {
        self.dock_state = default_dock_state();
        self.dirty = true;
    }

    pub fn debugger_layout(&mut self) {
        self.dock_state = debugger_dock_state();
        self.dirty = true;
    }

    pub fn mark_layout_changed(&mut self) {
        self.dirty = true;
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        screen_texture: Option<&TextureHandle>,
        emu: &mut Emu,
        debugger: &mut DebuggerUi,
        screen_captured: &mut bool,
    ) {
        debugger.drain_trace_events(emu);
        let mut viewer = WorkspaceViewer {
            screen_texture,
            emu,
            debugger,
            screen_captured,
            screen_visible: false,
            screen_clicked: false,
            events_visible: false,
        };
        DockArea::new(&mut self.dock_state)
            .id(egui::Id::new("rtvc_developer_workspace"))
            .show_add_buttons(false)
            .show_inside(ui, &mut viewer);
        let clicked_elsewhere =
            ui.input(|input| input.pointer.primary_clicked()) && !viewer.screen_clicked;
        let screen_visible = viewer.screen_visible;
        let events_visible = viewer.events_visible;
        drop(viewer);
        if !screen_visible || clicked_elsewhere {
            *screen_captured = false;
        }
        debugger.update_tracing(emu, events_visible);
    }

    pub fn save(&mut self, config_path: &Path) -> Result<(), String> {
        if !self.dirty {
            return Ok(());
        }
        let text = self.to_json()?;
        let location = persistence_location(config_path);
        if self.persisted_json.as_deref() != Some(text.as_str())
            || self.persisted_location.as_deref() != Some(location.as_str())
        {
            save_text(config_path, &text)?;
            self.persisted_json = Some(text);
            self.persisted_location = Some(location);
        }
        self.dirty = false;
        Ok(())
    }

    fn simple_default() -> Self {
        Self {
            mode: WorkspaceMode::Simple,
            dock_state: default_dock_state(),
            dirty: false,
            persisted_json: None,
            persisted_location: None,
        }
    }

    fn developer_default() -> Self {
        Self {
            mode: WorkspaceMode::Developer,
            dock_state: default_dock_state(),
            dirty: true,
            persisted_json: None,
            persisted_location: None,
        }
    }

    fn from_persisted_text(text: String) -> Self {
        match Self::from_json(&text) {
            Ok(mut workspace) => {
                workspace.persisted_json = Some(text);
                workspace
            }
            Err(_) => Self::developer_default(),
        }
    }

    fn from_json(text: &str) -> Result<Self, String> {
        let document: WorkspaceDocument =
            serde_json::from_str(text).map_err(|err| err.to_string())?;
        if document.version != WORKSPACE_VERSION {
            return Err(format!(
                "unsupported workspace version {}",
                document.version
            ));
        }
        let mut workspace = Self {
            mode: document.mode,
            dock_state: document.dock_state,
            dirty: false,
            persisted_json: None,
            persisted_location: None,
        };
        if !workspace.has_tab(WorkspaceTab::Screen) {
            workspace
                .dock_state
                .push_to_first_leaf(WorkspaceTab::Screen);
            workspace.dirty = true;
        }
        Ok(workspace)
    }

    fn to_json(&self) -> Result<String, String> {
        let mut value = serde_json::to_value(WorkspaceDocument {
            version: WORKSPACE_VERSION,
            mode: self.mode,
            dock_state: self.dock_state.clone(),
        })
        .map_err(|err| err.to_string())?;
        replace_non_finite_coordinates(&mut value);
        serde_json::to_string_pretty(&value).map_err(|err| err.to_string())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persistence_location(config_path: &Path) -> String {
    workspace_path(config_path).to_string_lossy().into_owned()
}

#[cfg(target_arch = "wasm32")]
fn persistence_location(_config_path: &Path) -> String {
    WORKSPACE_STORAGE_KEY.to_string()
}

fn replace_non_finite_coordinates(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (name, value) in fields {
                if matches!(name.as_str(), "x" | "y") && value.is_null() {
                    *value = serde_json::Value::from(0.0);
                } else {
                    replace_non_finite_coordinates(value);
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                replace_non_finite_coordinates(value);
            }
        }
        _ => {}
    }
}

fn default_dock_state() -> DockState<WorkspaceTab> {
    let mut dock_state = DockState::new(vec![WorkspaceTab::Screen]);
    dock_state
        .main_surface_mut()
        .split_below(NodeIndex::root(), 0.72, vec![WorkspaceTab::IoLog]);
    dock_state
}

fn debugger_dock_state() -> DockState<WorkspaceTab> {
    let mut dock_state = DockState::new(vec![WorkspaceTab::Screen]);
    let [main, _bottom] = dock_state.main_surface_mut().split_below(
        NodeIndex::root(),
        0.75,
        vec![WorkspaceTab::IoLog, WorkspaceTab::Events],
    );
    let [_screen, right] =
        dock_state
            .main_surface_mut()
            .split_right(main, 0.68, vec![WorkspaceTab::Cpu]);
    dock_state.main_surface_mut().split_below(
        right,
        0.38,
        vec![
            WorkspaceTab::Disassembly,
            WorkspaceTab::Memory,
            WorkspaceTab::Breakpoints,
            WorkspaceTab::RomSymbols,
        ],
    );
    dock_state
}

pub fn draw_screen(
    ui: &mut egui::Ui,
    screen_texture: Option<&TextureHandle>,
    capture: Option<&mut bool>,
) -> bool {
    let available_rect = ui.available_rect_before_wrap();
    let avail = available_rect.size();
    let display_height_per_width = 3.0 / 4.0;
    let width = avail.x.min(avail.y / display_height_per_width);
    let height = width * display_height_per_width;
    let image_size = egui::vec2(width, height);
    let rect = egui::Rect::from_center_size(available_rect.center(), image_size);
    let sense = if capture.is_some() {
        egui::Sense::click()
    } else {
        egui::Sense::hover()
    };
    let response = ui.allocate_rect(rect, sense);

    if let Some(texture) = screen_texture {
        ui.painter().image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        ui.painter().rect_filled(rect, 0.0, egui::Color32::BLACK);
    }

    if let Some(captured) = capture {
        if response.clicked() {
            *captured = true;
        }
        let stroke = if *captured {
            egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 170, 255))
        } else {
            egui::Stroke::new(1.0, egui::Color32::from_gray(70))
        };
        ui.painter()
            .rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
        if *captured {
            ui.painter().text(
                rect.left_top() + egui::vec2(8.0, 8.0),
                egui::Align2::LEFT_TOP,
                "Keyboard captured - Esc to release",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(180, 220, 255),
            );
        }
    }
    response.clicked()
}

fn draw_io_log(ui: &mut egui::Ui, emu: &mut Emu) {
    ui.horizontal(|ui| {
        if ui.small_button("Clear").clicked() {
            emu.tvc.clear_log();
        }
    });
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for entry in emu.tvc.log_entries().iter().rev() {
                ui.label(entry);
            }
        });
}

struct WorkspaceViewer<'a> {
    screen_texture: Option<&'a TextureHandle>,
    emu: &'a mut Emu,
    debugger: &'a mut DebuggerUi,
    screen_captured: &'a mut bool,
    screen_visible: bool,
    screen_clicked: bool,
    events_visible: bool,
}

impl TabViewer for WorkspaceViewer<'_> {
    type Tab = WorkspaceTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            WorkspaceTab::Screen => "Screen".into(),
            WorkspaceTab::IoLog => "IO Log".into(),
            WorkspaceTab::Cpu => "CPU".into(),
            WorkspaceTab::Disassembly => "Disassembly".into(),
            WorkspaceTab::Memory => "Memory".into(),
            WorkspaceTab::Breakpoints => "Breakpoints".into(),
            WorkspaceTab::RomSymbols => "ROM Symbols".into(),
            WorkspaceTab::Events => "Events".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            WorkspaceTab::Screen => {
                self.screen_visible = true;
                self.screen_clicked =
                    draw_screen(ui, self.screen_texture, Some(self.screen_captured));
            }
            WorkspaceTab::IoLog => draw_io_log(ui, self.emu),
            WorkspaceTab::Cpu => self.debugger.draw_cpu(ui, self.emu),
            WorkspaceTab::Disassembly => self.debugger.draw_disassembly(ui, self.emu),
            WorkspaceTab::Memory => self.debugger.draw_memory(ui, self.emu),
            WorkspaceTab::Breakpoints => self.debugger.draw_breakpoints(ui, self.emu),
            WorkspaceTab::RomSymbols => self.debugger.draw_rom_symbols(ui, self.emu),
            WorkspaceTab::Events => {
                self.events_visible = true;
                self.debugger.draw_events(ui, self.emu);
            }
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        *tab != WorkspaceTab::Screen
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn workspace_path(config_path: &Path) -> std::path::PathBuf {
    config_path.with_file_name(WORKSPACE_FILE_NAME)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_text(config_path: &Path) -> Result<Option<String>, String> {
    let path = workspace_path(config_path);
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_text(config_path: &Path, text: &str) -> Result<(), String> {
    std::fs::write(workspace_path(config_path), text).map_err(|err| err.to_string())
}

#[cfg(target_arch = "wasm32")]
fn load_text(_config_path: &std::path::Path) -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or_else(|| "browser window unavailable".to_string())?;
    let storage = window
        .local_storage()
        .map_err(js_value_string)?
        .ok_or_else(|| "browser local storage unavailable".to_string())?;
    storage
        .get_item(WORKSPACE_STORAGE_KEY)
        .map_err(js_value_string)
}

#[cfg(target_arch = "wasm32")]
fn save_text(_config_path: &std::path::Path, text: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or_else(|| "browser window unavailable".to_string())?;
    let storage = window
        .local_storage()
        .map_err(js_value_string)?
        .ok_or_else(|| "browser local storage unavailable".to_string())?;
    storage
        .set_item(WORKSPACE_STORAGE_KEY, text)
        .map_err(js_value_string)
}

#[cfg(target_arch = "wasm32")]
fn js_value_string(value: wasm_bindgen::JsValue) -> String {
    value
        .as_string()
        .unwrap_or_else(|| "browser workspace storage failed".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_round_trips() {
        let mut workspace = Workspace::developer_default();
        workspace.open_tab(WorkspaceTab::IoLog);
        let text = workspace.to_json().unwrap();
        let restored = Workspace::from_json(&text).unwrap();

        assert!(!text.contains("\"x\": null"));
        assert_eq!(restored.mode(), WorkspaceMode::Developer);
        assert!(restored.has_tab(WorkspaceTab::Screen));
        assert!(restored.has_tab(WorkspaceTab::IoLog));
    }

    #[test]
    fn rejects_unknown_workspace_version() {
        let workspace = Workspace::developer_default();
        let text = workspace
            .to_json()
            .unwrap()
            .replace("\"version\": 1", "\"version\": 99");

        assert!(Workspace::from_json(&text).is_err());
    }

    #[test]
    fn invalid_workspace_falls_back_to_default_developer_layout() {
        let workspace = Workspace::from_persisted_text("{not json".to_string());

        assert_eq!(workspace.mode(), WorkspaceMode::Developer);
        assert!(workspace.has_tab(WorkspaceTab::Screen));
        assert!(workspace.has_tab(WorkspaceTab::IoLog));
    }

    #[test]
    fn missing_screen_is_restored() {
        let mut workspace = Workspace::developer_default();
        let location = workspace
            .dock_state
            .find_tab(&WorkspaceTab::Screen)
            .unwrap();
        workspace.dock_state.remove_tab(location);
        let text = workspace.to_json().unwrap();
        let restored = Workspace::from_json(&text).unwrap();

        assert!(restored.has_tab(WorkspaceTab::Screen));
    }

    #[test]
    fn closed_log_can_be_reopened() {
        let mut workspace = Workspace::developer_default();
        workspace.close_tab(WorkspaceTab::IoLog);
        assert!(!workspace.has_tab(WorkspaceTab::IoLog));

        workspace.open_tab(WorkspaceTab::IoLog);

        assert!(workspace.has_tab(WorkspaceTab::IoLog));
    }

    #[test]
    fn debugger_layout_contains_all_phase_two_panes() {
        let mut workspace = Workspace::developer_default();
        workspace.debugger_layout();

        for tab in [
            WorkspaceTab::Screen,
            WorkspaceTab::IoLog,
            WorkspaceTab::Cpu,
            WorkspaceTab::Disassembly,
            WorkspaceTab::Memory,
            WorkspaceTab::Breakpoints,
            WorkspaceTab::RomSymbols,
            WorkspaceTab::Events,
        ] {
            assert!(workspace.has_tab(tab));
        }
        let restored = Workspace::from_json(&workspace.to_json().unwrap()).unwrap();
        assert!(restored.has_tab(WorkspaceTab::Disassembly));
        assert!(restored.has_tab(WorkspaceTab::Events));
    }

    #[test]
    fn tvc_input_requires_capture_only_in_developer_mode() {
        let simple = Workspace::simple_default();
        let developer = Workspace::developer_default();

        assert!(simple.accepts_tvc_input(false));
        assert!(!developer.accepts_tvc_input(false));
        assert!(developer.accepts_tvc_input(true));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn native_workspace_survives_restart() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("rtvc-workspace-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let config_path = directory.join("rtvc.toml");

        let mut workspace = Workspace::developer_default();
        workspace.close_tab(WorkspaceTab::IoLog);
        workspace.save(&config_path).unwrap();

        let restored = Workspace::load(&config_path);
        assert_eq!(restored.mode(), WorkspaceMode::Developer);
        assert!(restored.has_tab(WorkspaceTab::Screen));
        assert!(!restored.has_tab(WorkspaceTab::IoLog));

        std::fs::remove_file(workspace_path(&config_path)).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}

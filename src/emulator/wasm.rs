use std::sync::Arc;
#[cfg(feature = "wasm-full")]
use wasm_bindgen::JsCast;

use wasm_bindgen::prelude::*;

use crate::host_file::{HostFileStore, MemoryStore};
use crate::key::{KeyTyper, TypedKeyStep};
use crate::machine::VidModel;
use crate::machine_type::ExpansionCard;
use crate::tvc::Tvc;

pub const SCREEN_WIDTH: usize = 608;
pub const SCREEN_HEIGHT: usize = 288;

#[wasm_bindgen]
pub struct WasmTvc {
    tvc: Tvc,
    host_files: Arc<MemoryStore>,
    typer: KeyTyper,
    last_frame_seconds: f64,
}

impl WasmTvc {
    fn configure_host_file_cards(&mut self) {
        let store: Arc<dyn HostFileStore> = self.host_files.clone();
        for card in self.tvc.host_file_cards_mut() {
            card.set_store(Some(store.clone()));
        }
    }
}

#[wasm_bindgen]
impl WasmTvc {
    #[wasm_bindgen(constructor)]
    pub fn new(is_plus: bool) -> Self {
        WasmTvc {
            tvc: Tvc::new_with_vid_model(is_plus, default_web_vid_model()),
            host_files: Arc::new(MemoryStore::new()),
            typer: KeyTyper::default(),
            last_frame_seconds: 1.0 / 50.0,
        }
    }

    pub fn reset(&mut self) {
        self.typer.clear();
        self.tvc.reset();
    }

    #[wasm_bindgen(js_name = saveSnapshot)]
    pub fn save_snapshot(&self) -> Vec<u8> {
        self.tvc.save_snapshot()
    }

    #[wasm_bindgen(js_name = loadSnapshot)]
    pub fn load_snapshot(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.typer.clear();
        if let Some(machine) = snapshot_machine_type(data)? {
            let fast_boot = self.tvc.fast_boot();
            self.tvc = Tvc::new_with_vid_model(machine.is_plus, self.tvc.vid_model());
            load_builtin_roms(&mut self.tvc, machine);
            self.configure_host_file_cards();
            self.tvc.set_fast_boot(fast_boot);
        }
        self.tvc
            .load_snapshot(data)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen(js_name = setVidModel)]
    pub fn set_vid_model(&mut self, model: &str) -> Result<(), JsValue> {
        let vid_model = match model {
            "fast-frame" | "simple" => VidModel::FastFrame,
            "interleaved" | "realistic" => VidModel::Interleaved,
            _ => {
                return Err(JsValue::from_str("expected `fast-frame` or `interleaved`"));
            }
        };
        self.tvc.set_vid_model(vid_model);
        Ok(())
    }

    #[wasm_bindgen(js_name = vidModel)]
    pub fn vid_model(&self) -> String {
        match self.tvc.vid_model() {
            VidModel::FastFrame => "fast-frame",
            VidModel::Interleaved => "interleaved",
        }
        .to_string()
    }

    #[wasm_bindgen(js_name = runFrame)]
    pub fn run_frame(&mut self) -> bool {
        match self.typer.step() {
            TypedKeyStep::Idle => {}
            TypedKeyStep::Release(code) => self.tvc.key_up(code),
            TypedKeyStep::Press(ch) => {
                let code = self.tvc.press_char(ch);
                self.typer.pressed(code);
            }
        }
        let start = self.tvc.clock;
        let hit_breakpoint = self.tvc.run_for_a_frame();
        let elapsed = self.tvc.clock.wrapping_sub(start);
        if elapsed > 0 {
            self.last_frame_seconds = elapsed as f64 / crate::tvc::CPU_CLOCK_HZ as f64;
        }
        hit_breakpoint
    }

    /// Emulated seconds the last `runFrame` covered (one TV picture, about
    /// 20.1 ms); the player paces by it instead of a fixed 1/50 s.
    #[wasm_bindgen(js_name = lastFrameSeconds)]
    pub fn last_frame_seconds(&self) -> f64 {
        self.last_frame_seconds
    }

    #[wasm_bindgen(js_name = audioSampleRate)]
    pub fn audio_sample_rate(&self) -> u32 {
        self.tvc.sound_sample_rate()
    }

    #[wasm_bindgen(js_name = takeAudioSamples)]
    pub fn take_audio_samples(&mut self) -> Vec<f32> {
        self.tvc.take_audio_samples()
    }

    /// Loads a system or cartridge ROM. A `*DOS*` ROM name keeps its earlier
    /// meaning and plugs an HBF card into slot 0.
    #[wasm_bindgen(js_name = addRom)]
    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        if name.contains("DOS") && !name.contains("CART") {
            self.tvc.attach_card(0, ExpansionCard::VtDos12, data);
        } else {
            self.tvc.add_rom(name, data);
        }
    }

    /// Plugs a card into `slot` (0-3): `vtdos12` with its ROM in `data`, or
    /// `hostfile`, backed by the files managed with `putHostFile`.
    #[wasm_bindgen(js_name = attachCard)]
    pub fn attach_card(&mut self, slot: usize, card: &str, data: &[u8]) -> Result<(), JsValue> {
        let kind = ExpansionCard::from_id_name(card)
            .ok_or_else(|| JsValue::from_str("expected `vtdos12` or `hostfile`"))?;
        if slot > 3 {
            return Err(JsValue::from_str("slot must be 0-3"));
        }
        self.tvc.attach_card(slot, kind, data);
        self.configure_host_file_cards();
        Ok(())
    }

    #[wasm_bindgen(js_name = putHostFile)]
    pub fn put_host_file(&mut self, name: &str, data: &[u8]) -> Result<(), JsValue> {
        self.host_files
            .insert(name, data.to_vec())
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen(js_name = getHostFile)]
    pub fn get_host_file(&self, name: &str) -> Option<Vec<u8>> {
        self.host_files.get(name)
    }

    #[wasm_bindgen(js_name = listHostFiles)]
    pub fn list_host_files(&self) -> Vec<String> {
        self.host_files.names()
    }

    #[wasm_bindgen(js_name = loadCartRom)]
    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.tvc.load_cart_rom(data);
    }

    /// Inserts a disk into drive 0/1 of the first HBF card.
    #[wasm_bindgen(js_name = loadDisk)]
    pub fn load_disk(&mut self, drive: usize, name: &str, data: &[u8]) {
        if let Some(&slot) = self.tvc.hbf_slots().first() {
            self.tvc.load_disk(slot, drive, name, data);
        }
    }

    #[wasm_bindgen(js_name = loadSlotDisk)]
    pub fn load_slot_disk(&mut self, slot: usize, drive: usize, name: &str, data: &[u8]) {
        self.tvc.load_disk(slot, drive, name, data);
    }

    #[wasm_bindgen(js_name = keyDown)]
    pub fn key_down(&mut self, code: u32) -> bool {
        self.tvc.key_down(code)
    }

    #[wasm_bindgen(js_name = keyUp)]
    pub fn key_up(&mut self, code: u32) {
        self.tvc.key_up(code);
    }

    #[wasm_bindgen(js_name = keyPressText)]
    pub fn key_press_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.tvc.key_press(ch);
        }
    }

    /// Queues text to be typed one key at a time over the next frames, for
    /// keyboards that deliver characters without a usable key code (phone
    /// soft keyboards). `\r` is Return and `\x08` is Backspace.
    #[wasm_bindgen(js_name = typeText)]
    pub fn type_text(&mut self, text: &str) {
        self.typer.push_str(text);
    }

    #[wasm_bindgen(js_name = focusChange)]
    pub fn focus_change(&mut self, has_focus: bool) {
        if !has_focus {
            if let Some(code) = self.typer.clear() {
                self.tvc.key_up(code);
            }
        }
        self.tvc.focus_change(has_focus);
    }

    #[wasm_bindgen(js_name = framebufferPtr)]
    pub fn framebuffer_ptr(&self) -> *const u32 {
        self.tvc.framebuffer.as_ptr()
    }

    #[wasm_bindgen(js_name = framebufferLen)]
    pub fn framebuffer_len(&self) -> usize {
        self.tvc.framebuffer.len() * std::mem::size_of::<u32>()
    }

    #[wasm_bindgen(js_name = screenWidth)]
    pub fn screen_width(&self) -> usize {
        SCREEN_WIDTH
    }

    #[wasm_bindgen(js_name = screenHeight)]
    pub fn screen_height(&self) -> usize {
        SCREEN_HEIGHT
    }

    #[wasm_bindgen(js_name = takeFrameComplete)]
    pub fn take_frame_complete(&mut self) -> bool {
        let complete = self.tvc.frame_complete;
        self.tvc.frame_complete = false;
        complete
    }
}

/// True when the main pointer is a finger (phones and tablets).
#[cfg(feature = "wasm-full")]
fn is_touch_device(window: &web_sys::Window) -> bool {
    window
        .match_media("(pointer: coarse)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
}

fn default_web_vid_model() -> VidModel {
    VidModel::FastFrame
}

fn snapshot_machine_type(data: &[u8]) -> Result<Option<crate::machine_type::MachineType>, JsValue> {
    let chunks =
        crate::snapshot::read_file(data).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let Some(chunk) = chunks.iter().find(|chunk| chunk.id == *b"EMUT") else {
        return Ok(None);
    };
    crate::machine_type::MachineType::from_snapshot_chunk(chunk.data)
        .map(Some)
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

fn load_builtin_roms(tvc: &mut Tvc, machine: crate::machine_type::MachineType) {
    machine.attach_builtin_cards(tvc);
    for (name, use_) in machine.rom_uses() {
        let Some(bytes) = builtin_rom(&name) else {
            continue;
        };
        crate::machine_type::place_machine_rom(tvc, &name, bytes, use_);
    }
}

fn builtin_rom(name: &str) -> Option<&'static [u8]> {
    Some(match name {
        "TVC12_D3.64K" => include_bytes!("../../roms/TVC12_D3.64K"),
        "TVC12_D4.64K" => include_bytes!("../../roms/TVC12_D4.64K"),
        "TVC12_D7.64K" => include_bytes!("../../roms/TVC12_D7.64K"),
        "TVC22_D4.64K" => include_bytes!("../../roms/TVC22_D4.64K"),
        "TVC22_D6.64K" => include_bytes!("../../roms/TVC22_D6.64K"),
        "TVC22_D7.64K" => include_bytes!("../../roms/TVC22_D7.64K"),
        "VT-DOS12-DISK.ROM" => include_bytes!("../../roms/VT-DOS12-DISK.ROM"),
        "VT-DOS12-CART.crt" => include_bytes!("../../roms/VT-DOS12-CART.crt"),
        "pack_man.crt" => include_bytes!("../../roms/pack_man.crt"),
        _ => return None,
    })
}

#[cfg(feature = "wasm-full")]
#[wasm_bindgen]
pub struct WebHandle {
    runner: eframe::WebRunner,
}

#[cfg(feature = "wasm-full")]
#[wasm_bindgen]
impl WebHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    #[wasm_bindgen(js_name = start)]
    pub fn start(&self, canvas_id: &str) -> Result<(), JsValue> {
        use crate::emu::Emu;

        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("no window"))?
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str("canvas not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("element is not a canvas"))?;

        let app_state_file = crate::app_state::AppStateFile::load();
        let machine_type = app_state_file
            .state
            .machine_type
            .unwrap_or_else(|| crate::emu::MachineType::all_types()[0]);
        let mut emu = Emu::new(machine_type);
        // Without a saved choice, touch devices start in Fast frame: phones
        // have less headroom, and some (Safari with Lockdown Mode) run
        // WebAssembly without a compiler, where Interleaved misses 50 FPS.
        let vid_model = app_state_file
            .state
            .vid_model
            .or_else(|| is_touch_device(&web_sys::window()?).then_some(VidModel::FastFrame));
        if let Some(vid_model) = vid_model {
            emu.set_vid_model(vid_model);
        }
        emu.set_fast_boot(app_state_file.state.fast_boot);
        emu.set_warp_loading(app_state_file.state.warp_loading);
        if let Some(enabled) = app_state_file.state.host_file_default {
            emu.set_host_file_default(enabled);
        }
        emu.load_roms();
        let audio_error = startup_audio_error();
        let storage_error = startup_storage_error();
        let (recent_tapes, recent_disks, host_files) = load_recent_media()?;
        emu.recent_tapes_wasm = recent_tapes;
        emu.recent_disks_wasm = recent_disks;
        emu.set_bundled_media(load_bundled_media()?);
        for file in host_files {
            let _ = emu.host_file_memory().insert(&file.name, file.bytes);
        }

        let mut app = crate::ui::EmuApp::new(emu, app_state_file, None);
        if let Some(error) = audio_error {
            app.set_audio_status(format!("Audio unavailable: {error}"));
        }
        if let Some(error) = storage_error {
            app.set_file_status(format!("Browser storage unavailable: {error}"));
        }
        match load_startup_launch() {
            Ok(Some(file)) => app.queue_media_launch(file.name, file.bytes),
            Ok(None) => {}
            Err(error) => app.set_file_status(error),
        }

        let runner = self.runner.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(err) = runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|_cc| Ok(Box::new(app))),
                )
                .await
            {
                web_sys::console::error_1(&err);
            }
        });
        Ok(())
    }
}

#[cfg(feature = "wasm-full")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = globalThis, js_name = rtvcGetStartupAudioError)]
    fn web_startup_audio_error() -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = rtvcGetStartupStorageError)]
    fn web_startup_storage_error() -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = rtvcGetStartupRecentMedia)]
    fn web_startup_recent_media() -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = rtvcGetStartupBundledMedia)]
    fn web_startup_bundled_media() -> JsValue;

    #[wasm_bindgen(js_namespace = globalThis, js_name = rtvcGetStartupLaunch)]
    fn web_startup_launch() -> JsValue;
}

/// Media named by the page URL's `?run=` parameter, fetched by app.js.
/// app.js reports a failed fetch as `{ error }`.
#[cfg(feature = "wasm-full")]
fn load_startup_launch() -> Result<Option<crate::emu::WasmRecentFile>, String> {
    let record = web_startup_launch();
    if record.is_undefined() || record.is_null() {
        return Ok(None);
    }
    let get = |key: &str| js_sys::Reflect::get(&record, &JsValue::from_str(key)).ok();
    if let Some(error) = get("error").and_then(|value| value.as_string()) {
        return Err(error);
    }
    let name = get("name")
        .and_then(|value| value.as_string())
        .unwrap_or_default();
    let bytes = get("bytes")
        .filter(|value| value.is_object())
        .map(|value| js_sys::Uint8Array::new(&value).to_vec())
        .ok_or("run URL returned no data")?;
    Ok(Some(crate::emu::WasmRecentFile { name, bytes }))
}

/// Media files from the bundle's `bundled/` directory, fetched by app.js.
#[cfg(feature = "wasm-full")]
fn load_bundled_media() -> Result<Vec<crate::emu::WasmRecentFile>, JsValue> {
    let records = web_startup_bundled_media();
    if records.is_undefined() || records.is_null() {
        return Ok(Vec::new());
    }
    let records = records
        .dyn_into::<js_sys::Array>()
        .map_err(|_| JsValue::from_str("bundled media result is not an array"))?;
    let mut files = Vec::new();
    for record in records.iter() {
        let name = js_sys::Reflect::get(&record, &JsValue::from_str("name"))?
            .as_string()
            .unwrap_or_default();
        let bytes =
            js_sys::Uint8Array::new(&js_sys::Reflect::get(&record, &JsValue::from_str("bytes"))?)
                .to_vec();
        files.push(crate::emu::WasmRecentFile { name, bytes });
    }
    Ok(files)
}

#[cfg(feature = "wasm-full")]
fn startup_audio_error() -> Option<String> {
    web_startup_audio_error().as_string()
}

#[cfg(feature = "wasm-full")]
fn startup_storage_error() -> Option<String> {
    web_startup_storage_error().as_string()
}

#[cfg(feature = "wasm-full")]
fn load_recent_media() -> Result<
    (
        Vec<crate::emu::WasmRecentFile>,
        Vec<crate::emu::WasmRecentFile>,
        Vec<crate::emu::WasmRecentFile>,
    ),
    JsValue,
> {
    let records = web_startup_recent_media()
        .dyn_into::<js_sys::Array>()
        .map_err(|_| JsValue::from_str("recent media result is not an array"))?;
    let mut tapes = Vec::new();
    let mut disks = Vec::new();
    let mut host_files = Vec::new();

    for record in records.iter() {
        let kind = js_sys::Reflect::get(&record, &JsValue::from_str("kind"))?
            .as_string()
            .unwrap_or_default();
        let name = js_sys::Reflect::get(&record, &JsValue::from_str("name"))?
            .as_string()
            .unwrap_or_default();
        let bytes =
            js_sys::Uint8Array::new(&js_sys::Reflect::get(&record, &JsValue::from_str("bytes"))?)
                .to_vec();
        let recent = crate::emu::WasmRecentFile { name, bytes };
        match kind.as_str() {
            "tape" => tapes.push(recent),
            "disk" => disks.push(recent),
            "hostfile" => host_files.push(recent),
            _ => {}
        }
    }
    tapes.truncate(5);
    disks.truncate(5);
    Ok((tapes, disks, host_files))
}

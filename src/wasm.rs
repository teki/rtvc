use wasm_bindgen::prelude::*;

use crate::tvc::Tvc;
use crate::vid::VidModel;

pub const SCREEN_WIDTH: usize = 608;
pub const SCREEN_HEIGHT: usize = 288;

#[wasm_bindgen]
pub struct WasmTvc {
    tvc: Tvc,
}

#[wasm_bindgen]
impl WasmTvc {
    #[wasm_bindgen(constructor)]
    pub fn new(is_plus: bool) -> Self {
        WasmTvc {
            tvc: Tvc::new_with_vid_model(is_plus, default_web_vid_model()),
        }
    }

    pub fn reset(&mut self) {
        self.tvc.reset();
    }

    #[wasm_bindgen(js_name = saveSnapshot)]
    pub fn save_snapshot(&self) -> Vec<u8> {
        self.tvc.save_snapshot()
    }

    #[wasm_bindgen(js_name = loadSnapshot)]
    pub fn load_snapshot(&mut self, data: &[u8]) -> Result<(), JsValue> {
        self.tvc
            .load_snapshot(data)
            .map_err(|err| JsValue::from_str(&err.to_string()))
    }

    #[wasm_bindgen(js_name = setVidModel)]
    pub fn set_vid_model(&mut self, model: &str) -> Result<(), JsValue> {
        let vid_model = match model {
            "simple" => VidModel::Simple,
            "realistic" => VidModel::Realistic,
            _ => return Err(JsValue::from_str("expected `simple` or `realistic`")),
        };
        self.tvc.set_vid_model(vid_model);
        Ok(())
    }

    #[wasm_bindgen(js_name = vidModel)]
    pub fn vid_model(&self) -> String {
        match self.tvc.vid_model() {
            VidModel::Simple => "simple",
            VidModel::Realistic => "realistic",
        }
        .to_string()
    }

    #[wasm_bindgen(js_name = runFrame)]
    pub fn run_frame(&mut self) -> bool {
        self.tvc.run_for_a_frame()
    }

    #[wasm_bindgen(js_name = addRom)]
    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        self.tvc.add_rom(name, data);
    }

    #[wasm_bindgen(js_name = loadCartRom)]
    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.tvc.load_cart_rom(data);
    }

    #[wasm_bindgen(js_name = loadDisk)]
    pub fn load_disk(&mut self, name: &str, data: &[u8]) {
        self.tvc.load_disk(name, data);
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

    #[wasm_bindgen(js_name = focusChange)]
    pub fn focus_change(&mut self, has_focus: bool) {
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

fn default_web_vid_model() -> VidModel {
    #[cfg(feature = "web-vid-realistic")]
    {
        VidModel::Realistic
    }

    #[cfg(not(feature = "web-vid-realistic"))]
    {
        VidModel::Simple
    }
}

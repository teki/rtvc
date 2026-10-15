//! CPC 464 video: the HD6845S CRTC, the Gate Array's pixel decoding and
//! palette, and a simple monitor that locks to the CRTC's HSYNC and VSYNC.
//!
//! The Gate Array's register protocol, ROM mapping, and interrupt counter stay
//! in [`crate::cpc`]; this module reports the sync edges they depend on.

use crate::crtc::{Crtc, CrtcTick, CrtcVariant};

/// Visible window in mode 2 pixels: 48 characters of 16 pixels.
pub const FRAMEBUFFER_WIDTH: usize = VISIBLE_CHARS * 16;
pub const FRAMEBUFFER_HEIGHT: usize = 272;

const VISIBLE_CHARS: usize = 48;
/// Characters after the start of HSYNC where the visible window begins. With
/// the firmware's default CRTC setup this centres the 40-character display.
const VISIBLE_X0: usize = 13;
/// Lines after the start of VSYNC where the visible window begins.
const VISIBLE_Y0: usize = 36;
/// Free-running monitor limits used when the CRTC produces no sync.
const MAX_BEAM_X: usize = 80;
const MAX_BEAM_Y: usize = 330;
/// Pen 16 is the border.
pub const BORDER_PEN: usize = 16;

/// Hardware colour numbers 0..31 as 0xAABBGGRR.
const HARDWARE_PALETTE: [u32; 32] = [
    rgb(0x80, 0x80, 0x80),
    rgb(0x80, 0x80, 0x80),
    rgb(0x00, 0xFF, 0x80),
    rgb(0xFF, 0xFF, 0x80),
    rgb(0x00, 0x00, 0x80),
    rgb(0xFF, 0x00, 0x80),
    rgb(0x00, 0x80, 0x80),
    rgb(0xFF, 0x80, 0x80),
    rgb(0xFF, 0x00, 0x80),
    rgb(0xFF, 0xFF, 0x80),
    rgb(0xFF, 0xFF, 0x00),
    rgb(0xFF, 0xFF, 0xFF),
    rgb(0xFF, 0x00, 0x00),
    rgb(0xFF, 0x00, 0xFF),
    rgb(0xFF, 0x80, 0x00),
    rgb(0xFF, 0x80, 0xFF),
    rgb(0x00, 0x00, 0x80),
    rgb(0x00, 0xFF, 0x80),
    rgb(0x00, 0xFF, 0x00),
    rgb(0x00, 0xFF, 0xFF),
    rgb(0x00, 0x00, 0x00),
    rgb(0x00, 0x00, 0xFF),
    rgb(0x00, 0x80, 0x00),
    rgb(0x00, 0x80, 0xFF),
    rgb(0x80, 0x00, 0x80),
    rgb(0x80, 0xFF, 0x80),
    rgb(0x80, 0xFF, 0x00),
    rgb(0x80, 0xFF, 0xFF),
    rgb(0x80, 0x00, 0x00),
    rgb(0x80, 0x00, 0xFF),
    rgb(0x80, 0x80, 0x00),
    rgb(0x80, 0x80, 0xFF),
];

const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    0xFF000000 | ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

/// Sync edges seen during one character clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpcVidEvents {
    pub hsync_end: bool,
    pub vsync_start: bool,
}

pub struct CpcVid {
    crtc: Crtc,
    hsync: bool,
    vsync: bool,
    pens: [u8; 17],
    mode: u8,
    pending_mode: u8,
    beam_x: usize,
    beam_y: usize,
    back_buffer: Vec<u32>,
    framebuffer: Vec<u32>,
    frame_ready: bool,
}

impl CpcVid {
    pub fn new() -> Self {
        let pixels = FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT;
        let mut vid = Self {
            crtc: Crtc::new(CrtcVariant::Hd6845s),
            hsync: false,
            vsync: false,
            pens: [0; 17],
            mode: 1,
            pending_mode: 1,
            beam_x: 0,
            beam_y: 0,
            back_buffer: vec![0xFF000000; pixels],
            framebuffer: vec![0xFF000000; pixels],
            frame_ready: false,
        };
        vid.reset();
        vid
    }

    pub fn reset(&mut self) {
        self.crtc.reset();
        self.hsync = false;
        self.vsync = false;
        self.pens = [20; 17];
        self.mode = 1;
        self.pending_mode = 1;
        self.beam_x = 0;
        self.beam_y = 0;
    }

    pub fn crtc(&self) -> &Crtc {
        &self.crtc
    }

    pub fn crtc_mut(&mut self) -> &mut Crtc {
        &mut self.crtc
    }

    /// Sets a pen (0..15, or [`BORDER_PEN`]) to a hardware colour number.
    pub fn set_pen(&mut self, pen: usize, colour: u8) {
        if let Some(entry) = self.pens.get_mut(pen) {
            *entry = colour & 0x1F;
        }
    }

    /// Requests a screen mode; the Gate Array applies it at the next HSYNC.
    pub fn set_mode(&mut self, mode: u8) {
        self.pending_mode = mode & 0x03;
    }

    pub fn mode(&self) -> u8 {
        self.mode
    }

    pub fn vsync(&self) -> bool {
        self.vsync
    }

    pub fn framebuffer(&self) -> &[u32] {
        &self.framebuffer
    }

    pub fn back_buffer(&self) -> &[u32] {
        &self.back_buffer
    }

    /// Returns whether a frame completed since the last call.
    pub fn take_frame_ready(&mut self) -> bool {
        std::mem::take(&mut self.frame_ready)
    }

    /// Advances the CRTC and monitor by one 1 MHz character clock.
    pub fn tick(&mut self, ram: &[u8]) -> CpcVidEvents {
        let tick = self.crtc.tick();
        let hsync_start = tick.raw_hsync && !self.hsync;
        let events = CpcVidEvents {
            hsync_end: !tick.raw_hsync && self.hsync,
            vsync_start: tick.raw_vsync && !self.vsync,
        };
        self.hsync = tick.raw_hsync;
        self.vsync = tick.raw_vsync;

        self.render_char(tick, ram);
        self.beam_x += 1;
        if hsync_start {
            self.mode = self.pending_mode;
            self.next_beam_line();
        } else if self.beam_x >= MAX_BEAM_X {
            self.next_beam_line();
        }
        if events.vsync_start {
            self.finish_beam_frame();
        }
        events
    }

    fn next_beam_line(&mut self) {
        self.beam_x = 0;
        self.beam_y += 1;
        if self.beam_y >= MAX_BEAM_Y {
            self.finish_beam_frame();
        }
    }

    fn finish_beam_frame(&mut self) {
        self.beam_y = 0;
        std::mem::swap(&mut self.framebuffer, &mut self.back_buffer);
        self.frame_ready = true;
    }

    fn render_char(&mut self, tick: CrtcTick, ram: &[u8]) {
        if self.beam_x < VISIBLE_X0
            || self.beam_x >= VISIBLE_X0 + VISIBLE_CHARS
            || self.beam_y < VISIBLE_Y0
            || self.beam_y >= VISIBLE_Y0 + FRAMEBUFFER_HEIGHT
        {
            return;
        }
        let offset =
            (self.beam_y - VISIBLE_Y0) * FRAMEBUFFER_WIDTH + (self.beam_x - VISIBLE_X0) * 16;
        let out = &mut self.back_buffer[offset..offset + 16];

        if !tick.display_enable {
            out.fill(HARDWARE_PALETTE[self.pens[BORDER_PEN] as usize]);
            return;
        }

        let address = video_address(tick);
        for (byte_index, &byte) in [ram[address], ram[address | 1]].iter().enumerate() {
            let out = &mut out[byte_index * 8..byte_index * 8 + 8];
            match self.mode {
                0 => {
                    for (pixel, chunk) in out.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                        let b = byte << pixel;
                        let pen = ((b >> 7) & 1) | ((b >> 2) & 2) | ((b >> 3) & 4) | ((b << 2) & 8);
                        chunk.fill(HARDWARE_PALETTE[self.pens[pen as usize] as usize]);
                    }
                }
                1 => {
                    for (pixel, chunk) in out.as_chunks_mut::<2>().0.iter_mut().enumerate() {
                        let pen = ((byte >> (7 - pixel)) & 1) | (((byte >> (3 - pixel)) & 1) << 1);
                        chunk.fill(HARDWARE_PALETTE[self.pens[pen as usize] as usize]);
                    }
                }
                _ => {
                    // Mode 3 is undocumented; render it like mode 2.
                    for (pixel, dot) in out.iter_mut().enumerate() {
                        let pen = (byte >> (7 - pixel)) & 1;
                        *dot = HARDWARE_PALETTE[self.pens[pen as usize] as usize];
                    }
                }
            }
        }
    }
}

impl Default for CpcVid {
    fn default() -> Self {
        Self::new()
    }
}

/// Screen RAM address of a character's first byte: MA13..12 select the 16K
/// bank, RA2..0 the 2K block, and MA9..0 the even byte within it.
fn video_address(tick: CrtcTick) -> usize {
    let ma = tick.ma as usize;
    ((ma & 0x3000) << 2) | ((tick.ra as usize & 0x07) << 11) | ((ma & 0x03FF) << 1)
}

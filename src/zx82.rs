#![allow(dead_code)]

use crate::bus::CpuBus;
use crate::vid::VidModel;
use crate::z80::Z80;

pub const CPU_CLOCK_HZ: u64 = 3_500_000;
pub const FRAME_CLOCKS: u64 = 69_888;
pub const FRAMEBUFFER_WIDTH: usize = 352;
pub const FRAMEBUFFER_HEIGHT: usize = 296;

const ROM_SIZE: usize = 0x4000;
const RAM_SIZE: usize = 0xC000;
const ACTIVE_WIDTH: usize = 256;
const ACTIVE_HEIGHT: usize = 192;
const LEFT_BORDER: usize = (FRAMEBUFFER_WIDTH - ACTIVE_WIDTH) / 2;
const TOP_BORDER: usize = (FRAMEBUFFER_HEIGHT - ACTIVE_HEIGHT) / 2;

pub struct Zx82Bus {
    rom: [u8; ROM_SIZE],
    ram: [u8; RAM_SIZE],
    keyboard: [u8; 8],
    ula_latch: u8,
    ear_input: bool,
}

impl Zx82Bus {
    pub fn new() -> Self {
        Self {
            rom: [0; ROM_SIZE],
            ram: [0; RAM_SIZE],
            keyboard: [0x1F; 8],
            ula_latch: 0,
            ear_input: false,
        }
    }

    pub fn reset(&mut self) {
        self.keyboard = [0x1F; 8];
        self.ula_latch = 0;
        self.ear_input = false;
    }

    pub fn clear_ram(&mut self) {
        self.ram.fill(0);
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() != ROM_SIZE {
            return Err(format!(
                "ZX Spectrum 48K ROM must be {ROM_SIZE} bytes, got {}",
                data.len()
            ));
        }
        self.rom.copy_from_slice(data);
        Ok(())
    }

    pub fn rom(&self) -> &[u8] {
        &self.rom
    }

    pub fn ram(&self) -> &[u8] {
        &self.ram
    }

    pub fn ram_mut(&mut self) -> &mut [u8] {
        &mut self.ram
    }

    pub fn border_color(&self) -> u8 {
        self.ula_latch & 0x07
    }

    pub fn speaker_level(&self) -> bool {
        self.ula_latch & 0x10 != 0
    }

    pub fn set_ear_input(&mut self, high: bool) {
        self.ear_input = high;
    }

    pub fn set_key(&mut self, row: usize, column: usize, pressed: bool) {
        if row >= self.keyboard.len() || column >= 5 {
            return;
        }
        let mask = 1u8 << column;
        if pressed {
            self.keyboard[row] &= !mask;
        } else {
            self.keyboard[row] |= mask;
        }
    }

    fn read_ula(&self, high_addr: u8) -> u8 {
        let mut columns = 0x1F;
        for row in 0..8 {
            if high_addr & (1 << row) == 0 {
                columns &= self.keyboard[row];
            }
        }
        0xA0 | ((self.ear_input as u8) << 6) | columns
    }
}

impl Default for Zx82Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuBus for Zx82Bus {
    fn r8(&mut self, addr: u16) -> u8 {
        if addr < 0x4000 {
            self.rom[addr as usize]
        } else {
            self.ram[addr as usize - 0x4000]
        }
    }

    fn w8(&mut self, addr: u16, val: u8) {
        if addr >= 0x4000 {
            self.ram[addr as usize - 0x4000] = val;
        }
    }

    fn out8(&mut self, port: u8, val: u8, _high_addr: u8) {
        if port & 1 == 0 {
            self.ula_latch = val;
        }
    }

    fn in8(&mut self, port: u8, high_addr: u8) -> u8 {
        if port & 1 == 0 {
            self.read_ula(high_addr)
        } else {
            0xFF
        }
    }
}

pub struct Zx82 {
    pub bus: Zx82Bus,
    pub z80: Z80,
    pub framebuffer: Vec<u32>,
    pub frame_complete: bool,
    vid_model: VidModel,
    clock: u64,
    next_frame_clock: u64,
    frame_counter: u64,
    last_frame_interrupt_accepted: bool,
}

impl Zx82 {
    pub fn new() -> Self {
        Self::new_with_vid_model(VidModel::FastFrame)
    }

    pub fn new_with_vid_model(vid_model: VidModel) -> Self {
        let mut machine = Self {
            bus: Zx82Bus::new(),
            z80: Z80::new(),
            framebuffer: vec![0xFF000000; FRAMEBUFFER_WIDTH * FRAMEBUFFER_HEIGHT],
            frame_complete: false,
            vid_model,
            clock: 0,
            next_frame_clock: FRAME_CLOCKS,
            frame_counter: 0,
            last_frame_interrupt_accepted: false,
        };
        machine.reset();
        machine
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
        self.next_frame_clock = FRAME_CLOCKS;
        self.frame_counter = 0;
        self.last_frame_interrupt_accepted = false;
        self.frame_complete = false;
        self.draw_full_frame();
    }

    pub fn hard_reset(&mut self) {
        self.bus.clear_ram();
        self.reset();
    }

    pub fn load_rom(&mut self, data: &[u8]) -> Result<(), String> {
        self.bus.load_rom(data)
    }

    pub fn vid_model(&self) -> VidModel {
        self.vid_model
    }

    pub fn set_vid_model(&mut self, vid_model: VidModel) {
        self.vid_model = vid_model;
    }

    pub fn clock(&self) -> u64 {
        self.clock
    }

    pub fn frame_counter(&self) -> u64 {
        self.frame_counter
    }

    pub fn last_frame_interrupt_accepted(&self) -> bool {
        self.last_frame_interrupt_accepted
    }

    pub fn run_for_a_frame(&mut self) {
        while self.clock < self.next_frame_clock {
            self.clock += self.z80.step(&mut self.bus, 0) as u64;
        }

        let irq_cycles = self.z80.irq(&mut self.bus);
        self.last_frame_interrupt_accepted = irq_cycles != 0;
        self.clock += irq_cycles as u64;
        self.next_frame_clock += FRAME_CLOCKS;
        self.frame_counter += 1;

        // Interleaved timing remains a selectable model, but initially shares
        // the complete-frame renderer until raster effects are implemented.
        self.draw_full_frame();
        self.frame_complete = true;
    }

    pub fn draw_full_frame(&mut self) {
        let border = spectrum_color(self.bus.border_color(), false);
        self.framebuffer.fill(border);

        let flash_invert = (self.frame_counter / 16) & 1 != 0;
        let ram = self.bus.ram();

        for y in 0..ACTIVE_HEIGHT {
            let bitmap_offset = ((y & 0xC0) << 5) | ((y & 0x07) << 8) | ((y & 0x38) << 2);
            let attribute_row = (y >> 3) * 32;
            let output_row = (TOP_BORDER + y) * FRAMEBUFFER_WIDTH + LEFT_BORDER;

            for byte_x in 0..32 {
                let pixels = ram[bitmap_offset + byte_x];
                let attribute = ram[0x1800 + attribute_row + byte_x];
                let bright = attribute & 0x40 != 0;
                let invert = attribute & 0x80 != 0 && flash_invert;
                let ink = spectrum_color(attribute & 0x07, bright);
                let paper = spectrum_color((attribute >> 3) & 0x07, bright);

                for bit in 0..8 {
                    let set = pixels & (0x80 >> bit) != 0;
                    let use_ink = set ^ invert;
                    self.framebuffer[output_row + byte_x * 8 + bit] =
                        if use_ink { ink } else { paper };
                }
            }
        }
    }
}

impl Default for Zx82 {
    fn default() -> Self {
        Self::new()
    }
}

fn spectrum_color(color: u8, bright: bool) -> u32 {
    if color & 0x07 == 0 {
        return 0xFF000000;
    }
    let level = if bright { 0xFF } else { 0xCD };
    let r = if color & 0x02 != 0 { level } else { 0 };
    let g = if color & 0x04 != 0 { level } else { 0 };
    let b = if color & 0x01 != 0 { level } else { 0 };
    0xFF000000 | ((b as u32) << 16) | ((g as u32) << 8) | r as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_memory_map_keeps_rom_read_only() {
        let mut bus = Zx82Bus::new();
        let mut rom = [0; ROM_SIZE];
        rom[0x1234] = 0xA5;
        bus.load_rom(&rom).unwrap();

        bus.w8(0x1234, 0x5A);
        bus.w8(0x4000, 0x3C);

        assert_eq!(bus.r8(0x1234), 0xA5);
        assert_eq!(bus.r8(0x4000), 0x3C);
    }

    #[test]
    fn ula_uses_even_ports_and_high_address_for_keyboard_rows() {
        let mut bus = Zx82Bus::new();
        bus.set_key(0, 1, true);

        assert_eq!(bus.in8(0xFE, 0xFE) & 0x1F, 0x1D);
        assert_eq!(bus.in8(0xFE, 0xFF) & 0x1F, 0x1F);
        assert_eq!(bus.in8(0xFF, 0xFE), 0xFF);

        bus.out8(0xFE, 0x15, 0);
        assert_eq!(bus.border_color(), 5);
        assert!(bus.speaker_level());
    }

    #[test]
    fn full_frame_renderer_uses_spectrum_bitmap_layout_and_attributes() {
        let mut zx82 = Zx82::new();
        zx82.bus.ram_mut()[0] = 0x80;
        zx82.bus.ram_mut()[0x1800] = 0x02;
        zx82.draw_full_frame();

        let first_pixel = TOP_BORDER * FRAMEBUFFER_WIDTH + LEFT_BORDER;
        assert_eq!(zx82.framebuffer[first_pixel], spectrum_color(2, false));
        assert_eq!(zx82.framebuffer[first_pixel + 1], spectrum_color(0, false));
    }

    #[test]
    fn frame_interrupt_is_offered_every_69888_t_states() {
        let mut zx82 = Zx82::new();
        zx82.bus.ram_mut()[0] = 0x76;
        zx82.z80.state.r16[11] = 0x4000;
        zx82.z80.state.iff1 = 1;
        zx82.z80.state.iff2 = 1;

        zx82.run_for_a_frame();

        assert!(zx82.clock() >= FRAME_CLOCKS);
        assert!(zx82.last_frame_interrupt_accepted());
        assert_eq!(zx82.z80.state.r16[11], 0x0038);
    }

    #[test]
    fn supplied_48k_rom_reaches_an_initialized_screen() {
        let Ok(rom) = std::fs::read("roms/48.rom") else {
            return;
        };
        let mut zx82 = Zx82::new();
        zx82.load_rom(&rom).unwrap();

        for _ in 0..100 {
            zx82.run_for_a_frame();
        }

        assert!(zx82.bus.ram()[..0x1B00].iter().any(|&byte| byte != 0));
        assert!(zx82.framebuffer.iter().any(|&pixel| pixel != 0xFF000000));
        assert!(zx82.last_frame_interrupt_accepted());
    }

    #[test]
    fn supplied_rom_reads_keyboard_matrix_in_basic() {
        let Ok(rom) = std::fs::read("roms/48.rom") else {
            return;
        };
        let mut zx82 = Zx82::new();
        zx82.load_rom(&rom).unwrap();
        for _ in 0..100 {
            zx82.run_for_a_frame();
        }
        let before = zx82.bus.ram()[..0x1B00].to_vec();

        zx82.bus.set_key(5, 0, true);
        for _ in 0..3 {
            zx82.run_for_a_frame();
        }
        zx82.bus.set_key(5, 0, false);
        for _ in 0..3 {
            zx82.run_for_a_frame();
        }

        assert_ne!(&zx82.bus.ram()[..0x1B00], before.as_slice());
    }
}

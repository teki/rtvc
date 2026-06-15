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
const Z80_BASE_HEADER_SIZE: usize = 30;
const Z80_PAGE_SIZE: usize = 0x4000;
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

    pub fn set_border_color(&mut self, color: u8) {
        self.ula_latch = (self.ula_latch & !0x07) | (color & 0x07);
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

    pub fn load_z80(&mut self, data: &[u8]) -> Result<(), String> {
        if data.len() < Z80_BASE_HEADER_SIZE {
            return Err(format!(
                "Z80 snapshot must contain at least {Z80_BASE_HEADER_SIZE} bytes, got {}",
                data.len()
            ));
        }
        if data[29] & 0x03 > 2 {
            return Err(format!(
                "invalid Z80 interrupt mode {} in snapshot",
                data[29] & 0x03
            ));
        }

        let base_pc = read_word(data, 6);
        let (pc, ram) = if base_pc != 0 {
            let flags = normalized_z80_flags(data[12]);
            let memory = &data[Z80_BASE_HEADER_SIZE..];
            let ram = if flags & 0x20 != 0 {
                decompress_z80(memory, RAM_SIZE, true)?
            } else {
                if memory.len() != RAM_SIZE {
                    return Err(format!(
                        "uncompressed Z80 v1 memory must be {RAM_SIZE} bytes, got {}",
                        memory.len()
                    ));
                }
                memory.to_vec()
            };
            (base_pc, ram)
        } else {
            load_z80_extended_memory(data)?
        };

        let flags = normalized_z80_flags(data[12]);
        self.bus.ram_mut().copy_from_slice(&ram);
        self.restore_z80_registers(data, pc);
        self.bus.reset();
        self.bus.set_border_color((flags >> 1) & 0x07);
        self.clock = 0;
        self.next_frame_clock = FRAME_CLOCKS;
        self.frame_counter = 0;
        self.last_frame_interrupt_accepted = false;
        self.frame_complete = false;
        self.draw_full_frame();
        Ok(())
    }

    fn restore_z80_registers(&mut self, data: &[u8], pc: u16) {
        let state = &mut self.z80.state;
        state.set_reg16(0, u16::from_be_bytes([data[0], data[1]]));
        state.set_reg16(1, read_word(data, 2));
        state.set_reg16(3, read_word(data, 4));
        state.set_reg16(10, read_word(data, 8));
        state.set_reg8(20, data[10]);

        let flags = normalized_z80_flags(data[12]);
        state.set_reg8(21, (data[11] & 0x7F) | ((flags & 0x01) << 7));
        state.set_reg16(2, read_word(data, 13));
        state.set_reg16(7, read_word(data, 15));
        state.set_reg16(8, read_word(data, 17));
        state.set_reg16(9, read_word(data, 19));
        state.set_reg16(6, u16::from_be_bytes([data[21], data[22]]));
        state.set_reg16(5, read_word(data, 23));
        state.set_reg16(4, read_word(data, 25));
        state.set_reg16(11, pc);
        state.iff1 = (data[27] != 0) as u8;
        state.iff2 = (data[28] != 0) as u8;
        state.im = data[29] & 0x03;
        state.halted = 0;
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

fn read_word(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn normalized_z80_flags(flags: u8) -> u8 {
    if flags == 0xFF { 1 } else { flags }
}

fn load_z80_extended_memory(data: &[u8]) -> Result<(u16, Vec<u8>), String> {
    if data.len() < 32 {
        return Err("Z80 v2/v3 snapshot is missing its extended header".to_string());
    }
    let header_len = read_word(data, 30) as usize;
    if !matches!(header_len, 23 | 54 | 55) {
        return Err(format!(
            "unsupported Z80 extended header length {header_len}"
        ));
    }
    let block_start = 32usize
        .checked_add(header_len)
        .ok_or_else(|| "Z80 extended header length overflow".to_string())?;
    if data.len() < block_start {
        return Err("truncated Z80 extended header".to_string());
    }

    let hardware_mode = data[34];
    if hardware_mode != 0 {
        return Err(format!(
            "unsupported Z80 hardware mode {hardware_mode}; Zx82 accepts only plain 48K snapshots"
        ));
    }
    if data.get(37).is_some_and(|flags| flags & 0x80 != 0) {
        return Err("unsupported modified 16K Z80 hardware mode".to_string());
    }

    let mut ram = vec![0; RAM_SIZE];
    let mut loaded_pages = 0u8;
    let mut offset = block_start;
    while offset < data.len() {
        if data.len() - offset < 3 {
            return Err("truncated Z80 memory block header".to_string());
        }
        let compressed_len = read_word(data, offset);
        let page = data[offset + 2];
        offset += 3;

        let ram_offset = match page {
            8 => 0,
            4 => Z80_PAGE_SIZE,
            5 => Z80_PAGE_SIZE * 2,
            _ => {
                return Err(format!("unsupported Z80 page {page} in plain 48K snapshot"));
            }
        };
        let page_bit = match page {
            8 => 1,
            4 => 2,
            5 => 4,
            _ => unreachable!(),
        };
        if loaded_pages & page_bit != 0 {
            return Err(format!("duplicate Z80 memory page {page}"));
        }

        let page_data = if compressed_len == 0xFFFF {
            let end = offset
                .checked_add(Z80_PAGE_SIZE)
                .ok_or_else(|| "Z80 page length overflow".to_string())?;
            if end > data.len() {
                return Err(format!("truncated uncompressed Z80 page {page}"));
            }
            let decoded = data[offset..end].to_vec();
            offset = end;
            decoded
        } else {
            let compressed_len = compressed_len as usize;
            let end = offset
                .checked_add(compressed_len)
                .ok_or_else(|| "Z80 page length overflow".to_string())?;
            if end > data.len() {
                return Err(format!("truncated compressed Z80 page {page}"));
            }
            let decoded = decompress_z80(&data[offset..end], Z80_PAGE_SIZE, false)?;
            offset = end;
            decoded
        };

        ram[ram_offset..ram_offset + Z80_PAGE_SIZE].copy_from_slice(&page_data);
        loaded_pages |= page_bit;
    }

    if loaded_pages != 0x07 {
        return Err(format!(
            "incomplete 48K Z80 snapshot: expected pages 4, 5, and 8, mask is 0x{loaded_pages:02X}"
        ));
    }
    Ok((read_word(data, 32), ram))
}

fn decompress_z80(data: &[u8], expected_len: usize, v1: bool) -> Result<Vec<u8>, String> {
    let mut output = Vec::with_capacity(expected_len);
    let mut offset = 0;

    while output.len() < expected_len {
        if v1
            && data
                .get(offset..offset + 4)
                .is_some_and(|bytes| bytes == [0x00, 0xED, 0xED, 0x00])
        {
            break;
        }
        let Some(&byte) = data.get(offset) else {
            return Err(format!(
                "truncated Z80 compressed data at {} of {expected_len} output bytes",
                output.len()
            ));
        };
        if byte == 0xED && data.get(offset + 1) == Some(&0xED) {
            let count = *data
                .get(offset + 2)
                .ok_or_else(|| "truncated Z80 run length".to_string())?;
            let value = *data
                .get(offset + 3)
                .ok_or_else(|| "truncated Z80 run value".to_string())?;
            let count = if count == 0 { 256 } else { count as usize };
            if output.len() + count > expected_len {
                return Err("Z80 compressed run exceeds expected memory size".to_string());
            }
            output.resize(output.len() + count, value);
            offset += 4;
        } else {
            output.push(byte);
            offset += 1;
        }
    }

    if output.len() != expected_len {
        return Err(format!(
            "Z80 compressed data produced {} bytes, expected {expected_len}",
            output.len()
        ));
    }
    Ok(output)
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

    #[test]
    fn loads_uncompressed_z80_v1() {
        let mut snapshot = z80_base_header(0x4567);
        snapshot[0] = 0x12;
        snapshot[1] = 0x34;
        snapshot[2] = 0x78;
        snapshot[3] = 0x56;
        snapshot[10] = 0x9A;
        snapshot[11] = 0x2B;
        snapshot[12] = 0x0D;
        snapshot[27] = 1;
        snapshot[28] = 1;
        snapshot[29] = 2;
        let mut ram = vec![0; RAM_SIZE];
        ram[0] = 0xA5;
        ram[Z80_PAGE_SIZE] = 0xB6;
        ram[Z80_PAGE_SIZE * 2] = 0xC7;
        snapshot.extend_from_slice(&ram);

        let mut zx82 = Zx82::new();
        zx82.load_z80(&snapshot).unwrap();

        assert_eq!(zx82.z80.state.get_reg16(0), 0x1234);
        assert_eq!(zx82.z80.state.get_reg16(1), 0x5678);
        assert_eq!(zx82.z80.state.get_reg16(11), 0x4567);
        assert_eq!(zx82.z80.state.get_reg8(20), 0x9A);
        assert_eq!(zx82.z80.state.get_reg8(21), 0xAB);
        assert_eq!(zx82.z80.state.iff1, 1);
        assert_eq!(zx82.z80.state.iff2, 1);
        assert_eq!(zx82.z80.state.im, 2);
        assert_eq!(zx82.bus.border_color(), 6);
        assert_eq!(zx82.bus.ram()[0], 0xA5);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE], 0xB6);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE * 2], 0xC7);
    }

    #[test]
    fn loads_compressed_z80_v1() {
        let mut snapshot = z80_base_header(0x3456);
        snapshot[12] = 0x20;
        snapshot.extend(repeated_z80_runs(RAM_SIZE, 0xED));
        snapshot.extend_from_slice(&[0x00, 0xED, 0xED, 0x00]);

        let mut zx82 = Zx82::new();
        zx82.load_z80(&snapshot).unwrap();

        assert_eq!(zx82.z80.state.get_reg16(11), 0x3456);
        assert!(zx82.bus.ram().iter().all(|&byte| byte == 0xED));
    }

    #[test]
    fn loads_z80_v2_uncompressed_pages() {
        let mut snapshot = z80_base_header(0);
        snapshot.extend_from_slice(&23u16.to_le_bytes());
        snapshot.extend_from_slice(&0x2468u16.to_le_bytes());
        snapshot.push(0);
        snapshot.resize(32 + 23, 0);
        append_z80_page(&mut snapshot, 8, &[0x18; Z80_PAGE_SIZE], false);
        append_z80_page(&mut snapshot, 4, &[0x24; Z80_PAGE_SIZE], false);
        append_z80_page(&mut snapshot, 5, &[0x35; Z80_PAGE_SIZE], false);

        let mut zx82 = Zx82::new();
        zx82.load_z80(&snapshot).unwrap();

        assert_eq!(zx82.z80.state.get_reg16(11), 0x2468);
        assert_eq!(zx82.bus.ram()[0], 0x18);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE], 0x24);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE * 2], 0x35);
    }

    #[test]
    fn loads_z80_v3_compressed_pages() {
        let mut snapshot = z80_base_header(0);
        snapshot.extend_from_slice(&54u16.to_le_bytes());
        snapshot.extend_from_slice(&0x1357u16.to_le_bytes());
        snapshot.push(0);
        snapshot.resize(32 + 54, 0);
        append_z80_page(&mut snapshot, 5, &[0x55; Z80_PAGE_SIZE], true);
        append_z80_page(&mut snapshot, 8, &[0x88; Z80_PAGE_SIZE], true);
        append_z80_page(&mut snapshot, 4, &[0x44; Z80_PAGE_SIZE], true);

        let mut zx82 = Zx82::new();
        zx82.load_z80(&snapshot).unwrap();

        assert_eq!(zx82.z80.state.get_reg16(11), 0x1357);
        assert_eq!(zx82.bus.ram()[0], 0x88);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE], 0x44);
        assert_eq!(zx82.bus.ram()[Z80_PAGE_SIZE * 2], 0x55);
    }

    #[test]
    fn rejects_non_48k_and_incomplete_z80_snapshots() {
        let mut snapshot = z80_base_header(0);
        snapshot.extend_from_slice(&23u16.to_le_bytes());
        snapshot.extend_from_slice(&0x1234u16.to_le_bytes());
        snapshot.push(3);
        snapshot.resize(32 + 23, 0);

        let mut zx82 = Zx82::new();
        assert!(zx82.load_z80(&snapshot).is_err());

        snapshot[34] = 0;
        append_z80_page(&mut snapshot, 8, &[0; Z80_PAGE_SIZE], false);
        assert!(zx82.load_z80(&snapshot).is_err());
    }

    fn write_word(data: &mut [u8], offset: usize, value: u16) {
        data[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn z80_base_header(pc: u16) -> Vec<u8> {
        let mut header = vec![0; Z80_BASE_HEADER_SIZE];
        write_word(&mut header, 6, pc);
        header
    }

    fn repeated_z80_runs(length: usize, value: u8) -> Vec<u8> {
        let mut encoded = Vec::new();
        let mut remaining = length;
        while remaining > 0 {
            let count = remaining.min(256);
            encoded.extend_from_slice(&[
                0xED,
                0xED,
                if count == 256 { 0 } else { count as u8 },
                value,
            ]);
            remaining -= count;
        }
        encoded
    }

    fn append_z80_page(snapshot: &mut Vec<u8>, page: u8, data: &[u8], compressed: bool) {
        if compressed {
            let encoded = repeated_z80_runs(data.len(), data[0]);
            snapshot.extend_from_slice(&(encoded.len() as u16).to_le_bytes());
            snapshot.push(page);
            snapshot.extend_from_slice(&encoded);
        } else {
            snapshot.extend_from_slice(&0xFFFFu16.to_le_bytes());
            snapshot.push(page);
            snapshot.extend_from_slice(data);
        }
    }
}

#![allow(dead_code)]

use std::collections::HashSet;

use crate::hbf::HBF;
use crate::key::Key;
use crate::log::Log;
use crate::mmu::{Mmu, TvcMmu};
use crate::snapshot::{self, Reader, SnapshotError, Writer};
use crate::vid::{Vid, VidModel};
use crate::z80::Z80;

const FRAME_CLOCKS: u64 = 62500;

pub struct TvcBus {
    pub mmu: TvcMmu,
    pub vid: Vid,
    pub key: Key,
    pub log: Log,
    pend_it: u8,
    ext_types: u8,
    ext_cart_mapping: u8,
    ext0: Option<HBF>,
}

impl TvcBus {
    pub fn new(is_plus: bool) -> Self {
        TvcBus {
            mmu: TvcMmu::new(is_plus),
            vid: Vid::new(),
            key: Key::new(),
            log: Log::new(),
            pend_it: 0x1F,
            ext_types: 0xFF,
            ext_cart_mapping: 0,
            ext0: None,
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
        self.vid.reset();
        self.key.reset();
        self.pend_it = 0x1F;
        self.ext_types = 0xFF;
        self.ext_cart_mapping = 0;
    }

    pub fn extension_attach(&mut self, port: u8, ext: HBF) {
        if port == 0 {
            self.ext0 = Some(ext);
            self.ext_types &= !(3 << (port * 2));
            self.ext_types |= 2 << (port * 2);
        }
    }

    fn active_ext_mut(&mut self) -> Option<&mut HBF> {
        match self.ext_cart_mapping {
            0 => self.ext0.as_mut(),
            _ => None,
        }
    }

    fn is_ext_page3_access(&self, addr: u16) -> bool {
        let page = (addr >> 14) as usize;
        page == 3 && (self.mmu.get_map_val() & 0xC0) == 0xC0
    }

    fn write_port(&mut self, addr: u8, val: u8) {
        match addr {
            0x00 => self.vid.set_border(val),

            0x02 => self.mmu.set_map(val),

            0x03 => {
                self.key.select_row(val & 0x0F);
                self.ext_cart_mapping = val >> 6;
            }

            0x04 => {}

            0x05 => {}

            0x06 => {
                self.vid.set_mode(val & 0x03);
            }

            0x07 => self.pend_it |= 0x10,

            0x0C..=0x0F => self.mmu.set_vid_map(val),

            0x60..=0x63 => self.vid.set_palette(addr - 0x60, val),

            0x70 => self.vid.set_reg_idx(val),
            0x71 => self.vid.set_reg(val),

            0x58 => {}
            0x59 => {}
            0x5A => {}
            0x5B => {}

            _ => {
                if (0x10..=0x1F).contains(&addr) {
                    if let Some(ref mut ext) = self.ext0 {
                        ext.write_port(addr & 0x0F, val);
                    }
                } else if (0x20..=0x2F).contains(&addr) {
                }
            }
        }
    }

    fn read_port(&mut self, addr: u8) -> u8 {
        match addr {
            0x58 => self.key.read_row(),
            0x59 => 0x40 | self.pend_it,
            0x5A => self.ext_types,
            _ => {
                if (0x10..=0x1F).contains(&addr) {
                    if let Some(ref mut ext) = self.ext0 {
                        ext.read_port(addr & 0x0F)
                    } else {
                        0xFF
                    }
                } else {
                    0xFF
                }
            }
        }
    }
}

impl Mmu for TvcBus {
    fn r8(&mut self, addr: u16) -> u8 {
        if self.is_ext_page3_access(addr) {
            let offset = addr & 0x3FFF;
            if offset < 0x2000 {
                if let Some(ext) = self.active_ext_mut() {
                    return ext.r8(offset);
                }
                return 0xFF;
            }
        }
        self.mmu.r8(addr)
    }

    fn w8(&mut self, addr: u16, val: u8) {
        if self.is_ext_page3_access(addr) {
            let offset = addr & 0x3FFF;
            if offset < 0x2000 {
                if let Some(ext) = self.active_ext_mut() {
                    ext.w8(offset, val);
                }
                return;
            }
        }
        self.mmu.w8(addr, val);
    }

    fn out8(&mut self, port: u8, val: u8, _expected_val: u8) {
        self.write_port(port, val);
    }

    fn in8(&mut self, port: u8, _val: u8) -> u8 {
        self.read_port(port)
    }
}

pub struct Tvc {
    pub bus: TvcBus,
    pub z80: Z80,
    pub framebuffer: Vec<u32>,
    pub frame_complete: bool,
    vid_model: VidModel,
    clock: u64,
    breakpoints: HashSet<u16>,
}

impl Tvc {
    pub fn new(is_plus: bool) -> Self {
        Self::new_with_vid_model(is_plus, VidModel::Simple)
    }

    pub fn new_with_vid_model(is_plus: bool, vid_model: VidModel) -> Self {
        let mut tvc = Tvc {
            bus: TvcBus::new(is_plus),
            z80: Z80::new(),
            framebuffer: vec![0xFF000000; 608 * 288],
            frame_complete: false,
            vid_model,
            clock: 0,
            breakpoints: HashSet::new(),
        };
        tvc.reset();
        tvc
    }

    pub fn vid_model(&self) -> VidModel {
        self.vid_model
    }

    pub fn set_vid_model(&mut self, vid_model: VidModel) {
        self.vid_model = vid_model;
    }

    pub fn save_snapshot(&self) -> Vec<u8> {
        let mut chunks = Vec::new();

        let mut meta = Writer::new();
        meta.u8(self.bus.mmu.is_plus() as u8);
        meta.u8(match self.vid_model {
            VidModel::Simple => 0,
            VidModel::Realistic => 1,
        });
        meta.u64(self.clock);
        meta.u8(self.frame_complete as u8);
        chunks.push((*b"META", meta.into_inner()));

        let mut cpu = Writer::new();
        cpu.raw_bytes(&self.z80.state.r8);
        for reg in self.z80.state.r16 {
            cpu.u16(reg);
        }
        cpu.u8(self.z80.state.halted);
        cpu.u8(self.z80.state.im);
        cpu.u8(self.z80.state.iff1);
        cpu.u8(self.z80.state.iff2);
        chunks.push((*b"CPUZ", cpu.into_inner()));

        let mut mmu = Writer::new();
        self.bus.mmu.write_snapshot(&mut mmu);
        chunks.push((*b"MMU ", mmu.into_inner()));

        let mut vid = Writer::new();
        self.bus.vid.write_snapshot(&mut vid);
        chunks.push((*b"VID ", vid.into_inner()));

        if let Some(ext) = &self.bus.ext0 {
            let mut hbf = Writer::new();
            ext.write_snapshot(&mut hbf);
            chunks.push((*b"HBF ", hbf.into_inner()));
        }

        let mut bus = Writer::new();
        bus.u8(self.bus.pend_it);
        bus.u8(self.bus.ext_types);
        bus.u8(self.bus.ext_cart_mapping);
        chunks.push((*b"BUS ", bus.into_inner()));

        snapshot::write_file(&chunks)
    }

    pub fn load_snapshot(&mut self, data: &[u8]) -> snapshot::Result<()> {
        let chunks = snapshot::read_file(data)?;
        let meta = chunks
            .iter()
            .find(|chunk| chunk.id == *b"META")
            .ok_or(SnapshotError::InvalidChunk("META"))?;
        let mut meta = Reader::new(meta.data);
        let is_plus = meta.u8()? != 0;
        let vid_model = match meta.u8()? {
            0 => VidModel::Simple,
            1 => VidModel::Realistic,
            _ => return Err(SnapshotError::InvalidData("unknown video model".to_string())),
        };
        let clock = meta.u64()?;
        let frame_complete = meta.u8()? != 0;

        *self = Tvc::new_with_vid_model(is_plus, vid_model);
        self.clock = clock;
        self.frame_complete = frame_complete;

        for chunk in chunks {
            let mut reader = Reader::new(chunk.data);
            match &chunk.id {
                b"META" => {}
                b"CPUZ" => {
                    self.z80.state.r8.copy_from_slice(reader.raw_bytes(22)?);
                    for reg in &mut self.z80.state.r16 {
                        *reg = reader.u16()?;
                    }
                    self.z80.state.halted = reader.u8()?;
                    self.z80.state.im = reader.u8()?;
                    self.z80.state.iff1 = reader.u8()?;
                    self.z80.state.iff2 = reader.u8()?;
                }
                b"MMU " => self.bus.mmu.read_snapshot(&mut reader)?,
                b"VID " => self.bus.vid.read_snapshot(&mut reader)?,
                b"HBF " => {
                    self.bus.ext0 = Some(HBF::read_snapshot(&mut reader)?);
                }
                b"BUS " => {
                    self.bus.pend_it = reader.u8()?;
                    self.bus.ext_types = reader.u8()?;
                    self.bus.ext_cart_mapping = reader.u8()?;
                }
                _ => {}
            }
        }
        self.bus.key.reset();
        self.bus.log.clear();
        Ok(())
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        if name.contains("DOS") {
            let hbf = HBF::new(data);
            self.bus.extension_attach(0, hbf);
        } else {
            self.bus.mmu.add_rom(name, data);
        }
    }

    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.bus.mmu.load_cart_rom(data);
    }

    pub fn load_disk(&mut self, name: &str, data: &[u8]) {
        if let Some(ref mut ext) = self.bus.ext0 {
            ext.load_disk(name, data);
        }
    }

    pub fn key_down(&mut self, code: u32) -> bool {
        self.bus.key.key_down(code)
    }

    pub fn key_up(&mut self, code: u32) {
        self.bus.key.key_up(code);
    }

    pub fn key_press(&mut self, ch: char) {
        self.bus.key.key_press(ch);
    }

    pub fn log_entries(&self) -> &[String] {
        &self.bus.log.entries
    }

    pub fn clear_log(&mut self) {
        self.bus.log.clear();
    }

    pub fn focus_change(&mut self, has_focus: bool) {
        if !has_focus {
            self.bus.key.reset();
        }
    }

    pub fn set_breakpoint(&mut self, addr: u16) {
        self.breakpoints.insert(addr);
    }

    pub fn clear_breakpoint(&mut self, addr: u16) {
        self.breakpoints.remove(&addr);
    }

    pub fn clear_all_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub fn run_for_a_frame(&mut self) -> bool {
        let mut do_break = false;
        let mut remaining = FRAME_CLOCKS as u32;
        let mut frame_complete = false;

        while !do_break && remaining > 0 {
            let cpu_time = self.z80.step(&mut self.bus, 0) as u64;

            if self.breakpoints.contains(&self.z80.state.r16[11]) {
                do_break = true;
            }

            self.clock += cpu_time;
            remaining = remaining.saturating_sub(cpu_time as u32);

            if self.vid_model == VidModel::Realistic {
                let cursor_it = self.bus.vid.stream_some(
                    self.bus.mmu.get_vid_mem(),
                    cpu_time.try_into().unwrap_or(u32::MAX),
                );
                if cursor_it {
                    self.bus.pend_it &= !0x10;
                }
                frame_complete |= self.bus.vid.render_stream(&mut self.framebuffer, 608);
            }
        }

        if self.bus.vid.is_initialized() && self.z80.state.iff1 != 0 {
            let irq_duration = self.z80.irq(&mut self.bus);
            self.bus.pend_it &= !0x10;
            self.clock += irq_duration as u64;
        }

        if self.vid_model == VidModel::Simple {
            let vidmem = self.bus.mmu.get_vid_mem();
            self.bus.vid.draw_frame(vidmem, &mut self.framebuffer);
            frame_complete = true;
        }

        self.frame_complete = frame_complete;

        do_break
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trips_core_state() {
        let mut tvc = Tvc::new_with_vid_model(true, VidModel::Realistic);
        tvc.z80.state.r16[11] = 0x1234;
        tvc.bus.mmu.w8(0x4000, 0xAB);
        tvc.bus.vid.set_border(0x55);
        tvc.bus.pend_it = 0x0F;

        let snapshot = tvc.save_snapshot();
        let mut restored = Tvc::new(false);
        restored.load_snapshot(&snapshot).unwrap();

        assert!(restored.bus.mmu.is_plus());
        assert_eq!(restored.vid_model(), VidModel::Realistic);
        assert_eq!(restored.z80.state.r16[11], 0x1234);
        assert_eq!(restored.bus.mmu.r8(0x4000), 0xAB);
        assert_eq!(restored.bus.pend_it, 0x0F);
    }
}

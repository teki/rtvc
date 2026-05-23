#![allow(dead_code)]

use std::collections::HashSet;

use crate::hbf::HBF;
use crate::key::Key;
use crate::log::Log;
use crate::mmu::{Mmu, TvcMmu};
use crate::vid::Vid;
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
    clock: u64,
    breakpoints: HashSet<u16>,
}

impl Tvc {
    pub fn new(is_plus: bool) -> Self {
        let mut tvc = Tvc {
            bus: TvcBus::new(is_plus),
            z80: Z80::new(),
            framebuffer: vec![0xFF000000; 608 * 288],
            frame_complete: false,
            clock: 0,
            breakpoints: HashSet::new(),
        };
        tvc.reset();
        tvc
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

        while !do_break && remaining > 0 {
            let cpu_time = self.z80.step(&mut self.bus, 0) as u64;

            if self.breakpoints.contains(&self.z80.state.r16[11]) {
                do_break = true;
            }

            self.clock += cpu_time;
            remaining = remaining.saturating_sub(cpu_time as u32);
        }

        if self.bus.vid.is_initialized() && self.z80.state.iff1 != 0 {
            let irq_duration = self.z80.irq(&mut self.bus);
            self.bus.pend_it &= !0x10;
            self.clock += irq_duration as u64;
        }

        let vidmem = self.bus.mmu.get_vid_mem();
        self.bus.vid.draw_frame(vidmem, &mut self.framebuffer);
        self.frame_complete = true;

        do_break
    }
}

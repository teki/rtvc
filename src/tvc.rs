#![allow(dead_code)]

use std::collections::HashSet;

use crate::key::Key;
use crate::log::{Log, Logger};
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
        }
    }

    pub fn reset(&mut self) {
        self.mmu.reset();
        self.vid.reset();
        self.key.reset();
        self.pend_it = 0x1F;
        self.ext_cart_mapping = 0;
    }

    fn write_port(&mut self, addr: u8, val: u8) {
        self.log.log(&format!("OUT {:02X} <- {:02X}", addr, val));
        match addr {
            0x00 => self.vid.set_border(val),

            0x02 => self.mmu.set_map(val),

            0x03 => {
                self.key.select_row(val & 0x0F);
                self.ext_cart_mapping = val >> 6;
            }

            0x04 => { /* aud.setFreqL(val) - stub */ }

            0x05 => { /* aud.setFreqH(val & 0x0F); aud.setOn((val & 0x10) != 0) - stub */ }

            0x06 => {
                self.vid.set_mode(val & 0x03);
                // bits 2-5: sound amplitude (stub)
            }

            0x07 => self.pend_it |= 0x10,

            0x0C..=0x0F => self.mmu.set_vid_map(val),

            0x60..=0x63 => self.vid.set_palette(addr - 0x60, val),

            0x70 => self.vid.set_reg_idx(val),
            0x71 => self.vid.set_reg(val),

            0x58 => { /* ext0 interrupt enable - stub */ }
            0x59 => { /* ext1 interrupt enable - stub */ }
            0x5A => { /* ext2 interrupt enable - stub */ }
            0x5B => { /* ext3 interrupt enable - stub */ }

            _ => {
                if (0x10..=0x1F).contains(&addr) {
                    // ext0 write - stub
                } else if (0x20..=0x2F).contains(&addr) {
                    // ext1 write - stub
                }
                // unhandled port write - silently ignore
            }
        }
    }

    fn read_port(&mut self, addr: u8) -> u8 {
        let val = match addr {
            0x58 => self.key.read_row(),
            0x59 => 0x40 | self.pend_it,
            0x5A => self.ext_types,
            _ => {
                if (0x10..=0x1F).contains(&addr) {
                    // ext0 read - stub
                    0xFF
                } else if (0x20..=0x2F).contains(&addr) {
                    // ext1 read - stub
                    0xFF
                } else {
                    0xFF
                }
            }
        };
        self.log.log(&format!("IN  {:02X} -> {:02X}", addr, val));
        val
    }
}

impl Mmu for TvcBus {
    fn r8(&mut self, addr: u16) -> u8 {
        self.mmu.r8(addr)
    }

    fn w8(&mut self, addr: u16, val: u8) {
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
        Tvc {
            bus: TvcBus::new(is_plus),
            z80: Z80::new(),
            framebuffer: vec![0xFF000000; 608 * 288],
            frame_complete: false,
            clock: 0,
            breakpoints: HashSet::new(),
        }
    }

    pub fn reset(&mut self) {
        self.z80.reset();
        self.bus.reset();
        self.clock = 0;
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        self.bus.mmu.add_rom(name, data);
    }

    pub fn load_cart_rom(&mut self, data: &[u8]) {
        self.bus.mmu.load_cart_rom(data);
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

    /// Returns true if a breakpoint was hit.
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

 #![allow(dead_code)]
/// Memory Management Unit trait for the Videoton TV Computer emulators
pub trait Mmu {
    fn r8(&mut self, addr: u16) -> u8;
    fn w8(&mut self, addr: u16, val: u8);

    fn r8s(&mut self, addr: u16) -> i8 {
        self.r8(addr) as i8
    }

    fn r16(&mut self, addr: u16) -> u16 {
        self.r8(addr) as u16 | ((self.r8(addr.wrapping_add(1)) as u16) << 8)
    }

    fn r16nolog(&mut self, addr: u16) -> u16 {
        self.r16(addr)
    }

    fn w16(&mut self, addr: u16, val: u16) {
        self.w8(addr, (val & 0xFF) as u8);
        self.w8(addr.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
    }

    fn w16reverse(&mut self, addr: u16, val: u16) {
        self.w8(addr.wrapping_add(1), ((val >> 8) & 0xFF) as u8);
        self.w8(addr, (val & 0xFF) as u8);
    }

    fn out8(&mut self, _port: u8, _val: u8, _expected_val: u8) {}

    fn in8(&mut self, _port: u8, val: u8) -> u8 {
        val
    }
}

/// Memory Management Unit for the Videoton TV Computer
/// Based on the MMU documentation from jstvc
pub struct TvcMmu {
    // RAM banks (16KB each)
    u0: [u8; 0x4000],
    u1: [u8; 0x4000],
    u2: [u8; 0x4000],
    u3: [u8; 0x4000],

    // Video RAM banks
    vid0: [u8; 0x4000],
    vid1: Option<[u8; 0x4000]>,
    vid2: Option<[u8; 0x4000]>,
    vid3: Option<[u8; 0x4000]>,

    // ROM banks
    sys: [u8; 0x4000],
    cart: [u8; 0x4000],
    exth: [u8; 0x2000], // 8KB

    // Paging state
    map_val: u8,
    map_val_vid: u8,
    is_plus: bool,

    // Current memory map (4 pages -> pointers to banks)
    map: [Option<usize>; 4], // 0=page0, 1=page1, 2=page2, 3=page3
                             // Bank IDs: 0=u0, 1=u1, 2=u2, 3=u3, 4=vid0, 5=vid1, 6=vid2, 7=vid3, 8=sys, 9=cart, 10=ext
}

impl TvcMmu {
    pub fn new(is_plus: bool) -> Self {
        let mut mmu = TvcMmu {
            u0: [0; 0x4000],
            u1: [0; 0x4000],
            u2: [0; 0x4000],
            u3: [0; 0x4000],
            vid0: [0; 0x4000],
            vid1: if is_plus { Some([0; 0x4000]) } else { None },
            vid2: if is_plus { Some([0; 0x4000]) } else { None },
            vid3: if is_plus { Some([0; 0x4000]) } else { None },
            sys: [0; 0x4000],
            cart: [0; 0x4000],
            exth: [0; 0x2000],
            map_val: 0xFF,
            map_val_vid: 0xFF,
            is_plus,
            map: [None; 4],
        };
        mmu.set_vid_map(0);
        mmu.set_map(0);
        mmu
    }

    pub fn reset(&mut self) {
        self.set_vid_map(0);
        self.set_map(0);
    }

    pub fn set_map(&mut self, new_map: u8) {
        if self.map_val == new_map {
            return;
        }
        self.map_val = new_map;

        // Page 0 (0x0000-0x3FFF)
        self.map[0] = Some(match new_map & 0x18 {
            0x00 => 8, // SYS
            0x08 => 9, // CART
            0x10 => 0, // U0
            0x18 => {
                if self.is_plus {
                    3
                } else {
                    0
                }
            } // U3 or U0
            _ => unreachable!(),
        });

        // Page 1 (0x4000-0x7FFF)
        if self.is_plus && (new_map & 0x04) != 0 {
            self.map[1] = Some(match self.map_val_vid & 0x03 {
                0x00 => 4, // VID0
                0x01 => 5, // VID1
                0x02 => 6, // VID2
                0x03 => 7, // VID3
                _ => unreachable!(),
            });
        } else {
            self.map[1] = Some(1); // U1
        }

        // Page 2 (0x8000-0xBFFF)
        if (new_map & 0x20) != 0 {
            self.map[2] = Some(2); // U2
        } else if self.is_plus {
            self.map[2] = Some(match self.map_val_vid & 0x0C {
                0x00 => 4, // VID0
                0x04 => 5, // VID1
                0x08 => 6, // VID2
                0x0C => 7, // VID3
                _ => unreachable!(),
            });
        } else {
            self.map[2] = Some(4); // VID0
        }

        // Page 3 (0xC000-0xFFFF)
        self.map[3] = Some(match new_map & 0xC0 {
            0x00 => 9,  // CART
            0x40 => 8,  // SYS
            0x80 => 3,  // U3
            0xC0 => 10, // EXT
            _ => unreachable!(),
        });
    }

    pub fn set_vid_map(&mut self, new_vid_map: u8) {
        if !self.is_plus {
            return;
        }
        if self.map_val_vid == new_vid_map {
            return;
        }
        self.map_val_vid = new_vid_map;

        // Recompute page 1 if it shows video RAM
        if (self.map_val & 0x04) != 0 {
            self.map[1] = Some(match new_vid_map & 0x03 {
                0x00 => 4, // VID0
                0x01 => 5, // VID1
                0x02 => 6, // VID2
                0x03 => 7, // VID3
                _ => unreachable!(),
            });
        }

        // Recompute page 2 if it shows video RAM
        if (self.map_val & 0x20) == 0 {
            self.map[2] = Some(match new_vid_map & 0x0C {
                0x00 => 4, // VID0
                0x04 => 5, // VID1
                0x08 => 6, // VID2
                0x0C => 7, // VID3
                _ => unreachable!(),
            });
        }
    }

    fn get_bank_mut(&mut self, bank_id: usize) -> Option<&mut [u8; 0x4000]> {
        match bank_id {
            0 => Some(&mut self.u0),
            1 => Some(&mut self.u1),
            2 => Some(&mut self.u2),
            3 => Some(&mut self.u3),
            4 => Some(&mut self.vid0),
            5 => self.vid1.as_mut(),
            6 => self.vid2.as_mut(),
            7 => self.vid3.as_mut(),
            8 => Some(&mut self.sys),
            9 => Some(&mut self.cart),
            _ => None,
        }
    }

    fn get_bank(&self, bank_id: usize) -> Option<&[u8; 0x4000]> {
        match bank_id {
            0 => Some(&self.u0),
            1 => Some(&self.u1),
            2 => Some(&self.u2),
            3 => Some(&self.u3),
            4 => Some(&self.vid0),
            5 => self.vid1.as_ref(),
            6 => self.vid2.as_ref(),
            7 => self.vid3.as_ref(),
            8 => Some(&self.sys),
            9 => Some(&self.cart),
            _ => None,
        }
    }

    pub fn get_vid_mem(&self) -> &[u8] {
        &self.vid0
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        match name {
            "TVC12_D7.64K" | "TVC22_D7.64K" => {
                let len = data.len().min(self.exth.len());
                self.exth[..len].copy_from_slice(&data[..len]);
            }
            "TVC12_D4.64K" | "TVC22_D6.64K" => {
                let len = data.len().min(self.sys.len());
                self.sys[..len].copy_from_slice(&data[..len]);
            }
            "TVC12_D3.64K" | "TVC22_D4.64K" => {
                let offset = 0x2000;
                let space = self.sys.len() - offset;
                let len = data.len().min(space);
                self.sys[offset..offset + len].copy_from_slice(&data[..len]);
            }
            _ => {
                // Unknown ROM name, try loading as cartridge
                self.load_cart_rom(data);
            }
        }
    }

    pub fn load_cart_rom(&mut self, data: &[u8]) {
        let len = data.len().min(0x4000);
        self.cart[..len].copy_from_slice(&data[..len]);
    }
}

impl Mmu for TvcMmu {
    fn r8(&mut self, addr: u16) -> u8 {
        let page = (addr >> 14) as usize;
        let offset = (addr & 0x3FFF) as usize;

        let bank_id = match self.map[page] {
            Some(id) => id,
            None => return 0xFF,
        };

        // EXT handling for page 3
        if bank_id == 10 {
            if offset < 0x2000 {
                // External expansion - return 0xFF for now
                return 0xFF;
            } else {
                // EXTH ROM
                return self.exth[offset - 0x2000];
            }
        }

        match self.get_bank(bank_id) {
            Some(bank) => bank[offset],
            None => 0xFF,
        }
    }

    fn w8(&mut self, addr: u16, val: u8) {
        let page = (addr >> 14) as usize;
        let offset = (addr & 0x3FFF) as usize;

        let bank_id = match self.map[page] {
            Some(id) => id,
            None => return,
        };

        // EXT handling for page 3
        if bank_id == 10 {
            if offset < 0x2000 {
                // External expansion - for now, just ignore
            }
            // EXTH is read-only, ignore writes
            return;
        }

        // ROM banks are read-only
        if bank_id == 8 || bank_id == 9 {
            return;
        }

        if let Some(bank) = self.get_bank_mut(bank_id) {
            bank[offset] = val;
        }
    }
}

/// Simple flat memory for testing (FakeMmu equivalent)
pub struct FakeMmu {
    pub mem: [u8; 0x10000],
    pub log: Vec<String>,
    pub logging_enabled: bool,
}

impl FakeMmu {
    pub fn new() -> Self {
        FakeMmu {
            mem: [0; 0x10000],
            log: Vec::new(),
            logging_enabled: false,
        }
    }

    pub fn clear(&mut self) {
        self.mem = [0; 0x10000];
    }
}

impl Mmu for FakeMmu {
    fn r8(&mut self, addr: u16) -> u8 {
        let val = self.mem[addr as usize];
        if self.logging_enabled {
            self.log.push(format!("MR {:04X} {:02X}", addr, val));
        }
        val
    }

    fn w8(&mut self, addr: u16, val: u8) {
        if self.logging_enabled {
            self.log.push(format!("MW {:04X} {:02X}", addr, val));
        }
        self.mem[addr as usize] = val;
    }

    fn r16nolog(&mut self, addr: u16) -> u16 {
        let was_logging = self.logging_enabled;
        self.logging_enabled = false;
        let val = self.r16(addr);
        self.logging_enabled = was_logging;
        val
    }

    fn out8(&mut self, port: u8, val: u8, expected_val: u8) {
        if self.logging_enabled {
            self.log
                .push(format!("PW {:02X}{:02X} {:02X}", expected_val, port, val));
        }
    }

    fn in8(&mut self, port: u8, val: u8) -> u8 {
        if self.logging_enabled {
            self.log
                .push(format!("PR {:02X}{:02X} {:02X}", val, port, val));
        }
        val
    }
}

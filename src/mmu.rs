/// Memory Management Unit for the Videoton TV Computer
/// Based on the MMU documentation from jstvc
#[derive(Clone, Copy)]
enum FastBootRom {
    V1_2,
    V2_2,
}

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
    fast_boot: bool,
    fast_boot_rom: Option<FastBootRom>,

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
            fast_boot: false,
            fast_boot_rom: None,
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

    pub fn get_map_val(&self) -> u8 {
        self.map_val
    }

    #[inline]
    pub fn ext_card_offset(&self, addr: u16) -> Option<u16> {
        if (self.map_val & 0xC0) != 0xC0 {
            return None;
        }
        let is_page3 = (addr & 0xC000) == 0xC000;
        if !is_page3 {
            return None;
        }
        let offset = addr & 0x3FFF;
        if offset < 0x2000 { Some(offset) } else { None }
    }

    pub fn is_plus(&self) -> bool {
        self.is_plus
    }

    pub fn fast_boot(&self) -> bool {
        self.fast_boot
    }

    pub fn set_fast_boot(&mut self, enabled: bool) {
        self.fast_boot = enabled;
        self.apply_fast_boot_patch();
    }

    pub fn write_snapshot(&self, w: &mut crate::snapshot::Writer) {
        w.u8(self.is_plus as u8);
        w.u8(self.map_val);
        w.u8(self.map_val_vid);
        w.raw_bytes(&self.u0);
        w.raw_bytes(&self.u1);
        w.raw_bytes(&self.u2);
        w.raw_bytes(&self.u3);
        w.raw_bytes(&self.vid0);
        if self.is_plus {
            w.raw_bytes(self.vid1.as_ref().expect("plus video bank 1"));
            w.raw_bytes(self.vid2.as_ref().expect("plus video bank 2"));
            w.raw_bytes(self.vid3.as_ref().expect("plus video bank 3"));
        }
    }

    pub fn read_snapshot(
        &mut self,
        r: &mut crate::snapshot::Reader<'_>,
    ) -> crate::snapshot::Result<()> {
        let is_plus = r.u8()? != 0;
        if is_plus != self.is_plus {
            return Err(crate::snapshot::SnapshotError::InvalidData(
                "snapshot machine model does not match loaded ROMs".to_string(),
            ));
        }
        let map_val = r.u8()?;
        let map_val_vid = r.u8()?;
        self.u0.copy_from_slice(r.raw_bytes(0x4000)?);
        self.u1.copy_from_slice(r.raw_bytes(0x4000)?);
        self.u2.copy_from_slice(r.raw_bytes(0x4000)?);
        self.u3.copy_from_slice(r.raw_bytes(0x4000)?);
        self.vid0.copy_from_slice(r.raw_bytes(0x4000)?);
        if self.is_plus {
            self.vid1
                .as_mut()
                .expect("plus video bank 1")
                .copy_from_slice(r.raw_bytes(0x4000)?);
            self.vid2
                .as_mut()
                .expect("plus video bank 2")
                .copy_from_slice(r.raw_bytes(0x4000)?);
            self.vid3
                .as_mut()
                .expect("plus video bank 3")
                .copy_from_slice(r.raw_bytes(0x4000)?);
        }
        self.map_val = 0xFF;
        self.map_val_vid = 0xFF;
        self.set_vid_map(map_val_vid);
        self.set_map(map_val);
        Ok(())
    }

    pub fn add_rom(&mut self, name: &str, data: &[u8]) {
        match name {
            "TVC12_D7.64K" | "TVC22_D7.64K" => {
                let len = data.len().min(self.exth.len());
                self.exth[..len].copy_from_slice(&data[..len]);
            }
            "TVC12_D4.64K" => {
                let len = data.len().min(self.sys.len());
                self.sys[..len].copy_from_slice(&data[..len]);
                self.fast_boot_rom = Some(FastBootRom::V1_2);
                self.apply_fast_boot_patch();
            }
            "TVC22_D6.64K" => {
                let len = data.len().min(self.sys.len());
                self.sys[..len].copy_from_slice(&data[..len]);
                self.fast_boot_rom = Some(FastBootRom::V2_2);
                self.apply_fast_boot_patch();
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

    fn apply_fast_boot_patch(&mut self) {
        let Some(rom) = self.fast_boot_rom else {
            return;
        };

        // Original 1.2 entry points (2.2 is shifted by 0x0F):
        // C338:
        //   PUSH HL
        //   CALL 0348H      ; low SYS alias while testing a high mapped page
        //   JR C342H
        //
        // C33E:
        //   PUSH HL
        //   CALL C348H      ; high SYS alias
        //
        // Shared epilogue:
        // C342:
        //   POP DE
        //   RET NZ          ; return if the page is faulty
        //   EX DE,HL
        //   LD A,AAH
        //   LD BC,553EH
        //
        // C348 is an overlapping entry in the preceding LD BC instruction:
        // direct calls decode its bytes as LD A,55H.
        // C348:
        //   LD A,55H
        //   PUSH HL
        //   LD E,L
        //   LD D,H
        //   INC DE
        //   LD (HL),A
        //   LD BC,3FFFH
        //   LDIR
        //
        // Fast replacement keeps both entry points and the shared POP DE.
        // RET NZ becomes RET so the slow second AA-pattern pass is skipped.
        // C348:
        //   XOR A
        //   LD (HL),A
        //   LD D,H
        //   LD E,L
        //   INC DE
        //   LD BC,3FFFH
        //   LDIR
        //   EX DE,HL        ; HL = next 16 KB page boundary
        //   RET             ; Z remains set from XOR A
        const CLEAR_ORIGINAL: [u8; 12] = [
            0x3E, 0x55, 0xE5, 0x5D, 0x54, 0x13, 0x77, 0x01, 0xFF, 0x3F, 0xED, 0xB0,
        ];
        const CLEAR_FAST: [u8; 12] = [
            0xAF, 0x77, 0x54, 0x5D, 0x13, 0x01, 0xFF, 0x3F, 0xED, 0xB0, 0xEB, 0xC9,
        ];

        let (shared_ret_offset, clear_offset) = match rom {
            FastBootRom::V1_2 => (0x0343, 0x0348),
            FastBootRom::V2_2 => (0x0352, 0x0357),
        };
        self.apply_fast_boot_bytes(shared_ret_offset, &[0xC0], &[0xC9]);
        self.apply_fast_boot_bytes(clear_offset, &CLEAR_ORIGINAL, &CLEAR_FAST);

        if matches!(rom, FastBootRom::V1_2) {
            // Original at DA19:
            //   LD DE,DC15H
            //
            // Fast replacement:
            //   JR DA77H
            self.apply_fast_boot_bytes(0x1A19, &[0x11, 0x15], &[0x18, 0x5C]);
        } else {
            // Original at CF1F:
            //   BIT 0,A
            //   JR NZ,CF96H
            //   PUSH AF
            //   ... boot-screen drawing ...
            //   POP AF
            // CF96:
            //   BIT 1,A
            //
            // Fast replacement makes the existing branch unconditional. It
            // skips the balanced PUSH/POP drawing block and rejoins at CF96.
            self.apply_fast_boot_bytes(0x0F21, &[0x20, 0x73], &[0x18, 0x73]);
        }
    }

    fn apply_fast_boot_bytes(&mut self, offset: usize, original: &[u8], patched: &[u8]) {
        debug_assert_eq!(original.len(), patched.len());
        let bytes = &mut self.sys[offset..offset + original.len()];
        if self.fast_boot && bytes == original {
            bytes.copy_from_slice(patched);
        } else if !self.fast_boot && bytes == patched {
            bytes.copy_from_slice(original);
        }
    }

    pub fn read_raw_bank(&self, bank: &str, addr: usize, len: usize) -> Option<Vec<u8>> {
        let bank_data: &[u8] = match bank.to_lowercase().as_str() {
            "u0" => &self.u0,
            "u1" => &self.u1,
            "u2" => &self.u2,
            "u3" => &self.u3,
            "vid0" => &self.vid0,
            "vid1" => self.vid1.as_ref()?,
            "vid2" => self.vid2.as_ref()?,
            "vid3" => self.vid3.as_ref()?,
            "sys" => &self.sys,
            "cart" => &self.cart,
            "exth" => &self.exth,
            _ => return None,
        };

        if addr >= bank_data.len() {
            return Some(Vec::new());
        }
        let end = (addr + len).min(bank_data.len());
        Some(bank_data[addr..end].to_vec())
    }
}

impl TvcMmu {
    pub fn r8(&mut self, addr: u16) -> u8 {
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

    pub fn w8(&mut self, addr: u16, val: u8) {
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

#[cfg(test)]
mod tests {
    use super::{FastBootRom, TvcMmu};
    use crate::bus::FakeBus;
    use crate::z80::Z80;

    const CLEAR_FAST: [u8; 12] = [
        0xAF, 0x77, 0x54, 0x5D, 0x13, 0x01, 0xFF, 0x3F, 0xED, 0xB0, 0xEB, 0xC9,
    ];

    #[test]
    fn fast_boot_patches_and_restores_known_roms() {
        let mut mmu = TvcMmu::new(false);

        let rom_1_2 = include_bytes!("../roms/TVC12_D4.64K");
        let original_entry_1_2 = &rom_1_2[0x0338..0x0348];
        let original_clear_1_2 = &rom_1_2[0x0348..0x0354];
        assert_eq!(&rom_1_2[0x1A19..0x1A1B], &[0x11, 0x15]);
        mmu.set_fast_boot(true);
        mmu.add_rom("TVC12_D4.64K", rom_1_2);
        assert_eq!(
            mmu.read_raw_bank("sys", 0x0338, 11).unwrap(),
            original_entry_1_2[..11]
        );
        assert_eq!(mmu.read_raw_bank("sys", 0x0343, 1).unwrap(), [0xC9]);
        assert_eq!(mmu.read_raw_bank("sys", 0x0348, 12).unwrap(), CLEAR_FAST);
        assert_eq!(mmu.read_raw_bank("sys", 0x1A19, 2).unwrap(), [0x18, 0x5C]);
        mmu.set_fast_boot(false);
        assert_eq!(
            mmu.read_raw_bank("sys", 0x0338, 16).unwrap(),
            original_entry_1_2
        );
        assert_eq!(
            mmu.read_raw_bank("sys", 0x0348, 12).unwrap(),
            original_clear_1_2
        );
        assert_eq!(mmu.read_raw_bank("sys", 0x1A19, 2).unwrap(), [0x11, 0x15]);

        let rom_2_2 = include_bytes!("../roms/TVC22_D6.64K");
        let original_entry_2_2 = &rom_2_2[0x034D..0x0357];
        let original_clear_2_2 = &rom_2_2[0x0357..0x0363];
        assert_eq!(&rom_2_2[0x0F21..0x0F23], &[0x20, 0x73]);
        mmu.set_fast_boot(true);
        mmu.add_rom("TVC22_D6.64K", rom_2_2);
        assert_eq!(
            mmu.read_raw_bank("sys", 0x034D, 5).unwrap(),
            original_entry_2_2[..5]
        );
        assert_eq!(mmu.read_raw_bank("sys", 0x0352, 1).unwrap(), [0xC9]);
        assert_eq!(mmu.read_raw_bank("sys", 0x0357, 12).unwrap(), CLEAR_FAST);
        assert_eq!(mmu.read_raw_bank("sys", 0x0F21, 2).unwrap(), [0x18, 0x73]);
        mmu.set_fast_boot(false);
        assert_eq!(
            mmu.read_raw_bank("sys", 0x034D, 10).unwrap(),
            original_entry_2_2
        );
        assert_eq!(
            mmu.read_raw_bank("sys", 0x0357, 12).unwrap(),
            original_clear_2_2
        );
        assert_eq!(mmu.read_raw_bank("sys", 0x0F21, 2).unwrap(), [0x20, 0x73]);
    }

    #[test]
    fn fast_boot_does_not_patch_unexpected_bytes() {
        let rom = [0u8; 0x2000];
        let mut mmu = TvcMmu::new(false);
        mmu.set_fast_boot(true);

        mmu.add_rom("TVC12_D4.64K", &rom);
        assert_eq!(mmu.read_raw_bank("sys", 0x0343, 1).unwrap(), [0x00]);
        assert_eq!(mmu.read_raw_bank("sys", 0x0348, 12).unwrap(), [0x00; 12]);
        assert_eq!(mmu.read_raw_bank("sys", 0x1A19, 2).unwrap(), [0x00, 0x00]);

        mmu.add_rom("TVC22_D6.64K", &rom);
        assert_eq!(mmu.read_raw_bank("sys", 0x0352, 1).unwrap(), [0x00]);
        assert_eq!(mmu.read_raw_bank("sys", 0x0357, 12).unwrap(), [0x00; 12]);
        assert_eq!(mmu.read_raw_bank("sys", 0x0F21, 2).unwrap(), [0x00, 0x00]);
    }

    #[test]
    fn fast_ram_test_preserves_both_entry_point_stack_contracts() {
        run_fast_ram_test(FastBootRom::V1_2, 0xC338);
        run_fast_ram_test(FastBootRom::V1_2, 0xC33E);
        run_fast_ram_test(FastBootRom::V2_2, 0xC347);
        run_fast_ram_test(FastBootRom::V2_2, 0xC34D);
    }

    fn run_fast_ram_test(rom: FastBootRom, entry: u16) {
        let rom_bytes: &[u8] = match rom {
            FastBootRom::V1_2 => include_bytes!("../roms/TVC12_D4.64K"),
            FastBootRom::V2_2 => include_bytes!("../roms/TVC22_D6.64K"),
        };
        let mut mmu = TvcMmu::new(false);
        mmu.set_fast_boot(true);
        mmu.add_rom(
            match rom {
                FastBootRom::V1_2 => "TVC12_D4.64K",
                FastBootRom::V2_2 => "TVC22_D6.64K",
            },
            rom_bytes,
        );
        let sys = mmu.read_raw_bank("sys", 0, 0x2000).unwrap();

        let mut bus = FakeBus::new();
        bus.mem[..sys.len()].copy_from_slice(&sys);
        bus.mem[0xC000..0xC000 + sys.len()].copy_from_slice(&sys);
        bus.mem[0x4000..0x8000].fill(0xA5);
        bus.mem[0x9000] = 0x34;
        bus.mem[0x9001] = 0x12;

        let mut z80 = Z80::new();
        z80.state.set_reg16(3, 0x4000);
        z80.state.set_reg16(10, 0x9000);
        z80.state.set_reg16(11, entry);

        for _ in 0..0x5000 {
            z80.step(&mut bus, 0);
            if z80.state.get_reg16(11) == 0x1234 {
                break;
            }
        }

        assert_eq!(z80.state.get_reg16(11), 0x1234);
        assert_eq!(z80.state.get_reg16(3), 0x8000);
        assert_eq!(z80.state.get_reg16(2), 0x4000);
        assert_eq!(z80.state.get_reg16(10), 0x9002);
        assert_ne!(z80.state.get_reg16(0) as u8 & 0x40, 0);
        assert!(bus.mem[0x4000..0x8000].iter().all(|byte| *byte == 0));
    }
}

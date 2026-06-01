#![allow(dead_code)]
use crate::bus::CpuBus;

// Flag constants
const F_S: u8 = 0x80;
const F_Z: u8 = 0x40;
const F_5: u8 = 0x20;
const F_H: u8 = 0x10;
const F_3: u8 = 0x08;
const F_PV: u8 = 0x04;
const F_N: u8 = 0x02;
const F_C: u8 = 0x01;

// Register indices for 8-bit access
const R_A: usize = 0;
const R_F: usize = 1;
const R_B: usize = 2;
const R_C: usize = 3;
const R_D: usize = 4;
const R_E: usize = 5;
const R_H: usize = 6;
const R_L: usize = 7;
const R_XH: usize = 8;
const R_XL: usize = 9;
const R_YH: usize = 10;
const R_YL: usize = 11;
const R_AA: usize = 12;
const R_FA: usize = 13;
const R_BA: usize = 14;
const R_CA: usize = 15;
const R_DA: usize = 16;
const R_EA: usize = 17;
const R_HA: usize = 18;
const R_LA: usize = 19;
const R_I: usize = 20;
const R_R: usize = 21;

// Register indices for 16-bit access
const R_AF: usize = 0;
const R_BC: usize = 1;
const R_DE: usize = 2;
const R_HL: usize = 3;
const R_IX: usize = 4;
const R_IY: usize = 5;
const R_AFA: usize = 6;
const R_BCA: usize = 7;
const R_DEA: usize = 8;
const R_HLA: usize = 9;
const R_SP: usize = 10;
const R_PC: usize = 11;
const R_IR: usize = 12;

pub struct Z80State {
    // 8-bit registers: A,F,B,C,D,E,H,L,IXh,IXl,IYh,IYl,A',F',B',C',D',E',H',L',I,R
    pub r8: [u8; 22],
    // 16-bit registers: AF,BC,DE,HL,IX,IY,AF',BC',DE',HL',SP,PC,IR
    pub r16: [u16; 13],

    pub halted: u8,
    pub im: u8,
    pub iff1: u8,
    pub iff2: u8,
}

impl Z80State {
    pub fn new() -> Self {
        let mut state = Z80State {
            r8: [0; 22],
            r16: [0; 13],
            halted: 0,
            im: 0,
            iff1: 0,
            iff2: 0,
        };
        state.initialize();
        state
    }

    pub fn initialize(&mut self) {
        self.r8 = [0; 22];
        self.r16 = [0; 13];

        self.r16[R_AF] = 0xFFFF;
        self.r16[R_BC] = 0xFFFF;
        self.r16[R_DE] = 0xFFFF;
        self.r16[R_HL] = 0xFFFF;
        self.r16[R_IX] = 0xFFFF;
        self.r16[R_IY] = 0xFFFF;
        self.r16[R_SP] = 0xFFFF;
        self.r16[R_AFA] = 0xFFFF;
        self.r16[R_BCA] = 0xFFFF;
        self.r16[R_DEA] = 0xFFFF;
        self.r16[R_HLA] = 0xFFFF;
        self.sync_r8_from_r16();

        self.reset();
    }

    pub fn reset(&mut self) {
        // Hardware reset only defines PC, interrupt state, I, R, and HALT.
        // General registers and SP retain their previous contents.
        self.halted = 0;
        self.im = 0;
        self.iff1 = 0;
        self.iff2 = 0;

        self.r8[R_I] = 0x00;
        self.r8[R_R] = 0x00;

        self.r16[R_PC] = 0x0000;
    }

    // Sync 8-bit registers from 16-bit (high byte first)
    pub fn sync_r8_from_r16(&mut self) {
        for i in 0..=9 {
            let val = self.r16[i];
            self.r8[i * 2] = ((val >> 8) & 0xFF) as u8;
            self.r8[i * 2 + 1] = (val & 0xFF) as u8;
        }
    }

    // Sync 16-bit registers from 8-bit
    pub fn sync_r16_from_r8(&mut self) {
        for i in 0..=9 {
            self.r16[i] = ((self.r8[i * 2] as u16) << 8) | (self.r8[i * 2 + 1] as u16);
        }
    }

    pub fn get_reg16(&self, reg: usize) -> u16 {
        match reg {
            0..=9 => ((self.r8[reg * 2] as u16) << 8) | (self.r8[reg * 2 + 1] as u16),
            10 => self.r16[R_SP],
            11 => self.r16[R_PC],
            12 => ((self.r8[R_I] as u16) << 8) | (self.r8[R_R] as u16),
            _ => 0,
        }
    }

    pub fn set_reg16(&mut self, reg: usize, val: u16) {
        match reg {
            0..=9 => {
                self.r8[reg * 2] = ((val >> 8) & 0xFF) as u8;
                self.r8[reg * 2 + 1] = (val & 0xFF) as u8;
            }
            10 => self.r16[R_SP] = val,
            11 => self.r16[R_PC] = val,
            12 => {
                self.r8[R_I] = ((val >> 8) & 0xFF) as u8;
                self.r8[R_R] = (val & 0xFF) as u8;
            }
            _ => {}
        }
    }

    pub fn get_reg8(&self, reg: usize) -> u8 {
        self.r8[reg]
    }

    pub fn set_reg8(&mut self, reg: usize, val: u8) {
        self.r8[reg] = val;
    }
}

pub struct Z80 {
    pub state: Z80State,
    pub sz53_table: [u8; 256],
    pub sz53p_table: [u8; 256],

    // Port handlers
    pub port_out: Option<fn(u8, u8, u8)>,
    pub port_in: Option<fn(u8, u8) -> u8>,
}

impl Z80 {
    pub fn new() -> Self {
        let mut z80 = Z80 {
            state: Z80State::new(),
            sz53_table: [0; 256],
            sz53p_table: [0; 256],
            port_out: None,
            port_in: None,
        };

        // Precompute lookup tables
        for i in 0..256 {
            let mut flags = (i as u8 & F_S) | (i as u8 & F_5) | (i as u8 & F_3);
            if i == 0 {
                flags |= F_Z;
            }
            z80.sz53_table[i] = flags;

            // Parity
            let mut parity = 0;
            let mut temp = i as u8;
            for _ in 0..8 {
                parity ^= temp & 1;
                temp >>= 1;
            }
            z80.sz53p_table[i] = flags | if parity == 0 { F_PV } else { 0 };
        }

        z80
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    pub fn initialize(&mut self) {
        self.state.initialize();
    }

    pub fn push16<M: CpuBus>(&mut self, mmu: &mut M, val: u16) {
        let sp = self.state.r16[R_SP].wrapping_sub(1);
        mmu.w8(sp, ((val >> 8) & 0xFF) as u8);
        let sp = sp.wrapping_sub(1);
        mmu.w8(sp, (val & 0xFF) as u8);
        self.state.r16[R_SP] = sp;
    }

    pub fn pop16<M: CpuBus>(&mut self, mmu: &mut M) -> u16 {
        let sp = self.state.r16[R_SP];
        let lo = mmu.r8(sp) as u16;
        let sp = sp.wrapping_add(1);
        let hi = mmu.r8(sp) as u16;
        self.state.r16[R_SP] = sp.wrapping_add(1);
        (hi << 8) | lo
    }

    // Arithmetic helpers
    pub fn add8(&self, val1: u8, val2: u8, cin: bool) -> (u8, u8) {
        let cin_val = if cin { 1 } else { 0 };
        let res = (val1 as u16) + (val2 as u16) + cin_val;
        let res4 = ((val1 & 0x0F) as u16) + ((val2 & 0x0F) as u16) + cin_val;
        let res8 = (res & 0xFF) as u8;

        let val1s = (val1 & 0x80) != 0;
        let val2s = (val2 & 0x80) != 0;
        let ress = (res8 & 0x80) != 0;
        let overflow = val1s == val2s && val1s != ress;

        let chalf = res4 > 0x0F;
        let cout = res > 0xFF;

        let mut flags = self.sz53_table[res8 as usize];
        if overflow {
            flags |= F_PV;
        }
        if chalf {
            flags |= F_H;
        }
        if cout {
            flags |= F_C;
        }

        (res8, flags)
    }

    pub fn sub8(&self, val1: u8, val2: u8, cin: bool) -> (u8, u8) {
        let (res, mut flags) = self.add8(val1, !val2, !cin);
        flags ^= F_H | F_C;
        flags |= F_N;
        (res, flags)
    }

    pub fn add16(&self, val1: u16, val2: u16, cin: bool) -> (u16, u8) {
        let (res_l, flags_l) = self.add8((val1 & 0xFF) as u8, (val2 & 0xFF) as u8, cin);
        let (res_h, flags_h) = self.add8(
            ((val1 >> 8) & 0xFF) as u8,
            ((val2 >> 8) & 0xFF) as u8,
            flags_l & F_C != 0,
        );

        let res16 = ((res_h as u16) << 8) | (res_l as u16);

        let mut flags = 0;
        flags |= res_h & F_S;
        if res16 == 0 {
            flags |= F_Z;
        }
        flags |= res_h & F_5;
        flags |= flags_h & F_H;
        flags |= res_h & F_3;
        flags |= flags_h & F_PV;
        flags |= flags_h & F_C;

        (res16, flags)
    }

    pub fn sub16(&self, val1: u16, val2: u16, cin: bool) -> (u16, u8) {
        let (res, mut flags) = self.add16(val1, !val2, !cin);
        flags ^= F_C | F_H;
        flags |= F_N;
        (res, flags)
    }

    pub fn shl8(&self, val: u8, right_in: bool) -> (u8, u8) {
        let cout = (val & 0x80) != 0;
        let res = ((val << 1) | if right_in { 1 } else { 0 }) & 0xFF;
        let mut flags = self.sz53p_table[res as usize];
        if cout {
            flags |= F_C;
        }
        (res, flags)
    }

    pub fn shr8(&self, val: u8, left_in: bool) -> (u8, u8) {
        let cout = (val & 0x01) != 0;
        let res = ((val >> 1) | if left_in { 0x80 } else { 0 }) & 0xFF;
        let mut flags = self.sz53p_table[res as usize];
        if cout {
            flags |= F_C;
        }
        (res, flags)
    }

    pub fn get_reg_val(&self, name: &str) -> u16 {
        match name {
            "AF" => self.state.get_reg16(R_AF),
            "BC" => self.state.get_reg16(R_BC),
            "DE" => self.state.get_reg16(R_DE),
            "HL" => self.state.get_reg16(R_HL),
            "AFa" => self.state.get_reg16(R_AFA),
            "BCa" => self.state.get_reg16(R_BCA),
            "DEa" => self.state.get_reg16(R_DEA),
            "HLa" => self.state.get_reg16(R_HLA),
            "IX" => self.state.get_reg16(R_IX),
            "IY" => self.state.get_reg16(R_IY),
            "SP" => self.state.r16[R_SP],
            "PC" => self.state.r16[R_PC],
            "A" => self.state.r8[R_A] as u16,
            "F" => self.state.r8[R_F] as u16,
            "B" => self.state.r8[R_B] as u16,
            "C" => self.state.r8[R_C] as u16,
            "D" => self.state.r8[R_D] as u16,
            "E" => self.state.r8[R_E] as u16,
            "H" => self.state.r8[R_H] as u16,
            "L" => self.state.r8[R_L] as u16,
            "I" => self.state.r8[R_I] as u16,
            "R" => self.state.r8[R_R] as u16,
            "IFF1" => self.state.iff1 as u16,
            "IFF2" => self.state.iff2 as u16,
            "im" => self.state.im as u16,
            "halted" => self.state.halted as u16,
            _ => 0,
        }
    }

    pub fn set_reg_val(&mut self, name: &str, val: u16) {
        match name {
            "AF" => self.state.set_reg16(R_AF, val),
            "BC" => self.state.set_reg16(R_BC, val),
            "DE" => self.state.set_reg16(R_DE, val),
            "HL" => self.state.set_reg16(R_HL, val),
            "AFa" => self.state.set_reg16(R_AFA, val),
            "BCa" => self.state.set_reg16(R_BCA, val),
            "DEa" => self.state.set_reg16(R_DEA, val),
            "HLa" => self.state.set_reg16(R_HLA, val),
            "IX" => self.state.set_reg16(R_IX, val),
            "IY" => self.state.set_reg16(R_IY, val),
            "SP" => self.state.r16[R_SP] = val,
            "PC" => self.state.r16[R_PC] = val,
            "A" => self.state.r8[R_A] = (val & 0xFF) as u8,
            "F" => self.state.r8[R_F] = (val & 0xFF) as u8,
            "B" => self.state.r8[R_B] = (val & 0xFF) as u8,
            "C" => self.state.r8[R_C] = (val & 0xFF) as u8,
            "D" => self.state.r8[R_D] = (val & 0xFF) as u8,
            "E" => self.state.r8[R_E] = (val & 0xFF) as u8,
            "H" => self.state.r8[R_H] = (val & 0xFF) as u8,
            "L" => self.state.r8[R_L] = (val & 0xFF) as u8,
            "I" => self.state.r8[R_I] = (val & 0xFF) as u8,
            "R" => self.state.r8[R_R] = (val & 0xFF) as u8,
            "IFF1" => self.state.iff1 = (val & 1) as u8,
            "IFF2" => self.state.iff2 = (val & 1) as u8,
            "im" => self.state.im = (val & 3) as u8,
            "halted" => self.state.halted = (val & 1) as u8,
            _ => {}
        }
    }

    pub fn irq<M: CpuBus>(&mut self, mmu: &mut M) -> u32 {
        if self.state.iff1 != 0 {
            self.state.halted = 0;
            self.state.iff1 = 0;
            self.state.iff2 = 0;
            let pc = self.state.r16[R_PC];
            self.push16(mmu, pc);
            self.state.r16[R_PC] = 0x0038;
            13
        } else {
            0
        }
    }

    pub fn step<M: CpuBus>(&mut self, mmu: &mut M, _run_for: i32) -> u32 {
        if self.state.halted != 0 {
            return 4;
        }

        let mut pc = self.state.r16[R_PC];
        let mut r_add = 0u8;
        let mut t_add = 0u32;
        let mut displ = 0i8;
        let mut opcode = mmu.r8(pc) as u32;
        if opcode == 0xDD || opcode == 0xFD {
            let mut pc_loop = pc;
            let mut opcodeb2: u32;
            loop {
                opcodeb2 = mmu.r8(pc_loop.wrapping_add(1)) as u32;
                if opcodeb2 == 0xDD || opcodeb2 == 0xFD {
                    opcode = opcodeb2;
                    t_add += 4;
                    pc_loop = pc_loop.wrapping_add(1);
                    r_add += 1;
                } else {
                    break;
                }
            }
            self.state.r16[R_PC] = pc_loop;
            pc = pc_loop;
            opcode = (opcode << 8) | opcodeb2;
            r_add += 2;
            if opcode == 0xFDCB || opcode == 0xDDCB {
                displ = mmu.r8s(pc_loop.wrapping_add(2));
                opcode = (opcode << 8) | mmu.r8(pc_loop.wrapping_add(3)) as u32;
            }
        } else if opcode == 0xED || opcode == 0xCB {
            opcode = (opcode << 8) | mmu.r8(pc.wrapping_add(1)) as u32;
            r_add += 2;
        } else {
            r_add += 1;
        }
        self.state.r8[R_R] =
            (self.state.r8[R_R] & 0x80) | ((self.state.r8[R_R].wrapping_add(r_add)) & 0x7F);
        let (t, m) = self.execute(opcode, displ, mmu, &mut t_add, &mut pc);
        if t == 0 {
            panic!("Opcode not implemented: 0x{:04X}", opcode);
        }
        if m > 0 && (self.state.halted == 0 || opcode == 0x76) {
            self.state.r16[R_PC] = pc.wrapping_add(m as u16);
        }
        t + t_add
    }

    pub fn execute<M: CpuBus>(
        &mut self,
        opcode: u32,
        displ: i8,
        mmu: &mut M,
        t_add: &mut u32,
        pc: &mut u16,
    ) -> (u32, u8) {
        if opcode <= 0xFF {
            return self.execute_base(opcode as u8, mmu);
        }
        let hi = opcode >> 8;
        if hi == 0xCB {
            return self.execute_cb((opcode & 0xFF) as u8, mmu);
        }
        if hi == 0xED {
            return self.execute_ed((opcode & 0xFF) as u8, mmu);
        }
        if hi == 0xDD {
            let res = self.execute_dd((opcode & 0xFF) as u8, displ, mmu);
            if res.0 != 0 || res.1 != 0 {
                return res;
            }
            *t_add += 4;
            *pc = pc.wrapping_add(1);
            return self.execute_base((opcode & 0xFF) as u8, mmu);
        }
        if hi == 0xFD {
            let res = self.execute_fd((opcode & 0xFF) as u8, displ, mmu);
            if res.0 != 0 || res.1 != 0 {
                return res;
            }
            *t_add += 4;
            *pc = pc.wrapping_add(1);
            return self.execute_base((opcode & 0xFF) as u8, mmu);
        }
        if hi == 0xDDCB {
            return self.execute_ddcb((opcode & 0xFF) as u8, displ, mmu);
        }
        if hi == 0xFDCB {
            return self.execute_fdcb((opcode & 0xFF) as u8, displ, mmu);
        }
        (0, 0)
    }

    pub fn execute_base<M: CpuBus>(&mut self, opcode: u8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x00 => (4, 1),
            0x01 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(1));
                self.state.set_reg16(R_BC, nn);
                (10, 3)
            }
            0x02 => {
                mmu.w8(self.state.get_reg16(R_BC), self.state.r8[R_A]);
                (7, 1)
            }
            0x03 => {
                self.state
                    .set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_add(1));
                (6, 1)
            }
            0x04 => {
                let (res, flags) = self.add8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x05 => {
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x06 => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_B] = n;
                (7, 2)
            }
            0x07 => {
                let a = self.state.r8[R_A];
                let (res, flags) = self.shl8(a, (a & 0x80) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x08 => {
                let a = self.state.r8[R_A];
                self.state.r8[R_A] = self.state.r8[R_AA];
                self.state.r8[R_AA] = a;
                let f = self.state.r8[R_F];
                self.state.r8[R_F] = self.state.r8[R_FA];
                self.state.r8[R_FA] = f;
                (4, 1)
            }
            0x09 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x0A => {
                self.state.r8[R_A] = mmu.r8(self.state.get_reg16(R_BC));
                (7, 1)
            }
            0x0B => {
                self.state
                    .set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_sub(1));
                (6, 1)
            }
            0x0C => {
                let (res, flags) = self.add8(self.state.r8[R_C], 1, false);
                self.state.r8[R_C] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x0D => {
                let (res, flags) = self.sub8(self.state.r8[R_C], 1, false);
                self.state.r8[R_C] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x0E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_C] = n;
                (7, 2)
            }
            0x0F => {
                let a = self.state.r8[R_A];
                let (res, flags) = self.shr8(a, (a & 0x01) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x10 => {
                let pc = self.state.r16[R_PC];
                self.state.r8[R_B] = self.state.r8[R_B].wrapping_sub(1);
                if self.state.r8[R_B] == 0 {
                    (8, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                    (13, 0)
                }
            }
            0x11 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(1));
                self.state.set_reg16(R_DE, nn);
                (10, 3)
            }
            0x12 => {
                mmu.w8(self.state.get_reg16(R_DE), self.state.r8[R_A]);
                (7, 1)
            }
            0x13 => {
                self.state
                    .set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_add(1));
                (6, 1)
            }
            0x14 => {
                let (res, flags) = self.add8(self.state.r8[R_D], 1, false);
                self.state.r8[R_D] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x15 => {
                let (res, flags) = self.sub8(self.state.r8[R_D], 1, false);
                self.state.r8[R_D] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x16 => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_D] = n;
                (7, 2)
            }
            0x17 => {
                let a = self.state.r8[R_A];
                let (res, flags) = self.shl8(a, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x18 => {
                let pc = self.state.r16[R_PC];
                let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                (12, 0)
            }
            0x19 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x1A => {
                self.state.r8[R_A] = mmu.r8(self.state.get_reg16(R_DE));
                (7, 1)
            }
            0x1B => {
                self.state
                    .set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_sub(1));
                (6, 1)
            }
            0x1C => {
                let (res, flags) = self.add8(self.state.r8[R_E], 1, false);
                self.state.r8[R_E] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x1D => {
                let (res, flags) = self.sub8(self.state.r8[R_E], 1, false);
                self.state.r8[R_E] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x1E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_E] = n;
                (7, 2)
            }
            0x1F => {
                let a = self.state.r8[R_A];
                let (res, flags) = self.shr8(a, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x20 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    (7, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                    (12, 0)
                }
            }
            0x21 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(1));
                self.state.set_reg16(R_HL, nn);
                (10, 3)
            }
            0x22 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                mmu.w16(nn, self.state.get_reg16(R_HL));
                (16, 3)
            }
            0x23 => {
                self.state
                    .set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_add(1));
                (6, 1)
            }
            0x24 => {
                let (res, flags) = self.add8(self.state.r8[R_H], 1, false);
                self.state.r8[R_H] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x25 => {
                let (res, flags) = self.sub8(self.state.r8[R_H], 1, false);
                self.state.r8[R_H] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x26 => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_H] = n;
                (7, 2)
            }
            0x27 => {
                let a = self.state.r8[R_A];
                let mut add = 0u8;
                let carry = self.state.r8[R_F] & F_C;
                let lownibble = a & 0x0F;
                if (self.state.r8[R_F] & F_H) != 0 || lownibble > 9 {
                    add = 6;
                }
                let mut new_carry = carry;
                if carry != 0 || a > 0x99 {
                    add |= 0x60;
                    new_carry = F_C;
                }
                let (res, flags) = if (self.state.r8[R_F] & F_N) != 0 {
                    self.sub8(a, add, false)
                } else {
                    self.add8(a, add, false)
                };
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = (self.state.r8[R_F] & F_N)
                    | self.sz53p_table[res as usize]
                    | (flags & F_H)
                    | new_carry;
                (4, 1)
            }
            0x28 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                    (12, 0)
                } else {
                    (7, 2)
                }
            }
            0x29 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, hl, false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x2A => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.set_reg16(R_HL, mmu.r16(nn));
                (16, 3)
            }
            0x2B => {
                self.state
                    .set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_sub(1));
                (6, 1)
            }
            0x2C => {
                let (res, flags) = self.add8(self.state.r8[R_L], 1, false);
                self.state.r8[R_L] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x2D => {
                let (res, flags) = self.sub8(self.state.r8[R_L], 1, false);
                self.state.r8[R_L] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x2E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_L] = n;
                (7, 2)
            }
            0x2F => {
                self.state.r8[R_A] = !self.state.r8[R_A];
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV | F_C))
                    | F_H
                    | F_N
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3);
                (4, 1)
            }
            0x30 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    (7, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                    (12, 0)
                }
            }
            0x31 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(1));
                self.state.r16[R_SP] = nn;
                (10, 3)
            }
            0x32 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                mmu.w8(nn, self.state.r8[R_A]);
                (13, 3)
            }
            0x33 => {
                self.state.r16[R_SP] = self.state.r16[R_SP].wrapping_add(1);
                (6, 1)
            }
            0x34 => {
                let addr = self.state.get_reg16(R_HL);
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (11, 1)
            }
            0x35 => {
                let addr = self.state.get_reg16(R_HL);
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (11, 1)
            }
            0x36 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                mmu.w8(self.state.get_reg16(R_HL), n);
                (10, 2)
            }
            0x37 => {
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV))
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3)
                    | F_C;
                (4, 1)
            }
            0x38 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16;
                    (12, 0)
                } else {
                    (7, 2)
                }
            }
            0x39 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.r16[R_SP], false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x3A => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.r8[R_A] = mmu.r8(nn);
                (13, 3)
            }
            0x3B => {
                self.state.r16[R_SP] = self.state.r16[R_SP].wrapping_sub(1);
                (6, 1)
            }
            0x3C => {
                let (res, flags) = self.add8(self.state.r8[R_A], 1, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x3D => {
                let (res, flags) = self.sub8(self.state.r8[R_A], 1, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x3E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(1));
                self.state.r8[R_A] = n;
                (7, 2)
            }
            0x3F => {
                let cf = self.state.r8[R_F] & F_C;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV))
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3)
                    | (cf << 4)
                    | (cf ^ F_C);
                (4, 1)
            }
            0x40 => {
                self.state.r8[2] = self.state.r8[2];
                (4, 1)
            }
            0x41 => {
                self.state.r8[2] = self.state.r8[3];
                (4, 1)
            }
            0x42 => {
                self.state.r8[2] = self.state.r8[4];
                (4, 1)
            }
            0x43 => {
                self.state.r8[2] = self.state.r8[5];
                (4, 1)
            }
            0x44 => {
                self.state.r8[2] = self.state.r8[6];
                (4, 1)
            }
            0x45 => {
                self.state.r8[2] = self.state.r8[7];
                (4, 1)
            }
            0x46 => {
                self.state.r8[2] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x47 => {
                self.state.r8[2] = self.state.r8[0];
                (4, 1)
            }
            0x48 => {
                self.state.r8[3] = self.state.r8[2];
                (4, 1)
            }
            0x49 => {
                self.state.r8[3] = self.state.r8[3];
                (4, 1)
            }
            0x4A => {
                self.state.r8[3] = self.state.r8[4];
                (4, 1)
            }
            0x4B => {
                self.state.r8[3] = self.state.r8[5];
                (4, 1)
            }
            0x4C => {
                self.state.r8[3] = self.state.r8[6];
                (4, 1)
            }
            0x4D => {
                self.state.r8[3] = self.state.r8[7];
                (4, 1)
            }
            0x4E => {
                self.state.r8[3] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x4F => {
                self.state.r8[3] = self.state.r8[0];
                (4, 1)
            }
            0x50 => {
                self.state.r8[4] = self.state.r8[2];
                (4, 1)
            }
            0x51 => {
                self.state.r8[4] = self.state.r8[3];
                (4, 1)
            }
            0x52 => {
                self.state.r8[4] = self.state.r8[4];
                (4, 1)
            }
            0x53 => {
                self.state.r8[4] = self.state.r8[5];
                (4, 1)
            }
            0x54 => {
                self.state.r8[4] = self.state.r8[6];
                (4, 1)
            }
            0x55 => {
                self.state.r8[4] = self.state.r8[7];
                (4, 1)
            }
            0x56 => {
                self.state.r8[4] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x57 => {
                self.state.r8[4] = self.state.r8[0];
                (4, 1)
            }
            0x58 => {
                self.state.r8[5] = self.state.r8[2];
                (4, 1)
            }
            0x59 => {
                self.state.r8[5] = self.state.r8[3];
                (4, 1)
            }
            0x5A => {
                self.state.r8[5] = self.state.r8[4];
                (4, 1)
            }
            0x5B => {
                self.state.r8[5] = self.state.r8[5];
                (4, 1)
            }
            0x5C => {
                self.state.r8[5] = self.state.r8[6];
                (4, 1)
            }
            0x5D => {
                self.state.r8[5] = self.state.r8[7];
                (4, 1)
            }
            0x5E => {
                self.state.r8[5] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x5F => {
                self.state.r8[5] = self.state.r8[0];
                (4, 1)
            }
            0x60 => {
                self.state.r8[6] = self.state.r8[2];
                (4, 1)
            }
            0x61 => {
                self.state.r8[6] = self.state.r8[3];
                (4, 1)
            }
            0x62 => {
                self.state.r8[6] = self.state.r8[4];
                (4, 1)
            }
            0x63 => {
                self.state.r8[6] = self.state.r8[5];
                (4, 1)
            }
            0x64 => {
                self.state.r8[6] = self.state.r8[6];
                (4, 1)
            }
            0x65 => {
                self.state.r8[6] = self.state.r8[7];
                (4, 1)
            }
            0x66 => {
                self.state.r8[6] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x67 => {
                self.state.r8[6] = self.state.r8[0];
                (4, 1)
            }
            0x68 => {
                self.state.r8[7] = self.state.r8[2];
                (4, 1)
            }
            0x69 => {
                self.state.r8[7] = self.state.r8[3];
                (4, 1)
            }
            0x6A => {
                self.state.r8[7] = self.state.r8[4];
                (4, 1)
            }
            0x6B => {
                self.state.r8[7] = self.state.r8[5];
                (4, 1)
            }
            0x6C => {
                self.state.r8[7] = self.state.r8[6];
                (4, 1)
            }
            0x6D => {
                self.state.r8[7] = self.state.r8[7];
                (4, 1)
            }
            0x6E => {
                self.state.r8[7] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x6F => {
                self.state.r8[7] = self.state.r8[0];
                (4, 1)
            }
            0x70 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[2]);
                (7, 1)
            }
            0x71 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[3]);
                (7, 1)
            }
            0x72 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[4]);
                (7, 1)
            }
            0x73 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[5]);
                (7, 1)
            }
            0x74 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[6]);
                (7, 1)
            }
            0x75 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[7]);
                (7, 1)
            }
            0x76 => {
                self.state.halted = 1;
                (4, 1)
            }
            0x77 => {
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[0]);
                (7, 1)
            }
            0x78 => {
                self.state.r8[0] = self.state.r8[2];
                (4, 1)
            }
            0x79 => {
                self.state.r8[0] = self.state.r8[3];
                (4, 1)
            }
            0x7A => {
                self.state.r8[0] = self.state.r8[4];
                (4, 1)
            }
            0x7B => {
                self.state.r8[0] = self.state.r8[5];
                (4, 1)
            }
            0x7C => {
                self.state.r8[0] = self.state.r8[6];
                (4, 1)
            }
            0x7D => {
                self.state.r8[0] = self.state.r8[7];
                (4, 1)
            }
            0x7E => {
                self.state.r8[0] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x7F => {
                self.state.r8[0] = self.state.r8[0];
                (4, 1)
            }
            0x80 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x81 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x82 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x83 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x84 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x85 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x86 => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) = self.add8(self.state.r8[R_A], val, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x87 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x88 => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[2],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x89 => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[3],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x8A => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[4],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x8B => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[5],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x8C => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[6],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x8D => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[7],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x8E => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) =
                    self.add8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x8F => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[0],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x90 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x91 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x92 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x93 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x94 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x95 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x96 => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x97 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x98 => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[2],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x99 => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[3],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x9A => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[4],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x9B => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[5],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x9C => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[6],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x9D => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[7],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x9E => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) =
                    self.sub8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x9F => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[0],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0xA0 => {
                self.state.r8[R_A] &= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA1 => {
                self.state.r8[R_A] &= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA2 => {
                self.state.r8[R_A] &= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA3 => {
                self.state.r8[R_A] &= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA4 => {
                self.state.r8[R_A] &= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA5 => {
                self.state.r8[R_A] &= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA6 => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] &= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (7, 1)
            }
            0xA7 => {
                self.state.r8[R_A] &= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA8 => {
                self.state.r8[R_A] ^= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xA9 => {
                self.state.r8[R_A] ^= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAA => {
                self.state.r8[R_A] ^= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAB => {
                self.state.r8[R_A] ^= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAC => {
                self.state.r8[R_A] ^= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAD => {
                self.state.r8[R_A] ^= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAE => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] ^= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 1)
            }
            0xAF => {
                self.state.r8[R_A] ^= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB0 => {
                self.state.r8[R_A] |= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB1 => {
                self.state.r8[R_A] |= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB2 => {
                self.state.r8[R_A] |= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB3 => {
                self.state.r8[R_A] |= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB4 => {
                self.state.r8[R_A] |= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB5 => {
                self.state.r8[R_A] |= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB6 => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] |= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 1)
            }
            0xB7 => {
                self.state.r8[R_A] |= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB8 => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[2] & (F_5 | F_3));
                (4, 1)
            }
            0xB9 => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[3] & (F_5 | F_3));
                (4, 1)
            }
            0xBA => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[4] & (F_5 | F_3));
                (4, 1)
            }
            0xBB => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[5] & (F_5 | F_3));
                (4, 1)
            }
            0xBC => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[6] & (F_5 | F_3));
                (4, 1)
            }
            0xBD => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[7] & (F_5 | F_3));
                (4, 1)
            }
            0xBE => {
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (7, 1)
            }
            0xBF => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[0] & (F_5 | F_3));
                (4, 1)
            }
            0xC0 => {
                if self.state.r8[R_F] & F_Z != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                }
            }
            0xC1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(R_BC, val);
                (10, 1)
            }
            0xC2 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                }
            }
            0xC3 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.r16[R_PC] = nn;
                (10, 0)
            }
            0xC4 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                }
            }
            0xC5 => {
                self.push16(mmu, self.state.get_reg16(R_BC));
                (11, 1)
            }
            0xC6 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) = self.add8(self.state.r8[R_A], n, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xC7 => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x00;
                (11, 0)
            }
            0xC8 => {
                if self.state.r8[R_F] & F_Z != 0 {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xC9 => {
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (10, 0)
            }
            0xCA => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xCB => (4, 1),
            0xCC => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_Z != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xCD => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(1));
                self.push16(mmu, pc.wrapping_add(3));
                self.state.r16[R_PC] = nn;
                (17, 0)
            }
            0xCE => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) =
                    self.add8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xCF => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x08;
                (11, 0)
            }
            0xD0 => {
                if self.state.r8[R_F] & F_C != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                }
            }
            0xD1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(R_DE, val);
                (10, 1)
            }
            0xD2 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                }
            }
            0xD3 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                mmu.out8(n, self.state.r8[R_A], self.state.r8[R_A]);
                (11, 2)
            }
            0xD4 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                }
            }
            0xD5 => {
                self.push16(mmu, self.state.get_reg16(R_DE));
                (11, 1)
            }
            0xD6 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) = self.sub8(self.state.r8[R_A], n, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xD7 => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x10;
                (11, 0)
            }
            0xD8 => {
                if self.state.r8[R_F] & F_C != 0 {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xD9 => {
                let b = self.state.r8[R_B];
                self.state.r8[R_B] = self.state.r8[R_BA];
                self.state.r8[R_BA] = b;
                let c = self.state.r8[R_C];
                self.state.r8[R_C] = self.state.r8[R_CA];
                self.state.r8[R_CA] = c;
                let d = self.state.r8[R_D];
                self.state.r8[R_D] = self.state.r8[R_DA];
                self.state.r8[R_DA] = d;
                let e = self.state.r8[R_E];
                self.state.r8[R_E] = self.state.r8[R_EA];
                self.state.r8[R_EA] = e;
                let h = self.state.r8[R_H];
                self.state.r8[R_H] = self.state.r8[R_HA];
                self.state.r8[R_HA] = h;
                let l = self.state.r8[R_L];
                self.state.r8[R_L] = self.state.r8[R_LA];
                self.state.r8[R_LA] = l;
                (4, 1)
            }
            0xDA => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xDB => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] = mmu.in8(n, self.state.r8[R_A]);
                (11, 2)
            }
            0xDC => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_C != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xDD => (4, 1),
            0xDE => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) =
                    self.sub8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xDF => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x18;
                (11, 0)
            }
            0xE0 => {
                if self.state.r8[R_F] & F_PV != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                }
            }
            0xE1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(R_HL, val);
                (10, 1)
            }
            0xE2 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_PV != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                }
            }
            0xE3 => {
                let sp = self.state.r16[R_SP];
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(R_HL));
                self.state.set_reg16(R_HL, memval);
                (19, 1)
            }
            0xE4 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_PV != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                }
            }
            0xE5 => {
                self.push16(mmu, self.state.get_reg16(R_HL));
                (11, 1)
            }
            0xE6 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] &= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (7, 2)
            }
            0xE7 => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x20;
                (11, 0)
            }
            0xE8 => {
                if self.state.r8[R_F] & F_PV != 0 {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xE9 => {
                self.state.r16[R_PC] = self.state.get_reg16(R_HL);
                (4, 0)
            }
            0xEA => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_PV != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xEB => {
                let de = self.state.get_reg16(R_DE);
                self.state.set_reg16(R_DE, self.state.get_reg16(R_HL));
                self.state.set_reg16(R_HL, de);
                (4, 1)
            }
            0xEC => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_PV != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xED => (4, 1),
            0xEE => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] ^= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 2)
            }
            0xEF => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x28;
                (11, 0)
            }
            0xF0 => {
                if self.state.r8[R_F] & F_S != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                }
            }
            0xF1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(R_AF, val);
                (10, 1)
            }
            0xF2 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_S != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                }
            }
            0xF3 => {
                self.state.iff1 = 0;
                self.state.iff2 = 0;
                (4, 1)
            }
            0xF4 => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_S != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                }
            }
            0xF5 => {
                self.push16(mmu, self.state.get_reg16(R_AF));
                (11, 1)
            }
            0xF6 => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] |= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 2)
            }
            0xF7 => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x30;
                (11, 0)
            }
            0xF8 => {
                if self.state.r8[R_F] & F_S != 0 {
                    let addr = self.pop16(mmu);
                    self.state.r16[R_PC] = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xF9 => {
                self.state.r16[R_SP] = self.state.get_reg16(R_HL);
                (6, 1)
            }
            0xFA => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_S != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.r16[R_PC] = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xFB => {
                self.state.iff1 = 1;
                self.state.iff2 = 1;
                (4, 1)
            }
            0xFC => {
                let pc = self.state.r16[R_PC];
                if self.state.r8[R_F] & F_S != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.r16[R_PC] = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xFD => (4, 1),
            0xFE => {
                let pc = self.state.r16[R_PC];
                let n = mmu.r8(pc.wrapping_add(1));
                let (_, flags) = self.sub8(self.state.r8[R_A], n, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (n & (F_5 | F_3));
                (7, 2)
            }
            0xFF => {
                let pc = self.state.r16[R_PC];
                self.push16(mmu, pc.wrapping_add(1));
                self.state.r16[R_PC] = 0x38;
                (11, 0)
            }
        }
    }
    pub fn execute_cb<M: CpuBus>(&mut self, opcode: u8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x00 => {
                let (res, flags) = self.shl8(self.state.r8[2], (self.state.r8[2] & 0x80) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x01 => {
                let (res, flags) = self.shl8(self.state.r8[3], (self.state.r8[3] & 0x80) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x02 => {
                let (res, flags) = self.shl8(self.state.r8[4], (self.state.r8[4] & 0x80) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x03 => {
                let (res, flags) = self.shl8(self.state.r8[5], (self.state.r8[5] & 0x80) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x04 => {
                let (res, flags) = self.shl8(self.state.r8[6], (self.state.r8[6] & 0x80) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x05 => {
                let (res, flags) = self.shl8(self.state.r8[7], (self.state.r8[7] & 0x80) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x06 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x07 => {
                let (res, flags) = self.shl8(self.state.r8[0], (self.state.r8[0] & 0x80) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x08 => {
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[2] & 0x01) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x09 => {
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[3] & 0x01) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0A => {
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[4] & 0x01) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0B => {
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[5] & 0x01) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0C => {
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[6] & 0x01) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0D => {
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[7] & 0x01) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x0F => {
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[0] & 0x01) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x10 => {
                let (res, flags) = self.shl8(self.state.r8[2], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x11 => {
                let (res, flags) = self.shl8(self.state.r8[3], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x12 => {
                let (res, flags) = self.shl8(self.state.r8[4], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x13 => {
                let (res, flags) = self.shl8(self.state.r8[5], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x14 => {
                let (res, flags) = self.shl8(self.state.r8[6], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x15 => {
                let (res, flags) = self.shl8(self.state.r8[7], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x16 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x17 => {
                let (res, flags) = self.shl8(self.state.r8[0], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x18 => {
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x19 => {
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1A => {
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1B => {
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1C => {
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1D => {
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x1F => {
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x20 => {
                let (res, flags) = self.shl8(self.state.r8[2], false);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x21 => {
                let (res, flags) = self.shl8(self.state.r8[3], false);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x22 => {
                let (res, flags) = self.shl8(self.state.r8[4], false);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x23 => {
                let (res, flags) = self.shl8(self.state.r8[5], false);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x24 => {
                let (res, flags) = self.shl8(self.state.r8[6], false);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x25 => {
                let (res, flags) = self.shl8(self.state.r8[7], false);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x26 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x27 => {
                let (res, flags) = self.shl8(self.state.r8[0], false);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x28 => {
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[2] & 0x80) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x29 => {
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[3] & 0x80) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2A => {
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[4] & 0x80) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2B => {
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[5] & 0x80) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2C => {
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[6] & 0x80) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2D => {
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[7] & 0x80) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x2F => {
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[0] & 0x80) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x30 => {
                let (res, flags) = self.shl8(self.state.r8[2], true);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x31 => {
                let (res, flags) = self.shl8(self.state.r8[3], true);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x32 => {
                let (res, flags) = self.shl8(self.state.r8[4], true);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x33 => {
                let (res, flags) = self.shl8(self.state.r8[5], true);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x34 => {
                let (res, flags) = self.shl8(self.state.r8[6], true);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x35 => {
                let (res, flags) = self.shl8(self.state.r8[7], true);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x36 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x37 => {
                let (res, flags) = self.shl8(self.state.r8[0], true);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x38 => {
                let (res, flags) = self.shr8(self.state.r8[2], false);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x39 => {
                let (res, flags) = self.shr8(self.state.r8[3], false);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3A => {
                let (res, flags) = self.shr8(self.state.r8[4], false);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3B => {
                let (res, flags) = self.shr8(self.state.r8[5], false);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3C => {
                let (res, flags) = self.shr8(self.state.r8[6], false);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3D => {
                let (res, flags) = self.shr8(self.state.r8[7], false);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x3F => {
                let (res, flags) = self.shr8(self.state.r8[0], false);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x40 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x41 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x42 => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x43 => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x44 => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x45 => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x46 => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x47 => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x48 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x49 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x4A => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x4B => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x4C => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x4D => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x4E => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x4F => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x50 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x51 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x52 => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x53 => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x54 => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x55 => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x56 => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x57 => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x58 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x59 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x5A => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x5B => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x5C => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x5D => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x5E => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x5F => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x60 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x61 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x62 => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x63 => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x64 => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x65 => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x66 => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x67 => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x68 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x69 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x6A => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x6B => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x6C => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x6D => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x6E => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x6F => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x70 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x71 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x72 => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x73 => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x74 => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x75 => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x76 => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x77 => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x78 => {
                let srcval = self.state.r8[2];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x79 => {
                let srcval = self.state.r8[3];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x7A => {
                let srcval = self.state.r8[4];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x7B => {
                let srcval = self.state.r8[5];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x7C => {
                let srcval = self.state.r8[6];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x7D => {
                let srcval = self.state.r8[7];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x7E => {
                let addr = self.state.get_reg16(R_HL);
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (12, 2)
            }
            0x7F => {
                let srcval = self.state.r8[0];
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | (srcval & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x80 => {
                self.state.r8[2] &= 0xFE;
                (8, 2)
            }
            0x81 => {
                self.state.r8[3] &= 0xFE;
                (8, 2)
            }
            0x82 => {
                self.state.r8[4] &= 0xFE;
                (8, 2)
            }
            0x83 => {
                self.state.r8[5] &= 0xFE;
                (8, 2)
            }
            0x84 => {
                self.state.r8[6] &= 0xFE;
                (8, 2)
            }
            0x85 => {
                self.state.r8[7] &= 0xFE;
                (8, 2)
            }
            0x86 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x87 => {
                self.state.r8[0] &= 0xFE;
                (8, 2)
            }
            0x88 => {
                self.state.r8[2] &= 0xFD;
                (8, 2)
            }
            0x89 => {
                self.state.r8[3] &= 0xFD;
                (8, 2)
            }
            0x8A => {
                self.state.r8[4] &= 0xFD;
                (8, 2)
            }
            0x8B => {
                self.state.r8[5] &= 0xFD;
                (8, 2)
            }
            0x8C => {
                self.state.r8[6] &= 0xFD;
                (8, 2)
            }
            0x8D => {
                self.state.r8[7] &= 0xFD;
                (8, 2)
            }
            0x8E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x8F => {
                self.state.r8[0] &= 0xFD;
                (8, 2)
            }
            0x90 => {
                self.state.r8[2] &= 0xFB;
                (8, 2)
            }
            0x91 => {
                self.state.r8[3] &= 0xFB;
                (8, 2)
            }
            0x92 => {
                self.state.r8[4] &= 0xFB;
                (8, 2)
            }
            0x93 => {
                self.state.r8[5] &= 0xFB;
                (8, 2)
            }
            0x94 => {
                self.state.r8[6] &= 0xFB;
                (8, 2)
            }
            0x95 => {
                self.state.r8[7] &= 0xFB;
                (8, 2)
            }
            0x96 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x97 => {
                self.state.r8[0] &= 0xFB;
                (8, 2)
            }
            0x98 => {
                self.state.r8[2] &= 0xF7;
                (8, 2)
            }
            0x99 => {
                self.state.r8[3] &= 0xF7;
                (8, 2)
            }
            0x9A => {
                self.state.r8[4] &= 0xF7;
                (8, 2)
            }
            0x9B => {
                self.state.r8[5] &= 0xF7;
                (8, 2)
            }
            0x9C => {
                self.state.r8[6] &= 0xF7;
                (8, 2)
            }
            0x9D => {
                self.state.r8[7] &= 0xF7;
                (8, 2)
            }
            0x9E => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x9F => {
                self.state.r8[0] &= 0xF7;
                (8, 2)
            }
            0xA0 => {
                self.state.r8[2] &= 0xEF;
                (8, 2)
            }
            0xA1 => {
                self.state.r8[3] &= 0xEF;
                (8, 2)
            }
            0xA2 => {
                self.state.r8[4] &= 0xEF;
                (8, 2)
            }
            0xA3 => {
                self.state.r8[5] &= 0xEF;
                (8, 2)
            }
            0xA4 => {
                self.state.r8[6] &= 0xEF;
                (8, 2)
            }
            0xA5 => {
                self.state.r8[7] &= 0xEF;
                (8, 2)
            }
            0xA6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xA7 => {
                self.state.r8[0] &= 0xEF;
                (8, 2)
            }
            0xA8 => {
                self.state.r8[2] &= 0xDF;
                (8, 2)
            }
            0xA9 => {
                self.state.r8[3] &= 0xDF;
                (8, 2)
            }
            0xAA => {
                self.state.r8[4] &= 0xDF;
                (8, 2)
            }
            0xAB => {
                self.state.r8[5] &= 0xDF;
                (8, 2)
            }
            0xAC => {
                self.state.r8[6] &= 0xDF;
                (8, 2)
            }
            0xAD => {
                self.state.r8[7] &= 0xDF;
                (8, 2)
            }
            0xAE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xAF => {
                self.state.r8[0] &= 0xDF;
                (8, 2)
            }
            0xB0 => {
                self.state.r8[2] &= 0xBF;
                (8, 2)
            }
            0xB1 => {
                self.state.r8[3] &= 0xBF;
                (8, 2)
            }
            0xB2 => {
                self.state.r8[4] &= 0xBF;
                (8, 2)
            }
            0xB3 => {
                self.state.r8[5] &= 0xBF;
                (8, 2)
            }
            0xB4 => {
                self.state.r8[6] &= 0xBF;
                (8, 2)
            }
            0xB5 => {
                self.state.r8[7] &= 0xBF;
                (8, 2)
            }
            0xB6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xB7 => {
                self.state.r8[0] &= 0xBF;
                (8, 2)
            }
            0xB8 => {
                self.state.r8[2] &= 0x7F;
                (8, 2)
            }
            0xB9 => {
                self.state.r8[3] &= 0x7F;
                (8, 2)
            }
            0xBA => {
                self.state.r8[4] &= 0x7F;
                (8, 2)
            }
            0xBB => {
                self.state.r8[5] &= 0x7F;
                (8, 2)
            }
            0xBC => {
                self.state.r8[6] &= 0x7F;
                (8, 2)
            }
            0xBD => {
                self.state.r8[7] &= 0x7F;
                (8, 2)
            }
            0xBE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xBF => {
                self.state.r8[0] &= 0x7F;
                (8, 2)
            }
            0xC0 => {
                self.state.r8[2] |= 0x01;
                (8, 2)
            }
            0xC1 => {
                self.state.r8[3] |= 0x01;
                (8, 2)
            }
            0xC2 => {
                self.state.r8[4] |= 0x01;
                (8, 2)
            }
            0xC3 => {
                self.state.r8[5] |= 0x01;
                (8, 2)
            }
            0xC4 => {
                self.state.r8[6] |= 0x01;
                (8, 2)
            }
            0xC5 => {
                self.state.r8[7] |= 0x01;
                (8, 2)
            }
            0xC6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xC7 => {
                self.state.r8[0] |= 0x01;
                (8, 2)
            }
            0xC8 => {
                self.state.r8[2] |= 0x02;
                (8, 2)
            }
            0xC9 => {
                self.state.r8[3] |= 0x02;
                (8, 2)
            }
            0xCA => {
                self.state.r8[4] |= 0x02;
                (8, 2)
            }
            0xCB => {
                self.state.r8[5] |= 0x02;
                (8, 2)
            }
            0xCC => {
                self.state.r8[6] |= 0x02;
                (8, 2)
            }
            0xCD => {
                self.state.r8[7] |= 0x02;
                (8, 2)
            }
            0xCE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xCF => {
                self.state.r8[0] |= 0x02;
                (8, 2)
            }
            0xD0 => {
                self.state.r8[2] |= 0x04;
                (8, 2)
            }
            0xD1 => {
                self.state.r8[3] |= 0x04;
                (8, 2)
            }
            0xD2 => {
                self.state.r8[4] |= 0x04;
                (8, 2)
            }
            0xD3 => {
                self.state.r8[5] |= 0x04;
                (8, 2)
            }
            0xD4 => {
                self.state.r8[6] |= 0x04;
                (8, 2)
            }
            0xD5 => {
                self.state.r8[7] |= 0x04;
                (8, 2)
            }
            0xD6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xD7 => {
                self.state.r8[0] |= 0x04;
                (8, 2)
            }
            0xD8 => {
                self.state.r8[2] |= 0x08;
                (8, 2)
            }
            0xD9 => {
                self.state.r8[3] |= 0x08;
                (8, 2)
            }
            0xDA => {
                self.state.r8[4] |= 0x08;
                (8, 2)
            }
            0xDB => {
                self.state.r8[5] |= 0x08;
                (8, 2)
            }
            0xDC => {
                self.state.r8[6] |= 0x08;
                (8, 2)
            }
            0xDD => {
                self.state.r8[7] |= 0x08;
                (8, 2)
            }
            0xDE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xDF => {
                self.state.r8[0] |= 0x08;
                (8, 2)
            }
            0xE0 => {
                self.state.r8[2] |= 0x10;
                (8, 2)
            }
            0xE1 => {
                self.state.r8[3] |= 0x10;
                (8, 2)
            }
            0xE2 => {
                self.state.r8[4] |= 0x10;
                (8, 2)
            }
            0xE3 => {
                self.state.r8[5] |= 0x10;
                (8, 2)
            }
            0xE4 => {
                self.state.r8[6] |= 0x10;
                (8, 2)
            }
            0xE5 => {
                self.state.r8[7] |= 0x10;
                (8, 2)
            }
            0xE6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xE7 => {
                self.state.r8[0] |= 0x10;
                (8, 2)
            }
            0xE8 => {
                self.state.r8[2] |= 0x20;
                (8, 2)
            }
            0xE9 => {
                self.state.r8[3] |= 0x20;
                (8, 2)
            }
            0xEA => {
                self.state.r8[4] |= 0x20;
                (8, 2)
            }
            0xEB => {
                self.state.r8[5] |= 0x20;
                (8, 2)
            }
            0xEC => {
                self.state.r8[6] |= 0x20;
                (8, 2)
            }
            0xED => {
                self.state.r8[7] |= 0x20;
                (8, 2)
            }
            0xEE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xEF => {
                self.state.r8[0] |= 0x20;
                (8, 2)
            }
            0xF0 => {
                self.state.r8[2] |= 0x40;
                (8, 2)
            }
            0xF1 => {
                self.state.r8[3] |= 0x40;
                (8, 2)
            }
            0xF2 => {
                self.state.r8[4] |= 0x40;
                (8, 2)
            }
            0xF3 => {
                self.state.r8[5] |= 0x40;
                (8, 2)
            }
            0xF4 => {
                self.state.r8[6] |= 0x40;
                (8, 2)
            }
            0xF5 => {
                self.state.r8[7] |= 0x40;
                (8, 2)
            }
            0xF6 => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xF7 => {
                self.state.r8[0] |= 0x40;
                (8, 2)
            }
            0xF8 => {
                self.state.r8[2] |= 0x80;
                (8, 2)
            }
            0xF9 => {
                self.state.r8[3] |= 0x80;
                (8, 2)
            }
            0xFA => {
                self.state.r8[4] |= 0x80;
                (8, 2)
            }
            0xFB => {
                self.state.r8[5] |= 0x80;
                (8, 2)
            }
            0xFC => {
                self.state.r8[6] |= 0x80;
                (8, 2)
            }
            0xFD => {
                self.state.r8[7] |= 0x80;
                (8, 2)
            }
            0xFE => {
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xFF => {
                self.state.r8[0] |= 0x80;
                (8, 2)
            }
        }
    }
    pub fn execute_ed<M: CpuBus>(&mut self, opcode: u8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x40 => {
                self.state.r8[2] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[2] as usize];
                (12, 2)
            }
            0x41 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[2], self.state.r8[R_B]);
                (12, 2)
            }
            0x48 => {
                self.state.r8[3] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[3] as usize];
                (12, 2)
            }
            0x49 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[3], self.state.r8[R_B]);
                (12, 2)
            }
            0x50 => {
                self.state.r8[4] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[4] as usize];
                (12, 2)
            }
            0x51 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[4], self.state.r8[R_B]);
                (12, 2)
            }
            0x58 => {
                self.state.r8[5] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[5] as usize];
                (12, 2)
            }
            0x59 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[5], self.state.r8[R_B]);
                (12, 2)
            }
            0x60 => {
                self.state.r8[6] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[6] as usize];
                (12, 2)
            }
            0x61 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[6], self.state.r8[R_B]);
                (12, 2)
            }
            0x68 => {
                self.state.r8[7] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[7] as usize];
                (12, 2)
            }
            0x69 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[7], self.state.r8[R_B]);
                (12, 2)
            }
            0x70 => {
                let val = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C) | self.sz53p_table[val as usize];
                (12, 2)
            }
            0x71 => {
                mmu.out8(self.state.r8[R_C], 0, self.state.r8[R_B]);
                (12, 2)
            }
            0x78 => {
                self.state.r8[0] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[0] as usize];
                (12, 2)
            }
            0x79 => {
                mmu.out8(self.state.r8[R_C], self.state.r8[0], self.state.r8[R_B]);
                (12, 2)
            }
            0x42 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(1), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x43 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(1));
                (20, 4)
            }
            0x4A => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(1), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x4B => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(1, val);
                (20, 4)
            }
            0x52 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(2), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x53 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(2));
                (20, 4)
            }
            0x5A => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(2), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x5B => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(2, val);
                (20, 4)
            }
            0x62 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(3), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x63 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(3));
                (20, 4)
            }
            0x6A => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(3), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x6B => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(3, val);
                (20, 4)
            }
            0x72 => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.r16[R_SP], (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x73 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.r16[R_SP]);
                (20, 4)
            }
            0x7A => {
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.r16[R_SP], (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x7B => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.r16[R_SP] = val;
                (20, 4)
            }
            0x44 => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x4C => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x54 => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x5C => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x64 => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x6C => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x74 => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x7C => {
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x45 => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x55 => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x65 => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x75 => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x5D => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x6D => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x7D => {
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x4D => {
                let addr = self.pop16(mmu);
                self.state.r16[R_PC] = addr;
                (14, 0)
            }
            0x46 => {
                self.state.im = 0;
                (8, 2)
            }
            0x4E => {
                self.state.im = 0;
                (8, 2)
            }
            0x66 => {
                self.state.im = 0;
                (8, 2)
            }
            0x6E => {
                self.state.im = 0;
                (8, 2)
            }
            0x56 => {
                self.state.im = 1;
                (8, 2)
            }
            0x76 => {
                self.state.im = 1;
                (8, 2)
            }
            0x5E => {
                self.state.im = 2;
                (8, 2)
            }
            0x7E => {
                self.state.im = 2;
                (8, 2)
            }
            0x47 => {
                self.state.r8[R_I] = self.state.r8[R_A];
                (9, 2)
            }
            0x4F => {
                self.state.r8[R_R] = self.state.r8[R_A];
                (9, 2)
            }
            0x57 => {
                self.state.r8[R_A] = self.state.r8[R_I];
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C)
                    | self.sz53_table[self.state.r8[R_A] as usize]
                    | (if self.state.iff2 != 0 { F_PV } else { 0 });
                (9, 2)
            }
            0x5F => {
                self.state.r8[R_A] = self.state.r8[R_R];
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C)
                    | self.sz53_table[self.state.r8[R_A] as usize]
                    | (if self.state.iff2 != 0 { F_PV } else { 0 });
                (9, 2)
            }
            0x67 => {
                let addr = self.state.get_reg16(R_HL);
                let memval = mmu.r8(addr);
                mmu.w8(addr, ((self.state.r8[R_A] & 0x0F) << 4) | (memval >> 4));
                self.state.r8[R_A] = (self.state.r8[R_A] & 0xF0) | (memval & 0x0F);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[R_A] as usize];
                (18, 2)
            }
            0x6F => {
                let addr = self.state.get_reg16(R_HL);
                let memval = mmu.r8(addr);
                mmu.w8(addr, ((memval & 0x0F) << 4) | (self.state.r8[R_A] & 0x0F));
                self.state.r8[R_A] = (self.state.r8[R_A] & 0xF0) | (memval >> 4);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[R_A] as usize];
                (18, 2)
            }
            0x77 => (8, 2),
            0x7F => (8, 2),
            0xA0 => {
                let de = self.state.get_reg16(R_DE);
                let hl = self.state.get_reg16(R_HL);
                let bc = self.state.get_reg16(R_BC);
                let memval = mmu.r8(hl);
                mmu.w8(de, memval);
                let de = de.wrapping_add(1);
                let hl = hl.wrapping_add(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_DE, de);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let memval = (memval.wrapping_add(self.state.r8[R_A])) & 0xFF;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_C))
                    | (if bc != 0 { F_PV } else { 0 })
                    | (memval & F_3)
                    | (if (memval & 0x02) != 0 { F_5 } else { 0 });
                (16, 2)
            }
            0xA1 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let (_, flags) = self.sub8(self.state.r8[R_A], memval, false);
                let hl = hl.wrapping_add(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let tmp = (self.state.r8[R_A]
                    .wrapping_sub(memval)
                    .wrapping_sub(if (flags & F_H) != 0 { 1 } else { 0 }))
                    & 0xFF;
                self.state.r8[R_F] = F_N
                    | (self.state.r8[R_F] & F_C)
                    | (flags & (F_S | F_Z | F_H))
                    | (tmp & F_3)
                    | (if (tmp & 0x02) != 0 { F_5 } else { 0 })
                    | (if bc != 0 { F_PV } else { 0 });
                (16, 2)
            }
            0xA2 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let regval = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                mmu.w8(hl, regval);
                let hl = hl.wrapping_add(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                let tmp = (regval as u16).wrapping_add((bc.wrapping_add(1)) & 0xFF as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (regval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((regval.wrapping_add((bc.wrapping_add(1) & 0xFF) as u8))
                        & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                (16, 2)
            }
            0xA3 => {
                //let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let hl = hl.wrapping_add(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                mmu.out8(self.state.r8[R_C], memval, self.state.r8[R_B]);
                let tmp = (memval as u16).wrapping_add(self.state.r8[R_L] as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (memval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((memval.wrapping_add(self.state.r8[R_L])) & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                (16, 2)
            }
            0xA8 => {
                let de = self.state.get_reg16(R_DE);
                let hl = self.state.get_reg16(R_HL);
                let bc = self.state.get_reg16(R_BC);
                let memval = mmu.r8(hl);
                mmu.w8(de, memval);
                let de = de.wrapping_sub(1);
                let hl = hl.wrapping_sub(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_DE, de);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let memval = (memval.wrapping_add(self.state.r8[R_A])) & 0xFF;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_C))
                    | (if bc != 0 { F_PV } else { 0 })
                    | (memval & F_3)
                    | (if (memval & 0x02) != 0 { F_5 } else { 0 });
                (16, 2)
            }
            0xA9 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let (_, flags) = self.sub8(self.state.r8[R_A], memval, false);
                let hl = hl.wrapping_sub(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let tmp = (self.state.r8[R_A]
                    .wrapping_sub(memval)
                    .wrapping_sub(if (flags & F_H) != 0 { 1 } else { 0 }))
                    & 0xFF;
                self.state.r8[R_F] = F_N
                    | (self.state.r8[R_F] & F_C)
                    | (flags & (F_S | F_Z | F_H))
                    | (tmp & F_3)
                    | (if (tmp & 0x02) != 0 { F_5 } else { 0 })
                    | (if bc != 0 { F_PV } else { 0 });
                (16, 2)
            }
            0xAA => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let regval = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                mmu.w8(hl, regval);
                let hl = hl.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                let tmp = (regval as u16).wrapping_add((bc.wrapping_sub(1)) & 0xFF as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (regval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((regval.wrapping_add((bc.wrapping_sub(1) & 0xFF) as u8))
                        & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                (16, 2)
            }
            0xAB => {
                //let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let hl = hl.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                mmu.out8(self.state.r8[R_C], memval, self.state.r8[R_B]);
                let tmp = (memval as u16).wrapping_add(self.state.r8[R_L] as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (memval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((memval.wrapping_add(self.state.r8[R_L])) & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                (16, 2)
            }
            0xB0 => {
                let de = self.state.get_reg16(R_DE);
                let hl = self.state.get_reg16(R_HL);
                let bc = self.state.get_reg16(R_BC);
                let memval = mmu.r8(hl);
                mmu.w8(de, memval);
                let de = de.wrapping_add(1);
                let hl = hl.wrapping_add(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_DE, de);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let memval = (memval.wrapping_add(self.state.r8[R_A])) & 0xFF;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_C))
                    | (if bc != 0 { F_PV } else { 0 })
                    | (memval & F_3)
                    | (if (memval & 0x02) != 0 { F_5 } else { 0 });
                if self.state.get_reg16(R_BC) != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xB1 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let (_, flags) = self.sub8(self.state.r8[R_A], memval, false);
                let hl = hl.wrapping_add(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let tmp = (self.state.r8[R_A]
                    .wrapping_sub(memval)
                    .wrapping_sub(if (flags & F_H) != 0 { 1 } else { 0 }))
                    & 0xFF;
                self.state.r8[R_F] = F_N
                    | (self.state.r8[R_F] & F_C)
                    | (flags & (F_S | F_Z | F_H))
                    | (tmp & F_3)
                    | (if (tmp & 0x02) != 0 { F_5 } else { 0 })
                    | (if bc != 0 { F_PV } else { 0 });
                if self.state.get_reg16(R_BC) != 0 && (self.state.r8[R_F] & F_Z) == 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xB2 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let regval = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                mmu.w8(hl, regval);
                let hl = hl.wrapping_add(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                let tmp = (regval as u16).wrapping_add((bc.wrapping_add(1)) & 0xFF as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (regval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((regval.wrapping_add((bc.wrapping_add(1) & 0xFF) as u8))
                        & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                if self.state.r8[R_B] != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xB3 => {
                //let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let hl = hl.wrapping_add(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                mmu.out8(self.state.r8[R_C], memval, self.state.r8[R_B]);
                let tmp = (memval as u16).wrapping_add(self.state.r8[R_L] as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (memval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((memval.wrapping_add(self.state.r8[R_L])) & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                if self.state.r8[R_B] != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xB8 => {
                let de = self.state.get_reg16(R_DE);
                let hl = self.state.get_reg16(R_HL);
                let bc = self.state.get_reg16(R_BC);
                let memval = mmu.r8(hl);
                mmu.w8(de, memval);
                let de = de.wrapping_sub(1);
                let hl = hl.wrapping_sub(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_DE, de);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let memval = (memval.wrapping_add(self.state.r8[R_A])) & 0xFF;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_C))
                    | (if bc != 0 { F_PV } else { 0 })
                    | (memval & F_3)
                    | (if (memval & 0x02) != 0 { F_5 } else { 0 });
                if self.state.get_reg16(R_BC) != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xB9 => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let (_, flags) = self.sub8(self.state.r8[R_A], memval, false);
                let hl = hl.wrapping_sub(1);
                let bc = bc.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                self.state.set_reg16(R_BC, bc);
                let tmp = (self.state.r8[R_A]
                    .wrapping_sub(memval)
                    .wrapping_sub(if (flags & F_H) != 0 { 1 } else { 0 }))
                    & 0xFF;
                self.state.r8[R_F] = F_N
                    | (self.state.r8[R_F] & F_C)
                    | (flags & (F_S | F_Z | F_H))
                    | (tmp & F_3)
                    | (if (tmp & 0x02) != 0 { F_5 } else { 0 })
                    | (if bc != 0 { F_PV } else { 0 });
                if self.state.get_reg16(R_BC) != 0 && (self.state.r8[R_F] & F_Z) == 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xBA => {
                let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let regval = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                mmu.w8(hl, regval);
                let hl = hl.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                let tmp = (regval as u16).wrapping_add((bc.wrapping_sub(1)) & 0xFF as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (regval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((regval.wrapping_add((bc.wrapping_sub(1) & 0xFF) as u8))
                        & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                if self.state.r8[R_B] != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            0xBB => {
                //let bc = self.state.get_reg16(R_BC);
                let hl = self.state.get_reg16(R_HL);
                let memval = mmu.r8(hl);
                let hl = hl.wrapping_sub(1);
                self.state.set_reg16(R_HL, hl);
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                mmu.out8(self.state.r8[R_C], memval, self.state.r8[R_B]);
                let tmp = (memval as u16).wrapping_add(self.state.r8[R_L] as u16);
                self.state.r8[R_F] = (flags & (F_S | F_Z | F_5 | F_3))
                    | (if (memval & 0x80) != 0 { F_N } else { 0 })
                    | (if tmp > 0xFF { F_H | F_C } else { 0 })
                    | (self.sz53p_table[(((memval.wrapping_add(self.state.r8[R_L])) & 7)
                        ^ self.state.r8[R_B]) as usize]
                        & F_PV);
                if self.state.r8[R_B] != 0 {
                    (21, 0)
                } else {
                    (16, 2)
                }
            }
            _ => (0, 0),
        }
    }
    pub fn execute_dd<M: CpuBus>(&mut self, opcode: u8, _displ: i8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x09 => {
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x19 => {
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x29 => {
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, ix, false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x39 => {
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.r16[R_SP], false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x21 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(2));
                self.state.set_reg16(4, nn);
                (14, 4)
            }
            0x22 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(4));
                (20, 4)
            }
            0x2A => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                self.state.set_reg16(4, mmu.r16(nn));
                (20, 4)
            }
            0x23 => {
                self.state
                    .set_reg16(4, self.state.get_reg16(4).wrapping_add(1));
                (10, 2)
            }
            0x2B => {
                self.state
                    .set_reg16(4, self.state.get_reg16(4).wrapping_sub(1));
                (10, 2)
            }
            0x24 => {
                let (res, flags) = self.add8(self.state.r8[8], 1, false);
                self.state.r8[8] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x25 => {
                let (res, flags) = self.sub8(self.state.r8[8], 1, false);
                self.state.r8[8] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x26 => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(2));
                self.state.r8[8] = n;
                (11, 3)
            }
            0x2C => {
                let (res, flags) = self.add8(self.state.r8[9], 1, false);
                self.state.r8[9] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2D => {
                let (res, flags) = self.sub8(self.state.r8[9], 1, false);
                self.state.r8[9] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(2));
                self.state.r8[9] = n;
                (11, 3)
            }
            0x34 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x35 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x36 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(3));
                mmu.w8(addr, n);
                (19, 4)
            }
            0x44 => {
                self.state.r8[R_B] = self.state.r8[8];
                (8, 2)
            }
            0x45 => {
                self.state.r8[R_B] = self.state.r8[9];
                (8, 2)
            }
            0x46 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_B] = mmu.r8(addr);
                (19, 3)
            }
            0x4C => {
                self.state.r8[R_C] = self.state.r8[8];
                (8, 2)
            }
            0x4D => {
                self.state.r8[R_C] = self.state.r8[9];
                (8, 2)
            }
            0x4E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_C] = mmu.r8(addr);
                (19, 3)
            }
            0x54 => {
                self.state.r8[R_D] = self.state.r8[8];
                (8, 2)
            }
            0x55 => {
                self.state.r8[R_D] = self.state.r8[9];
                (8, 2)
            }
            0x56 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_D] = mmu.r8(addr);
                (19, 3)
            }
            0x5C => {
                self.state.r8[R_E] = self.state.r8[8];
                (8, 2)
            }
            0x5D => {
                self.state.r8[R_E] = self.state.r8[9];
                (8, 2)
            }
            0x5E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_E] = mmu.r8(addr);
                (19, 3)
            }
            0x66 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_H] = mmu.r8(addr);
                (19, 3)
            }
            0x6E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_L] = mmu.r8(addr);
                (19, 3)
            }
            0x7C => {
                self.state.r8[R_A] = self.state.r8[8];
                (8, 2)
            }
            0x7D => {
                self.state.r8[R_A] = self.state.r8[9];
                (8, 2)
            }
            0x7E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] = mmu.r8(addr);
                (19, 3)
            }
            0x60 => {
                self.state.r8[8] = self.state.r8[R_B];
                (8, 2)
            }
            0x61 => {
                self.state.r8[8] = self.state.r8[R_C];
                (8, 2)
            }
            0x62 => {
                self.state.r8[8] = self.state.r8[R_D];
                (8, 2)
            }
            0x63 => {
                self.state.r8[8] = self.state.r8[R_E];
                (8, 2)
            }
            0x64 => (8, 2),
            0x65 => {
                self.state.r8[8] = self.state.r8[9];
                (8, 2)
            }
            0x67 => {
                self.state.r8[8] = self.state.r8[R_A];
                (8, 2)
            }
            0x68 => {
                self.state.r8[9] = self.state.r8[R_B];
                (8, 2)
            }
            0x69 => {
                self.state.r8[9] = self.state.r8[R_C];
                (8, 2)
            }
            0x6A => {
                self.state.r8[9] = self.state.r8[R_D];
                (8, 2)
            }
            0x6B => {
                self.state.r8[9] = self.state.r8[R_E];
                (8, 2)
            }
            0x6C => {
                self.state.r8[9] = self.state.r8[8];
                (8, 2)
            }
            0x6D => (8, 2),
            0x6F => {
                self.state.r8[9] = self.state.r8[R_A];
                (8, 2)
            }
            0x70 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[2]);
                (19, 3)
            }
            0x71 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[3]);
                (19, 3)
            }
            0x72 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[4]);
                (19, 3)
            }
            0x73 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[5]);
                (19, 3)
            }
            0x74 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[6]);
                (19, 3)
            }
            0x75 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[7]);
                (19, 3)
            }
            0x77 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[0]);
                (19, 3)
            }
            0x84 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x85 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x86 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x8C => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[8],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x8D => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[9],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x8E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    mmu.r8(addr),
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x94 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x95 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x96 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x9C => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[8],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x9D => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[9],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x9E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    mmu.r8(addr),
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0xA4 => {
                self.state.r8[R_A] &= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA5 => {
                self.state.r8[R_A] &= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA6 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] &= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (19, 3)
            }
            0xAC => {
                self.state.r8[R_A] ^= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAD => {
                self.state.r8[R_A] ^= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAE => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] ^= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xB4 => {
                self.state.r8[R_A] |= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB5 => {
                self.state.r8[R_A] |= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB6 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] |= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xBC => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[8] & (F_5 | F_3));
                (8, 2)
            }
            0xBD => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[9] & (F_5 | F_3));
                (8, 2)
            }
            0xBE => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let val = mmu.r8(addr);
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (19, 3)
            }
            0xE1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(4, val);
                (14, 2)
            }
            0xE3 => {
                let sp = self.state.r16[R_SP];
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(4));
                self.state.set_reg16(4, memval);
                (23, 2)
            }
            0xE5 => {
                self.push16(mmu, self.state.get_reg16(4));
                (15, 2)
            }
            0xE9 => {
                self.state.r16[R_PC] = self.state.get_reg16(4);
                (8, 0)
            }
            0xF9 => {
                self.state.r16[R_SP] = self.state.get_reg16(4);
                (10, 2)
            }
            _ => (0, 0),
        }
    }
    pub fn execute_fd<M: CpuBus>(&mut self, opcode: u8, _displ: i8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x09 => {
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x19 => {
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x29 => {
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, ix, false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x39 => {
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.r16[R_SP], false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x21 => {
                let nn = mmu.r16(self.state.r16[R_PC].wrapping_add(2));
                self.state.set_reg16(5, nn);
                (14, 4)
            }
            0x22 => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(5));
                (20, 4)
            }
            0x2A => {
                let pc = self.state.r16[R_PC];
                let nn = mmu.r16(pc.wrapping_add(2));
                self.state.set_reg16(5, mmu.r16(nn));
                (20, 4)
            }
            0x23 => {
                self.state
                    .set_reg16(5, self.state.get_reg16(5).wrapping_add(1));
                (10, 2)
            }
            0x2B => {
                self.state
                    .set_reg16(5, self.state.get_reg16(5).wrapping_sub(1));
                (10, 2)
            }
            0x24 => {
                let (res, flags) = self.add8(self.state.r8[10], 1, false);
                self.state.r8[10] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x25 => {
                let (res, flags) = self.sub8(self.state.r8[10], 1, false);
                self.state.r8[10] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x26 => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(2));
                self.state.r8[10] = n;
                (11, 3)
            }
            0x2C => {
                let (res, flags) = self.add8(self.state.r8[11], 1, false);
                self.state.r8[11] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2D => {
                let (res, flags) = self.sub8(self.state.r8[11], 1, false);
                self.state.r8[11] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2E => {
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(2));
                self.state.r8[11] = n;
                (11, 3)
            }
            0x34 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x35 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x36 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let n = mmu.r8(self.state.r16[R_PC].wrapping_add(3));
                mmu.w8(addr, n);
                (19, 4)
            }
            0x44 => {
                self.state.r8[R_B] = self.state.r8[10];
                (8, 2)
            }
            0x45 => {
                self.state.r8[R_B] = self.state.r8[11];
                (8, 2)
            }
            0x46 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_B] = mmu.r8(addr);
                (19, 3)
            }
            0x4C => {
                self.state.r8[R_C] = self.state.r8[10];
                (8, 2)
            }
            0x4D => {
                self.state.r8[R_C] = self.state.r8[11];
                (8, 2)
            }
            0x4E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_C] = mmu.r8(addr);
                (19, 3)
            }
            0x54 => {
                self.state.r8[R_D] = self.state.r8[10];
                (8, 2)
            }
            0x55 => {
                self.state.r8[R_D] = self.state.r8[11];
                (8, 2)
            }
            0x56 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_D] = mmu.r8(addr);
                (19, 3)
            }
            0x5C => {
                self.state.r8[R_E] = self.state.r8[10];
                (8, 2)
            }
            0x5D => {
                self.state.r8[R_E] = self.state.r8[11];
                (8, 2)
            }
            0x5E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_E] = mmu.r8(addr);
                (19, 3)
            }
            0x66 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_H] = mmu.r8(addr);
                (19, 3)
            }
            0x6E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_L] = mmu.r8(addr);
                (19, 3)
            }
            0x7C => {
                self.state.r8[R_A] = self.state.r8[10];
                (8, 2)
            }
            0x7D => {
                self.state.r8[R_A] = self.state.r8[11];
                (8, 2)
            }
            0x7E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] = mmu.r8(addr);
                (19, 3)
            }
            0x60 => {
                self.state.r8[10] = self.state.r8[R_B];
                (8, 2)
            }
            0x61 => {
                self.state.r8[10] = self.state.r8[R_C];
                (8, 2)
            }
            0x62 => {
                self.state.r8[10] = self.state.r8[R_D];
                (8, 2)
            }
            0x63 => {
                self.state.r8[10] = self.state.r8[R_E];
                (8, 2)
            }
            0x64 => (8, 2),
            0x65 => {
                self.state.r8[10] = self.state.r8[11];
                (8, 2)
            }
            0x67 => {
                self.state.r8[10] = self.state.r8[R_A];
                (8, 2)
            }
            0x68 => {
                self.state.r8[11] = self.state.r8[R_B];
                (8, 2)
            }
            0x69 => {
                self.state.r8[11] = self.state.r8[R_C];
                (8, 2)
            }
            0x6A => {
                self.state.r8[11] = self.state.r8[R_D];
                (8, 2)
            }
            0x6B => {
                self.state.r8[11] = self.state.r8[R_E];
                (8, 2)
            }
            0x6C => {
                self.state.r8[11] = self.state.r8[10];
                (8, 2)
            }
            0x6D => (8, 2),
            0x6F => {
                self.state.r8[11] = self.state.r8[R_A];
                (8, 2)
            }
            0x70 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[2]);
                (19, 3)
            }
            0x71 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[3]);
                (19, 3)
            }
            0x72 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[4]);
                (19, 3)
            }
            0x73 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[5]);
                (19, 3)
            }
            0x74 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[6]);
                (19, 3)
            }
            0x75 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[7]);
                (19, 3)
            }
            0x77 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[0]);
                (19, 3)
            }
            0x84 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x85 => {
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x86 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x8C => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[10],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x8D => {
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    self.state.r8[11],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x8E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(
                    self.state.r8[R_A],
                    mmu.r8(addr),
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x94 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x95 => {
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x96 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x9C => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[10],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x9D => {
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    self.state.r8[11],
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x9E => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(
                    self.state.r8[R_A],
                    mmu.r8(addr),
                    (self.state.r8[R_F] & F_C) != 0,
                );
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0xA4 => {
                self.state.r8[R_A] &= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA5 => {
                self.state.r8[R_A] &= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA6 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] &= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (19, 3)
            }
            0xAC => {
                self.state.r8[R_A] ^= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAD => {
                self.state.r8[R_A] ^= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAE => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] ^= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xB4 => {
                self.state.r8[R_A] |= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB5 => {
                self.state.r8[R_A] |= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB6 => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] |= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xBC => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[10] & (F_5 | F_3));
                (8, 2)
            }
            0xBD => {
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[11] & (F_5 | F_3));
                (8, 2)
            }
            0xBE => {
                let displ = mmu.r8s(self.state.r16[R_PC].wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let val = mmu.r8(addr);
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (19, 3)
            }
            0xE1 => {
                let val = self.pop16(mmu);
                self.state.set_reg16(5, val);
                (14, 2)
            }
            0xE3 => {
                let sp = self.state.r16[R_SP];
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(5));
                self.state.set_reg16(5, memval);
                (23, 2)
            }
            0xE5 => {
                self.push16(mmu, self.state.get_reg16(5));
                (15, 2)
            }
            0xE9 => {
                self.state.r16[R_PC] = self.state.get_reg16(5);
                (8, 0)
            }
            0xF9 => {
                self.state.r16[R_SP] = self.state.get_reg16(5);
                (10, 2)
            }
            _ => (0, 0),
        }
    }
    pub fn execute_ddcb<M: CpuBus>(&mut self, opcode: u8, displ: i8, mmu: &mut M) -> (u32, u8) {
        let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
        match opcode {
            0x00 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x01 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x02 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x03 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x04 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x05 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x06 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x07 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x08 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x09 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x10 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x11 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x12 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x13 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x14 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x15 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x16 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x17 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x18 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x19 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x20 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x21 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x22 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x23 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x24 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x25 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x26 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x27 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x28 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x29 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x30 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x31 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x32 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x33 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x34 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x35 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x36 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x37 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x38 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x39 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x40 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x41 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x42 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x43 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x44 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x45 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x46 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x47 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x48 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x49 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x50 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x51 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x52 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x53 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x54 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x55 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x56 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x57 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x58 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x59 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x60 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x61 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x62 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x63 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x64 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x65 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x66 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x67 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x68 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x69 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x70 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x71 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x72 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x73 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x74 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x75 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x76 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x77 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x78 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x79 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x80 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x81 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x82 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x83 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x84 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x85 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x86 => {
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x87 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x88 => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x89 => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8A => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8B => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8C => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8D => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8E => {
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8F => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x90 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x91 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x92 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x93 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x94 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x95 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x96 => {
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x97 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x98 => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x99 => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9A => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9B => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9C => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9D => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9E => {
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9F => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA0 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA1 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA2 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA3 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA4 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA5 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA6 => {
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA7 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA8 => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA9 => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAA => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAB => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAC => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAD => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAE => {
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAF => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB0 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB1 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB2 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB3 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB4 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB5 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB6 => {
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB7 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB8 => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB9 => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBA => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBB => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBC => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBD => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBE => {
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBF => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC0 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC1 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC2 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC3 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC4 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC5 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC6 => {
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC7 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC8 => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC9 => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCA => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCB => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCC => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCD => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCE => {
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCF => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD0 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD1 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD2 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD3 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD4 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD5 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD6 => {
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD7 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD8 => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD9 => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDA => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDB => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDC => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDD => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDE => {
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDF => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE0 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE1 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE2 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE3 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE4 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE5 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE6 => {
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE7 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE8 => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE9 => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEA => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEB => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEC => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xED => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEE => {
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEF => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF0 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF1 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF2 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF3 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF4 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF5 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF6 => {
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF7 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF8 => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF9 => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFA => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFB => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFC => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFD => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFE => {
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFF => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
        }
    }
    pub fn execute_fdcb<M: CpuBus>(&mut self, opcode: u8, displ: i8, mmu: &mut M) -> (u32, u8) {
        let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
        match opcode {
            0x00 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x01 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x02 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x03 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x04 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x05 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x06 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x07 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x08 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x09 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x10 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x11 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x12 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x13 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x14 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x15 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x16 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x17 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x18 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x19 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x20 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x21 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x22 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x23 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x24 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x25 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x26 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x27 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x28 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x29 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x30 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x31 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x32 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x33 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x34 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x35 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x36 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x37 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x38 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x39 => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3A => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3B => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3C => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3D => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3E => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3F => {
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x40 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x41 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x42 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x43 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x44 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x45 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x46 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x47 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x01;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x48 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x49 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x4F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x02;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x50 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x51 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x52 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x53 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x54 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x55 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x56 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x57 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x04;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x58 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x59 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x5F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x08;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x60 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x61 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x62 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x63 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x64 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x65 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x66 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x67 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x10;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x68 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x69 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x6F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x20;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x70 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x71 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x72 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x73 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x74 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x75 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x76 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x77 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x40;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x78 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x79 => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7A => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7B => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7C => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7D => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7E => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x7F => {
                let srcval = mmu.r8(addr);
                let val = srcval & 0x80;
                self.state.r8[R_F] = (val & F_S)
                    | (if val != 0 { 0 } else { F_Z | F_PV })
                    | ((addr >> 8) as u8 & (F_3 | F_5))
                    | F_H
                    | (self.state.r8[R_F] & F_C);
                (20, 4)
            }
            0x80 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x81 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x82 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x83 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x84 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x85 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x86 => {
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x87 => {
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x88 => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x89 => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8A => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8B => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8C => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8D => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8E => {
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8F => {
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x90 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x91 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x92 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x93 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x94 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x95 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x96 => {
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x97 => {
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x98 => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x99 => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9A => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9B => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9C => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9D => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9E => {
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9F => {
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA0 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA1 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA2 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA3 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA4 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA5 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA6 => {
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA7 => {
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA8 => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA9 => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAA => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAB => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAC => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAD => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAE => {
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAF => {
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB0 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB1 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB2 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB3 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB4 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB5 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB6 => {
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB7 => {
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB8 => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB9 => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBA => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBB => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBC => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBD => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBE => {
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBF => {
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC0 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC1 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC2 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC3 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC4 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC5 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC6 => {
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC7 => {
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC8 => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC9 => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCA => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCB => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCC => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCD => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCE => {
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCF => {
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD0 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD1 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD2 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD3 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD4 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD5 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD6 => {
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD7 => {
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD8 => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD9 => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDA => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDB => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDC => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDD => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDE => {
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDF => {
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE0 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE1 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE2 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE3 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE4 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE5 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE6 => {
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE7 => {
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE8 => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE9 => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEA => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEB => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEC => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xED => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEE => {
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEF => {
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF0 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF1 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF2 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF3 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF4 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF5 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF6 => {
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF7 => {
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF8 => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF9 => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFA => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFB => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFC => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFD => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFE => {
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFF => {
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
        }
    }
}

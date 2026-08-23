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

pub struct Z80State {
    // 8-bit registers: A,F,B,C,D,E,H,L,IXh,IXl,IYh,IYl,A',F',B',C',D',E',H',L',I,R
    pub r8: [u8; 22],
    pub sp: u16,
    pub pc: u16,

    pub halted: u8,
    pub im: u8,
    pub iff1: u8,
    pub iff2: u8,
}

impl Z80State {
    pub fn new() -> Self {
        let mut state = Z80State {
            r8: [0; 22],
            sp: 0,
            pc: 0,
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
        self.r8[0..=19].fill(0xFF);
        self.sp = 0xFFFF;

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

        self.pc = 0x0000;
    }

    pub fn get_reg16(&self, reg: usize) -> u16 {
        match reg {
            0..=9 => ((self.r8[reg * 2] as u16) << 8) | (self.r8[reg * 2 + 1] as u16),
            10 => self.sp,
            11 => self.pc,
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
            10 => self.sp = val,
            11 => self.pc = val,
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
        let sp = self.state.sp.wrapping_sub(1);
        mmu.w8(sp, ((val >> 8) & 0xFF) as u8);
        let sp = sp.wrapping_sub(1);
        mmu.w8(sp, (val & 0xFF) as u8);
        self.state.sp = sp;
    }

    pub fn pop16<M: CpuBus>(&mut self, mmu: &mut M) -> u16 {
        let sp = self.state.sp;
        let lo = mmu.r8(sp) as u16;
        let sp = sp.wrapping_add(1);
        let hi = mmu.r8(sp) as u16;
        self.state.sp = sp.wrapping_add(1);
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
            "SP" => self.state.sp,
            "PC" => self.state.pc,
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
            "SP" => self.state.sp = val,
            "PC" => self.state.pc = val,
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
            let pc = self.state.pc;
            self.push16(mmu, pc);
            self.state.pc = 0x0038;
            13
        } else {
            0
        }
    }

    pub fn step<M: CpuBus>(&mut self, mmu: &mut M, _run_for: i32) -> u32 {
        if self.state.halted != 0 {
            return 4;
        }

        let mut pc = self.state.pc;
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
            self.state.pc = pc_loop;
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
            self.state.pc = pc.wrapping_add(m as u16);
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
            0x00 => (4, 1), // NOP
            0x01 => {
                // LD BC,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(1));
                self.state.set_reg16(R_BC, nn);
                (10, 3)
            }
            0x02 => {
                // LD (BC),A
                mmu.w8(self.state.get_reg16(R_BC), self.state.r8[R_A]);
                (7, 1)
            }
            0x03 => {
                // INC BC
                self.state
                    .set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_add(1));
                (6, 1)
            }
            0x04 => {
                // INC B
                let (res, flags) = self.add8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x05 => {
                // DEC B
                let (res, flags) = self.sub8(self.state.r8[R_B], 1, false);
                self.state.r8[R_B] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x06 => {
                // LD B,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_B] = n;
                (7, 2)
            }
            0x07 => {
                // RLCA
                let a = self.state.r8[R_A];
                let (res, flags) = self.shl8(a, (a & 0x80) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x08 => {
                // EX AF,AF'
                let a = self.state.r8[R_A];
                self.state.r8[R_A] = self.state.r8[R_AA];
                self.state.r8[R_AA] = a;
                let f = self.state.r8[R_F];
                self.state.r8[R_F] = self.state.r8[R_FA];
                self.state.r8[R_FA] = f;
                (4, 1)
            }
            0x09 => {
                // ADD HL,BC
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x0A => {
                // LD A,(BC)
                self.state.r8[R_A] = mmu.r8(self.state.get_reg16(R_BC));
                (7, 1)
            }
            0x0B => {
                // DEC BC
                self.state
                    .set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_sub(1));
                (6, 1)
            }
            0x0C => {
                // INC C
                let (res, flags) = self.add8(self.state.r8[R_C], 1, false);
                self.state.r8[R_C] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x0D => {
                // DEC C
                let (res, flags) = self.sub8(self.state.r8[R_C], 1, false);
                self.state.r8[R_C] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x0E => {
                // LD C,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_C] = n;
                (7, 2)
            }
            0x0F => {
                // RRCA
                let a = self.state.r8[R_A];
                let (res, flags) = self.shr8(a, (a & 0x01) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x10 => {
                // DJNZ e
                let pc = self.state.pc;
                self.state.r8[R_B] = self.state.r8[R_B].wrapping_sub(1);
                if self.state.r8[R_B] == 0 {
                    (8, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                    (13, 0)
                }
            }
            0x11 => {
                // LD DE,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(1));
                self.state.set_reg16(R_DE, nn);
                (10, 3)
            }
            0x12 => {
                // LD (DE),A
                mmu.w8(self.state.get_reg16(R_DE), self.state.r8[R_A]);
                (7, 1)
            }
            0x13 => {
                // INC DE
                self.state
                    .set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_add(1));
                (6, 1)
            }
            0x14 => {
                // INC D
                let (res, flags) = self.add8(self.state.r8[R_D], 1, false);
                self.state.r8[R_D] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x15 => {
                // DEC D
                let (res, flags) = self.sub8(self.state.r8[R_D], 1, false);
                self.state.r8[R_D] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x16 => {
                // LD D,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_D] = n;
                (7, 2)
            }
            0x17 => {
                // RLA
                let a = self.state.r8[R_A];
                let (res, flags) = self.shl8(a, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x18 => {
                // JR e
                let pc = self.state.pc;
                let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                (12, 0)
            }
            0x19 => {
                // ADD HL,DE
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x1A => {
                // LD A,(DE)
                self.state.r8[R_A] = mmu.r8(self.state.get_reg16(R_DE));
                (7, 1)
            }
            0x1B => {
                // DEC DE
                self.state
                    .set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_sub(1));
                (6, 1)
            }
            0x1C => {
                // INC E
                let (res, flags) = self.add8(self.state.r8[R_E], 1, false);
                self.state.r8[R_E] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x1D => {
                // DEC E
                let (res, flags) = self.sub8(self.state.r8[R_E], 1, false);
                self.state.r8[R_E] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x1E => {
                // LD E,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_E] = n;
                (7, 2)
            }
            0x1F => {
                // RRA
                let a = self.state.r8[R_A];
                let (res, flags) = self.shr8(a, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & (F_S | F_Z | F_PV)) | (flags & !(F_S | F_Z | F_PV));
                (4, 1)
            }
            0x20 => {
                // JR NZ e
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    (7, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                    (12, 0)
                }
            }
            0x21 => {
                // LD HL,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(1));
                self.state.set_reg16(R_HL, nn);
                (10, 3)
            }
            0x22 => {
                // LD (nn),HL
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                mmu.w16(nn, self.state.get_reg16(R_HL));
                (16, 3)
            }
            0x23 => {
                // INC HL
                self.state
                    .set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_add(1));
                (6, 1)
            }
            0x24 => {
                // INC H
                let (res, flags) = self.add8(self.state.r8[R_H], 1, false);
                self.state.r8[R_H] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x25 => {
                // DEC H
                let (res, flags) = self.sub8(self.state.r8[R_H], 1, false);
                self.state.r8[R_H] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x26 => {
                // LD H,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_H] = n;
                (7, 2)
            }
            0x27 => {
                // DAA
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
                // JR Z e
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                    (12, 0)
                } else {
                    (7, 2)
                }
            }
            0x29 => {
                // ADD HL,HL
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, hl, false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x2A => {
                // LD HL,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.set_reg16(R_HL, mmu.r16(nn));
                (16, 3)
            }
            0x2B => {
                // DEC HL
                self.state
                    .set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_sub(1));
                (6, 1)
            }
            0x2C => {
                // INC L
                let (res, flags) = self.add8(self.state.r8[R_L], 1, false);
                self.state.r8[R_L] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x2D => {
                // DEC L
                let (res, flags) = self.sub8(self.state.r8[R_L], 1, false);
                self.state.r8[R_L] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x2E => {
                // LD L,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_L] = n;
                (7, 2)
            }
            0x2F => {
                // CPL
                self.state.r8[R_A] = !self.state.r8[R_A];
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV | F_C))
                    | F_H
                    | F_N
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3);
                (4, 1)
            }
            0x30 => {
                // JR NC e
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    (7, 2)
                } else {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                    (12, 0)
                }
            }
            0x31 => {
                // LD SP,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(1));
                self.state.sp = nn;
                (10, 3)
            }
            0x32 => {
                // LD (nn),A
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                mmu.w8(nn, self.state.r8[R_A]);
                (13, 3)
            }
            0x33 => {
                // INC SP
                self.state.sp = self.state.sp.wrapping_add(1);
                (6, 1)
            }
            0x34 => {
                // INC (HL)
                let addr = self.state.get_reg16(R_HL);
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (11, 1)
            }
            0x35 => {
                // DEC (HL)
                let addr = self.state.get_reg16(R_HL);
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (11, 1)
            }
            0x36 => {
                // LD (HL),n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                mmu.w8(self.state.get_reg16(R_HL), n);
                (10, 2)
            }
            0x37 => {
                // SCF
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV))
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3)
                    | F_C;
                (4, 1)
            }
            0x38 => {
                // JR C e
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    let e = mmu.r8s(pc.wrapping_add(1)) as i16;
                    self.state.pc = pc.wrapping_add(2).wrapping_add(e as u16);
                    (12, 0)
                } else {
                    (7, 2)
                }
            }
            0x39 => {
                // ADD HL,SP
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.sp, false);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (11, 1)
            }
            0x3A => {
                // LD A,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.r8[R_A] = mmu.r8(nn);
                (13, 3)
            }
            0x3B => {
                // DEC SP
                self.state.sp = self.state.sp.wrapping_sub(1);
                (6, 1)
            }
            0x3C => {
                // INC A
                let (res, flags) = self.add8(self.state.r8[R_A], 1, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x3D => {
                // DEC A
                let (res, flags) = self.sub8(self.state.r8[R_A], 1, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (4, 1)
            }
            0x3E => {
                // LD A,n
                let n = mmu.r8(self.state.pc.wrapping_add(1));
                self.state.r8[R_A] = n;
                (7, 2)
            }
            0x3F => {
                // CCF
                let cf = self.state.r8[R_F] & F_C;
                self.state.r8[R_F] = (self.state.r8[R_F] & (F_S | F_Z | F_PV))
                    | (self.state.r8[R_A] & F_5)
                    | (self.state.r8[R_A] & F_3)
                    | (cf << 4)
                    | (cf ^ F_C);
                (4, 1)
            }
            0x40 => {
                // LD B,B
                self.state.r8[2] = self.state.r8[2];
                (4, 1)
            }
            0x41 => {
                // LD B,C
                self.state.r8[2] = self.state.r8[3];
                (4, 1)
            }
            0x42 => {
                // LD B,D
                self.state.r8[2] = self.state.r8[4];
                (4, 1)
            }
            0x43 => {
                // LD B,E
                self.state.r8[2] = self.state.r8[5];
                (4, 1)
            }
            0x44 => {
                // LD B,H
                self.state.r8[2] = self.state.r8[6];
                (4, 1)
            }
            0x45 => {
                // LD B,L
                self.state.r8[2] = self.state.r8[7];
                (4, 1)
            }
            0x46 => {
                // LD B,(HL)
                self.state.r8[2] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x47 => {
                // LD B,A
                self.state.r8[2] = self.state.r8[0];
                (4, 1)
            }
            0x48 => {
                // LD C,B
                self.state.r8[3] = self.state.r8[2];
                (4, 1)
            }
            0x49 => {
                // LD C,C
                self.state.r8[3] = self.state.r8[3];
                (4, 1)
            }
            0x4A => {
                // LD C,D
                self.state.r8[3] = self.state.r8[4];
                (4, 1)
            }
            0x4B => {
                // LD C,E
                self.state.r8[3] = self.state.r8[5];
                (4, 1)
            }
            0x4C => {
                // LD C,H
                self.state.r8[3] = self.state.r8[6];
                (4, 1)
            }
            0x4D => {
                // LD C,L
                self.state.r8[3] = self.state.r8[7];
                (4, 1)
            }
            0x4E => {
                // LD C,(HL)
                self.state.r8[3] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x4F => {
                // LD C,A
                self.state.r8[3] = self.state.r8[0];
                (4, 1)
            }
            0x50 => {
                // LD D,B
                self.state.r8[4] = self.state.r8[2];
                (4, 1)
            }
            0x51 => {
                // LD D,C
                self.state.r8[4] = self.state.r8[3];
                (4, 1)
            }
            0x52 => {
                // LD D,D
                self.state.r8[4] = self.state.r8[4];
                (4, 1)
            }
            0x53 => {
                // LD D,E
                self.state.r8[4] = self.state.r8[5];
                (4, 1)
            }
            0x54 => {
                // LD D,H
                self.state.r8[4] = self.state.r8[6];
                (4, 1)
            }
            0x55 => {
                // LD D,L
                self.state.r8[4] = self.state.r8[7];
                (4, 1)
            }
            0x56 => {
                // LD D,(HL)
                self.state.r8[4] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x57 => {
                // LD D,A
                self.state.r8[4] = self.state.r8[0];
                (4, 1)
            }
            0x58 => {
                // LD E,B
                self.state.r8[5] = self.state.r8[2];
                (4, 1)
            }
            0x59 => {
                // LD E,C
                self.state.r8[5] = self.state.r8[3];
                (4, 1)
            }
            0x5A => {
                // LD E,D
                self.state.r8[5] = self.state.r8[4];
                (4, 1)
            }
            0x5B => {
                // LD E,E
                self.state.r8[5] = self.state.r8[5];
                (4, 1)
            }
            0x5C => {
                // LD E,H
                self.state.r8[5] = self.state.r8[6];
                (4, 1)
            }
            0x5D => {
                // LD E,L
                self.state.r8[5] = self.state.r8[7];
                (4, 1)
            }
            0x5E => {
                // LD E,(HL)
                self.state.r8[5] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x5F => {
                // LD E,A
                self.state.r8[5] = self.state.r8[0];
                (4, 1)
            }
            0x60 => {
                // LD H,B
                self.state.r8[6] = self.state.r8[2];
                (4, 1)
            }
            0x61 => {
                // LD H,C
                self.state.r8[6] = self.state.r8[3];
                (4, 1)
            }
            0x62 => {
                // LD H,D
                self.state.r8[6] = self.state.r8[4];
                (4, 1)
            }
            0x63 => {
                // LD H,E
                self.state.r8[6] = self.state.r8[5];
                (4, 1)
            }
            0x64 => {
                // LD H,H
                self.state.r8[6] = self.state.r8[6];
                (4, 1)
            }
            0x65 => {
                // LD H,L
                self.state.r8[6] = self.state.r8[7];
                (4, 1)
            }
            0x66 => {
                // LD H,(HL)
                self.state.r8[6] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x67 => {
                // LD H,A
                self.state.r8[6] = self.state.r8[0];
                (4, 1)
            }
            0x68 => {
                // LD L,B
                self.state.r8[7] = self.state.r8[2];
                (4, 1)
            }
            0x69 => {
                // LD L,C
                self.state.r8[7] = self.state.r8[3];
                (4, 1)
            }
            0x6A => {
                // LD L,D
                self.state.r8[7] = self.state.r8[4];
                (4, 1)
            }
            0x6B => {
                // LD L,E
                self.state.r8[7] = self.state.r8[5];
                (4, 1)
            }
            0x6C => {
                // LD L,H
                self.state.r8[7] = self.state.r8[6];
                (4, 1)
            }
            0x6D => {
                // LD L,L
                self.state.r8[7] = self.state.r8[7];
                (4, 1)
            }
            0x6E => {
                // LD L,(HL)
                self.state.r8[7] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x6F => {
                // LD L,A
                self.state.r8[7] = self.state.r8[0];
                (4, 1)
            }
            0x70 => {
                // LD (HL),B
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[2]);
                (7, 1)
            }
            0x71 => {
                // LD (HL),C
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[3]);
                (7, 1)
            }
            0x72 => {
                // LD (HL),D
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[4]);
                (7, 1)
            }
            0x73 => {
                // LD (HL),E
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[5]);
                (7, 1)
            }
            0x74 => {
                // LD (HL),H
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[6]);
                (7, 1)
            }
            0x75 => {
                // LD (HL),L
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[7]);
                (7, 1)
            }
            0x76 => {
                // HALT
                self.state.halted = 1;
                (4, 1)
            }
            0x77 => {
                // LD (HL),A
                mmu.w8(self.state.get_reg16(R_HL), self.state.r8[0]);
                (7, 1)
            }
            0x78 => {
                // LD A,B
                self.state.r8[0] = self.state.r8[2];
                (4, 1)
            }
            0x79 => {
                // LD A,C
                self.state.r8[0] = self.state.r8[3];
                (4, 1)
            }
            0x7A => {
                // LD A,D
                self.state.r8[0] = self.state.r8[4];
                (4, 1)
            }
            0x7B => {
                // LD A,E
                self.state.r8[0] = self.state.r8[5];
                (4, 1)
            }
            0x7C => {
                // LD A,H
                self.state.r8[0] = self.state.r8[6];
                (4, 1)
            }
            0x7D => {
                // LD A,L
                self.state.r8[0] = self.state.r8[7];
                (4, 1)
            }
            0x7E => {
                // LD A,(HL)
                self.state.r8[0] = mmu.r8(self.state.get_reg16(R_HL));
                (7, 1)
            }
            0x7F => {
                // LD A,A
                self.state.r8[0] = self.state.r8[0];
                (4, 1)
            }
            0x80 => {
                // ADD A,B
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x81 => {
                // ADD A,C
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x82 => {
                // ADD A,D
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x83 => {
                // ADD A,E
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x84 => {
                // ADD A,H
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x85 => {
                // ADD A,L
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x86 => {
                // ADD A,(HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) = self.add8(self.state.r8[R_A], val, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x87 => {
                // ADD A,A
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x88 => {
                // ADC A,B
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
                // ADC A,C
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
                // ADC A,D
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
                // ADC A,E
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
                // ADC A,H
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
                // ADC A,L
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
                // ADC A,(HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) =
                    self.add8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x8F => {
                // ADC A,A
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
                // SUB B
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x91 => {
                // SUB C
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x92 => {
                // SUB D
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x93 => {
                // SUB E
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x94 => {
                // SUB H
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x95 => {
                // SUB L
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x96 => {
                // SUB (HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x97 => {
                // SUB A
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (4, 1)
            }
            0x98 => {
                // SBC A,B
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
                // SBC A,C
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
                // SBC A,D
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
                // SBC A,E
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
                // SBC A,H
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
                // SBC A,L
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
                // SBC A,(HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (res, flags) =
                    self.sub8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 1)
            }
            0x9F => {
                // SBC A,A
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
                // AND B
                self.state.r8[R_A] &= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA1 => {
                // AND C
                self.state.r8[R_A] &= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA2 => {
                // AND D
                self.state.r8[R_A] &= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA3 => {
                // AND E
                self.state.r8[R_A] &= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA4 => {
                // AND H
                self.state.r8[R_A] &= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA5 => {
                // AND L
                self.state.r8[R_A] &= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA6 => {
                // AND (HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] &= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (7, 1)
            }
            0xA7 => {
                // AND A
                self.state.r8[R_A] &= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (4, 1)
            }
            0xA8 => {
                // XOR B
                self.state.r8[R_A] ^= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xA9 => {
                // XOR C
                self.state.r8[R_A] ^= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAA => {
                // XOR D
                self.state.r8[R_A] ^= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAB => {
                // XOR E
                self.state.r8[R_A] ^= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAC => {
                // XOR H
                self.state.r8[R_A] ^= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAD => {
                // XOR L
                self.state.r8[R_A] ^= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xAE => {
                // XOR (HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] ^= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 1)
            }
            0xAF => {
                // XOR A
                self.state.r8[R_A] ^= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB0 => {
                // OR B
                self.state.r8[R_A] |= self.state.r8[2];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB1 => {
                // OR C
                self.state.r8[R_A] |= self.state.r8[3];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB2 => {
                // OR D
                self.state.r8[R_A] |= self.state.r8[4];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB3 => {
                // OR E
                self.state.r8[R_A] |= self.state.r8[5];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB4 => {
                // OR H
                self.state.r8[R_A] |= self.state.r8[6];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB5 => {
                // OR L
                self.state.r8[R_A] |= self.state.r8[7];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB6 => {
                // OR (HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                self.state.r8[R_A] |= val;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 1)
            }
            0xB7 => {
                // OR A
                self.state.r8[R_A] |= self.state.r8[0];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (4, 1)
            }
            0xB8 => {
                // CP B
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[2], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[2] & (F_5 | F_3));
                (4, 1)
            }
            0xB9 => {
                // CP C
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[3], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[3] & (F_5 | F_3));
                (4, 1)
            }
            0xBA => {
                // CP D
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[4], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[4] & (F_5 | F_3));
                (4, 1)
            }
            0xBB => {
                // CP E
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[5], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[5] & (F_5 | F_3));
                (4, 1)
            }
            0xBC => {
                // CP H
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[6], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[6] & (F_5 | F_3));
                (4, 1)
            }
            0xBD => {
                // CP L
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[7], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[7] & (F_5 | F_3));
                (4, 1)
            }
            0xBE => {
                // CP (HL)
                let val = mmu.r8(self.state.get_reg16(R_HL));
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (7, 1)
            }
            0xBF => {
                // CP A
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[0], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[0] & (F_5 | F_3));
                (4, 1)
            }
            0xC0 => {
                // RET NZ
                if self.state.r8[R_F] & F_Z != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                }
            }
            0xC1 => {
                // POP BC
                let val = self.pop16(mmu);
                self.state.set_reg16(R_BC, val);
                (10, 1)
            }
            0xC2 => {
                // JP NZ,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                }
            }
            0xC3 => {
                // JP nn
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                self.state.pc = nn;
                (10, 0)
            }
            0xC4 => {
                // CALL NZ,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                }
            }
            0xC5 => {
                // PUSH BC
                self.push16(mmu, self.state.get_reg16(R_BC));
                (11, 1)
            }
            0xC6 => {
                // ADD A,n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) = self.add8(self.state.r8[R_A], n, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xC7 => {
                // RST 00H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x00;
                (11, 0)
            }
            0xC8 => {
                // RET Z
                if self.state.r8[R_F] & F_Z != 0 {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xC9 => {
                // RET
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (10, 0)
            }
            0xCA => {
                // JP Z,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xCB => (4, 1), // CB
            0xCC => {
                // CALL Z,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_Z != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xCD => {
                // CALL nn
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(1));
                self.push16(mmu, pc.wrapping_add(3));
                self.state.pc = nn;
                (17, 0)
            }
            0xCE => {
                // ADC A,n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) =
                    self.add8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xCF => {
                // RST 08H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x08;
                (11, 0)
            }
            0xD0 => {
                // RET NC
                if self.state.r8[R_F] & F_C != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                }
            }
            0xD1 => {
                // POP DE
                let val = self.pop16(mmu);
                self.state.set_reg16(R_DE, val);
                (10, 1)
            }
            0xD2 => {
                // JP NC,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                }
            }
            0xD3 => {
                // OUT (n),A
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                mmu.out8(n, self.state.r8[R_A], self.state.r8[R_A]);
                (11, 2)
            }
            0xD4 => {
                // CALL NC,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                }
            }
            0xD5 => {
                // PUSH DE
                self.push16(mmu, self.state.get_reg16(R_DE));
                (11, 1)
            }
            0xD6 => {
                // SUB n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) = self.sub8(self.state.r8[R_A], n, false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xD7 => {
                // RST 10H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x10;
                (11, 0)
            }
            0xD8 => {
                // RET C
                if self.state.r8[R_F] & F_C != 0 {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xD9 => {
                // EXX
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
                // JP C,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xDB => {
                // IN A,(n)
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] = mmu.in8(n, self.state.r8[R_A]);
                (11, 2)
            }
            0xDC => {
                // CALL C,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_C != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xDD => (4, 1), // DD
            0xDE => {
                // SBC A,n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                let (res, flags) =
                    self.sub8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (7, 2)
            }
            0xDF => {
                // RST 18H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x18;
                (11, 0)
            }
            0xE0 => {
                // RET PO
                if self.state.r8[R_F] & F_PV != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                }
            }
            0xE1 => {
                // POP HL
                let val = self.pop16(mmu);
                self.state.set_reg16(R_HL, val);
                (10, 1)
            }
            0xE2 => {
                // JP PO,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_PV != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                }
            }
            0xE3 => {
                // EX (SP),HL
                let sp = self.state.sp;
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(R_HL));
                self.state.set_reg16(R_HL, memval);
                (19, 1)
            }
            0xE4 => {
                // CALL PO,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_PV != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                }
            }
            0xE5 => {
                // PUSH HL
                self.push16(mmu, self.state.get_reg16(R_HL));
                (11, 1)
            }
            0xE6 => {
                // AND n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] &= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (7, 2)
            }
            0xE7 => {
                // RST 20H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x20;
                (11, 0)
            }
            0xE8 => {
                // RET PE
                if self.state.r8[R_F] & F_PV != 0 {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xE9 => {
                // JP (HL)
                self.state.pc = self.state.get_reg16(R_HL);
                (4, 0)
            }
            0xEA => {
                // JP PE,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_PV != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xEB => {
                // EX DE,HL
                let de = self.state.get_reg16(R_DE);
                self.state.set_reg16(R_DE, self.state.get_reg16(R_HL));
                self.state.set_reg16(R_HL, de);
                (4, 1)
            }
            0xEC => {
                // CALL PE,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_PV != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xED => (4, 1), // ED
            0xEE => {
                // XOR n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] ^= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 2)
            }
            0xEF => {
                // RST 28H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x28;
                (11, 0)
            }
            0xF0 => {
                // RET P
                if self.state.r8[R_F] & F_S != 0 {
                    (5, 1)
                } else {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                }
            }
            0xF1 => {
                // POP AF
                let val = self.pop16(mmu);
                self.state.set_reg16(R_AF, val);
                (10, 1)
            }
            0xF2 => {
                // JP P,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_S != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                }
            }
            0xF3 => {
                // DI
                self.state.iff1 = 0;
                self.state.iff2 = 0;
                (4, 1)
            }
            0xF4 => {
                // CALL P,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_S != 0 {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                } else {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                }
            }
            0xF5 => {
                // PUSH AF
                self.push16(mmu, self.state.get_reg16(R_AF));
                (11, 1)
            }
            0xF6 => {
                // OR n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                self.state.r8[R_A] |= n;
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (7, 2)
            }
            0xF7 => {
                // RST 30H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x30;
                (11, 0)
            }
            0xF8 => {
                // RET M
                if self.state.r8[R_F] & F_S != 0 {
                    let addr = self.pop16(mmu);
                    self.state.pc = addr;
                    (11, 0)
                } else {
                    (5, 1)
                }
            }
            0xF9 => {
                // LD SP,HL
                self.state.sp = self.state.get_reg16(R_HL);
                (6, 1)
            }
            0xFA => {
                // JP M,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_S != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.state.pc = nn;
                    (10, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xFB => {
                // EI
                self.state.iff1 = 1;
                self.state.iff2 = 1;
                (4, 1)
            }
            0xFC => {
                // CALL M,nn
                let pc = self.state.pc;
                if self.state.r8[R_F] & F_S != 0 {
                    let nn = mmu.r16(pc.wrapping_add(1));
                    self.push16(mmu, pc.wrapping_add(3));
                    self.state.pc = nn;
                    (17, 0)
                } else {
                    mmu.r16nolog(pc.wrapping_add(1));
                    (10, 3)
                }
            }
            0xFD => (4, 1), // FD
            0xFE => {
                // CP n
                let pc = self.state.pc;
                let n = mmu.r8(pc.wrapping_add(1));
                let (_, flags) = self.sub8(self.state.r8[R_A], n, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (n & (F_5 | F_3));
                (7, 2)
            }
            0xFF => {
                // RST 38H
                let pc = self.state.pc;
                self.push16(mmu, pc.wrapping_add(1));
                self.state.pc = 0x38;
                (11, 0)
            }
        }
    }
    pub fn execute_cb<M: CpuBus>(&mut self, opcode: u8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x00 => {
                // RLC B
                let (res, flags) = self.shl8(self.state.r8[2], (self.state.r8[2] & 0x80) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x01 => {
                // RLC C
                let (res, flags) = self.shl8(self.state.r8[3], (self.state.r8[3] & 0x80) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x02 => {
                // RLC D
                let (res, flags) = self.shl8(self.state.r8[4], (self.state.r8[4] & 0x80) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x03 => {
                // RLC E
                let (res, flags) = self.shl8(self.state.r8[5], (self.state.r8[5] & 0x80) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x04 => {
                // RLC H
                let (res, flags) = self.shl8(self.state.r8[6], (self.state.r8[6] & 0x80) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x05 => {
                // RLC L
                let (res, flags) = self.shl8(self.state.r8[7], (self.state.r8[7] & 0x80) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x06 => {
                // RLC (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x07 => {
                // RLC A
                let (res, flags) = self.shl8(self.state.r8[0], (self.state.r8[0] & 0x80) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x08 => {
                // RRC B
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[2] & 0x01) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x09 => {
                // RRC C
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[3] & 0x01) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0A => {
                // RRC D
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[4] & 0x01) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0B => {
                // RRC E
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[5] & 0x01) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0C => {
                // RRC H
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[6] & 0x01) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0D => {
                // RRC L
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[7] & 0x01) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x0E => {
                // RRC (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x0F => {
                // RRC A
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[0] & 0x01) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x10 => {
                // RL B
                let (res, flags) = self.shl8(self.state.r8[2], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x11 => {
                // RL C
                let (res, flags) = self.shl8(self.state.r8[3], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x12 => {
                // RL D
                let (res, flags) = self.shl8(self.state.r8[4], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x13 => {
                // RL E
                let (res, flags) = self.shl8(self.state.r8[5], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x14 => {
                // RL H
                let (res, flags) = self.shl8(self.state.r8[6], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x15 => {
                // RL L
                let (res, flags) = self.shl8(self.state.r8[7], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x16 => {
                // RL (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x17 => {
                // RL A
                let (res, flags) = self.shl8(self.state.r8[0], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x18 => {
                // RR B
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x19 => {
                // RR C
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1A => {
                // RR D
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1B => {
                // RR E
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1C => {
                // RR H
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1D => {
                // RR L
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x1E => {
                // RR (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x1F => {
                // RR A
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x20 => {
                // SLA B
                let (res, flags) = self.shl8(self.state.r8[2], false);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x21 => {
                // SLA C
                let (res, flags) = self.shl8(self.state.r8[3], false);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x22 => {
                // SLA D
                let (res, flags) = self.shl8(self.state.r8[4], false);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x23 => {
                // SLA E
                let (res, flags) = self.shl8(self.state.r8[5], false);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x24 => {
                // SLA H
                let (res, flags) = self.shl8(self.state.r8[6], false);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x25 => {
                // SLA L
                let (res, flags) = self.shl8(self.state.r8[7], false);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x26 => {
                // SLA (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x27 => {
                // SLA A
                let (res, flags) = self.shl8(self.state.r8[0], false);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x28 => {
                // SRA B
                let (res, flags) = self.shr8(self.state.r8[2], (self.state.r8[2] & 0x80) != 0);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x29 => {
                // SRA C
                let (res, flags) = self.shr8(self.state.r8[3], (self.state.r8[3] & 0x80) != 0);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2A => {
                // SRA D
                let (res, flags) = self.shr8(self.state.r8[4], (self.state.r8[4] & 0x80) != 0);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2B => {
                // SRA E
                let (res, flags) = self.shr8(self.state.r8[5], (self.state.r8[5] & 0x80) != 0);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2C => {
                // SRA H
                let (res, flags) = self.shr8(self.state.r8[6], (self.state.r8[6] & 0x80) != 0);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2D => {
                // SRA L
                let (res, flags) = self.shr8(self.state.r8[7], (self.state.r8[7] & 0x80) != 0);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x2E => {
                // SRA (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x2F => {
                // SRA A
                let (res, flags) = self.shr8(self.state.r8[0], (self.state.r8[0] & 0x80) != 0);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x30 => {
                // SLL B
                let (res, flags) = self.shl8(self.state.r8[2], true);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x31 => {
                // SLL C
                let (res, flags) = self.shl8(self.state.r8[3], true);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x32 => {
                // SLL D
                let (res, flags) = self.shl8(self.state.r8[4], true);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x33 => {
                // SLL E
                let (res, flags) = self.shl8(self.state.r8[5], true);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x34 => {
                // SLL H
                let (res, flags) = self.shl8(self.state.r8[6], true);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x35 => {
                // SLL L
                let (res, flags) = self.shl8(self.state.r8[7], true);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x36 => {
                // SLL (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x37 => {
                // SLL A
                let (res, flags) = self.shl8(self.state.r8[0], true);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x38 => {
                // SRL B
                let (res, flags) = self.shr8(self.state.r8[2], false);
                self.state.r8[2] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x39 => {
                // SRL C
                let (res, flags) = self.shr8(self.state.r8[3], false);
                self.state.r8[3] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3A => {
                // SRL D
                let (res, flags) = self.shr8(self.state.r8[4], false);
                self.state.r8[4] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3B => {
                // SRL E
                let (res, flags) = self.shr8(self.state.r8[5], false);
                self.state.r8[5] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3C => {
                // SRL H
                let (res, flags) = self.shr8(self.state.r8[6], false);
                self.state.r8[6] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3D => {
                // SRL L
                let (res, flags) = self.shr8(self.state.r8[7], false);
                self.state.r8[7] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x3E => {
                // SRL (HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x3F => {
                // SRL A
                let (res, flags) = self.shr8(self.state.r8[0], false);
                self.state.r8[0] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x40 => {
                // BIT 0,B
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
                // BIT 0,C
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
                // BIT 0,D
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
                // BIT 0,E
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
                // BIT 0,H
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
                // BIT 0,L
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
                // BIT 0,(HL)
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
                // BIT 0,A
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
                // BIT 1,B
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
                // BIT 1,C
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
                // BIT 1,D
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
                // BIT 1,E
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
                // BIT 1,H
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
                // BIT 1,L
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
                // BIT 1,(HL)
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
                // BIT 1,A
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
                // BIT 2,B
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
                // BIT 2,C
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
                // BIT 2,D
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
                // BIT 2,E
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
                // BIT 2,H
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
                // BIT 2,L
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
                // BIT 2,(HL)
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
                // BIT 2,A
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
                // BIT 3,B
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
                // BIT 3,C
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
                // BIT 3,D
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
                // BIT 3,E
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
                // BIT 3,H
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
                // BIT 3,L
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
                // BIT 3,(HL)
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
                // BIT 3,A
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
                // BIT 4,B
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
                // BIT 4,C
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
                // BIT 4,D
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
                // BIT 4,E
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
                // BIT 4,H
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
                // BIT 4,L
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
                // BIT 4,(HL)
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
                // BIT 4,A
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
                // BIT 5,B
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
                // BIT 5,C
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
                // BIT 5,D
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
                // BIT 5,E
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
                // BIT 5,H
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
                // BIT 5,L
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
                // BIT 5,(HL)
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
                // BIT 5,A
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
                // BIT 6,B
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
                // BIT 6,C
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
                // BIT 6,D
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
                // BIT 6,E
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
                // BIT 6,H
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
                // BIT 6,L
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
                // BIT 6,(HL)
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
                // BIT 6,A
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
                // BIT 7,B
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
                // BIT 7,C
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
                // BIT 7,D
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
                // BIT 7,E
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
                // BIT 7,H
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
                // BIT 7,L
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
                // BIT 7,(HL)
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
                // BIT 7,A
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
                // RES 0,B
                self.state.r8[2] &= 0xFE;
                (8, 2)
            }
            0x81 => {
                // RES 0,C
                self.state.r8[3] &= 0xFE;
                (8, 2)
            }
            0x82 => {
                // RES 0,D
                self.state.r8[4] &= 0xFE;
                (8, 2)
            }
            0x83 => {
                // RES 0,E
                self.state.r8[5] &= 0xFE;
                (8, 2)
            }
            0x84 => {
                // RES 0,H
                self.state.r8[6] &= 0xFE;
                (8, 2)
            }
            0x85 => {
                // RES 0,L
                self.state.r8[7] &= 0xFE;
                (8, 2)
            }
            0x86 => {
                // RES 0,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x87 => {
                // RES 0,A
                self.state.r8[0] &= 0xFE;
                (8, 2)
            }
            0x88 => {
                // RES 1,B
                self.state.r8[2] &= 0xFD;
                (8, 2)
            }
            0x89 => {
                // RES 1,C
                self.state.r8[3] &= 0xFD;
                (8, 2)
            }
            0x8A => {
                // RES 1,D
                self.state.r8[4] &= 0xFD;
                (8, 2)
            }
            0x8B => {
                // RES 1,E
                self.state.r8[5] &= 0xFD;
                (8, 2)
            }
            0x8C => {
                // RES 1,H
                self.state.r8[6] &= 0xFD;
                (8, 2)
            }
            0x8D => {
                // RES 1,L
                self.state.r8[7] &= 0xFD;
                (8, 2)
            }
            0x8E => {
                // RES 1,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x8F => {
                // RES 1,A
                self.state.r8[0] &= 0xFD;
                (8, 2)
            }
            0x90 => {
                // RES 2,B
                self.state.r8[2] &= 0xFB;
                (8, 2)
            }
            0x91 => {
                // RES 2,C
                self.state.r8[3] &= 0xFB;
                (8, 2)
            }
            0x92 => {
                // RES 2,D
                self.state.r8[4] &= 0xFB;
                (8, 2)
            }
            0x93 => {
                // RES 2,E
                self.state.r8[5] &= 0xFB;
                (8, 2)
            }
            0x94 => {
                // RES 2,H
                self.state.r8[6] &= 0xFB;
                (8, 2)
            }
            0x95 => {
                // RES 2,L
                self.state.r8[7] &= 0xFB;
                (8, 2)
            }
            0x96 => {
                // RES 2,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x97 => {
                // RES 2,A
                self.state.r8[0] &= 0xFB;
                (8, 2)
            }
            0x98 => {
                // RES 3,B
                self.state.r8[2] &= 0xF7;
                (8, 2)
            }
            0x99 => {
                // RES 3,C
                self.state.r8[3] &= 0xF7;
                (8, 2)
            }
            0x9A => {
                // RES 3,D
                self.state.r8[4] &= 0xF7;
                (8, 2)
            }
            0x9B => {
                // RES 3,E
                self.state.r8[5] &= 0xF7;
                (8, 2)
            }
            0x9C => {
                // RES 3,H
                self.state.r8[6] &= 0xF7;
                (8, 2)
            }
            0x9D => {
                // RES 3,L
                self.state.r8[7] &= 0xF7;
                (8, 2)
            }
            0x9E => {
                // RES 3,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (15, 2)
            }
            0x9F => {
                // RES 3,A
                self.state.r8[0] &= 0xF7;
                (8, 2)
            }
            0xA0 => {
                // RES 4,B
                self.state.r8[2] &= 0xEF;
                (8, 2)
            }
            0xA1 => {
                // RES 4,C
                self.state.r8[3] &= 0xEF;
                (8, 2)
            }
            0xA2 => {
                // RES 4,D
                self.state.r8[4] &= 0xEF;
                (8, 2)
            }
            0xA3 => {
                // RES 4,E
                self.state.r8[5] &= 0xEF;
                (8, 2)
            }
            0xA4 => {
                // RES 4,H
                self.state.r8[6] &= 0xEF;
                (8, 2)
            }
            0xA5 => {
                // RES 4,L
                self.state.r8[7] &= 0xEF;
                (8, 2)
            }
            0xA6 => {
                // RES 4,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xA7 => {
                // RES 4,A
                self.state.r8[0] &= 0xEF;
                (8, 2)
            }
            0xA8 => {
                // RES 5,B
                self.state.r8[2] &= 0xDF;
                (8, 2)
            }
            0xA9 => {
                // RES 5,C
                self.state.r8[3] &= 0xDF;
                (8, 2)
            }
            0xAA => {
                // RES 5,D
                self.state.r8[4] &= 0xDF;
                (8, 2)
            }
            0xAB => {
                // RES 5,E
                self.state.r8[5] &= 0xDF;
                (8, 2)
            }
            0xAC => {
                // RES 5,H
                self.state.r8[6] &= 0xDF;
                (8, 2)
            }
            0xAD => {
                // RES 5,L
                self.state.r8[7] &= 0xDF;
                (8, 2)
            }
            0xAE => {
                // RES 5,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xAF => {
                // RES 5,A
                self.state.r8[0] &= 0xDF;
                (8, 2)
            }
            0xB0 => {
                // RES 6,B
                self.state.r8[2] &= 0xBF;
                (8, 2)
            }
            0xB1 => {
                // RES 6,C
                self.state.r8[3] &= 0xBF;
                (8, 2)
            }
            0xB2 => {
                // RES 6,D
                self.state.r8[4] &= 0xBF;
                (8, 2)
            }
            0xB3 => {
                // RES 6,E
                self.state.r8[5] &= 0xBF;
                (8, 2)
            }
            0xB4 => {
                // RES 6,H
                self.state.r8[6] &= 0xBF;
                (8, 2)
            }
            0xB5 => {
                // RES 6,L
                self.state.r8[7] &= 0xBF;
                (8, 2)
            }
            0xB6 => {
                // RES 6,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xB7 => {
                // RES 6,A
                self.state.r8[0] &= 0xBF;
                (8, 2)
            }
            0xB8 => {
                // RES 7,B
                self.state.r8[2] &= 0x7F;
                (8, 2)
            }
            0xB9 => {
                // RES 7,C
                self.state.r8[3] &= 0x7F;
                (8, 2)
            }
            0xBA => {
                // RES 7,D
                self.state.r8[4] &= 0x7F;
                (8, 2)
            }
            0xBB => {
                // RES 7,E
                self.state.r8[5] &= 0x7F;
                (8, 2)
            }
            0xBC => {
                // RES 7,H
                self.state.r8[6] &= 0x7F;
                (8, 2)
            }
            0xBD => {
                // RES 7,L
                self.state.r8[7] &= 0x7F;
                (8, 2)
            }
            0xBE => {
                // RES 7,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xBF => {
                // RES 7,A
                self.state.r8[0] &= 0x7F;
                (8, 2)
            }
            0xC0 => {
                // SET 0,B
                self.state.r8[2] |= 0x01;
                (8, 2)
            }
            0xC1 => {
                // SET 0,C
                self.state.r8[3] |= 0x01;
                (8, 2)
            }
            0xC2 => {
                // SET 0,D
                self.state.r8[4] |= 0x01;
                (8, 2)
            }
            0xC3 => {
                // SET 0,E
                self.state.r8[5] |= 0x01;
                (8, 2)
            }
            0xC4 => {
                // SET 0,H
                self.state.r8[6] |= 0x01;
                (8, 2)
            }
            0xC5 => {
                // SET 0,L
                self.state.r8[7] |= 0x01;
                (8, 2)
            }
            0xC6 => {
                // SET 0,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xC7 => {
                // SET 0,A
                self.state.r8[0] |= 0x01;
                (8, 2)
            }
            0xC8 => {
                // SET 1,B
                self.state.r8[2] |= 0x02;
                (8, 2)
            }
            0xC9 => {
                // SET 1,C
                self.state.r8[3] |= 0x02;
                (8, 2)
            }
            0xCA => {
                // SET 1,D
                self.state.r8[4] |= 0x02;
                (8, 2)
            }
            0xCB => {
                // SET 1,E
                self.state.r8[5] |= 0x02;
                (8, 2)
            }
            0xCC => {
                // SET 1,H
                self.state.r8[6] |= 0x02;
                (8, 2)
            }
            0xCD => {
                // SET 1,L
                self.state.r8[7] |= 0x02;
                (8, 2)
            }
            0xCE => {
                // SET 1,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xCF => {
                // SET 1,A
                self.state.r8[0] |= 0x02;
                (8, 2)
            }
            0xD0 => {
                // SET 2,B
                self.state.r8[2] |= 0x04;
                (8, 2)
            }
            0xD1 => {
                // SET 2,C
                self.state.r8[3] |= 0x04;
                (8, 2)
            }
            0xD2 => {
                // SET 2,D
                self.state.r8[4] |= 0x04;
                (8, 2)
            }
            0xD3 => {
                // SET 2,E
                self.state.r8[5] |= 0x04;
                (8, 2)
            }
            0xD4 => {
                // SET 2,H
                self.state.r8[6] |= 0x04;
                (8, 2)
            }
            0xD5 => {
                // SET 2,L
                self.state.r8[7] |= 0x04;
                (8, 2)
            }
            0xD6 => {
                // SET 2,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xD7 => {
                // SET 2,A
                self.state.r8[0] |= 0x04;
                (8, 2)
            }
            0xD8 => {
                // SET 3,B
                self.state.r8[2] |= 0x08;
                (8, 2)
            }
            0xD9 => {
                // SET 3,C
                self.state.r8[3] |= 0x08;
                (8, 2)
            }
            0xDA => {
                // SET 3,D
                self.state.r8[4] |= 0x08;
                (8, 2)
            }
            0xDB => {
                // SET 3,E
                self.state.r8[5] |= 0x08;
                (8, 2)
            }
            0xDC => {
                // SET 3,H
                self.state.r8[6] |= 0x08;
                (8, 2)
            }
            0xDD => {
                // SET 3,L
                self.state.r8[7] |= 0x08;
                (8, 2)
            }
            0xDE => {
                // SET 3,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xDF => {
                // SET 3,A
                self.state.r8[0] |= 0x08;
                (8, 2)
            }
            0xE0 => {
                // SET 4,B
                self.state.r8[2] |= 0x10;
                (8, 2)
            }
            0xE1 => {
                // SET 4,C
                self.state.r8[3] |= 0x10;
                (8, 2)
            }
            0xE2 => {
                // SET 4,D
                self.state.r8[4] |= 0x10;
                (8, 2)
            }
            0xE3 => {
                // SET 4,E
                self.state.r8[5] |= 0x10;
                (8, 2)
            }
            0xE4 => {
                // SET 4,H
                self.state.r8[6] |= 0x10;
                (8, 2)
            }
            0xE5 => {
                // SET 4,L
                self.state.r8[7] |= 0x10;
                (8, 2)
            }
            0xE6 => {
                // SET 4,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xE7 => {
                // SET 4,A
                self.state.r8[0] |= 0x10;
                (8, 2)
            }
            0xE8 => {
                // SET 5,B
                self.state.r8[2] |= 0x20;
                (8, 2)
            }
            0xE9 => {
                // SET 5,C
                self.state.r8[3] |= 0x20;
                (8, 2)
            }
            0xEA => {
                // SET 5,D
                self.state.r8[4] |= 0x20;
                (8, 2)
            }
            0xEB => {
                // SET 5,E
                self.state.r8[5] |= 0x20;
                (8, 2)
            }
            0xEC => {
                // SET 5,H
                self.state.r8[6] |= 0x20;
                (8, 2)
            }
            0xED => {
                // SET 5,L
                self.state.r8[7] |= 0x20;
                (8, 2)
            }
            0xEE => {
                // SET 5,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xEF => {
                // SET 5,A
                self.state.r8[0] |= 0x20;
                (8, 2)
            }
            0xF0 => {
                // SET 6,B
                self.state.r8[2] |= 0x40;
                (8, 2)
            }
            0xF1 => {
                // SET 6,C
                self.state.r8[3] |= 0x40;
                (8, 2)
            }
            0xF2 => {
                // SET 6,D
                self.state.r8[4] |= 0x40;
                (8, 2)
            }
            0xF3 => {
                // SET 6,E
                self.state.r8[5] |= 0x40;
                (8, 2)
            }
            0xF4 => {
                // SET 6,H
                self.state.r8[6] |= 0x40;
                (8, 2)
            }
            0xF5 => {
                // SET 6,L
                self.state.r8[7] |= 0x40;
                (8, 2)
            }
            0xF6 => {
                // SET 6,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xF7 => {
                // SET 6,A
                self.state.r8[0] |= 0x40;
                (8, 2)
            }
            0xF8 => {
                // SET 7,B
                self.state.r8[2] |= 0x80;
                (8, 2)
            }
            0xF9 => {
                // SET 7,C
                self.state.r8[3] |= 0x80;
                (8, 2)
            }
            0xFA => {
                // SET 7,D
                self.state.r8[4] |= 0x80;
                (8, 2)
            }
            0xFB => {
                // SET 7,E
                self.state.r8[5] |= 0x80;
                (8, 2)
            }
            0xFC => {
                // SET 7,H
                self.state.r8[6] |= 0x80;
                (8, 2)
            }
            0xFD => {
                // SET 7,L
                self.state.r8[7] |= 0x80;
                (8, 2)
            }
            0xFE => {
                // SET 7,(HL)
                let addr = self.state.get_reg16(R_HL);
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (15, 2)
            }
            0xFF => {
                // SET 7,A
                self.state.r8[0] |= 0x80;
                (8, 2)
            }
        }
    }
    pub fn execute_ed<M: CpuBus>(&mut self, opcode: u8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x40 => {
                // IN B,(C)
                self.state.r8[2] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[2] as usize];
                (12, 2)
            }
            0x41 => {
                // OUT (C),B
                mmu.out8(self.state.r8[R_C], self.state.r8[2], self.state.r8[R_B]);
                (12, 2)
            }
            0x48 => {
                // IN C,(C)
                self.state.r8[3] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[3] as usize];
                (12, 2)
            }
            0x49 => {
                // OUT (C),C
                mmu.out8(self.state.r8[R_C], self.state.r8[3], self.state.r8[R_B]);
                (12, 2)
            }
            0x50 => {
                // IN D,(C)
                self.state.r8[4] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[4] as usize];
                (12, 2)
            }
            0x51 => {
                // OUT (C),D
                mmu.out8(self.state.r8[R_C], self.state.r8[4], self.state.r8[R_B]);
                (12, 2)
            }
            0x58 => {
                // IN E,(C)
                self.state.r8[5] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[5] as usize];
                (12, 2)
            }
            0x59 => {
                // OUT (C),E
                mmu.out8(self.state.r8[R_C], self.state.r8[5], self.state.r8[R_B]);
                (12, 2)
            }
            0x60 => {
                // IN H,(C)
                self.state.r8[6] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[6] as usize];
                (12, 2)
            }
            0x61 => {
                // OUT (C),H
                mmu.out8(self.state.r8[R_C], self.state.r8[6], self.state.r8[R_B]);
                (12, 2)
            }
            0x68 => {
                // IN L,(C)
                self.state.r8[7] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[7] as usize];
                (12, 2)
            }
            0x69 => {
                // OUT (C),L
                mmu.out8(self.state.r8[R_C], self.state.r8[7], self.state.r8[R_B]);
                (12, 2)
            }
            0x70 => {
                // IN F,(C)
                let val = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C) | self.sz53p_table[val as usize];
                (12, 2)
            }
            0x71 => {
                // OUT (C),0
                mmu.out8(self.state.r8[R_C], 0, self.state.r8[R_B]);
                (12, 2)
            }
            0x78 => {
                // IN A,(C)
                self.state.r8[0] = mmu.in8(self.state.r8[R_C], self.state.r8[R_B]);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[0] as usize];
                (12, 2)
            }
            0x79 => {
                // OUT (C),A
                mmu.out8(self.state.r8[R_C], self.state.r8[0], self.state.r8[R_B]);
                (12, 2)
            }
            0x42 => {
                // SBC HL,BC
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(1), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x43 => {
                // LD (nn),BC
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(1));
                (20, 4)
            }
            0x4A => {
                // ADC HL,BC
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(1), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x4B => {
                // LD BC,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(1, val);
                (20, 4)
            }
            0x52 => {
                // SBC HL,DE
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(2), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x53 => {
                // LD (nn),DE
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(2));
                (20, 4)
            }
            0x5A => {
                // ADC HL,DE
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(2), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x5B => {
                // LD DE,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(2, val);
                (20, 4)
            }
            0x62 => {
                // SBC HL,HL
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.sub16(hl, self.state.get_reg16(3), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x63 => {
                // LD (nn),HL
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(3));
                (20, 4)
            }
            0x6A => {
                // ADC HL,HL
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) =
                    self.add16(hl, self.state.get_reg16(3), (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x6B => {
                // LD HL,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.set_reg16(3, val);
                (20, 4)
            }
            0x72 => {
                // SBC HL,SP
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.sub16(hl, self.state.sp, (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x73 => {
                // LD (nn),SP
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.sp);
                (20, 4)
            }
            0x7A => {
                // ADC HL,SP
                let hl = self.state.get_reg16(R_HL);
                let (res, flags) = self.add16(hl, self.state.sp, (self.state.r8[R_F] & F_C) != 0);
                self.state.set_reg16(R_HL, res);
                self.state.r8[R_F] = flags;
                (15, 2)
            }
            0x7B => {
                // LD SP,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                let val = mmu.r16(nn);
                self.state.sp = val;
                (20, 4)
            }
            0x44 => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x4C => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x54 => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x5C => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x64 => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x6C => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x74 => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x7C => {
                // NEG
                let (res, flags) = self.sub8(0, self.state.r8[R_A], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x45 => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x55 => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x65 => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x75 => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x5D => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x6D => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x7D => {
                // RETN
                self.state.iff1 = self.state.iff2;
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x4D => {
                // RETI
                let addr = self.pop16(mmu);
                self.state.pc = addr;
                (14, 0)
            }
            0x46 => {
                // IM 0
                self.state.im = 0;
                (8, 2)
            }
            0x4E => {
                // IM 0
                self.state.im = 0;
                (8, 2)
            }
            0x66 => {
                // IM 0
                self.state.im = 0;
                (8, 2)
            }
            0x6E => {
                // IM 0
                self.state.im = 0;
                (8, 2)
            }
            0x56 => {
                // IM 1
                self.state.im = 1;
                (8, 2)
            }
            0x76 => {
                // IM 1
                self.state.im = 1;
                (8, 2)
            }
            0x5E => {
                // IM 2
                self.state.im = 2;
                (8, 2)
            }
            0x7E => {
                // IM 2
                self.state.im = 2;
                (8, 2)
            }
            0x47 => {
                // LD I,A
                self.state.r8[R_I] = self.state.r8[R_A];
                (9, 2)
            }
            0x4F => {
                // LD R,A
                self.state.r8[R_R] = self.state.r8[R_A];
                (9, 2)
            }
            0x57 => {
                // LD A,I
                self.state.r8[R_A] = self.state.r8[R_I];
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C)
                    | self.sz53_table[self.state.r8[R_A] as usize]
                    | (if self.state.iff2 != 0 { F_PV } else { 0 });
                (9, 2)
            }
            0x5F => {
                // LD A,R
                self.state.r8[R_A] = self.state.r8[R_R];
                self.state.r8[R_F] = (self.state.r8[R_F] & F_C)
                    | self.sz53_table[self.state.r8[R_A] as usize]
                    | (if self.state.iff2 != 0 { F_PV } else { 0 });
                (9, 2)
            }
            0x67 => {
                // RRD
                let addr = self.state.get_reg16(R_HL);
                let memval = mmu.r8(addr);
                mmu.w8(addr, ((self.state.r8[R_A] & 0x0F) << 4) | (memval >> 4));
                self.state.r8[R_A] = (self.state.r8[R_A] & 0xF0) | (memval & 0x0F);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[R_A] as usize];
                (18, 2)
            }
            0x6F => {
                // RLD
                let addr = self.state.get_reg16(R_HL);
                let memval = mmu.r8(addr);
                mmu.w8(addr, ((memval & 0x0F) << 4) | (self.state.r8[R_A] & 0x0F));
                self.state.r8[R_A] = (self.state.r8[R_A] & 0xF0) | (memval >> 4);
                self.state.r8[R_F] =
                    (self.state.r8[R_F] & F_C) | self.sz53p_table[self.state.r8[R_A] as usize];
                (18, 2)
            }
            0x77 => (8, 2), // NOP
            0x7F => (8, 2), // NOP
            0xA0 => {
                // LDI
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
                // CPI
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
                // INI
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
                // OUTI
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
                // LDD
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
                // CPD
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
                // IND
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
                // OUTD
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
                // LDIR
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
                // CPIR
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
                // INIR
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
                // OTIR
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
                // LDDR
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
                // CPDR
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
                // INDR
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
                // OTDR
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
                // ADD IX,BC
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x19 => {
                // ADD IX,DE
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x29 => {
                // ADD IX,IX
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, ix, false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x39 => {
                // ADD IX,SP
                let ix = self.state.get_reg16(4);
                let (res, flags) = self.add16(ix, self.state.sp, false);
                self.state.set_reg16(4, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x21 => {
                // LD IX,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(2));
                self.state.set_reg16(4, nn);
                (14, 4)
            }
            0x22 => {
                // LD (nn),IX
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(4));
                (20, 4)
            }
            0x2A => {
                // LD IX,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                self.state.set_reg16(4, mmu.r16(nn));
                (20, 4)
            }
            0x23 => {
                // INC IX
                self.state
                    .set_reg16(4, self.state.get_reg16(4).wrapping_add(1));
                (10, 2)
            }
            0x2B => {
                // DEC IX
                self.state
                    .set_reg16(4, self.state.get_reg16(4).wrapping_sub(1));
                (10, 2)
            }
            0x24 => {
                // INC IXH
                let (res, flags) = self.add8(self.state.r8[8], 1, false);
                self.state.r8[8] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x25 => {
                // DEC IXH
                let (res, flags) = self.sub8(self.state.r8[8], 1, false);
                self.state.r8[8] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x26 => {
                // LD IXH,n
                let n = mmu.r8(self.state.pc.wrapping_add(2));
                self.state.r8[8] = n;
                (11, 3)
            }
            0x2C => {
                // INC IXL
                let (res, flags) = self.add8(self.state.r8[9], 1, false);
                self.state.r8[9] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2D => {
                // DEC IXL
                let (res, flags) = self.sub8(self.state.r8[9], 1, false);
                self.state.r8[9] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2E => {
                // LD IXL,n
                let n = mmu.r8(self.state.pc.wrapping_add(2));
                self.state.r8[9] = n;
                (11, 3)
            }
            0x34 => {
                // INC (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x35 => {
                // DEC (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x36 => {
                // LD (IX+d),n
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let n = mmu.r8(self.state.pc.wrapping_add(3));
                mmu.w8(addr, n);
                (19, 4)
            }
            0x44 => {
                // LD B,IXH
                self.state.r8[R_B] = self.state.r8[8];
                (8, 2)
            }
            0x45 => {
                // LD B,IXL
                self.state.r8[R_B] = self.state.r8[9];
                (8, 2)
            }
            0x46 => {
                // LD B,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_B] = mmu.r8(addr);
                (19, 3)
            }
            0x4C => {
                // LD C,IXH
                self.state.r8[R_C] = self.state.r8[8];
                (8, 2)
            }
            0x4D => {
                // LD C,IXL
                self.state.r8[R_C] = self.state.r8[9];
                (8, 2)
            }
            0x4E => {
                // LD C,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_C] = mmu.r8(addr);
                (19, 3)
            }
            0x54 => {
                // LD D,IXH
                self.state.r8[R_D] = self.state.r8[8];
                (8, 2)
            }
            0x55 => {
                // LD D,IXL
                self.state.r8[R_D] = self.state.r8[9];
                (8, 2)
            }
            0x56 => {
                // LD D,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_D] = mmu.r8(addr);
                (19, 3)
            }
            0x5C => {
                // LD E,IXH
                self.state.r8[R_E] = self.state.r8[8];
                (8, 2)
            }
            0x5D => {
                // LD E,IXL
                self.state.r8[R_E] = self.state.r8[9];
                (8, 2)
            }
            0x5E => {
                // LD E,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_E] = mmu.r8(addr);
                (19, 3)
            }
            0x66 => {
                // LD H,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_H] = mmu.r8(addr);
                (19, 3)
            }
            0x6E => {
                // LD L,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_L] = mmu.r8(addr);
                (19, 3)
            }
            0x7C => {
                // LD A,IXH
                self.state.r8[R_A] = self.state.r8[8];
                (8, 2)
            }
            0x7D => {
                // LD A,IXL
                self.state.r8[R_A] = self.state.r8[9];
                (8, 2)
            }
            0x7E => {
                // LD A,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] = mmu.r8(addr);
                (19, 3)
            }
            0x60 => {
                // LD IXH,B
                self.state.r8[8] = self.state.r8[R_B];
                (8, 2)
            }
            0x61 => {
                // LD IXH,C
                self.state.r8[8] = self.state.r8[R_C];
                (8, 2)
            }
            0x62 => {
                // LD IXH,D
                self.state.r8[8] = self.state.r8[R_D];
                (8, 2)
            }
            0x63 => {
                // LD IXH,E
                self.state.r8[8] = self.state.r8[R_E];
                (8, 2)
            }
            0x64 => (8, 2), // LD IXH,IXH
            0x65 => {
                // LD IXH,IXL
                self.state.r8[8] = self.state.r8[9];
                (8, 2)
            }
            0x67 => {
                // LD IXH,A
                self.state.r8[8] = self.state.r8[R_A];
                (8, 2)
            }
            0x68 => {
                // LD IXL,B
                self.state.r8[9] = self.state.r8[R_B];
                (8, 2)
            }
            0x69 => {
                // LD IXL,C
                self.state.r8[9] = self.state.r8[R_C];
                (8, 2)
            }
            0x6A => {
                // LD IXL,D
                self.state.r8[9] = self.state.r8[R_D];
                (8, 2)
            }
            0x6B => {
                // LD IXL,E
                self.state.r8[9] = self.state.r8[R_E];
                (8, 2)
            }
            0x6C => {
                // LD IXL,IXH
                self.state.r8[9] = self.state.r8[8];
                (8, 2)
            }
            0x6D => (8, 2), // LD IXL,IXL
            0x6F => {
                // LD IXL,A
                self.state.r8[9] = self.state.r8[R_A];
                (8, 2)
            }
            0x70 => {
                // LD (IX+d),B
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[2]);
                (19, 3)
            }
            0x71 => {
                // LD (IX+d),C
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[3]);
                (19, 3)
            }
            0x72 => {
                // LD (IX+d),D
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[4]);
                (19, 3)
            }
            0x73 => {
                // LD (IX+d),E
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[5]);
                (19, 3)
            }
            0x74 => {
                // LD (IX+d),H
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[6]);
                (19, 3)
            }
            0x75 => {
                // LD (IX+d),L
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[7]);
                (19, 3)
            }
            0x77 => {
                // LD (IX+d),A
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[0]);
                (19, 3)
            }
            0x84 => {
                // ADD A,IXH
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x85 => {
                // ADD A,IXL
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x86 => {
                // ADD A,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x8C => {
                // ADC A,IXH
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
                // ADC A,IXL
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
                // ADC A,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
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
                // SUB IXH
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x95 => {
                // SUB IXL
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x96 => {
                // SUB (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x9C => {
                // SBC A,IXH
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
                // SBC A,IXL
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
                // SBC A,(IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
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
                // AND IXH
                self.state.r8[R_A] &= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA5 => {
                // AND IXL
                self.state.r8[R_A] &= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA6 => {
                // AND (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] &= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (19, 3)
            }
            0xAC => {
                // XOR IXH
                self.state.r8[R_A] ^= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAD => {
                // XOR IXL
                self.state.r8[R_A] ^= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAE => {
                // XOR (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] ^= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xB4 => {
                // OR IXH
                self.state.r8[R_A] |= self.state.r8[8];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB5 => {
                // OR IXL
                self.state.r8[R_A] |= self.state.r8[9];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB6 => {
                // OR (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                self.state.r8[R_A] |= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xBC => {
                // CP IXH
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[8], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[8] & (F_5 | F_3));
                (8, 2)
            }
            0xBD => {
                // CP IXL
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[9], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[9] & (F_5 | F_3));
                (8, 2)
            }
            0xBE => {
                // CP (IX+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
                let val = mmu.r8(addr);
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (19, 3)
            }
            0xE1 => {
                // POP IX
                let val = self.pop16(mmu);
                self.state.set_reg16(4, val);
                (14, 2)
            }
            0xE3 => {
                // EX (SP),IX
                let sp = self.state.sp;
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(4));
                self.state.set_reg16(4, memval);
                (23, 2)
            }
            0xE5 => {
                // PUSH IX
                self.push16(mmu, self.state.get_reg16(4));
                (15, 2)
            }
            0xE9 => {
                // JP (IX)
                self.state.pc = self.state.get_reg16(4);
                (8, 0)
            }
            0xF9 => {
                // LD SP,IX
                self.state.sp = self.state.get_reg16(4);
                (10, 2)
            }
            _ => (0, 0),
        }
    }
    pub fn execute_fd<M: CpuBus>(&mut self, opcode: u8, _displ: i8, mmu: &mut M) -> (u32, u8) {
        match opcode {
            0x09 => {
                // ADD IY,BC
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_BC), false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x19 => {
                // ADD IY,DE
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.get_reg16(R_DE), false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x29 => {
                // ADD IY,IY
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, ix, false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x39 => {
                // ADD IY,SP
                let ix = self.state.get_reg16(5);
                let (res, flags) = self.add16(ix, self.state.sp, false);
                self.state.set_reg16(5, res);
                self.state.r8[R_F] =
                    (flags & !(F_S | F_Z | F_PV)) | (self.state.r8[R_F] & (F_S | F_Z | F_PV));
                (15, 2)
            }
            0x21 => {
                // LD IY,nn
                let nn = mmu.r16(self.state.pc.wrapping_add(2));
                self.state.set_reg16(5, nn);
                (14, 4)
            }
            0x22 => {
                // LD (nn),IY
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                mmu.w16(nn, self.state.get_reg16(5));
                (20, 4)
            }
            0x2A => {
                // LD IY,(nn)
                let pc = self.state.pc;
                let nn = mmu.r16(pc.wrapping_add(2));
                self.state.set_reg16(5, mmu.r16(nn));
                (20, 4)
            }
            0x23 => {
                // INC IY
                self.state
                    .set_reg16(5, self.state.get_reg16(5).wrapping_add(1));
                (10, 2)
            }
            0x2B => {
                // DEC IY
                self.state
                    .set_reg16(5, self.state.get_reg16(5).wrapping_sub(1));
                (10, 2)
            }
            0x24 => {
                // INC IYH
                let (res, flags) = self.add8(self.state.r8[10], 1, false);
                self.state.r8[10] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x25 => {
                // DEC IYH
                let (res, flags) = self.sub8(self.state.r8[10], 1, false);
                self.state.r8[10] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x26 => {
                // LD IYH,n
                let n = mmu.r8(self.state.pc.wrapping_add(2));
                self.state.r8[10] = n;
                (11, 3)
            }
            0x2C => {
                // INC IYL
                let (res, flags) = self.add8(self.state.r8[11], 1, false);
                self.state.r8[11] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2D => {
                // DEC IYL
                let (res, flags) = self.sub8(self.state.r8[11], 1, false);
                self.state.r8[11] = res;
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (8, 2)
            }
            0x2E => {
                // LD IYL,n
                let n = mmu.r8(self.state.pc.wrapping_add(2));
                self.state.r8[11] = n;
                (11, 3)
            }
            0x34 => {
                // INC (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.add8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x35 => {
                // DEC (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let v = mmu.r8(addr);
                let (res, flags) = self.sub8(v, 1, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = (flags & !F_C) | (self.state.r8[R_F] & F_C);
                (23, 3)
            }
            0x36 => {
                // LD (IY+d),n
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let n = mmu.r8(self.state.pc.wrapping_add(3));
                mmu.w8(addr, n);
                (19, 4)
            }
            0x44 => {
                // LD B,IYH
                self.state.r8[R_B] = self.state.r8[10];
                (8, 2)
            }
            0x45 => {
                // LD B,IYL
                self.state.r8[R_B] = self.state.r8[11];
                (8, 2)
            }
            0x46 => {
                // LD B,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_B] = mmu.r8(addr);
                (19, 3)
            }
            0x4C => {
                // LD C,IYH
                self.state.r8[R_C] = self.state.r8[10];
                (8, 2)
            }
            0x4D => {
                // LD C,IYL
                self.state.r8[R_C] = self.state.r8[11];
                (8, 2)
            }
            0x4E => {
                // LD C,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_C] = mmu.r8(addr);
                (19, 3)
            }
            0x54 => {
                // LD D,IYH
                self.state.r8[R_D] = self.state.r8[10];
                (8, 2)
            }
            0x55 => {
                // LD D,IYL
                self.state.r8[R_D] = self.state.r8[11];
                (8, 2)
            }
            0x56 => {
                // LD D,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_D] = mmu.r8(addr);
                (19, 3)
            }
            0x5C => {
                // LD E,IYH
                self.state.r8[R_E] = self.state.r8[10];
                (8, 2)
            }
            0x5D => {
                // LD E,IYL
                self.state.r8[R_E] = self.state.r8[11];
                (8, 2)
            }
            0x5E => {
                // LD E,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_E] = mmu.r8(addr);
                (19, 3)
            }
            0x66 => {
                // LD H,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_H] = mmu.r8(addr);
                (19, 3)
            }
            0x6E => {
                // LD L,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_L] = mmu.r8(addr);
                (19, 3)
            }
            0x7C => {
                // LD A,IYH
                self.state.r8[R_A] = self.state.r8[10];
                (8, 2)
            }
            0x7D => {
                // LD A,IYL
                self.state.r8[R_A] = self.state.r8[11];
                (8, 2)
            }
            0x7E => {
                // LD A,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] = mmu.r8(addr);
                (19, 3)
            }
            0x60 => {
                // LD IYH,B
                self.state.r8[10] = self.state.r8[R_B];
                (8, 2)
            }
            0x61 => {
                // LD IYH,C
                self.state.r8[10] = self.state.r8[R_C];
                (8, 2)
            }
            0x62 => {
                // LD IYH,D
                self.state.r8[10] = self.state.r8[R_D];
                (8, 2)
            }
            0x63 => {
                // LD IYH,E
                self.state.r8[10] = self.state.r8[R_E];
                (8, 2)
            }
            0x64 => (8, 2), // LD IYH,IYH
            0x65 => {
                // LD IYH,IYL
                self.state.r8[10] = self.state.r8[11];
                (8, 2)
            }
            0x67 => {
                // LD IYH,A
                self.state.r8[10] = self.state.r8[R_A];
                (8, 2)
            }
            0x68 => {
                // LD IYL,B
                self.state.r8[11] = self.state.r8[R_B];
                (8, 2)
            }
            0x69 => {
                // LD IYL,C
                self.state.r8[11] = self.state.r8[R_C];
                (8, 2)
            }
            0x6A => {
                // LD IYL,D
                self.state.r8[11] = self.state.r8[R_D];
                (8, 2)
            }
            0x6B => {
                // LD IYL,E
                self.state.r8[11] = self.state.r8[R_E];
                (8, 2)
            }
            0x6C => {
                // LD IYL,IYH
                self.state.r8[11] = self.state.r8[10];
                (8, 2)
            }
            0x6D => (8, 2), // LD IYL,IYL
            0x6F => {
                // LD IYL,A
                self.state.r8[11] = self.state.r8[R_A];
                (8, 2)
            }
            0x70 => {
                // LD (IY+d),B
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[2]);
                (19, 3)
            }
            0x71 => {
                // LD (IY+d),C
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[3]);
                (19, 3)
            }
            0x72 => {
                // LD (IY+d),D
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[4]);
                (19, 3)
            }
            0x73 => {
                // LD (IY+d),E
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[5]);
                (19, 3)
            }
            0x74 => {
                // LD (IY+d),H
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[6]);
                (19, 3)
            }
            0x75 => {
                // LD (IY+d),L
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[7]);
                (19, 3)
            }
            0x77 => {
                // LD (IY+d),A
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                mmu.w8(addr, self.state.r8[0]);
                (19, 3)
            }
            0x84 => {
                // ADD A,IYH
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x85 => {
                // ADD A,IYL
                let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x86 => {
                // ADD A,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.add8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x8C => {
                // ADC A,IYH
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
                // ADC A,IYL
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
                // ADC A,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
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
                // SUB IYH
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x95 => {
                // SUB IYL
                let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (8, 2)
            }
            0x96 => {
                // SUB (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let (res, flags) = self.sub8(self.state.r8[R_A], mmu.r8(addr), false);
                self.state.r8[R_A] = res;
                self.state.r8[R_F] = flags;
                (19, 3)
            }
            0x9C => {
                // SBC A,IYH
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
                // SBC A,IYL
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
                // SBC A,(IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
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
                // AND IYH
                self.state.r8[R_A] &= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA5 => {
                // AND IYL
                self.state.r8[R_A] &= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (8, 2)
            }
            0xA6 => {
                // AND (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] &= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H;
                (19, 3)
            }
            0xAC => {
                // XOR IYH
                self.state.r8[R_A] ^= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAD => {
                // XOR IYL
                self.state.r8[R_A] ^= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xAE => {
                // XOR (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] ^= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xB4 => {
                // OR IYH
                self.state.r8[R_A] |= self.state.r8[10];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB5 => {
                // OR IYL
                self.state.r8[R_A] |= self.state.r8[11];
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (8, 2)
            }
            0xB6 => {
                // OR (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                self.state.r8[R_A] |= mmu.r8(addr);
                self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize];
                (19, 3)
            }
            0xBC => {
                // CP IYH
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[10], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[10] & (F_5 | F_3));
                (8, 2)
            }
            0xBD => {
                // CP IYL
                let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[11], false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (self.state.r8[11] & (F_5 | F_3));
                (8, 2)
            }
            0xBE => {
                // CP (IY+d)
                let displ = mmu.r8s(self.state.pc.wrapping_add(2));
                let addr = (self.state.get_reg16(5) as i32 + displ as i32) as u16;
                let val = mmu.r8(addr);
                let (_, flags) = self.sub8(self.state.r8[R_A], val, false);
                self.state.r8[R_F] = (flags & !(F_5 | F_3)) | (val & (F_5 | F_3));
                (19, 3)
            }
            0xE1 => {
                // POP IY
                let val = self.pop16(mmu);
                self.state.set_reg16(5, val);
                (14, 2)
            }
            0xE3 => {
                // EX (SP),IY
                let sp = self.state.sp;
                let memval = mmu.r16(sp);
                mmu.w16reverse(sp, self.state.get_reg16(5));
                self.state.set_reg16(5, memval);
                (23, 2)
            }
            0xE5 => {
                // PUSH IY
                self.push16(mmu, self.state.get_reg16(5));
                (15, 2)
            }
            0xE9 => {
                // JP (IY)
                self.state.pc = self.state.get_reg16(5);
                (8, 0)
            }
            0xF9 => {
                // LD SP,IY
                self.state.sp = self.state.get_reg16(5);
                (10, 2)
            }
            _ => (0, 0),
        }
    }
    pub fn execute_ddcb<M: CpuBus>(&mut self, opcode: u8, displ: i8, mmu: &mut M) -> (u32, u8) {
        let addr = (self.state.get_reg16(4) as i32 + displ as i32) as u16;
        match opcode {
            0x00 => {
                // RLC (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x01 => {
                // RLC (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x02 => {
                // RLC (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x03 => {
                // RLC (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x04 => {
                // RLC (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x05 => {
                // RLC (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x06 => {
                // RLC (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x07 => {
                // RLC (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x08 => {
                // RRC (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x09 => {
                // RRC (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0A => {
                // RRC (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0B => {
                // RRC (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0C => {
                // RRC (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0D => {
                // RRC (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0E => {
                // RRC (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0F => {
                // RRC (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x10 => {
                // RL (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x11 => {
                // RL (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x12 => {
                // RL (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x13 => {
                // RL (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x14 => {
                // RL (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x15 => {
                // RL (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x16 => {
                // RL (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x17 => {
                // RL (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x18 => {
                // RR (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x19 => {
                // RR (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1A => {
                // RR (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1B => {
                // RR (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1C => {
                // RR (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1D => {
                // RR (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1E => {
                // RR (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1F => {
                // RR (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x20 => {
                // SLA (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x21 => {
                // SLA (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x22 => {
                // SLA (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x23 => {
                // SLA (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x24 => {
                // SLA (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x25 => {
                // SLA (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x26 => {
                // SLA (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x27 => {
                // SLA (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x28 => {
                // SRA (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x29 => {
                // SRA (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2A => {
                // SRA (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2B => {
                // SRA (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2C => {
                // SRA (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2D => {
                // SRA (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2E => {
                // SRA (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2F => {
                // SRA (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x30 => {
                // SLL (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x31 => {
                // SLL (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x32 => {
                // SLL (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x33 => {
                // SLL (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x34 => {
                // SLL (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x35 => {
                // SLL (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x36 => {
                // SLL (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x37 => {
                // SLL (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x38 => {
                // SRL (IX+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x39 => {
                // SRL (IX+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3A => {
                // SRL (IX+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3B => {
                // SRL (IX+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3C => {
                // SRL (IX+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3D => {
                // SRL (IX+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3E => {
                // SRL (IX+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3F => {
                // SRL (IX+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x40 => {
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 0,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 1,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 2,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 3,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 4,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 5,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 6,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // BIT 7,(IX+d)
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
                // RES 0,(IX+d),B
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x81 => {
                // RES 0,(IX+d),C
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x82 => {
                // RES 0,(IX+d),D
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x83 => {
                // RES 0,(IX+d),E
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x84 => {
                // RES 0,(IX+d),H
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x85 => {
                // RES 0,(IX+d),L
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x86 => {
                // RES 0,(IX+d)
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x87 => {
                // RES 0,(IX+d),A
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x88 => {
                // RES 1,(IX+d),B
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x89 => {
                // RES 1,(IX+d),C
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8A => {
                // RES 1,(IX+d),D
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8B => {
                // RES 1,(IX+d),E
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8C => {
                // RES 1,(IX+d),H
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8D => {
                // RES 1,(IX+d),L
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8E => {
                // RES 1,(IX+d)
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8F => {
                // RES 1,(IX+d),A
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x90 => {
                // RES 2,(IX+d),B
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x91 => {
                // RES 2,(IX+d),C
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x92 => {
                // RES 2,(IX+d),D
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x93 => {
                // RES 2,(IX+d),E
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x94 => {
                // RES 2,(IX+d),H
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x95 => {
                // RES 2,(IX+d),L
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x96 => {
                // RES 2,(IX+d)
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x97 => {
                // RES 2,(IX+d),A
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x98 => {
                // RES 3,(IX+d),B
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x99 => {
                // RES 3,(IX+d),C
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9A => {
                // RES 3,(IX+d),D
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9B => {
                // RES 3,(IX+d),E
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9C => {
                // RES 3,(IX+d),H
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9D => {
                // RES 3,(IX+d),L
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9E => {
                // RES 3,(IX+d)
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9F => {
                // RES 3,(IX+d),A
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA0 => {
                // RES 4,(IX+d),B
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA1 => {
                // RES 4,(IX+d),C
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA2 => {
                // RES 4,(IX+d),D
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA3 => {
                // RES 4,(IX+d),E
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA4 => {
                // RES 4,(IX+d),H
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA5 => {
                // RES 4,(IX+d),L
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA6 => {
                // RES 4,(IX+d)
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA7 => {
                // RES 4,(IX+d),A
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA8 => {
                // RES 5,(IX+d),B
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA9 => {
                // RES 5,(IX+d),C
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAA => {
                // RES 5,(IX+d),D
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAB => {
                // RES 5,(IX+d),E
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAC => {
                // RES 5,(IX+d),H
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAD => {
                // RES 5,(IX+d),L
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAE => {
                // RES 5,(IX+d)
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAF => {
                // RES 5,(IX+d),A
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB0 => {
                // RES 6,(IX+d),B
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB1 => {
                // RES 6,(IX+d),C
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB2 => {
                // RES 6,(IX+d),D
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB3 => {
                // RES 6,(IX+d),E
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB4 => {
                // RES 6,(IX+d),H
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB5 => {
                // RES 6,(IX+d),L
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB6 => {
                // RES 6,(IX+d)
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB7 => {
                // RES 6,(IX+d),A
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB8 => {
                // RES 7,(IX+d),B
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB9 => {
                // RES 7,(IX+d),C
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBA => {
                // RES 7,(IX+d),D
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBB => {
                // RES 7,(IX+d),E
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBC => {
                // RES 7,(IX+d),H
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBD => {
                // RES 7,(IX+d),L
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBE => {
                // RES 7,(IX+d)
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBF => {
                // RES 7,(IX+d),A
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC0 => {
                // SET 0,(IX+d),B
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC1 => {
                // SET 0,(IX+d),C
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC2 => {
                // SET 0,(IX+d),D
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC3 => {
                // SET 0,(IX+d),E
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC4 => {
                // SET 0,(IX+d),H
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC5 => {
                // SET 0,(IX+d),L
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC6 => {
                // SET 0,(IX+d)
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC7 => {
                // SET 0,(IX+d),A
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC8 => {
                // SET 1,(IX+d),B
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC9 => {
                // SET 1,(IX+d),C
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCA => {
                // SET 1,(IX+d),D
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCB => {
                // SET 1,(IX+d),E
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCC => {
                // SET 1,(IX+d),H
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCD => {
                // SET 1,(IX+d),L
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCE => {
                // SET 1,(IX+d)
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCF => {
                // SET 1,(IX+d),A
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD0 => {
                // SET 2,(IX+d),B
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD1 => {
                // SET 2,(IX+d),C
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD2 => {
                // SET 2,(IX+d),D
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD3 => {
                // SET 2,(IX+d),E
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD4 => {
                // SET 2,(IX+d),H
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD5 => {
                // SET 2,(IX+d),L
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD6 => {
                // SET 2,(IX+d)
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD7 => {
                // SET 2,(IX+d),A
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD8 => {
                // SET 3,(IX+d),B
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD9 => {
                // SET 3,(IX+d),C
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDA => {
                // SET 3,(IX+d),D
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDB => {
                // SET 3,(IX+d),E
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDC => {
                // SET 3,(IX+d),H
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDD => {
                // SET 3,(IX+d),L
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDE => {
                // SET 3,(IX+d)
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDF => {
                // SET 3,(IX+d),A
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE0 => {
                // SET 4,(IX+d),B
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE1 => {
                // SET 4,(IX+d),C
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE2 => {
                // SET 4,(IX+d),D
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE3 => {
                // SET 4,(IX+d),E
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE4 => {
                // SET 4,(IX+d),H
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE5 => {
                // SET 4,(IX+d),L
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE6 => {
                // SET 4,(IX+d)
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE7 => {
                // SET 4,(IX+d),A
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE8 => {
                // SET 5,(IX+d),B
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE9 => {
                // SET 5,(IX+d),C
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEA => {
                // SET 5,(IX+d),D
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEB => {
                // SET 5,(IX+d),E
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEC => {
                // SET 5,(IX+d),H
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xED => {
                // SET 5,(IX+d),L
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEE => {
                // SET 5,(IX+d)
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEF => {
                // SET 5,(IX+d),A
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF0 => {
                // SET 6,(IX+d),B
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF1 => {
                // SET 6,(IX+d),C
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF2 => {
                // SET 6,(IX+d),D
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF3 => {
                // SET 6,(IX+d),E
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF4 => {
                // SET 6,(IX+d),H
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF5 => {
                // SET 6,(IX+d),L
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF6 => {
                // SET 6,(IX+d)
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF7 => {
                // SET 6,(IX+d),A
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF8 => {
                // SET 7,(IX+d),B
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF9 => {
                // SET 7,(IX+d),C
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFA => {
                // SET 7,(IX+d),D
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFB => {
                // SET 7,(IX+d),E
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFC => {
                // SET 7,(IX+d),H
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFD => {
                // SET 7,(IX+d),L
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFE => {
                // SET 7,(IX+d)
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFF => {
                // SET 7,(IX+d),A
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
                // RLC (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x01 => {
                // RLC (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x02 => {
                // RLC (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x03 => {
                // RLC (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x04 => {
                // RLC (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x05 => {
                // RLC (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x06 => {
                // RLC (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x07 => {
                // RLC (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x08 => {
                // RRC (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x09 => {
                // RRC (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0A => {
                // RRC (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0B => {
                // RRC (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0C => {
                // RRC (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0D => {
                // RRC (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0E => {
                // RRC (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x0F => {
                // RRC (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x01) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x10 => {
                // RL (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x11 => {
                // RL (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x12 => {
                // RL (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x13 => {
                // RL (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x14 => {
                // RL (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x15 => {
                // RL (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x16 => {
                // RL (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x17 => {
                // RL (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x18 => {
                // RR (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x19 => {
                // RR (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1A => {
                // RR (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1B => {
                // RR (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1C => {
                // RR (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1D => {
                // RR (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1E => {
                // RR (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x1F => {
                // RR (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (self.state.r8[R_F] & F_C) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x20 => {
                // SLA (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x21 => {
                // SLA (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x22 => {
                // SLA (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x23 => {
                // SLA (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x24 => {
                // SLA (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x25 => {
                // SLA (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x26 => {
                // SLA (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x27 => {
                // SLA (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x28 => {
                // SRA (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x29 => {
                // SRA (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2A => {
                // SRA (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2B => {
                // SRA (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2C => {
                // SRA (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2D => {
                // SRA (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2E => {
                // SRA (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x2F => {
                // SRA (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, (val & 0x80) != 0);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x30 => {
                // SLL (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x31 => {
                // SLL (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x32 => {
                // SLL (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x33 => {
                // SLL (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x34 => {
                // SLL (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x35 => {
                // SLL (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x36 => {
                // SLL (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x37 => {
                // SLL (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shl8(val, true);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x38 => {
                // SRL (IY+d),B
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[2] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x39 => {
                // SRL (IY+d),C
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[3] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3A => {
                // SRL (IY+d),D
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[4] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3B => {
                // SRL (IY+d),E
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[5] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3C => {
                // SRL (IY+d),H
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[6] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3D => {
                // SRL (IY+d),L
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[7] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3E => {
                // SRL (IY+d)
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x3F => {
                // SRL (IY+d),A
                let val = mmu.r8(addr);
                let (res, flags) = self.shr8(val, false);
                self.state.r8[0] = res;
                mmu.w8(addr, res);
                self.state.r8[R_F] = flags;
                (23, 4)
            }
            0x40 => {
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 0,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 1,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 2,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 3,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 4,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 5,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 6,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // BIT 7,(IY+d)
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
                // RES 0,(IY+d),B
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x81 => {
                // RES 0,(IY+d),C
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x82 => {
                // RES 0,(IY+d),D
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x83 => {
                // RES 0,(IY+d),E
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x84 => {
                // RES 0,(IY+d),H
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x85 => {
                // RES 0,(IY+d),L
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x86 => {
                // RES 0,(IY+d)
                let val = mmu.r8(addr) & 0xFE;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x87 => {
                // RES 0,(IY+d),A
                let val = mmu.r8(addr) & 0xFE;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x88 => {
                // RES 1,(IY+d),B
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x89 => {
                // RES 1,(IY+d),C
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8A => {
                // RES 1,(IY+d),D
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8B => {
                // RES 1,(IY+d),E
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8C => {
                // RES 1,(IY+d),H
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8D => {
                // RES 1,(IY+d),L
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8E => {
                // RES 1,(IY+d)
                let val = mmu.r8(addr) & 0xFD;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x8F => {
                // RES 1,(IY+d),A
                let val = mmu.r8(addr) & 0xFD;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x90 => {
                // RES 2,(IY+d),B
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x91 => {
                // RES 2,(IY+d),C
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x92 => {
                // RES 2,(IY+d),D
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x93 => {
                // RES 2,(IY+d),E
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x94 => {
                // RES 2,(IY+d),H
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x95 => {
                // RES 2,(IY+d),L
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x96 => {
                // RES 2,(IY+d)
                let val = mmu.r8(addr) & 0xFB;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x97 => {
                // RES 2,(IY+d),A
                let val = mmu.r8(addr) & 0xFB;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x98 => {
                // RES 3,(IY+d),B
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x99 => {
                // RES 3,(IY+d),C
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9A => {
                // RES 3,(IY+d),D
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9B => {
                // RES 3,(IY+d),E
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9C => {
                // RES 3,(IY+d),H
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9D => {
                // RES 3,(IY+d),L
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9E => {
                // RES 3,(IY+d)
                let val = mmu.r8(addr) & 0xF7;
                mmu.w8(addr, val);
                (23, 4)
            }
            0x9F => {
                // RES 3,(IY+d),A
                let val = mmu.r8(addr) & 0xF7;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA0 => {
                // RES 4,(IY+d),B
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA1 => {
                // RES 4,(IY+d),C
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA2 => {
                // RES 4,(IY+d),D
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA3 => {
                // RES 4,(IY+d),E
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA4 => {
                // RES 4,(IY+d),H
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA5 => {
                // RES 4,(IY+d),L
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA6 => {
                // RES 4,(IY+d)
                let val = mmu.r8(addr) & 0xEF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA7 => {
                // RES 4,(IY+d),A
                let val = mmu.r8(addr) & 0xEF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA8 => {
                // RES 5,(IY+d),B
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xA9 => {
                // RES 5,(IY+d),C
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAA => {
                // RES 5,(IY+d),D
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAB => {
                // RES 5,(IY+d),E
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAC => {
                // RES 5,(IY+d),H
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAD => {
                // RES 5,(IY+d),L
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAE => {
                // RES 5,(IY+d)
                let val = mmu.r8(addr) & 0xDF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xAF => {
                // RES 5,(IY+d),A
                let val = mmu.r8(addr) & 0xDF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB0 => {
                // RES 6,(IY+d),B
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB1 => {
                // RES 6,(IY+d),C
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB2 => {
                // RES 6,(IY+d),D
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB3 => {
                // RES 6,(IY+d),E
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB4 => {
                // RES 6,(IY+d),H
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB5 => {
                // RES 6,(IY+d),L
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB6 => {
                // RES 6,(IY+d)
                let val = mmu.r8(addr) & 0xBF;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB7 => {
                // RES 6,(IY+d),A
                let val = mmu.r8(addr) & 0xBF;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB8 => {
                // RES 7,(IY+d),B
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xB9 => {
                // RES 7,(IY+d),C
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBA => {
                // RES 7,(IY+d),D
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBB => {
                // RES 7,(IY+d),E
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBC => {
                // RES 7,(IY+d),H
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBD => {
                // RES 7,(IY+d),L
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBE => {
                // RES 7,(IY+d)
                let val = mmu.r8(addr) & 0x7F;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xBF => {
                // RES 7,(IY+d),A
                let val = mmu.r8(addr) & 0x7F;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC0 => {
                // SET 0,(IY+d),B
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC1 => {
                // SET 0,(IY+d),C
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC2 => {
                // SET 0,(IY+d),D
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC3 => {
                // SET 0,(IY+d),E
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC4 => {
                // SET 0,(IY+d),H
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC5 => {
                // SET 0,(IY+d),L
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC6 => {
                // SET 0,(IY+d)
                let val = mmu.r8(addr) | 0x01;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC7 => {
                // SET 0,(IY+d),A
                let val = mmu.r8(addr) | 0x01;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC8 => {
                // SET 1,(IY+d),B
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xC9 => {
                // SET 1,(IY+d),C
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCA => {
                // SET 1,(IY+d),D
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCB => {
                // SET 1,(IY+d),E
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCC => {
                // SET 1,(IY+d),H
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCD => {
                // SET 1,(IY+d),L
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCE => {
                // SET 1,(IY+d)
                let val = mmu.r8(addr) | 0x02;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xCF => {
                // SET 1,(IY+d),A
                let val = mmu.r8(addr) | 0x02;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD0 => {
                // SET 2,(IY+d),B
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD1 => {
                // SET 2,(IY+d),C
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD2 => {
                // SET 2,(IY+d),D
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD3 => {
                // SET 2,(IY+d),E
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD4 => {
                // SET 2,(IY+d),H
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD5 => {
                // SET 2,(IY+d),L
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD6 => {
                // SET 2,(IY+d)
                let val = mmu.r8(addr) | 0x04;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD7 => {
                // SET 2,(IY+d),A
                let val = mmu.r8(addr) | 0x04;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD8 => {
                // SET 3,(IY+d),B
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xD9 => {
                // SET 3,(IY+d),C
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDA => {
                // SET 3,(IY+d),D
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDB => {
                // SET 3,(IY+d),E
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDC => {
                // SET 3,(IY+d),H
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDD => {
                // SET 3,(IY+d),L
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDE => {
                // SET 3,(IY+d)
                let val = mmu.r8(addr) | 0x08;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xDF => {
                // SET 3,(IY+d),A
                let val = mmu.r8(addr) | 0x08;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE0 => {
                // SET 4,(IY+d),B
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE1 => {
                // SET 4,(IY+d),C
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE2 => {
                // SET 4,(IY+d),D
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE3 => {
                // SET 4,(IY+d),E
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE4 => {
                // SET 4,(IY+d),H
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE5 => {
                // SET 4,(IY+d),L
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE6 => {
                // SET 4,(IY+d)
                let val = mmu.r8(addr) | 0x10;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE7 => {
                // SET 4,(IY+d),A
                let val = mmu.r8(addr) | 0x10;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE8 => {
                // SET 5,(IY+d),B
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xE9 => {
                // SET 5,(IY+d),C
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEA => {
                // SET 5,(IY+d),D
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEB => {
                // SET 5,(IY+d),E
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEC => {
                // SET 5,(IY+d),H
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xED => {
                // SET 5,(IY+d),L
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEE => {
                // SET 5,(IY+d)
                let val = mmu.r8(addr) | 0x20;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xEF => {
                // SET 5,(IY+d),A
                let val = mmu.r8(addr) | 0x20;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF0 => {
                // SET 6,(IY+d),B
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF1 => {
                // SET 6,(IY+d),C
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF2 => {
                // SET 6,(IY+d),D
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF3 => {
                // SET 6,(IY+d),E
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF4 => {
                // SET 6,(IY+d),H
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF5 => {
                // SET 6,(IY+d),L
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF6 => {
                // SET 6,(IY+d)
                let val = mmu.r8(addr) | 0x40;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF7 => {
                // SET 6,(IY+d),A
                let val = mmu.r8(addr) | 0x40;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF8 => {
                // SET 7,(IY+d),B
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[2] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xF9 => {
                // SET 7,(IY+d),C
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[3] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFA => {
                // SET 7,(IY+d),D
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[4] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFB => {
                // SET 7,(IY+d),E
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[5] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFC => {
                // SET 7,(IY+d),H
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[6] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFD => {
                // SET 7,(IY+d),L
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[7] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFE => {
                // SET 7,(IY+d)
                let val = mmu.r8(addr) | 0x80;
                mmu.w8(addr, val);
                (23, 4)
            }
            0xFF => {
                // SET 7,(IY+d),A
                let val = mmu.r8(addr) | 0x80;
                self.state.r8[0] = val;
                mmu.w8(addr, val);
                (23, 4)
            }
        }
    }
}

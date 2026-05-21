#!/usr/bin/env python3
"""Generate Rust Z80 opcode implementations from JS reference."""

# This script generates Rust match arms for Z80 opcodes based on patterns.
# It outputs code that should be inserted into z80.rs.

def generate_base_opcodes():
    """Generate execute_base match arms for opcodes 0x00-0xFF."""
    lines = []
    
    # Opcode patterns based on JS implementation
    for opcode in range(256):
        arm = generate_base_arm(opcode)
        lines.append(f"        0x{opcode:02X} => {arm}")
    
    return "\n".join(lines)

def generate_base_arm(opcode):
    """Generate a single base opcode arm."""
    
    # NOP
    if opcode == 0x00:
        return "{ (4, 1) }"
    
    # LD BC,nn; DE,nn; HL,nn; SP,nn
    if opcode in (0x01, 0x11, 0x21, 0x31):
        reg = {0x01: "R_BC", 0x11: "R_DE", 0x21: "R_HL", 0x31: "R_SP"}[opcode]
        return f"{{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); self.state.set_reg16({reg}, nn); (10, 3) }}"
    
    # LD (BC),A; LD (DE),A
    if opcode == 0x02:
        return "{ let addr = self.state.get_reg16(R_BC); mmu.w8(addr, self.state.r8[R_A]); (7, 1) }"
    if opcode == 0x12:
        return "{ let addr = self.state.get_reg16(R_DE); mmu.w8(addr, self.state.r8[R_A]); (7, 1) }"
    
    # LD (nn),HL; LD (nn),A
    if opcode == 0x22:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); mmu.w16(nn, self.state.get_reg16(R_HL)); (16, 3) }"
    if opcode == 0x32:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); mmu.w8(nn, self.state.r8[R_A]); (13, 3) }"
    
    # LD A,(BC); LD A,(DE); LD HL,(nn); LD A,(nn)
    if opcode == 0x0A:
        return "{ let addr = self.state.get_reg16(R_BC); self.state.r8[R_A] = mmu.r8(addr); (7, 1) }"
    if opcode == 0x1A:
        return "{ let addr = self.state.get_reg16(R_DE); self.state.r8[R_A] = mmu.r8(addr); (7, 1) }"
    if opcode == 0x2A:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); self.state.set_reg16(R_HL, mmu.r16(nn)); (16, 3) }"
    if opcode == 0x3A:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); self.state.r8[R_A] = mmu.r8(nn); (13, 3) }"
    
    # INC ss: BC, DE, HL, SP
    if opcode in (0x03, 0x13, 0x23, 0x33):
        reg = {0x03: "R_BC", 0x13: "R_DE", 0x23: "R_HL", 0x33: "R_SP"}[opcode]
        return f"{{ self.state.set_reg16({reg}, self.state.get_reg16({reg}).wrapping_add(1)); ({6 + (4 if reg == 'R_SP' else 0)}, 1) }}".replace("(6 + (4 if reg == 'R_SP' else 0))", "6" if reg != "R_SP" else "6")
    
    if opcode == 0x03:
        return "{ self.state.set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_add(1)); (6, 1) }"
    if opcode == 0x13:
        return "{ self.state.set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_add(1)); (6, 1) }"
    if opcode == 0x23:
        return "{ self.state.set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_add(1)); (6, 1) }"
    if opcode == 0x33:
        return "{ self.state.r16[R_SP] = self.state.r16[R_SP].wrapping_add(1); (6, 1) }"
    
    # DEC ss
    if opcode == 0x0B:
        return "{ self.state.set_reg16(R_BC, self.state.get_reg16(R_BC).wrapping_sub(1)); (6, 1) }"
    if opcode == 0x1B:
        return "{ self.state.set_reg16(R_DE, self.state.get_reg16(R_DE).wrapping_sub(1)); (6, 1) }"
    if opcode == 0x2B:
        return "{ self.state.set_reg16(R_HL, self.state.get_reg16(R_HL).wrapping_sub(1)); (6, 1) }"
    if opcode == 0x3B:
        return "{ self.state.r16[R_SP] = self.state.r16[R_SP].wrapping_sub(1); (6, 1) }"
    
    # INC r
    inc_regs = {0x04: R_B, 0x0C: R_C, 0x14: R_D, 0x1C: R_E, 0x24: R_H, 0x2C: R_L, 0x3C: R_A}
    if opcode in inc_regs:
        reg = inc_regs[opcode]
        return f"{{ let (res, mut flags) = self.add8(self.state.r8[{reg}], 1, false); let mask = F_C; self.state.r8[{reg}] = res; self.state.r8[R_F] = (flags & !mask) | (self.state.r8[R_F] & mask); (4, 1) }}"
    
    # INC (HL)
    if opcode == 0x34:
        return "{ let addr = self.state.get_reg16(R_HL); let (res, mut flags) = self.add8(mmu.r8(addr), 1, false); mmu.w8(addr, res); let mask = F_C; self.state.r8[R_F] = (flags & !mask) | (self.state.r8[R_F] & mask); (11, 1) }"
    
    # DEC r
    dec_regs = {0x05: R_B, 0x0D: R_C, 0x15: R_D, 0x1D: R_E, 0x25: R_H, 0x2D: R_L, 0x3D: R_A}
    if opcode in dec_regs:
        reg = dec_regs[opcode]
        return f"{{ let (res, mut flags) = self.sub8(self.state.r8[{reg}], 1, false); let mask = F_C; self.state.r8[{reg}] = res; self.state.r8[R_F] = (flags & !mask) | (self.state.r8[R_F] & mask); (4, 1) }}"
    
    # DEC (HL)
    if opcode == 0x35:
        return "{ let addr = self.state.get_reg16(R_HL); let (res, mut flags) = self.sub8(mmu.r8(addr), 1, false); mmu.w8(addr, res); let mask = F_C; self.state.r8[R_F] = (flags & !mask) | (self.state.r8[R_F] & mask); (11, 1) }"
    
    # LD r,n
    ld_r_n_regs = {0x06: R_B, 0x0E: R_C, 0x16: R_D, 0x1E: R_E, 0x26: R_H, 0x2E: R_L, 0x3E: R_A}
    if opcode in ld_r_n_regs:
        reg = ld_r_n_regs[opcode]
        return f"{{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); self.state.r8[{reg}] = n; (7, 2) }}"
    
    # LD (HL),n
    if opcode == 0x36:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); mmu.w8(self.state.get_reg16(R_HL), n); (10, 2) }"
    
    # RLCA
    if opcode == 0x07:
        return "{ let a = self.state.r8[R_A]; let (res, flags) = self.shl8(a, (a & 0x80) != 0); self.state.r8[R_A] = res; let mask = F_S | F_Z | F_PV; self.state.r8[R_F] = (self.state.r8[R_F] & mask) | (flags & !mask); (4, 1) }"
    
    # RRCA
    if opcode == 0x0F:
        return "{ let a = self.state.r8[R_A]; let (res, flags) = self.shr8(a, (a & 0x01) != 0); self.state.r8[R_A] = res; let mask = F_S | F_Z | F_PV; self.state.r8[R_F] = (self.state.r8[R_F] & mask) | (flags & !mask); (4, 1) }"
    
    # RLA
    if opcode == 0x17:
        return "{ let a = self.state.r8[R_A]; let (res, flags) = self.shl8(a, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; let mask = F_S | F_Z | F_PV; self.state.r8[R_F] = (self.state.r8[R_F] & mask) | (flags & !mask); (4, 1) }"
    
    # RRA
    if opcode == 0x1F:
        return "{ let a = self.state.r8[R_A]; let (res, flags) = self.shr8(a, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; let mask = F_S | F_Z | F_PV; self.state.r8[R_F] = (self.state.r8[R_F] & mask) | (flags & !mask); (4, 1) }"
    
    # DJNZ
    if opcode == 0x10:
        return "{ let pc = self.state.r16[R_PC]; self.state.r8[R_B] = self.state.r8[R_B].wrapping_sub(1); if self.state.r8[R_B] == 0 { (8, 2) } else { let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (13, 0) } }"
    
    # JR e
    if opcode == 0x18:
        return "{ let pc = self.state.r16[R_PC]; let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (12, 0) }"
    
    # JR NZ,e; JR Z,e; JR NC,e; JR C,e
    if opcode == 0x20:
        return "{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & F_Z != 0 { (8, 2) } else { let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (12, 0) } }"
    if opcode == 0x28:
        return "{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & F_Z != 0 { let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (12, 0) } else { (8, 2) } }"
    if opcode == 0x30:
        return "{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & F_C != 0 { (8, 2) } else { let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (12, 0) } }"
    if opcode == 0x38:
        return "{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & F_C != 0 { let e = mmu.r8s(pc + 1) as i16; self.state.r16[R_PC] = (pc as i16 + 2 + e) as u16; (12, 0) } else { (8, 2) } }"
    
    # DAA
    if opcode == 0x27:
        return "{ let a = self.state.r8[R_A]; let mut add = 0; let carry = self.state.r8[R_F] & F_C; let lownibble = a & 0x0F; if (self.state.r8[R_F] & F_H) != 0 || lownibble > 9 { add = 6; } let mut new_carry = carry; if carry != 0 || a > 0x99 { add |= 0x60; new_carry = F_C; } let (res, mut flags) = if self.state.r8[R_F] & F_N != 0 { self.sub8(a, add, false) } else { self.add8(a, add, false) }; self.state.r8[R_A] = res; self.state.r8[R_F] = (self.state.r8[R_F] & F_N) | self.sz53p_table[res as usize] | (flags & F_H) | new_carry; (4, 1) }"
    
    # CPL
    if opcode == 0x2F:
        return "{ self.state.r8[R_A] = !self.state.r8[R_A]; self.state.r8[R_F] = (self.state.r8[R_F] & (F_S|F_Z|F_PV|F_C)) | F_H | F_N | (self.state.r8[R_A] & F_5) | (self.state.r8[R_A] & F_3); (4, 1) }"
    
    # SCF
    if opcode == 0x37:
        return "{ self.state.r8[R_F] = (self.state.r8[R_F] & (F_S|F_Z|F_PV)) | (self.state.r8[R_A] & F_5) | (self.state.r8[R_A] & F_3) | F_C; (4, 1) }"
    
    # CCF
    if opcode == 0x3F:
        return "{ let cf = self.state.r8[R_F] & F_C; self.state.r8[R_F] = (self.state.r8[R_F] & (F_S|F_Z|F_PV)) | (self.state.r8[R_A] & F_5) | (self.state.r8[R_A] & F_3) | (cf << 4) | (cf ^ F_C); (4, 1) }"
    
    # EX AF,AF'
    if opcode == 0x08:
        return "{ let a = self.state.r8[R_A]; self.state.r8[R_A] = self.state.r8[R_Aa]; self.state.r8[R_Aa] = a; let f = self.state.r8[R_F]; self.state.r8[R_F] = self.state.r8[R_Fa]; self.state.r8[R_Fa] = f; (4, 1) }"
    
    # ADD HL,ss
    add_hl_regs = {0x09: "R_BC", 0x19: "R_DE", 0x29: "R_HL", 0x39: "R_SP"}
    if opcode in add_hl_regs:
        reg = add_hl_regs[opcode]
        return f"{{ let hl = self.state.get_reg16(R_HL); let ss = {'self.state.r16[R_SP]' if reg == 'R_SP' else f'self.state.get_reg16({reg})'}; let (res, flags) = self.add16(hl, ss, false); self.state.set_reg16(R_HL, res); let mask = F_S | F_Z | F_PV; self.state.r8[R_F] = (flags & !mask) | (self.state.r8[R_F] & mask); (11, 1) }}"
    
    # HALT
    if opcode == 0x76:
        return "{ self.state.halted = 1; (4, 1) }"
    
    # LD r,r' (0x40-0x7F)
    if 0x40 <= opcode <= 0x7F:
        if opcode == 0x76:
            return "{ self.state.halted = 1; (4, 1) }"  # HALT
        src = (opcode & 0x07)
        dst = (opcode >> 3) & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        dst_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        dname = dst_names[dst]
        if sname == "HL" and dname == "HL":
            return "{ (4, 1) }"  # LD (HL),(HL) - actually HALT is 0x76, this is 0x66? No wait.
        if sname == "HL":
            return f"{{ self.state.r8[{dname}] = mmu.r8(self.state.get_reg16(R_HL)); (7, 1) }}"
        if dname == "HL":
            return f"{{ mmu.w8(self.state.get_reg16(R_HL), self.state.r8[{sname}]); (7, 1) }}"
        return f"{{ self.state.r8[{dname}] = self.state.r8[{sname}]; (4, 1) }}"
    
    # ADD A,r (0x80-0x87)
    if 0x80 <= opcode <= 0x87:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ let val = mmu.r8(self.state.get_reg16(R_HL)); let (res, flags) = self.add8(self.state.r8[R_A], val, false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 1) }"
        return f"{{ let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[{sname}], false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (4, 1) }}"
    
    # ADC A,r (0x88-0x8F)
    if 0x88 <= opcode <= 0x8F:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ let val = mmu.r8(self.state.get_reg16(R_HL)); let (res, flags) = self.add8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 1) }"
        return f"{{ let (res, flags) = self.add8(self.state.r8[R_A], self.state.r8[{sname}], (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (4, 1) }}"
    
    # SUB r (0x90-0x97)
    if 0x90 <= opcode <= 0x97:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ let val = mmu.r8(self.state.get_reg16(R_HL)); let (res, flags) = self.sub8(self.state.r8[R_A], val, false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 1) }"
        return f"{{ let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[{sname}], false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (4, 1) }}"
    
    # SBC A,r (0x98-0x9F)
    if 0x98 <= opcode <= 0x9F:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ let val = mmu.r8(self.state.get_reg16(R_HL)); let (res, flags) = self.sub8(self.state.r8[R_A], val, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 1) }"
        return f"{{ let (res, flags) = self.sub8(self.state.r8[R_A], self.state.r8[{sname}], (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (4, 1) }}"
    
    # AND r (0xA0-0xA7)
    if 0xA0 <= opcode <= 0xA7:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ self.state.r8[R_A] &= mmu.r8(self.state.get_reg16(R_HL)); self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H; (7, 1) }"
        return f"{{ self.state.r8[R_A] &= self.state.r8[{sname}]; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H; (4, 1) }}"
    
    # XOR r (0xA8-0xAF)
    if 0xA8 <= opcode <= 0xAF:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ self.state.r8[R_A] ^= mmu.r8(self.state.get_reg16(R_HL)); self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (7, 1) }"
        return f"{{ self.state.r8[R_A] ^= self.state.r8[{sname}]; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (4, 1) }}"
    
    # OR r (0xB0-0xB7)
    if 0xB0 <= opcode <= 0xB7:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ self.state.r8[R_A] |= mmu.r8(self.state.get_reg16(R_HL)); self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (7, 1) }"
        return f"{{ self.state.r8[R_A] |= self.state.r8[{sname}]; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (4, 1) }}"
    
    # CP r (0xB8-0xBF)
    if 0xB8 <= opcode <= 0xBF:
        src = opcode & 0x07
        src_names = ["R_B", "R_C", "R_D", "R_E", "R_H", "R_L", "HL", "R_A"]
        sname = src_names[src]
        if sname == "HL":
            return "{ let val = mmu.r8(self.state.get_reg16(R_HL)); let (_, flags) = self.sub8(self.state.r8[R_A], val, false); self.state.r8[R_F] = (flags & !(F_5|F_3)) | (val & (F_5|F_3)); (7, 1) }"
        return f"{{ let (_, flags) = self.sub8(self.state.r8[R_A], self.state.r8[{sname}], false); self.state.r8[R_F] = (flags & !(F_5|F_3)) | (self.state.r8[{sname}] & (F_5|F_3)); (4, 1) }}"
    
    # RET conditionals and RET
    if opcode == 0xC0:
        return "{ if self.state.r8[R_F] & F_Z != 0 { (5, 1) } else { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } }"
    if opcode == 0xC8:
        return "{ if self.state.r8[R_F] & F_Z != 0 { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } else { (5, 1) } }"
    if opcode == 0xC9:
        return "{ let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (10, 0) }"
    if opcode == 0xD0:
        return "{ if self.state.r8[R_F] & F_C != 0 { (5, 1) } else { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } }"
    if opcode == 0xD8:
        return "{ if self.state.r8[R_F] & F_C != 0 { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } else { (5, 1) } }"
    if opcode == 0xE0:
        return "{ if self.state.r8[R_F] & F_PV != 0 { (5, 1) } else { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } }"
    if opcode == 0xE8:
        return "{ if self.state.r8[R_F] & F_PV != 0 { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } else { (5, 1) } }"
    if opcode == 0xF0:
        return "{ if self.state.r8[R_F] & F_S != 0 { (5, 1) } else { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } }"
    if opcode == 0xF8:
        return "{ if self.state.r8[R_F] & F_S != 0 { let addr = self.pop16(mmu); self.state.r16[R_PC] = addr; (11, 0) } else { (5, 1) } }"
    
    # POP rr
    if opcode == 0xC1:
        return "{ let val = self.pop16(mmu); self.state.set_reg16(R_BC, val); (10, 1) }"
    if opcode == 0xD1:
        return "{ let val = self.pop16(mmu); self.state.set_reg16(R_DE, val); (10, 1) }"
    if opcode == 0xE1:
        return "{ let val = self.pop16(mmu); self.state.set_reg16(R_HL, val); (10, 1) }"
    if opcode == 0xF1:
        return "{ let val = self.pop16(mmu); self.state.set_reg16(R_AF, val); (10, 1) }"
    
    # PUSH rr
    if opcode == 0xC5:
        return "{ self.push16(mmu, self.state.get_reg16(R_BC)); (11, 1) }"
    if opcode == 0xD5:
        return "{ self.push16(mmu, self.state.get_reg16(R_DE)); (11, 1) }"
    if opcode == 0xE5:
        return "{ self.push16(mmu, self.state.get_reg16(R_HL)); (11, 1) }"
    if opcode == 0xF5:
        return "{ self.push16(mmu, self.state.get_reg16(R_AF)); (11, 1) }"
    
    # JP conditionals and JP
    jp_cond = {
        0xC2: ("F_Z", False, 3, 10), 0xCA: ("F_Z", True, 0, 10),
        0xD2: ("F_C", False, 3, 10), 0xDA: ("F_C", True, 0, 10),
        0xE2: ("F_PV", False, 3, 10), 0xEA: ("F_PV", True, 0, 10),
        0xF2: ("F_S", False, 3, 10), 0xFA: ("F_S", True, 0, 10),
    }
    if opcode in jp_cond:
        flag, cond_set, m_nojump, t = jp_cond[opcode]
        if cond_set:
            return f"{{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & {flag} != 0 {{ let nn = mmu.r16(pc + 1); self.state.r16[R_PC] = nn; ({t}, 0) }} else {{ mmu.r16nolog(pc + 1); ({t}, {m_nojump}) }} }}"
        else:
            return f"{{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & {flag} != 0 {{ mmu.r16nolog(pc + 1); ({t}, {m_nojump}) }} else {{ let nn = mmu.r16(pc + 1); self.state.r16[R_PC] = nn; ({t}, 0) }} }}"
    
    if opcode == 0xC3:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); self.state.r16[R_PC] = nn; (10, 0) }"
    
    # JP (HL)
    if opcode == 0xE9:
        return "{ self.state.r16[R_PC] = self.state.get_reg16(R_HL); (4, 0) }"
    
    # CALL conditionals and CALL
    call_cond = {
        0xC4: ("F_Z", False, 3, 10, 17), 0xCC: ("F_Z", True, 0, 10, 17),
        0xD4: ("F_C", False, 3, 10, 17), 0xDC: ("F_C", True, 0, 10, 17),
        0xE4: ("F_PV", False, 3, 10, 17), 0xEC: ("F_PV", True, 0, 10, 17),
        0xF4: ("F_S", False, 3, 10, 17), 0xFC: ("F_S", True, 0, 10, 17),
    }
    if opcode in call_cond:
        flag, cond_set, m_nojump, t_nojump, t_jump = call_cond[opcode]
        if cond_set:
            return f"{{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & {flag} != 0 {{ let nn = mmu.r16(pc + 1); self.push16(mmu, pc + 3); self.state.r16[R_PC] = nn; ({t_jump}, 0) }} else {{ mmu.r16nolog(pc + 1); ({t_nojump}, {m_nojump}) }} }}"
        else:
            return f"{{ let pc = self.state.r16[R_PC]; if self.state.r8[R_F] & {flag} != 0 {{ mmu.r16nolog(pc + 1); ({t_nojump}, {m_nojump}) }} else {{ let nn = mmu.r16(pc + 1); self.push16(mmu, pc + 3); self.state.r16[R_PC] = nn; ({t_jump}, 0) }} }}"
    
    if opcode == 0xCD:
        return "{ let pc = self.state.r16[R_PC]; let nn = mmu.r16(pc + 1); self.push16(mmu, pc + 3); self.state.r16[R_PC] = nn; (17, 0) }"
    
    # RST
    rst_addrs = {0xC7: 0x00, 0xCF: 0x08, 0xD7: 0x10, 0xDF: 0x18, 0xE7: 0x20, 0xEF: 0x28, 0xF7: 0x30, 0xFF: 0x38}
    if opcode in rst_addrs:
        addr = rst_addrs[opcode]
        return f"{{ let pc = self.state.r16[R_PC]; self.push16(mmu, pc + 1); self.state.r16[R_PC] = {addr}; (11, 0) }}"
    
    # ALU with immediate
    if opcode == 0xC6:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); let (res, flags) = self.add8(self.state.r8[R_A], n, false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 2) }"
    if opcode == 0xCE:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); let (res, flags) = self.add8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 2) }"
    if opcode == 0xD6:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); let (res, flags) = self.sub8(self.state.r8[R_A], n, false); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 2) }"
    if opcode == 0xDE:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); let (res, flags) = self.sub8(self.state.r8[R_A], n, (self.state.r8[R_F] & F_C) != 0); self.state.r8[R_A] = res; self.state.r8[R_F] = flags; (7, 2) }"
    if opcode == 0xE6:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); self.state.r8[R_A] &= n; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize] | F_H; (7, 2) }"
    if opcode == 0xEE:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); self.state.r8[R_A] ^= n; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (7, 2) }"
    if opcode == 0xF6:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); self.state.r8[R_A] |= n; self.state.r8[R_F] = self.sz53p_table[self.state.r8[R_A] as usize]; (7, 2) }"
    if opcode == 0xFE:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); let (_, flags) = self.sub8(self.state.r8[R_A], n, false); self.state.r8[R_F] = (flags & !(F_5|F_3)) | (n & (F_5|F_3)); (7, 2) }"
    
    # EXX
    if opcode == 0xD9:
        return "{ let b = self.state.r8[R_B]; self.state.r8[R_B] = self.state.r8[R_Ba]; self.state.r8[R_Ba] = b; let c = self.state.r8[R_C]; self.state.r8[R_C] = self.state.r8[R_Ca]; self.state.r8[R_Ca] = c; let d = self.state.r8[R_D]; self.state.r8[R_D] = self.state.r8[R_Da]; self.state.r8[R_Da] = d; let e = self.state.r8[R_E]; self.state.r8[R_E] = self.state.r8[R_Ea]; self.state.r8[R_Ea] = e; let h = self.state.r8[R_H]; self.state.r8[R_H] = self.state.r8[R_Ha]; self.state.r8[R_Ha] = h; let l = self.state.r8[R_L]; self.state.r8[R_L] = self.state.r8[R_La]; self.state.r8[R_La] = l; (4, 1) }"
    
    # EX DE,HL
    if opcode == 0xEB:
        return "{ let de = self.state.get_reg16(R_DE); self.state.set_reg16(R_DE, self.state.get_reg16(R_HL)); self.state.set_reg16(R_HL, de); (4, 1) }"
    
    # EX (SP),HL
    if opcode == 0xE3:
        return "{ let sp = self.state.r16[R_SP]; let memval = mmu.r16(sp); mmu.w16reverse(sp, self.state.get_reg16(R_HL)); self.state.set_reg16(R_HL, memval); (19, 1) }"
    
    # DI, EI
    if opcode == 0xF3:
        return "{ self.state.iff1 = 0; self.state.iff2 = 0; (4, 1) }"
    if opcode == 0xFB:
        return "{ self.state.iff1 = 1; self.state.iff2 = 1; (4, 1) }"
    
    # IN A,(n); OUT (n),A
    if opcode == 0xDB:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); if let Some(in_fn) = self.port_in { self.state.r8[R_A] = in_fn(n, self.state.r8[R_A]); } (11, 2) }"
    if opcode == 0xD3:
        return "{ let pc = self.state.r16[R_PC]; let n = mmu.r8(pc + 1); if let Some(out_fn) = self.port_out { out_fn(n, self.state.r8[R_A], self.state.r8[R_A]); } (11, 2) }"
    
    # LD SP,HL
    if opcode == 0xF9:
        return "{ self.state.r16[R_SP] = self.state.get_reg16(R_HL); (6, 1) }"
    
    return "{ (4, 1) }"  # Default to NOP for truly undefined base opcodes


# Register constants mapping
R_A = 0; R_F = 1; R_B = 2; R_C = 3; R_D = 4; R_E = 5; R_H = 6; R_L = 7
R_Xh = 8; R_Xl = 9; R_Yh = 10; R_Yl = 11
R_Aa = 12; R_Fa = 13; R_Ba = 14; R_Ca = 15; R_Da = 16; R_Ea = 17; R_Ha = 18; R_La = 19
R_I = 20; R_R = 21
R_AF = 0; R_BC = 1; R_DE = 2; R_HL = 3; R_IX = 4; R_IY = 5
R_AFa = 6; R_BCa = 7; R_DEa = 8; R_HLa = 9; R_SP = 10; R_PC = 11

F_S = 0x80; F_Z = 0x40; F_5 = 0x20; F_H = 0x10; F_3 = 0x08; F_PV = 0x04; F_N = 0x02; F_C = 0x01

print("=== execute_base ===")
print(generate_base_opcodes())

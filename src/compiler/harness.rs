//! Bounded Z80 execution for compiled C80 functions.

use super::ast::CallConv;
use super::z80::{GeneratedProgram, R8, RegHome, Rr};
use crate::bus::CpuBus;
use crate::disasm::disassemble_at;
use crate::z80::Z80;
use crate::z80_state::{R_A, R_B, R_C, R_D, R_E, R_F, R_H, R_L};
use std::collections::HashMap;

pub const DEFAULT_SP: u16 = 0xFF00;
pub const DEFAULT_SENTINEL: u16 = 0x0000;
pub const DEFAULT_INSN_LIMIT: u32 = 10_000;
pub const DEFAULT_TSTATE_LIMIT: u64 = 100_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    Fetch,
    DataRead,
    DataWrite,
    PortIn,
    PortOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemAccess {
    pub kind: AccessKind,
    pub addr: u16,
    pub value: u8,
}

#[derive(Debug, Clone)]
pub struct ExecConfig {
    pub sp: u16,
    pub sentinel: u16,
    pub insn_limit: u32,
    pub tstate_limit: u64,
    pub initial_mem: Vec<(u16, u8)>,
    pub scripted_reads: Vec<(u16, Vec<u8>)>,
    pub scripted_ports: Vec<(u8, Vec<u8>)>,
}

impl Default for ExecConfig {
    fn default() -> Self {
        Self {
            sp: DEFAULT_SP,
            sentinel: DEFAULT_SENTINEL,
            insn_limit: DEFAULT_INSN_LIMIT,
            tstate_limit: DEFAULT_TSTATE_LIMIT,
            initial_mem: Vec::new(),
            scripted_reads: Vec::new(),
            scripted_ports: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub a: u8,
    pub f: u8,
    pub bc: u16,
    pub de: u16,
    pub hl: u16,
    pub ix: u16,
    pub iy: u16,
    pub sp: u16,
    pub pc: u16,
    pub tstates: u64,
    pub insns: u32,
    pub accesses: Vec<MemAccess>,
    pub timed_out: bool,
    /// Extra bytes below the function's entry SP (return address already pushed).
    pub sp_used: u16,
}

impl ExecResult {
    pub fn return_byte(&self) -> u8 {
        self.a
    }

    pub fn return_word(&self) -> u16 {
        self.hl
    }
}

struct TraceBus {
    mem: [u8; 0x10000],
    fetch_lo: u16,
    fetch_hi: u16,
    accesses: Vec<MemAccess>,
    scripted: HashMap<u16, Vec<u8>>,
    scripted_ports: HashMap<u8, Vec<u8>>,
}

impl TraceBus {
    fn in_fetch_range(&self, addr: u16) -> bool {
        if self.fetch_lo <= self.fetch_hi {
            addr >= self.fetch_lo && addr < self.fetch_hi
        } else {
            addr >= self.fetch_lo || addr < self.fetch_hi
        }
    }
}

struct PeekBus<'a> {
    mem: &'a [u8; 0x10000],
}

impl CpuBus for PeekBus<'_> {
    fn r8(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn w8(&mut self, _addr: u16, _val: u8) {}
}

impl CpuBus for TraceBus {
    fn r8(&mut self, addr: u16) -> u8 {
        let kind = if self.in_fetch_range(addr) {
            AccessKind::Fetch
        } else {
            AccessKind::DataRead
        };
        let value = if kind == AccessKind::DataRead {
            if let Some(queue) = self.scripted.get_mut(&addr) {
                if !queue.is_empty() {
                    let value = queue.remove(0);
                    self.mem[addr as usize] = value;
                    value
                } else {
                    self.mem[addr as usize]
                }
            } else {
                self.mem[addr as usize]
            }
        } else {
            self.mem[addr as usize]
        };
        self.accesses.push(MemAccess { kind, addr, value });
        value
    }

    fn w8(&mut self, addr: u16, val: u8) {
        self.accesses.push(MemAccess {
            kind: AccessKind::DataWrite,
            addr,
            value: val,
        });
        self.mem[addr as usize] = val;
    }

    fn out8(&mut self, port: u8, val: u8, _expected_val: u8) {
        self.accesses.push(MemAccess {
            kind: AccessKind::PortOut,
            addr: u16::from(port),
            value: val,
        });
    }

    fn in8(&mut self, port: u8, val: u8) -> u8 {
        let value = if let Some(queue) = self.scripted_ports.get_mut(&port) {
            if !queue.is_empty() {
                queue.remove(0)
            } else {
                val
            }
        } else {
            val
        };
        self.accesses.push(MemAccess {
            kind: AccessKind::PortIn,
            addr: u16::from(port),
            value,
        });
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecError(pub String);

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ExecError {}

pub fn execute_function(
    code: &GeneratedProgram,
    name: &str,
    args: &[u16],
) -> Result<ExecResult, ExecError> {
    execute_function_with(code, name, args, &ExecConfig::default())
}

pub fn execute_function_with(
    code: &GeneratedProgram,
    name: &str,
    args: &[u16],
    config: &ExecConfig,
) -> Result<ExecResult, ExecError> {
    let func = code
        .function(name)
        .ok_or_else(|| ExecError(format!("no generated function '{name}'")))?;
    let expected_args = if func.conv == CallConv::Stack {
        func.param_types.len()
    } else {
        func.param_homes.len()
    };
    if args.len() != expected_args {
        return Err(ExecError(format!(
            "function '{name}' expects {expected_args} args, got {}",
            args.len()
        )));
    }

    let mut bus = TraceBus {
        mem: [0; 0x10000],
        fetch_lo: 0,
        fetch_hi: 0,
        accesses: Vec::new(),
        scripted: config.scripted_reads.iter().cloned().collect(),
        scripted_ports: config.scripted_ports.iter().cloned().collect(),
    };
    for segment in &code.assembled.segments {
        for (i, byte) in segment.bytes.iter().enumerate() {
            bus.mem[segment.addr.wrapping_add(i as u16) as usize] = *byte;
        }
    }
    for &(addr, value) in &config.initial_mem {
        bus.mem[addr as usize] = value;
    }

    let mut cpu = Z80::new();
    cpu.state.r8.fill(0xA5);
    cpu.state.set_ix(0x1111);
    cpu.state.set_iy(0x2222);
    cpu.state.sp = config.sp;
    cpu.state.pc = func.addr;

    if func.conv == CallConv::Stack {
        for (ty, value) in func.param_types.iter().zip(args.iter()).rev() {
            let slot = stack_slot(*ty, *value);
            cpu.state.sp = cpu.state.sp.wrapping_sub(2);
            bus.mem[cpu.state.sp as usize] = slot as u8;
            bus.mem[cpu.state.sp.wrapping_add(1) as usize] = (slot >> 8) as u8;
        }
    } else {
        for (home, value) in func.param_homes.iter().zip(args.iter()) {
            set_home(&mut cpu, *home, *value);
        }
    }

    let ret_sp = cpu.state.sp.wrapping_sub(2);
    bus.mem[ret_sp as usize] = config.sentinel as u8;
    bus.mem[ret_sp.wrapping_add(1) as usize] = (config.sentinel >> 8) as u8;
    cpu.state.sp = ret_sp;
    let entry_sp = ret_sp;
    let mut min_sp = ret_sp;

    let mut tstates = 0u64;
    let mut insns = 0u32;
    let mut timed_out = false;
    loop {
        if cpu.state.pc == config.sentinel {
            break;
        }
        if insns >= config.insn_limit || tstates >= config.tstate_limit {
            timed_out = true;
            break;
        }
        let peek = {
            let mut peek_bus = PeekBus { mem: &bus.mem };
            disassemble_at(&mut peek_bus, cpu.state.pc)
        };
        bus.fetch_lo = cpu.state.pc;
        bus.fetch_hi = cpu.state.pc.wrapping_add(u16::from(peek.len));
        tstates += u64::from(cpu.step(&mut bus, 0));
        insns += 1;
        let depth = entry_sp.wrapping_sub(cpu.state.sp);
        if depth > 0 && depth < 0x8000 && depth > entry_sp.wrapping_sub(min_sp) {
            min_sp = cpu.state.sp;
        }
    }

    if timed_out {
        return Err(ExecError(format!(
            "execution timed out after {insns} instructions / {tstates} T-states (pc={:04X})",
            cpu.state.pc
        )));
    }

    if func.conv == CallConv::Stack {
        cpu.state.sp = cpu
            .state
            .sp
            .wrapping_add(2u16.wrapping_mul(args.len() as u16));
    }

    Ok(ExecResult {
        a: cpu.state.r8[R_A],
        f: cpu.state.r8[R_F],
        bc: cpu.state.get_bc(),
        de: cpu.state.get_de(),
        hl: cpu.state.get_hl(),
        ix: cpu.state.get_ix(),
        iy: cpu.state.get_iy(),
        sp: cpu.state.sp,
        pc: cpu.state.pc,
        tstates,
        insns,
        accesses: bus.accesses,
        timed_out,
        sp_used: entry_sp.wrapping_sub(min_sp),
    })
}

fn stack_slot(ty: super::types::CType, value: u16) -> u16 {
    match ty {
        super::types::CType::I8 => value as u8 as i8 as i16 as u16,
        super::types::CType::U8 | super::types::CType::Bool => u16::from(value as u8),
        _ => value,
    }
}

fn set_home(cpu: &mut Z80, home: RegHome, value: u16) {
    match home {
        RegHome::Byte(r) => cpu.state.r8[r8_index(r)] = value as u8,
        RegHome::Word(rr) => match rr {
            Rr::Bc => cpu.state.set_bc(value),
            Rr::De => cpu.state.set_de(value),
            Rr::Hl => cpu.state.set_hl(value),
        },
    }
}

fn r8_index(r: R8) -> usize {
    match r {
        R8::A => R_A,
        R8::B => R_B,
        R8::C => R_C,
        R8::D => R_D,
        R8::E => R_E,
        R8::H => R_H,
        R8::L => R_L,
    }
}

impl ExecResult {
    pub fn data_accesses(&self) -> impl Iterator<Item = MemAccess> + '_ {
        self.accesses
            .iter()
            .copied()
            .filter(|a| a.kind != AccessKind::Fetch)
    }
}

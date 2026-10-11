use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use super::*;
use crate::bus::{CpuBus, FakeBus};
use crate::z80::Z80;

const TABLE_PATH: &str = "src/emulator/z80_bus_cycles_table.rs";
const REGEN_ENV: &str = "RTVC_REGEN_BUS_CYCLES";

struct FuseTest {
    name: String,
    regs: Vec<u16>,
    state: Vec<String>,
    memory: Vec<(u16, Vec<u8>)>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EventKind {
    Contend,
    Read,
    Write,
    Other,
}

struct Event {
    tstate: u32,
    kind: EventKind,
    addr: u16,
}

fn parse_tests(input: &str) -> Vec<FuseTest> {
    let mut tests = Vec::new();
    let mut lines = input.lines().map(str::trim_end);
    while let Some(name) = lines.next() {
        if name.is_empty() {
            continue;
        }
        let regs = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(|v| u16::from_str_radix(v, 16).unwrap())
            .collect();
        let state = lines
            .next()
            .unwrap()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let mut memory = Vec::new();
        for line in lines.by_ref() {
            let mut fields = line.split_whitespace();
            let addr = fields.next().unwrap();
            if addr == "-1" {
                break;
            }
            let bytes = fields
                .take_while(|v| *v != "-1")
                .map(|v| u8::from_str_radix(v, 16).unwrap())
                .collect();
            memory.push((u16::from_str_radix(addr, 16).unwrap(), bytes));
        }
        tests.push(FuseTest {
            name: name.to_string(),
            regs,
            state,
            memory,
        });
    }
    tests
}

fn parse_expected(input: &str) -> HashMap<String, Vec<Event>> {
    let mut expected = HashMap::new();
    let mut lines = input.lines().peekable();
    while let Some(name) = lines.next() {
        if name.trim().is_empty() {
            continue;
        }
        let mut events = Vec::new();
        while let Some(line) = lines.next_if(|line| line.starts_with(' ')) {
            let fields: Vec<&str> = line.split_whitespace().collect();
            let kind = match fields[1] {
                "MC" => EventKind::Contend,
                "MR" => EventKind::Read,
                "MW" => EventKind::Write,
                _ => EventKind::Other,
            };
            events.push(Event {
                tstate: fields[0].parse().unwrap(),
                kind,
                addr: u16::from_str_radix(fields[2], 16).unwrap(),
            });
        }
        while lines.next_if(|line| !line.trim().is_empty()).is_some() {}
        expected.insert(name.trim().to_string(), events);
    }
    expected
}

fn setup(test: &FuseTest) -> (Z80, FakeBus, i32) {
    let mut z80 = Z80::new();
    let mut bus = FakeBus::new();
    for (name, value) in [
        "AF", "BC", "DE", "HL", "AFa", "BCa", "DEa", "HLa", "IX", "IY", "SP", "PC",
    ]
    .iter()
    .zip(&test.regs)
    {
        z80.set_reg_val(name, *value);
    }
    for (name, value) in ["I", "R", "IFF1", "IFF2", "im", "halted"]
        .iter()
        .zip(&test.state)
    {
        z80.set_reg_val(name, u16::from_str_radix(value, 16).unwrap());
    }
    for (addr, bytes) in &test.memory {
        for (offset, byte) in bytes.iter().enumerate() {
            bus.w8(addr.wrapping_add(offset as u16), *byte);
        }
    }
    bus.logging_enabled = true;
    (z80, bus, test.state[6].parse().unwrap())
}

struct LoggedAccess {
    kind: EventKind,
    addr: u16,
    value: u8,
}

fn parse_log(entry: &str) -> LoggedAccess {
    let fields: Vec<&str> = entry.split_whitespace().collect();
    LoggedAccess {
        kind: if fields[0] == "MW" {
            EventKind::Write
        } else {
            EventKind::Read
        },
        addr: u16::from_str_radix(fields[1], 16).unwrap(),
        value: u8::from_str_radix(fields[2], 16).unwrap(),
    }
}

fn table_key(bytes: &[u8]) -> u32 {
    match (bytes[0], bytes.get(1).copied()) {
        (0xDD | 0xFD, Some(0xCB)) => (u32::from(bytes[0]) << 16) | 0xCB00 | u32::from(bytes[3]),
        (0xCB | 0xED | 0xDD | 0xFD, Some(b1)) => (u32::from(bytes[0]) << 8) | u32::from(b1),
        (b0, _) => u32::from(b0),
    }
}

struct Instruction {
    test: String,
    bytes: Vec<u8>,
    tstates: u32,
    cycles: Vec<u8>,
}

/// Runs every FUSE test through the Z80 core, pairing each memory access the
/// core makes with the bus cycle FUSE recorded for it.
fn fuse_instructions() -> Vec<Instruction> {
    let tests = parse_tests(&std::fs::read_to_string("tests/tests.in").unwrap());
    let expected = parse_expected(&std::fs::read_to_string("tests/tests.expected").unwrap());
    let mut instructions = Vec::new();
    let mut mismatched = Vec::new();

    'tests: for test in &tests {
        let events = &expected[&test.name];
        let accesses: Vec<usize> = (0..events.len())
            .filter(|&i| matches!(events[i].kind, EventKind::Read | EventKind::Write))
            .collect();
        let (mut z80, mut bus, mut remaining) = setup(test);
        let mut next_access = 0;
        let mut start = 0u32;
        while remaining > 0 {
            let logged = bus.log.len();
            let tstates = z80.step(&mut bus, remaining);
            remaining -= tstates as i32;
            let new: Vec<LoggedAccess> = bus.log[logged..]
                .iter()
                .filter(|e| e.starts_with('M'))
                .map(|e| parse_log(e))
                .collect();
            if new.is_empty() {
                start += tstates;
                continue;
            }
            let mut cycles = Vec::new();
            for access in &new {
                let Some(&index) = accesses.get(next_access) else {
                    mismatched.push(test.name.clone());
                    continue 'tests;
                };
                next_access += 1;
                let event = &events[index];
                if event.kind != access.kind || event.addr != access.addr {
                    mismatched.push(test.name.clone());
                    continue 'tests;
                }
                let contend = events[..index]
                    .iter()
                    .rev()
                    .find(|e| e.kind == EventKind::Contend)
                    .unwrap();
                let m1 = match event.tstate - contend.tstate {
                    4 => true,
                    3 => false,
                    other => panic!("{}: {other}-T-state memory cycle", test.name),
                };
                let tstate = contend.tstate - start;
                assert!(tstate < 0x80, "{}: cycle at T{tstate}", test.name);
                cycles.push(tstate as u8 | if m1 { M1_FLAG } else { 0 });
            }
            let bytes: Vec<u8> = new.iter().take(8).map(|a| a.value).collect();
            let prefixes = bytes
                .iter()
                .take_while(|b| matches!(b, 0xDD | 0xFD))
                .count();
            let fetches = prefixes
                + match bytes.get(prefixes) {
                    Some(0xED) => 2,
                    Some(0xCB) if prefixes == 0 => 2,
                    _ => 1,
                };
            for (index, cycle) in cycles.iter().enumerate() {
                assert_eq!(
                    cycle & M1_FLAG != 0,
                    index < fetches,
                    "{}: unexpected M1 pattern {cycles:?}",
                    test.name
                );
            }
            instructions.push(Instruction {
                test: test.name.clone(),
                bytes,
                tstates,
                cycles,
            });
            start += tstates;
        }
    }
    assert!(
        mismatched.len() <= 1,
        "core memory accesses differ from FUSE in {mismatched:?}"
    );
    instructions
}

fn render_table(instructions: &[Instruction]) -> String {
    let mut table: BTreeMap<(u32, u32, usize), (&str, &[u8])> = BTreeMap::new();
    for instruction in instructions {
        if matches!(instruction.bytes[..], [0xDD | 0xFD, 0xDD | 0xFD, ..]) {
            continue;
        }
        let key = (
            table_key(&instruction.bytes),
            instruction.tstates,
            instruction.cycles.len(),
        );
        let cycles = instruction.cycles.as_slice();
        if let Some((other, existing)) = table.insert(key, (&instruction.test, cycles)) {
            assert_eq!(
                existing, cycles,
                "{other} and {} disagree on {key:X?}",
                instruction.test
            );
        }
    }
    let mut out = String::from(
        "// Generated by `RTVC_REGEN_BUS_CYCLES=1 cargo test --lib z80_bus_cycles` from\n\
         // tests/tests.in and tests/tests.expected. Do not edit.\n\n\
         /// (opcode key, instruction T-states, cycles as start T-state | 0x80 for M1)\n\
         static BUS_CYCLES: &[(u32, u8, &[u8])] = &[\n",
    );
    for ((key, tstates, _), (_, cycles)) in table {
        let cycles: Vec<String> = cycles
            .iter()
            .map(|c| match (c & M1_FLAG != 0, c & TSTATE_MASK) {
                (true, 0) => "M1_FLAG".to_string(),
                (true, t) => format!("M1_FLAG | {t}"),
                (false, t) => t.to_string(),
            })
            .collect();
        writeln!(
            out,
            "    (0x{key:06X}, {tstates}, &[{}]),",
            cycles.join(", ")
        )
        .unwrap();
    }
    out.push_str("];\n");
    out
}

#[test]
fn table_matches_fuse_bus_cycles() {
    let instructions = fuse_instructions();
    let rendered = render_table(&instructions);
    if std::env::var_os(REGEN_ENV).is_some() {
        std::fs::write(TABLE_PATH, &rendered).unwrap();
        return;
    }
    assert!(
        std::fs::read_to_string(TABLE_PATH).unwrap() == rendered,
        "{TABLE_PATH} is stale; regenerate with {REGEN_ENV}=1 cargo test --lib z80_bus_cycles"
    );
    for instruction in &instructions {
        let found = BusCycles::lookup(
            &instruction.bytes,
            Some(instruction.tstates),
            instruction.cycles.len(),
        )
        .unwrap_or_else(|| panic!("{}: no bus cycles", instruction.test));
        let cycles: Vec<u8> = (0..found.len())
            .map(|i| {
                let cycle = found.get(i).unwrap();
                cycle.tstate as u8 | if cycle.m1 { M1_FLAG } else { 0 }
            })
            .collect();
        assert_eq!(cycles, instruction.cycles, "{}", instruction.test);
    }
}

#[test]
fn untaken_call_reads_its_operand_like_a_taken_one() {
    // FUSE omits these reads, but the CPU performs them and so does the core
    // on buses without a logging override.
    let cycles = BusCycles::lookup(&[0xC4, 0x34, 0x12], Some(10), 3).unwrap();
    let cycles: Vec<MemCycle> = (0..3).map(|i| cycles.get(i).unwrap()).collect();
    assert_eq!(
        cycles,
        [
            MemCycle {
                tstate: 0,
                m1: true
            },
            MemCycle {
                tstate: 4,
                m1: false
            },
            MemCycle {
                tstate: 7,
                m1: false
            },
        ]
    );
}

#[test]
fn redundant_index_prefix_is_an_extra_opcode_fetch() {
    let cycles = BusCycles::lookup(&[0xDD, 0x3E, 0x12], Some(11), 3).unwrap();
    let cycles: Vec<MemCycle> = (0..cycles.len()).map(|i| cycles.get(i).unwrap()).collect();
    assert_eq!(
        cycles,
        [
            MemCycle {
                tstate: 0,
                m1: true
            },
            MemCycle {
                tstate: 4,
                m1: true
            },
            MemCycle {
                tstate: 8,
                m1: false
            },
        ]
    );
}

//! H4: C80 programs on the emulated TVC (BASIC 1.2 ROM, cold boot).
//!
//! The bare harness (`rtvc_c80::harness`) runs one function on a flat
//! 64 KiB bus. These tests build a project in process, inject its cassette
//! into a booted TVC, type `RUN`, and check the result in emulated memory:
//! the real ROM, its 50 Hz interrupt, BASIC's stack, and the memory map.
//!
//! - `selfcheck_*`: [info/c80/selfcheck](../../info/c80/selfcheck) runs a
//!   battery of 44 checks 40 times and BASIC prints the result. The run is
//!   stepped instruction by instruction to see interrupts land inside
//!   generated code and to measure the stack.
//! - `pong_*`: [info/c80/pong](../../info/c80/pong) is played: serve, the
//!   joystick moves the paddle, and an autopilot plays on while the screen
//!   is checked against the game's own positions.

use rtvc_c80::{CompilationResult, GeneratedProgram};
use rtvc_core::bus::CpuBus;
use rtvc_core::tvc::Tvc;

mod support;
use support::{boot_with_cas, build_cas, examples_dir, type_chars, type_line, wait_until};

/// TVC CPU clocks per 50 Hz frame (3.125 MHz).
const FRAME_T: u64 = 62_500;

fn global(code: &GeneratedProgram, name: &str) -> u16 {
    code.global(name)
        .unwrap_or_else(|| panic!("no global {name}"))
        .addr
}

/// Every address that holds an instruction of a generated function.
fn c80_code_map(code: &GeneratedProgram) -> Vec<bool> {
    let mut map = vec![false; 0x10000];
    for f in &code.functions {
        for m in &f.mapped {
            for k in 0..m.bytes.len() {
                map[usize::from(m.address.wrapping_add(k as u16))] = true;
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// Self-check battery
// ---------------------------------------------------------------------------

/// What each bit of `failed` / `seen` checks (selfcheck.c80 keeps the same
/// list next to the code).
const CHECK_NAMES: [&str; 44] = [
    "u8 add wrap",
    "u8 sub wrap",
    "u8 and/or/xor",
    "u8 not",
    "i8 add overflow",
    "i8 sub overflow",
    "i8 negate -128",
    "i8 or",
    "u16 add wrap",
    "u16 sub wrap",
    "u16 and/xor",
    "u16 not",
    "i16 add overflow",
    "i16 sub overflow",
    "i16 negate -32768",
    "i16 or",
    "u8 compare 255 > 1",
    "i8 compare -128 < 1",
    "i8 compare 127 > -128",
    "u16 compare 0x8000 > 0x7FFF",
    "i16 compare -32768 < 32767",
    "i16 compare at the boundary",
    "u8 <= and >=",
    "sign of casts to i16/i8",
    "u8 constant shifts",
    "i8 constant arithmetic shift",
    "u16 constant shifts",
    "i16 constant arithmetic shift",
    "u8 variable shifts",
    "u8 variable shift count >= 8",
    "i8 variable shift count >= 8",
    "u16/i16 variable shift count >= 16",
    "static-slot spill (nine byte locals)",
    "PUSH/POP save around a call",
    "strict @fastcall with a pin",
    "@preserves callee keeps a counter",
    "static local persists",
    "@inline helper",
    "@stackcall",
    "nested calls",
    "struct array fields",
    "static linked list",
    "str length and characters",
    "cpu::ldir",
];

struct SelfCheckRun {
    seen: Vec<u8>,
    failed: Vec<u8>,
    checks: u16,
    passes: u8,
    /// SP right after the `CALL` into `run` (return address pushed).
    entry_sp: u16,
    /// Lowest SP while the PC was in generated code (ISR excluded).
    min_sp: u16,
    /// SP once `run` returned to the ROM.
    return_sp: u16,
    interrupts: u32,
    interrupts_in_c80: u32,
    cycles: u64,
}

fn run_selfcheck(optimize: bool) -> (CompilationResult, SelfCheckRun) {
    let (result, cas) = build_cas(&examples_dir().join("selfcheck/rtvc-c80.toml"), optimize);
    let code = result.code.as_ref().unwrap();
    let run_addr = code.function("run").expect("run").addr;
    let in_c80 = c80_code_map(code);
    let done = global(code, "basic_done");
    let seen = global(code, "seen");
    let failed = global(code, "failed");
    let checks = global(code, "checks");
    let passes = global(code, "passes");

    let mut tvc = boot_with_cas(&cas);
    assert_eq!(tvc.bus.r8(done), 0);
    type_chars(&mut tvc, "RUN");
    tvc.key_down(13);
    let mut cycles = 0u64;
    let mut released = false;
    let mut entry_sp = None;
    let mut return_sp = None;
    let mut min_sp = 0xFFFFu16;
    let mut interrupts = 0u32;
    let mut interrupts_in_c80 = 0u32;
    let budget = 600 * FRAME_T;
    while tvc.bus.r8(done) != 1 {
        assert!(
            cycles < budget,
            "optimize={optimize}: BASIC did not POKE basic_done within {} frames \
             (pc={:04X}, passes={}, checks={})",
            budget / FRAME_T,
            tvc.z80.state.pc,
            tvc.bus.r8(passes),
            u16::from_le_bytes([tvc.bus.r8(checks), tvc.bus.r8(checks + 1)])
        );
        let pc = tvc.z80.state.pc;
        let iff1 = tvc.z80.state.iff1;
        let opcode = tvc.bus.r8(pc);
        cycles += u64::from(tvc.debug_step_instruction());
        let state = &tvc.z80.state;
        // IFF1 dropping without a DI means the CPU accepted an interrupt.
        if iff1 == 1 && state.iff1 == 0 && opcode != 0xF3 {
            interrupts += 1;
            if in_c80[usize::from(pc)] {
                interrupts_in_c80 += 1;
            }
        } else if in_c80[usize::from(state.pc)] {
            if state.pc == run_addr && entry_sp.is_none() {
                entry_sp = Some(state.sp);
            }
            min_sp = min_sp.min(state.sp);
        } else if in_c80[usize::from(pc)] && entry_sp.is_some() && return_sp.is_none() {
            return_sp = Some(state.sp);
        }
        if !released && cycles >= 8 * FRAME_T {
            tvc.key_up(13);
            released = true;
        }
    }
    let read = |tvc: &mut Tvc, addr: u16, n: u16| (0..n).map(|k| tvc.bus.r8(addr + k)).collect();
    let run = SelfCheckRun {
        seen: read(&mut tvc, seen, 6),
        failed: read(&mut tvc, failed, 6),
        checks: u16::from_le_bytes([tvc.bus.r8(checks), tvc.bus.r8(checks + 1)]),
        passes: tvc.bus.r8(passes),
        entry_sp: entry_sp.expect("run was never entered"),
        min_sp,
        return_sp: return_sp.expect("run never returned"),
        interrupts,
        interrupts_in_c80,
        cycles,
    };
    (result, run)
}

fn check_selfcheck(optimize: bool) {
    let started = std::time::Instant::now();
    let (result, run) = run_selfcheck(optimize);
    let code = result.code.as_ref().unwrap();
    let mode = if optimize { "opt" } else { "base" };
    let names = |bits: &[u8], want: bool| -> Vec<String> {
        (0..CHECK_NAMES.len())
            .filter(|&i| ((bits[i / 8] >> (i % 8)) & 1 == 1) == want)
            .map(|i| format!("{i} {}", CHECK_NAMES[i]))
            .collect()
    };
    let failed = names(&run.failed, true);
    assert!(failed.is_empty(), "[{mode}] failed on the TVC: {failed:#?}");
    let not_run = names(&run.seen, false);
    assert!(not_run.is_empty(), "[{mode}] never ran: {not_run:#?}");
    assert_eq!(run.passes, 40, "[{mode}] passes");
    assert_eq!(run.checks, 44 * 40, "[{mode}] checks");
    // Interrupts were live and landed inside generated code.
    assert!(
        run.interrupts_in_c80 >= 5,
        "[{mode}] only {} of {} interrupts hit generated code",
        run.interrupts_in_c80,
        run.interrupts
    );
    // BASIC's stack: `run` returned with SP one return address above its
    // entry SP, and BASIC went on to print and POKE basic_done.
    assert_eq!(
        run.return_sp,
        run.entry_sp.wrapping_add(2),
        "[{mode}] SP after USR"
    );
    // Stack bound: the compiler's proven bound covers what generated code
    // used below the return address (ISR pushes excluded).
    let f = code.function("run").unwrap();
    let used = run.entry_sp.wrapping_sub(run.min_sp);
    eprintln!(
        "selfcheck [{mode}]: {} interrupts ({} in generated code), {} frames, \
         stack bound {} ({:?}), observed {} (entry SP {:04X}, low {:04X}), {:.2} s",
        run.interrupts,
        run.interrupts_in_c80,
        run.cycles / FRAME_T,
        f.stack_bound,
        f.stack_provenance,
        used,
        run.entry_sp,
        run.min_sp,
        started.elapsed().as_secs_f64()
    );
    if f.stack_provenance == rtvc_c80::StackProvenance::Proven {
        assert!(
            used <= f.stack_bound,
            "[{mode}] used {used} stack bytes, proven bound {}",
            f.stack_bound
        );
    }
}

#[test]
fn selfcheck_passes_on_the_tvc_optimized() {
    check_selfcheck(true);
}

#[test]
fn selfcheck_passes_on_the_tvc_unoptimized() {
    check_selfcheck(false);
}

// ---------------------------------------------------------------------------
// Pong
// ---------------------------------------------------------------------------

struct Pong {
    code: GeneratedProgram,
    tvc: Tvc,
    /// First instruction of the frame wait: every loop pass starts there,
    /// with the previous pass's drawing complete.
    mark: u16,
    in_c80: Vec<bool>,
}

impl Pong {
    fn g(&self, name: &str) -> u16 {
        global(&self.code, name)
    }

    fn get(&mut self, name: &str) -> u8 {
        let a = self.g(name);
        self.tvc.bus.r8(a)
    }

    /// Steps to the start of the next loop pass.
    fn next_pass(&mut self) {
        let mut cycles = 0u64;
        // Leave the mark first if we are on it.
        if self.tvc.z80.state.pc == self.mark {
            cycles += u64::from(self.tvc.debug_step_instruction());
        }
        while self.tvc.z80.state.pc != self.mark {
            let pc = self.tvc.z80.state.pc;
            assert!(
                self.in_c80[usize::from(pc)],
                "pong left generated code: PC {pc:04X} (state {})",
                self.get("state")
            );
            assert!(
                cycles < 4 * FRAME_T,
                "no loop pass within 4 frames (PC {pc:04X})"
            );
            cycles += u64::from(self.tvc.debug_step_instruction());
        }
    }

    /// VID0 holds exactly the left paddle, the right paddle, the ball and
    /// the score pips at the positions in the game's globals: 0xFF there,
    /// 0 everywhere else.
    fn check_scene(&mut self, when: &str) {
        let (py, ay, bx, by) = (
            self.get("py"),
            self.get("ay"),
            self.get("bx"),
            self.get("by"),
        );
        let (ps, sa) = (self.get("pscore"), self.get("ascore"));
        let mut wrong = Vec::new();
        for row in 0u16..240 {
            for col in 0u16..64 {
                let paddle = |top: u8| row >= u16::from(top) && row < u16::from(top) + 24;
                // Score pip n (1..=score) covers rows [8n, 8n + 4).
                let pips = |score: u8| row >= 8 && row % 8 < 4 && row / 8 <= u16::from(score);
                let lit = (col == 2 && paddle(py))
                    || (col == 61 && paddle(ay))
                    || (col == 0 && pips(ps))
                    || (col == 63 && pips(sa))
                    || (col == u16::from(bx) && row >= u16::from(by) && row < u16::from(by) + 4);
                let got = self.tvc.bus.r8(0x8000 + row * 64 + col);
                let want = if lit { 0xFF } else { 0 };
                if got != want {
                    wrong.push(format!("({col},{row})={got:02X}"));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "{when}: screen does not match py={py} ay={ay} bx={bx} by={by}: {} wrong bytes, first {:?}",
            wrong.len(),
            &wrong[..wrong.len().min(8)]
        );
    }
}

fn start_pong(optimize: bool) -> Pong {
    let (result, cas) = build_cas(&examples_dir().join("pong/rtvc-c80.toml"), optimize);
    let code = result.code.unwrap();
    // wait_frame is its own function in --no-optimize builds and expanded
    // into play otherwise.
    let mark = match code.function("wait_frame") {
        Some(f) => f.addr,
        None => {
            code.function("play")
                .unwrap()
                .mapped
                .iter()
                .find(|m| m.inlined.first().is_some_and(|fr| fr.name == "wait_frame"))
                .expect("wait_frame expansion in play")
                .address
        }
    };
    // The frame interrupt runs the JP at 0038H and the handler in irq.asm,
    // so those addresses count as the game's code too.
    let mut in_c80 = c80_code_map(&code);
    let sym = |name: &str| {
        *code
            .assembled
            .symbols
            .get(name)
            .unwrap_or_else(|| panic!("no symbol {name}"))
    };
    for a in sym("U1_FRAME")..sym("U1_SAVED") {
        in_c80[usize::from(a)] = true;
    }
    for a in 0x38..0x3B {
        in_c80[a] = true;
    }
    let mut tvc = boot_with_cas(&cas);
    type_line(&mut tvc, "RUN");
    let ball = 0x8000 + 118 * 64 + 32;
    wait_until(&mut tvc, 600, |tvc| {
        tvc.bus.mmu.get_map_val() == 0x50 && tvc.bus.r8(ball) == 0xFF
    });
    let mut pong = Pong {
        code,
        tvc,
        mark,
        in_c80,
    };
    pong.next_pass();
    pong
}

fn play_pong(optimize: bool) {
    let started = std::time::Instant::now();
    let mut p = start_pong(optimize);
    let mode = if optimize { "opt" } else { "base" };
    assert_eq!(p.tvc.bus.mmu.get_map_val(), 0x50, "[{mode}] VID0 at 8000H");
    assert_eq!(p.tvc.bus.r8(0x38), 0xC3, "[{mode}] JP at the IM 1 vector");
    // The start layout from play(): paddles at raster 108, ball centred.
    assert_eq!(
        (
            p.get("py"),
            p.get("ay"),
            p.get("bx"),
            p.get("by"),
            p.get("state")
        ),
        (108, 108, 32, 118, 0)
    );
    p.check_scene(&format!("[{mode}] start"));
    for _ in 0..10 {
        p.next_pass();
    }
    assert_eq!(p.get("bx"), 32, "[{mode}] the ball waits for Space");

    // Space serves: vx = vy = 1, and the ball moves one column and one
    // raster per pass.
    p.tvc.key_down(32);
    p.next_pass();
    p.next_pass();
    p.tvc.key_up(32);
    assert_eq!(p.get("state"), 1, "[{mode}] served");
    let (bx0, by0) = (p.get("bx"), p.get("by"));
    assert!(bx0 > 32 && by0 > 118, "[{mode}] ball moved: {bx0},{by0}");
    for _ in 0..5 {
        p.next_pass();
    }
    assert_eq!(
        (p.get("bx"), p.get("by")),
        (bx0 + 5, by0 + 5),
        "[{mode}] ball step"
    );
    p.check_scene(&format!("[{mode}] after serve"));

    // Joystick down (cursor down, key 40) for 10 passes: 2 rasters each.
    let py0 = p.get("py");
    p.tvc.key_down(40);
    for _ in 0..10 {
        p.next_pass();
    }
    p.tvc.key_up(40);
    assert_eq!(p.get("py"), py0 + 20, "[{mode}] paddle down");
    p.next_pass();
    p.check_scene(&format!("[{mode}] after down"));
    // And up for 6.
    let py1 = p.get("py");
    p.tvc.key_down(38);
    for _ in 0..6 {
        p.next_pass();
    }
    p.tvc.key_up(38);
    assert_eq!(p.get("py"), py1 - 12, "[{mode}] paddle up");
    p.next_pass();
    p.check_scene(&format!("[{mode}] after up"));

    // Autopilot: follow the ball with the joystick, serve again after a
    // point. The game must stay in its loop with a consistent screen.
    let mut points = 0u8;
    let mut held: Option<u32> = None;
    let mut score = (p.get("pscore"), p.get("ascore"));
    for pass in 0..PONG_PASSES {
        let (py, by, state) = (p.get("py"), p.get("by"), p.get("state"));
        assert!(state < 2, "[{mode}] game over after pass {pass}");
        let want = if state == 0 {
            Some(32)
        } else if u16::from(by) + 2 < u16::from(py) + 10 {
            Some(38)
        } else if u16::from(by) + 2 > u16::from(py) + 14 {
            Some(40)
        } else {
            None
        };
        if want != held {
            if let Some(k) = held {
                p.tvc.key_up(k);
            }
            if let Some(k) = want {
                p.tvc.key_down(k);
            }
            held = want;
        }
        p.next_pass();
        let now = (p.get("pscore"), p.get("ascore"));
        if now != score {
            points += 1;
            score = now;
        }
        if pass % 25 == 0 {
            p.check_scene(&format!("[{mode}] autopilot pass {pass}"));
        }
    }
    if let Some(k) = held {
        p.tvc.key_up(k);
    }
    p.check_scene(&format!("[{mode}] end"));
    let pc = p.tvc.z80.state.pc;
    assert!(
        p.in_c80[usize::from(pc)],
        "[{mode}] PC {pc:04X} outside pong"
    );

    // Miss on purpose: keep the paddle away from the ball until the AI
    // scores, then check its pip and the re-centred ball.
    let lost = p.get("ascore");
    let mut held: Option<u32> = None;
    let mut missed = false;
    for _ in 0..1000 {
        let (by, state) = (p.get("by"), p.get("state"));
        let want = if state == 0 {
            32
        } else if by < 120 {
            40
        } else {
            38
        };
        if held != Some(want) {
            if let Some(k) = held {
                p.tvc.key_up(k);
            }
            p.tvc.key_down(want);
            held = Some(want);
        }
        p.next_pass();
        if p.get("ascore") != lost {
            missed = true;
            break;
        }
    }
    if let Some(k) = held {
        p.tvc.key_up(k);
    }
    assert!(missed, "[{mode}] the AI never scored");
    assert_eq!(p.get("ascore"), lost + 1);
    p.check_scene(&format!("[{mode}] after a miss"));

    // Keep missing until the AI wins. play() then returns through irq.asm,
    // which puts the ROM stub back at 0038H (PUSH AF) and EIs, and BASIC
    // runs again with U2 mapped.
    let mut held: Option<u32> = None;
    let mut over = false;
    for _ in 0..3000 {
        if p.tvc.bus.mmu.get_map_val() == 0x70 {
            over = true;
            break;
        }
        let (by, state) = (p.get("by"), p.get("state"));
        let want = if state == 0 {
            32
        } else if by < 120 {
            40
        } else {
            38
        };
        if held != Some(want) {
            if let Some(k) = held {
                p.tvc.key_up(k);
            }
            p.tvc.key_down(want);
            held = Some(want);
        }
        p.tvc.run_for_a_frame();
    }
    if let Some(k) = held {
        p.tvc.key_up(k);
    }
    assert!(over, "[{mode}] the game did not end");
    assert_eq!(p.get("state"), 3, "[{mode}] the AI won");
    assert_eq!(p.get("ascore"), 5);
    assert_eq!(p.tvc.bus.r8(0x38), 0xF5, "[{mode}] ROM stub restored");
    for _ in 0..10 {
        p.tvc.run_for_a_frame();
    }
    assert_ne!(p.tvc.z80.state.iff1, 0, "[{mode}] interrupts back on");
    eprintln!(
        "pong [{mode}]: {PONG_PASSES} autopilot passes, {points} points, score {}:{}, {:.2} s",
        score.0,
        score.1,
        started.elapsed().as_secs_f64()
    );
}

/// Loop passes (frames) the autopilot plays.
const PONG_PASSES: u32 = 300;

#[test]
fn pong_plays_on_the_tvc_optimized() {
    play_pong(true);
}

#[test]
fn pong_plays_on_the_tvc_unoptimized() {
    play_pong(false);
}

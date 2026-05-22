use std::fs;
use std::time::Instant;

mod mmu;
mod z80;

use mmu::{FakeMmu, Mmu};
use z80::Z80;

fn run_perf() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Running Performance Benchmark (zexdoc) ---");
    let mut fakemmu = FakeMmu::new();

    let test_data = fs::read("tests/zexdoc.com")?;
    // 0x0100 = start address
    for (i, &byte) in test_data.iter().enumerate() {
        fakemmu.w8(0x0100 + i as u16, byte);
    }

    // stack address
    fakemmu.w8(6, 0xE4);
    fakemmu.w8(7, 0x00);
    // first test address
    fakemmu.w8(0x0120, 0x3A);

    let mut z80 = Z80::new();
    z80.set_reg_val("PC", 0x0100);

    let mut instructions = 0u64;
    let mut t_states = 0u64;

    let start = Instant::now();

    loop {
        let t = z80.step(&mut fakemmu, 0);
        t_states += t as u64;
        instructions += 1;

        let pc = z80.get_reg_val("PC");

        // 0 = soft reset
        if pc == 0 {
            break;
        }
        // 5 = system call
        else if pc == 5 {
            let c = z80.get_reg_val("C");
            if c == 9 {
                let txt_addr = z80.get_reg_val("DE");
                let mut addr = txt_addr;
                loop {
                    let txt_chr = fakemmu.r8(addr);
                    if txt_chr == b'$' {
                        break;
                    }
                    addr = addr.wrapping_add(1);
                }
            }
            let ret_addr = z80.pop16(&mut fakemmu);
            z80.set_reg_val("PC", ret_addr);
        }
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    println!("\nBenchmark Finished!");
    println!("Elapsed time: {:.4} seconds", elapsed_secs);
    println!("Total instructions: {}", instructions);
    println!("Total T-states: {}", t_states);

    if elapsed_secs > 0.0 {
        let mips = (instructions as f64) / (elapsed_secs * 1_000_000.0);
        let mhz = (t_states as f64) / (elapsed_secs * 1_000_000.0);
        println!("Performance: {:.2} MIPS ({:.2} MHz equivalent)", mips, mhz);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_perf()?;
    Ok(())
}

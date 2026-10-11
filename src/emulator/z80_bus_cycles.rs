//! Start T-states of the memory machine cycles of each Z80 instruction, in
//! bus order, for buses that stretch individual cycles.

include!("z80_bus_cycles_table.rs");

const M1_FLAG: u8 = 0x80;
const TSTATE_MASK: u8 = 0x7F;

/// One memory machine cycle of an instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemCycle {
    /// T-states from the start of the instruction to the start of the cycle.
    pub tstate: u32,
    /// Opcode fetch (M1) rather than an ordinary memory read or write.
    pub m1: bool,
}

/// Memory cycles of one executed instruction.
#[derive(Clone, Copy, Debug)]
pub struct BusCycles {
    prefix_m1s: u32,
    cycles: &'static [u8],
}

impl BusCycles {
    /// Looks up an instruction from the bytes read by its first bus cycles.
    /// `tstates` selects the taken/not-taken or repeat variant when known;
    /// `accesses` is the number of memory cycles the instruction performed,
    /// or a lower bound while it is still executing.
    pub fn lookup(bytes: &[u8], tstates: Option<u32>, accesses: usize) -> Option<Self> {
        // Only the last of consecutive DD/FD prefixes applies; each earlier
        // one is a separate 4-T-state opcode fetch.
        let prefixes = bytes
            .iter()
            .take_while(|byte| matches!(byte, 0xDD | 0xFD))
            .count();
        let redundant = prefixes.saturating_sub(1);
        let tstates = match tstates {
            Some(tstates) => Some(tstates.checked_sub(4 * redundant as u32)?),
            None => None,
        };
        let mut bus_cycles = Self::lookup_unprefixed(
            &bytes[redundant..],
            tstates,
            accesses.saturating_sub(redundant),
        )?;
        bus_cycles.prefix_m1s += redundant as u32;
        Some(bus_cycles)
    }

    fn lookup_unprefixed(bytes: &[u8], tstates: Option<u32>, accesses: usize) -> Option<Self> {
        let byte = |index: usize| bytes.get(index).copied();
        let b0 = byte(0)?;
        let key = match b0 {
            0xCB | 0xED => (u32::from(b0) << 8) | u32::from(byte(1)?),
            0xDD | 0xFD => {
                let b1 = byte(1)?;
                if b1 == 0xCB {
                    (u32::from(b0) << 16) | 0xCB00 | u32::from(byte(3)?)
                } else if let Some(cycles) =
                    find((u32::from(b0) << 8) | u32::from(b1), tstates, accesses)
                {
                    return Some(Self {
                        prefix_m1s: 0,
                        cycles,
                    });
                } else if b1 == 0xED {
                    return None;
                } else {
                    // A prefix that does not affect the instruction is a
                    // separate opcode fetch.
                    let cycles = find(
                        u32::from(b1),
                        tstates.map(|t| t.saturating_sub(4)),
                        accesses.saturating_sub(1),
                    )?;
                    return Some(Self {
                        prefix_m1s: 1,
                        cycles,
                    });
                }
            }
            _ => u32::from(b0),
        };
        Some(Self {
            prefix_m1s: 0,
            cycles: find(key, tstates, accesses)?,
        })
    }

    pub fn len(&self) -> usize {
        self.prefix_m1s as usize + self.cycles.len()
    }

    pub fn get(&self, index: usize) -> Option<MemCycle> {
        let prefix = self.prefix_m1s as usize;
        if index < prefix {
            return Some(MemCycle {
                tstate: 4 * index as u32,
                m1: true,
            });
        }
        let cycle = *self.cycles.get(index - prefix)?;
        Some(MemCycle {
            tstate: u32::from(cycle & TSTATE_MASK) + 4 * self.prefix_m1s,
            m1: cycle & M1_FLAG != 0,
        })
    }
}

fn find(key: u32, tstates: Option<u32>, accesses: usize) -> Option<&'static [u8]> {
    let start = BUS_CYCLES.partition_point(|entry| entry.0 < key);
    let variants = BUS_CYCLES[start..]
        .iter()
        .take_while(|entry| entry.0 == key);
    let same_tstates = |entry: &&(u32, u8, &[u8])| Some(u32::from(entry.1)) == tstates;
    variants
        .clone()
        .find(|entry| same_tstates(entry) && entry.2.len() == accesses)
        .or_else(|| {
            variants
                .clone()
                .find(|entry| same_tstates(entry) && entry.2.len() >= accesses)
        })
        .or_else(|| variants.clone().find(|entry| entry.2.len() >= accesses))
        .map(|entry| entry.2)
}

#[cfg(test)]
#[path = "z80_bus_cycles_tests.rs"]
mod tests;

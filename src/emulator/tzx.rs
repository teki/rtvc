//! Pulse-level cassette images shared by the ZX Spectrum and the Amstrad CPC:
//! TZX (`.tzx`, and `.cdt`, which is the same format) and Spectrum `.tap`.
//!
//! An image is compiled once into [`Segment`]s timed in TZX T-states
//! (3.5 MHz). A [`TapePlayer`] replays them against any machine clock and
//! exposes the current signal level; each machine decides where that level is
//! read and whether a motor relay gates playback.

/// Clock that all TZX timings are expressed in.
pub const TZX_CLOCK_HZ: u64 = 3_500_000;

const TZX_SIGNATURE: &[u8] = b"ZXTape!\x1A";
const T_STATES_PER_MS: u32 = 3_500;

// Spectrum ROM timings used by standard-speed blocks.
const ROM_PILOT: u32 = 2168;
const ROM_SYNC1: u32 = 667;
const ROM_SYNC2: u32 = 735;
const ROM_ZERO: u32 = 855;
const ROM_ONE: u32 = 1710;
const ROM_HEADER_PILOT_PULSES: u32 = 8063;
const ROM_DATA_PILOT_PULSES: u32 = 3223;
const TAP_PAUSE_MS: u32 = 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Segment {
    /// Toggle the level, then hold it for the given T-states.
    Pulse(u32),
    /// Drive the level low (an edge if it was high), then hold it.
    Pause(u32),
    /// Drive the level to a fixed value and hold it (direct recordings).
    Level { high: bool, t_states: u32 },
    /// Stop the tape; the user or the machine must press play again.
    Stop,
    /// "Stop the tape if in 48K mode"; honored only when the player asks.
    Stop48k,
}

#[derive(Clone, Debug, Default)]
pub struct Tape {
    pub segments: Vec<Segment>,
    /// Text descriptions found in the image (block 0x30).
    pub descriptions: Vec<String>,
    /// IDs of blocks that were skipped because they are not emulated.
    pub skipped_blocks: Vec<u8>,
}

impl Segment {
    fn t_states(self) -> u32 {
        match self {
            Self::Pulse(t_states) | Self::Pause(t_states) | Self::Level { t_states, .. } => {
                t_states
            }
            Self::Stop | Self::Stop48k => 0,
        }
    }
}

impl Tape {
    /// Parses a TZX/CDT image, or a Spectrum TAP file when the TZX signature
    /// is absent.
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.starts_with(TZX_SIGNATURE) {
            parse_tzx(data)
        } else {
            parse_tap(data)
        }
    }
}

fn parse_tap(data: &[u8]) -> Result<Tape, String> {
    let mut tape = Tape::default();
    let mut offset = 0;
    while offset < data.len() {
        let len = read_u16(data, offset)? as usize;
        let block = data
            .get(offset + 2..offset + 2 + len)
            .ok_or_else(|| format!("truncated TAP block at offset {offset}"))?;
        push_standard_block(&mut tape.segments, block, TAP_PAUSE_MS);
        offset += 2 + len;
    }
    if tape.segments.is_empty() {
        return Err("tape image contains no blocks".to_string());
    }
    Ok(tape)
}

fn parse_tzx(data: &[u8]) -> Result<Tape, String> {
    if data.len() < 10 {
        return Err("truncated TZX header".to_string());
    }
    let mut tape = Tape::default();
    let segments = &mut tape.segments;
    let mut loop_start: Option<(usize, u16)> = None;
    let mut offset = 10;
    while offset < data.len() {
        let id = data[offset];
        let at = offset + 1;
        let len = match id {
            0x10 => {
                let pause = read_u16(data, at)? as u32;
                let n = read_u16(data, at + 2)? as usize;
                push_standard_block(segments, slice(data, at + 4, n)?, pause);
                4 + n
            }
            0x11 => {
                let n = read_u24(data, at + 15)? as usize;
                let pilot = read_u16(data, at)? as u32;
                let sync1 = read_u16(data, at + 2)? as u32;
                let sync2 = read_u16(data, at + 4)? as u32;
                let zero = read_u16(data, at + 6)? as u32;
                let one = read_u16(data, at + 8)? as u32;
                let pilot_pulses = read_u16(data, at + 10)? as u32;
                let last_bits = data[at + 12];
                let pause = read_u16(data, at + 13)? as u32;
                segments.extend((0..pilot_pulses).map(|_| Segment::Pulse(pilot)));
                segments.push(Segment::Pulse(sync1));
                segments.push(Segment::Pulse(sync2));
                push_data(segments, slice(data, at + 18, n)?, zero, one, last_bits);
                push_pause(segments, pause);
                18 + n
            }
            0x12 => {
                let pulse = read_u16(data, at)? as u32;
                let count = read_u16(data, at + 2)?;
                segments.extend((0..count).map(|_| Segment::Pulse(pulse)));
                4
            }
            0x13 => {
                let count = *data.get(at).ok_or("truncated TZX pulse sequence")? as usize;
                for index in 0..count {
                    segments.push(Segment::Pulse(read_u16(data, at + 1 + index * 2)? as u32));
                }
                1 + count * 2
            }
            0x14 => {
                let zero = read_u16(data, at)? as u32;
                let one = read_u16(data, at + 2)? as u32;
                let last_bits = *data.get(at + 4).ok_or("truncated TZX pure data")?;
                let pause = read_u16(data, at + 5)? as u32;
                let n = read_u24(data, at + 7)? as usize;
                push_data(segments, slice(data, at + 10, n)?, zero, one, last_bits);
                push_pause(segments, pause);
                10 + n
            }
            0x15 => {
                let sample = read_u16(data, at)? as u32;
                let pause = read_u16(data, at + 2)? as u32;
                let last_bits = *data.get(at + 4).ok_or("truncated TZX direct recording")?;
                let n = read_u24(data, at + 5)? as usize;
                push_direct(segments, slice(data, at + 8, n)?, sample, last_bits);
                push_pause(segments, pause);
                8 + n
            }
            0x20 => {
                let pause = read_u16(data, at)? as u32;
                if pause == 0 {
                    segments.push(Segment::Stop);
                } else {
                    push_pause(segments, pause);
                }
                2
            }
            0x24 => {
                loop_start = Some((segments.len(), read_u16(data, at)?));
                2
            }
            0x25 => {
                if let Some((start, count)) = loop_start.take() {
                    let body = segments[start..].to_vec();
                    for _ in 1..count {
                        segments.extend_from_slice(&body);
                    }
                }
                0
            }
            0x2A => {
                segments.push(Segment::Stop48k);
                4 + read_u32(data, at)? as usize
            }
            0x30 => {
                let n = *data.get(at).ok_or("truncated TZX text block")? as usize;
                let text = slice(data, at + 1, n)?;
                tape.descriptions
                    .push(String::from_utf8_lossy(text).trim().to_string());
                1 + n
            }
            // Blocks that carry no signal or are not emulated: skip by length.
            0x16 | 0x17 | 0x18 | 0x19 | 0x2B => {
                if !matches!(id, 0x2B) {
                    tape.skipped_blocks.push(id);
                }
                4 + read_u32(data, at)? as usize
            }
            0x21 => 1 + *data.get(at).ok_or("truncated TZX group block")? as usize,
            0x22 | 0x27 => 0,
            0x23 => 2,
            0x26 => 2 + read_u16(data, at)? as usize * 2,
            0x28 | 0x32 => 2 + read_u16(data, at)? as usize,
            0x31 => 2 + *data.get(at + 1).ok_or("truncated TZX message block")? as usize,
            0x33 => 1 + *data.get(at).ok_or("truncated TZX hardware block")? as usize * 3,
            0x35 => 20 + read_u32(data, at + 16)? as usize,
            0x40 => 4 + read_u24(data, at + 1)? as usize,
            0x5A => 9,
            _ => {
                return Err(format!(
                    "unsupported TZX block 0x{id:02X} at offset {offset}"
                ));
            }
        };
        offset = at + len;
    }
    if segments.is_empty() {
        return Err("tape image contains no signal blocks".to_string());
    }
    Ok(tape)
}

fn push_standard_block(segments: &mut Vec<Segment>, block: &[u8], pause_ms: u32) {
    let pilot_pulses = if block.first().is_some_and(|&flag| flag < 0x80) {
        ROM_HEADER_PILOT_PULSES
    } else {
        ROM_DATA_PILOT_PULSES
    };
    segments.extend((0..pilot_pulses).map(|_| Segment::Pulse(ROM_PILOT)));
    segments.push(Segment::Pulse(ROM_SYNC1));
    segments.push(Segment::Pulse(ROM_SYNC2));
    push_data(segments, block, ROM_ZERO, ROM_ONE, 8);
    push_pause(segments, pause_ms);
}

fn push_data(segments: &mut Vec<Segment>, data: &[u8], zero: u32, one: u32, last_bits: u8) {
    for (index, &byte) in data.iter().enumerate() {
        let bits = if index + 1 == data.len() {
            last_bits.clamp(1, 8)
        } else {
            8
        };
        for bit in 0..bits {
            let pulse = if byte & (0x80 >> bit) != 0 { one } else { zero };
            segments.push(Segment::Pulse(pulse));
            segments.push(Segment::Pulse(pulse));
        }
    }
}

fn push_direct(segments: &mut Vec<Segment>, data: &[u8], sample: u32, last_bits: u8) {
    for (index, &byte) in data.iter().enumerate() {
        let bits = if index + 1 == data.len() {
            last_bits.clamp(1, 8)
        } else {
            8
        };
        for bit in 0..bits {
            let high = byte & (0x80 >> bit) != 0;
            match segments.last_mut() {
                Some(Segment::Level {
                    high: previous,
                    t_states,
                }) if *previous == high => *t_states += sample,
                _ => segments.push(Segment::Level {
                    high,
                    t_states: sample,
                }),
            }
        }
    }
}

fn push_pause(segments: &mut Vec<Segment>, pause_ms: u32) {
    if pause_ms != 0 {
        segments.push(Segment::Pause(pause_ms * T_STATES_PER_MS));
    }
}

fn slice(data: &[u8], offset: usize, len: usize) -> Result<&[u8], String> {
    data.get(offset..offset + len)
        .ok_or_else(|| format!("truncated tape block at offset {offset}"))
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    Ok(u16::from_le_bytes(
        slice(data, offset, 2)?.try_into().unwrap(),
    ))
}

fn read_u24(data: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = slice(data, offset, 3)?;
    Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    Ok(u32::from_le_bytes(
        slice(data, offset, 4)?.try_into().unwrap(),
    ))
}

/// Replays a [`Tape`] against a machine clock.
pub struct TapePlayer {
    tape: Tape,
    clock_hz: u64,
    honor_stop_48k: bool,
    index: usize,
    /// Time left in the current segment, in T-states × `clock_hz`, so that
    /// machine cycles (scaled by [`TZX_CLOCK_HZ`]) subtract exactly.
    remaining: u64,
    level: bool,
    playing: bool,
    /// Tape length, and the time up to the end of the current segment, in
    /// T-states; for progress reporting.
    total_t_states: u64,
    played_t_states: u64,
}

impl TapePlayer {
    pub fn new(tape: Tape, clock_hz: u64, honor_stop_48k: bool) -> Self {
        let total_t_states = tape.segments.iter().map(|s| s.t_states() as u64).sum();
        Self {
            tape,
            clock_hz,
            honor_stop_48k,
            index: 0,
            remaining: 0,
            level: false,
            playing: false,
            total_t_states,
            played_t_states: 0,
        }
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }

    pub fn level(&self) -> bool {
        self.level
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.tape.segments.len() && self.remaining == 0
    }

    pub fn play(&mut self) {
        if !self.is_finished() {
            self.playing = true;
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }

    pub fn rewind(&mut self) {
        self.index = 0;
        self.remaining = 0;
        self.level = false;
        self.playing = false;
        self.played_t_states = 0;
    }

    /// Playing time elapsed, as a percentage of the tape's length.
    pub fn progress_percent(&self) -> u8 {
        let remaining = self.remaining / self.clock_hz;
        let played = self.played_t_states.saturating_sub(remaining);
        (played * 100 / self.total_t_states.max(1)).min(100) as u8
    }

    /// Advances playback by `cycles` of the machine clock.
    pub fn advance(&mut self, cycles: u32) {
        let mut budget = cycles as u64 * TZX_CLOCK_HZ;
        while self.playing && budget != 0 {
            if self.remaining == 0 && !self.start_next_segment() {
                break;
            }
            let step = budget.min(self.remaining);
            self.remaining -= step;
            budget -= step;
        }
    }

    fn start_next_segment(&mut self) -> bool {
        let Some(&segment) = self.tape.segments.get(self.index) else {
            self.playing = false;
            return false;
        };
        self.index += 1;
        self.played_t_states += segment.t_states() as u64;
        let t_states = match segment {
            Segment::Pulse(t_states) => {
                self.level = !self.level;
                t_states
            }
            Segment::Pause(t_states) => {
                self.level = false;
                t_states
            }
            Segment::Level { high, t_states } => {
                self.level = high;
                t_states
            }
            Segment::Stop => {
                self.playing = false;
                return false;
            }
            Segment::Stop48k => {
                if self.honor_stop_48k {
                    self.playing = false;
                    return false;
                }
                0
            }
        };
        self.remaining = t_states as u64 * self.clock_hz;
        true
    }
}

#[cfg(test)]
#[path = "tzx_tests.rs"]
mod tests;

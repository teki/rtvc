use super::*;

fn tzx(blocks: &[u8]) -> Vec<u8> {
    let mut data = TZX_SIGNATURE.to_vec();
    data.extend_from_slice(&[1, 20]);
    data.extend_from_slice(blocks);
    data
}

fn pulses(tape: &Tape) -> usize {
    tape.segments
        .iter()
        .filter(|segment| matches!(segment, Segment::Pulse(_)))
        .count()
}

#[test]
fn tap_header_block_uses_rom_timings() {
    // One 2-byte header block (flag 0x00).
    let tape = Tape::parse(&[2, 0, 0x00, 0xFF]).unwrap();
    assert_eq!(pulses(&tape), 8063 + 2 + 2 * 16);
    assert_eq!(tape.segments[0], Segment::Pulse(ROM_PILOT));
    // 0x00 is zeros, 0xFF is ones.
    assert_eq!(tape.segments[8065], Segment::Pulse(ROM_ZERO));
    assert_eq!(tape.segments[8065 + 16], Segment::Pulse(ROM_ONE));
    assert_eq!(
        tape.segments.last(),
        Some(&Segment::Pause(TAP_PAUSE_MS * T_STATES_PER_MS))
    );
}

#[test]
fn tzx_turbo_block_honors_last_byte_bits() {
    let mut block = vec![0x11];
    for value in [1000u16, 300, 400, 500, 900, 2] {
        block.extend_from_slice(&value.to_le_bytes());
    }
    block.push(3); // last byte uses 3 bits
    block.extend_from_slice(&0u16.to_le_bytes());
    block.extend_from_slice(&[2, 0, 0]);
    block.extend_from_slice(&[0xFF, 0xA0]);
    let tape = Tape::parse(&tzx(&block)).unwrap();
    assert_eq!(pulses(&tape), 2 + 2 + 2 * (8 + 3));
    let last_bits: Vec<_> = tape.segments[tape.segments.len() - 6..].to_vec();
    assert_eq!(
        last_bits,
        [900, 900, 500, 500, 900, 900].map(|t| Segment::Pulse(t as u32))
    );
}

#[test]
fn tzx_pause_zero_stops_and_loops_repeat() {
    let blocks = [
        0x24, 3, 0, // loop 3 times
        0x12, 100, 0, 2, 0,    // two 100 T pulses
        0x25, // loop end
        0x20, 0, 0, // stop the tape
        0x30, 2, b'h', b'i',
    ];
    let tape = Tape::parse(&tzx(&blocks)).unwrap();
    assert_eq!(pulses(&tape), 6);
    assert_eq!(tape.segments.last(), Some(&Segment::Stop));
    assert_eq!(tape.descriptions, ["hi"]);
}

#[test]
fn player_scales_tzx_time_to_the_machine_clock() {
    let tape = Tape {
        segments: vec![Segment::Pulse(3500), Segment::Pulse(3500)],
        ..Tape::default()
    };
    // 3500 T at 3.5 MHz is 1 ms, which is 4000 cycles at 4 MHz.
    let mut player = TapePlayer::new(tape, 4_000_000, false);
    player.play();
    player.advance(1);
    assert!(player.level());
    player.advance(3999);
    assert!(player.level());
    player.advance(1);
    assert!(!player.level());
    player.advance(4000);
    player.advance(1);
    assert!(player.is_finished());
    assert!(!player.is_playing());
}

#[test]
fn player_stops_at_stop_blocks() {
    let tape = Tape {
        segments: vec![Segment::Pulse(10), Segment::Stop48k, Segment::Pulse(10)],
        ..Tape::default()
    };
    let mut spectrum = TapePlayer::new(tape.clone(), TZX_CLOCK_HZ, true);
    spectrum.play();
    spectrum.advance(100);
    assert!(!spectrum.is_playing());
    assert!(!spectrum.is_finished());

    let mut cpc = TapePlayer::new(tape, TZX_CLOCK_HZ, false);
    cpc.play();
    cpc.advance(100);
    assert!(cpc.is_finished());
}

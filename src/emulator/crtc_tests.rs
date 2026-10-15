use super::*;

/// R0..R9 for a small 10-character, 8-row, 1-raster frame with VSYNC on row 4.
fn configure(crtc: &mut Crtc, r3: u8) {
    for (index, value) in [9, 4, 6, r3, 7, 0, 4, 4, 0, 0].into_iter().enumerate() {
        crtc.select(index as u8);
        crtc.write(value);
    }
}

/// Lengths of the HSYNC and VSYNC pulses seen over two frames, in character
/// clocks and lines.
fn sync_lengths(crtc: &mut Crtc) -> (usize, usize) {
    let (mut hsync, mut vsync_lines) = (0, 0);
    let (mut max_hsync, mut max_vsync) = (0, 0);
    for _ in 0..2 * 10 * 8 {
        let tick = crtc.tick();
        hsync = if tick.raw_hsync { hsync + 1 } else { 0 };
        max_hsync = usize::max(max_hsync, hsync);
        if tick.line_start {
            vsync_lines = if tick.raw_vsync { vsync_lines + 1 } else { 0 };
            max_vsync = usize::max(max_vsync, vsync_lines);
        }
    }
    (max_hsync, max_vsync)
}

#[test]
fn mc6845_vsync_is_always_16_lines() {
    let mut crtc = Crtc::new(CrtcVariant::Mc6845);
    configure(&mut crtc, 0x22);
    // The 8-line frame retriggers VSYNC before 16 lines expire.
    let (hsync, vsync) = sync_lengths(&mut crtc);
    assert_eq!(hsync, 2);
    assert!(vsync > 2);
}

#[test]
fn hd6845s_vsync_width_comes_from_r3_high_nibble() {
    let mut crtc = Crtc::new(CrtcVariant::Hd6845s);
    configure(&mut crtc, 0x22);
    assert_eq!(sync_lengths(&mut crtc), (2, 2));
}

#[test]
fn zero_hsync_width_differs_by_variant() {
    let mut mc = Crtc::new(CrtcVariant::Mc6845);
    configure(&mut mc, 0x20);
    // Sixteen clocks overrun the 10-clock line and retrigger at R2.
    assert!(sync_lengths(&mut mc).0 >= 10);

    let mut hd = Crtc::new(CrtcVariant::Hd6845s);
    configure(&mut hd, 0x20);
    assert_eq!(sync_lengths(&mut hd).0, 0);
}

#[test]
fn write_only_registers_read_back_by_variant() {
    let mut mc = Crtc::new(CrtcVariant::Mc6845);
    let mut hd = Crtc::new(CrtcVariant::Hd6845s);
    for crtc in [&mut mc, &mut hd] {
        crtc.select(12);
        crtc.write(0xF0);
    }
    assert_eq!(mc.read(), 0x30);
    assert_eq!(hd.read(), 0x30);
    mc.select(1);
    hd.select(1);
    assert_eq!(mc.read(), 0xFF);
    assert_eq!(hd.read(), 0x00);
}

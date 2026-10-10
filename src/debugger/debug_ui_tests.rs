use super::*;

#[test]
fn parses_debugger_addresses_as_hex() {
    assert_eq!(parse_address("C229"), Some(0xC229));
    assert_eq!(parse_address("0xc229"), Some(0xC229));
    assert_eq!(parse_address("$C229"), Some(0xC229));
    assert_eq!(parse_address("C229H"), Some(0xC229));
    assert_eq!(parse_address("not-an-address"), None);
}

#[test]
fn embedded_rom_symbols_resolve_by_bank_and_offset() {
    let symbols = rom_symbols();
    assert!(!symbols.is_empty());
    // (bank, image offset) is the lookup key; the canonical CPU address is
    // display info, not identity, so no alias table is needed.
    assert_eq!(
        symbol_at(RomBank::Sys, 0x0229).map(|symbol| symbol.name.as_str()),
        Some("BASIC_COLD_START")
    );
    assert_eq!(
        symbol_at(RomBank::Sys, 0x0229).map(|symbol| symbol.address),
        Some(0xC229)
    );
    assert!(symbol_at(RomBank::Exth, 0x0229).is_none());
    // CPU F000H is SYS offset 0x3000 or EXTH offset 0x1000. The bank selects
    // the image; the listing address does not.
    assert_eq!(
        symbol_at(RomBank::Exth, 0x1000).map(|symbol| symbol.name.as_str()),
        Some("EXT_INIT")
    );
    assert_ne!(
        symbol_at(RomBank::Sys, 0x3000).map(|symbol| symbol.name.as_str()),
        Some("EXT_INIT")
    );
}

#[test]
fn comments_follow_the_image_that_owns_the_byte() {
    let reset = comment_at(RomBank::Sys, 0x0000).expect("D4 reset note");
    assert!(reset.text.contains("Reset starts"));
    let copied = comment_at(RomBank::Sys, 0x3B5B).expect("D3 RAM-copy note");
    assert!(copied.text.contains("copied into RAM"));
    // The same CPU address, read as an EXTH offset, is a different byte.
    assert!(comment_at(RomBank::Exth, 0x1B5B).is_none());
    let extension = comment_at(RomBank::Exth, 0x1000).expect("D7 init note");
    assert!(extension.text.contains("Extension ROM initialization"));
}

#[test]
fn mapped_address_prefers_the_page_being_viewed() {
    assert_eq!(choose_mapped_address(&[0xC229], &[0x0100]), Some(0xC229));
    assert_eq!(choose_mapped_address(&[], &[0xC229]), None);
    assert_eq!(
        choose_mapped_address(&[0x0229, 0xC229], &[0xC100]),
        Some(0xC229)
    );
    assert_eq!(
        choose_mapped_address(&[0x0229, 0xC229], &[0x0100]),
        Some(0x0229)
    );
    assert_eq!(
        choose_mapped_address(&[0x0229, 0xC229], &[0x8000, 0xC000]),
        Some(0xC229)
    );
    assert_eq!(
        choose_mapped_address(&[0x0229, 0xC229], &[0x8000]),
        Some(0x0229)
    );
}

#[test]
fn stacked_labels_share_one_key_with_stable_primary() {
    // Three ASM labels share sys offset 0x098F; the curated name stays
    // primary instead of flipping to whichever sorts last.
    let symbol = symbol_at(RomBank::Sys, 0x098F).expect("stacked key resolves");
    assert_eq!(symbol.name.as_str(), "CALL_WITH_SYS_PAGED");
    assert_eq!(
        symbol.alt_names,
        vec!["CALL_WITH_VIDEO_PAGED", "VIDEO_PAGE_GUARD"]
    );
    assert!(symbol.matches("video_page_guard"));
}

#[test]
fn event_history_is_capped() {
    let mut debugger = DebuggerUi::default();
    for index in 0..EVENT_LIMIT + 20 {
        debugger.record_control(&format!("event {index}"));
    }

    assert_eq!(debugger.events.len(), EVENT_LIMIT);
    assert_eq!(debugger.events.front().unwrap().sequence, 21);
}

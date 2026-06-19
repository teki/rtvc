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
fn embedded_rom_symbols_load_and_resolve_aliases() {
    let symbols = rom_symbols();
    assert!(!symbols.is_empty());
    assert_eq!(
        symbol_at(RomBank::Sys, 0xC229).map(|symbol| symbol.name.as_str()),
        Some("BASIC_COLD_START")
    );
    assert_eq!(
        symbol_at(RomBank::Sys, 0x0229).map(|symbol| symbol.name.as_str()),
        Some("BASIC_COLD_START")
    );
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

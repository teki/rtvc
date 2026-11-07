use super::*;

fn assert_all_released(key: &Key) {
    assert_eq!(key.state, [0xFF; 11]);
    assert_eq!(key.mod_state, 0);
}

#[test]
fn synthesized_shift_is_released_with_host_key() {
    let mut key = Key::new();

    key.key_down(65);
    key.key_press('A');
    assert_ne!(key.state, [0xFF; 11]);

    key.key_up(65);
    assert_all_released(&key);
}

#[test]
fn early_physical_shift_release_does_not_leave_matrix_keys_stuck() {
    let mut key = Key::new();

    key.key_down(KC_SHIFT);
    key.key_down(65);
    key.key_press('a');
    key.key_up(KC_SHIFT);
    key.key_up(65);

    assert_all_released(&key);
}

#[test]
fn altgr_character_mapping_releases_cleanly() {
    let mut key = Key::new();

    key.key_down(KC_ALTGR);
    key.key_down(81);
    key.key_press('@');
    key.key_up(81);
    key.key_up(KC_ALTGR);

    assert_all_released(&key);
}

#[test]
fn f9_and_insert_press_ins_and_leave_backtick_typeable() {
    for code in [120, 45] {
        let mut key = Key::new();
        key.key_down(code);
        assert_eq!(key.state[8], 0xFF & !1);
        key.key_up(code);
        assert_all_released(&key);
    }

    let mut key = Key::new();
    key.key_down(192);
    key.key_press('`');
    assert_eq!(key.state[8], 0xFF);
    assert_ne!(key.state[2], 0xFF);
    key.key_up(192);
    assert_all_released(&key);
}

#[test]
fn press_char_types_without_a_host_key_and_releases_cleanly() {
    let mut key = Key::new();

    let code = key.press_char('a');
    assert_ne!(key.state, [0xFF; 11]);
    key.key_up(code);
    assert_all_released(&key);

    // Synthesized Shift for an uppercase character is released with it.
    let code = key.press_char('A');
    assert_eq!(key.state[6] & (1 << 3), 0);
    key.key_up(code);
    assert_all_released(&key);
}

#[test]
fn press_char_is_independent_of_cached_host_key_mappings() {
    let mut key = Key::new();

    // Physical KeyA (code 65) typing 'a' caches 'a' under code 65 and marks
    // 'a' as mapped; neither may affect a later soft-keyboard 'A' or 'a'.
    key.key_down(65);
    key.key_press('a');
    let a_state = key.state;
    key.key_up(65);
    assert_all_released(&key);

    let code = key.press_char('a');
    assert_eq!(key.state, a_state);
    key.key_up(code);
    assert_all_released(&key);

    let code = key.press_char('A');
    assert_eq!(key.state[6] & (1 << 3), 0, "uppercase needs synthesized Shift");
    key.key_up(code);
    assert_all_released(&key);
}

#[test]
fn press_char_maps_return_and_backspace() {
    let mut key = Key::new();

    for ch in ['\r', '\n', '\x08', ' '] {
        let code = key.press_char(ch);
        assert_ne!(key.state, [0xFF; 11], "{ch:?} should press a key");
        key.key_up(code);
        assert_all_released(&key);
    }
}

#[test]
fn key_typer_holds_releases_and_separates_repeated_characters() {
    let mut typer = KeyTyper::default();
    typer.push_str("ll");

    assert_eq!(typer.step(), TypedKeyStep::Press('l'));
    typer.pressed(7);
    for _ in 1..TYPED_KEY_HOLD_FRAMES {
        assert_eq!(typer.step(), TypedKeyStep::Idle);
    }
    assert_eq!(typer.step(), TypedKeyStep::Release(7));
    for _ in 0..TYPED_KEY_GAP_FRAMES {
        assert_eq!(typer.step(), TypedKeyStep::Idle);
    }
    assert_eq!(typer.step(), TypedKeyStep::Press('l'));
    typer.pressed(7);
    assert_eq!(typer.clear(), Some(7));
    assert_eq!(typer.step(), TypedKeyStep::Idle);
}

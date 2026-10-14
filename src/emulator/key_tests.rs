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

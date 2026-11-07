#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};

const KSADD: u8 = 1 << 0;
const KSDEL: u8 = 1 << 1;

const SHIFT_ON: u8 = 1 << 0;
const ALTGR_ON: u8 = 1 << 2;
const SHIFT_ALTGR_ON: u8 = SHIFT_ON | ALTGR_ON;

pub const KC_SHIFT: u32 = 16;
pub const KC_ALT: u32 = 18;
pub const KC_ALTGR: u32 = 225;
const SYNTHETIC_CHAR_CODE_BASE: u32 = 0x11_0000;
/// Emulated frames a queued character's key stays down.
pub const TYPED_KEY_HOLD_FRAMES: u32 = 3;
/// Emulated frames with no key down between queued characters, so a repeated
/// character registers as two presses.
pub const TYPED_KEY_GAP_FRAMES: u32 = 1;

type Mapping = (u8, u8, u8);

pub struct Key {
    state: [u8; 11],
    row: u8,
    keymap: HashMap<u8, HashMap<u32, Mapping>>,
    is_mapped: HashSet<u32>,
    mod_state: u8,
    last_press: u32,
    ntable: String,
    stable: String,
}

impl Key {
    pub fn new() -> Self {
        let mut key = Key {
            state: [0xFF; 11],
            row: 0,
            keymap: HashMap::new(),
            is_mapped: HashSet::new(),
            mod_state: 0,
            last_press: 0,
            ntable: String::from(
                "53206í14^89ü*óö7tew;z@qr]ioő[úpugds\\h<af klá űéjbcx n yv ,.   -m",
            ),
            stable: String::from(
                "%+\"&/Í'!~()Ü#ÓÖ=TEW$Z`QR}IOŐ{ÚPUGDS|H>AF KLÁ ŰÉJBCX N YV ?:   _M",
            ),
        };

        key.keymap.insert(0, HashMap::new());
        key.keymap.insert(SHIFT_ON, HashMap::new());
        key.keymap.insert(ALTGR_ON, HashMap::new());
        key.keymap.insert(SHIFT_ALTGR_ON, HashMap::new());

        // Pre-mapped non-typable keys (unshifted)
        key.keymap.get_mut(&0).unwrap().insert(46, (5, 0, 0)); // del
        key.keymap.get_mut(&0).unwrap().insert(8, (5, 0, 0)); // backspace
        key.keymap.get_mut(&0).unwrap().insert(13, (5, 4, 0)); // return
        key.keymap.get_mut(&0).unwrap().insert(16, (6, 3, 0)); // shift
        key.keymap.get_mut(&0).unwrap().insert(20, (6, 5, 0)); // lock
        key.keymap.get_mut(&0).unwrap().insert(18, (7, 0, 0)); // alt
        key.keymap.get_mut(&0).unwrap().insert(27, (7, 3, 0)); // esc
        key.keymap.get_mut(&0).unwrap().insert(17, (7, 4, 0)); // ctrl
        key.keymap.get_mut(&0).unwrap().insert(32, (7, 5, 0)); // space
        key.keymap.get_mut(&0).unwrap().insert(38, (8, 1, 0)); // up
        key.keymap.get_mut(&0).unwrap().insert(40, (8, 2, 0)); // down
        key.keymap.get_mut(&0).unwrap().insert(9, (8, 3, 0)); // tab -> fire
        key.keymap.get_mut(&0).unwrap().insert(39, (8, 5, 0)); // right
        key.keymap.get_mut(&0).unwrap().insert(37, (8, 6, 0)); // left
        key.keymap.get_mut(&0).unwrap().insert(120, (8, 0, 0)); // F9 -> ins
        key.keymap.get_mut(&0).unwrap().insert(45, (8, 0, 0)); // insert -> ins

        // Copy base mappings to all modifier states
        let base: HashMap<u32, Mapping> = key.keymap[&0].clone();
        for (kc, m) in &base {
            key.keymap.get_mut(&SHIFT_ON).unwrap().insert(*kc, *m);
            key.keymap.get_mut(&ALTGR_ON).unwrap().insert(*kc, *m);
            key.keymap.get_mut(&SHIFT_ALTGR_ON).unwrap().insert(*kc, *m);
        }

        // Pre-mark these character codes as already mapped
        key.is_mapped.insert(8);
        key.is_mapped.insert(9);
        key.is_mapped.insert(13);
        key.is_mapped.insert(32);

        key
    }

    pub fn reset(&mut self) {
        self.state = [0xFF; 11];
        self.mod_state = 0;
        self.last_press = 0;
    }

    fn add_mapping(&mut self, ch: char) -> Option<Mapping> {
        // Don't auto-map over modifier key entries
        if self.last_press == KC_SHIFT
            || self.last_press == 17
            || self.last_press == KC_ALT
            || self.last_press == KC_ALTGR
        {
            return None;
        }

        let mut flags = 0u8;

        if let Some(idx) = self.ntable.chars().position(|c| c == ch) {
            if self.mod_state & SHIFT_ON != 0 {
                flags |= KSDEL;
            }
            let mapping: Mapping = ((idx / 8) as u8, (idx % 8) as u8, flags);
            if let Some(map) = self.keymap.get_mut(&self.mod_state) {
                map.insert(self.last_press, mapping);
            }
            return Some(mapping);
        }

        if let Some(idx) = self.stable.chars().position(|c| c == ch) {
            if self.mod_state & SHIFT_ON == 0 {
                flags |= KSADD;
            }
            let mapping: Mapping = ((idx / 8) as u8, (idx % 8) as u8, flags);
            if let Some(map) = self.keymap.get_mut(&self.mod_state) {
                map.insert(self.last_press, mapping);
            }
            return Some(mapping);
        }

        None
    }

    fn fix_state(&mut self, val: u8, down: bool) {
        if val & KSADD != 0 {
            self.key_set(6, 3, down || (self.mod_state & SHIFT_ON) != 0);
        }
        if val & KSDEL != 0 {
            self.key_set(6, 3, !down && (self.mod_state & SHIFT_ON) != 0);
        }
    }

    fn key_update(&mut self, code: u32, down: bool) -> bool {
        if code == KC_SHIFT {
            if down {
                self.mod_state |= SHIFT_ON;
            } else {
                self.mod_state &= !SHIFT_ON;
            }
        }
        if code == KC_ALTGR {
            if down {
                self.mod_state |= ALTGR_ON;
            } else {
                self.mod_state &= !ALTGR_ON;
            }
        }

        let entry = if let Some(map) = self.keymap.get(&self.mod_state) {
            map.get(&code).copied()
        } else {
            None
        };
        if let Some(m) = entry {
            self.apply_mapping(m, down);
        }
        let found = entry.is_some();

        if !down {
            let mut to_release: Vec<Mapping> = Vec::new();
            for (_, map) in &self.keymap {
                if let Some(&m) = map.get(&code) {
                    to_release.push(m);
                }
            }
            for m in to_release {
                self.key_set(m.0, m.1, false);
                self.fix_state(m.2, false);
            }
        }

        found
    }

    fn apply_mapping(&mut self, m: Mapping, down: bool) {
        self.key_set(m.0, m.1, down);
        self.fix_state(m.2, down);
    }

    pub fn key_down(&mut self, code: u32) -> bool {
        self.last_press = code;
        if code != 0 {
            self.key_update(code, true)
        } else {
            true
        }
    }

    pub fn key_up(&mut self, code: u32) {
        if code != 0 {
            self.key_update(code, false);
        }
    }

    pub fn key_press(&mut self, ch: char) {
        let code = ch as u32;
        if code != 0 && !self.is_mapped.contains(&code) {
            if let Some(m) = self.add_mapping(ch) {
                self.is_mapped.insert(code);
                self.apply_mapping(m, true);
            }
        }
    }

    /// Presses the matrix key that types `ch`, independent of any host key
    /// code, and returns the code that releases it through [`Key::key_up`].
    ///
    /// Soft keyboards and queued text deliver characters without a reliable
    /// physical key, so each character gets its own synthetic code above the
    /// Unicode range; it cannot collide with a host key code whose cached
    /// mapping produces a different character.
    pub fn press_char(&mut self, ch: char) -> u32 {
        let code = match ch {
            '\r' | '\n' => 13,
            '\x08' => 8,
            '\t' => 9,
            ' ' => 32,
            _ => SYNTHETIC_CHAR_CODE_BASE + ch as u32,
        };
        if !self.key_down(code) {
            if let Some(m) = self.add_mapping(ch) {
                self.apply_mapping(m, true);
            }
        }
        code
    }

    pub fn select_row(&mut self, val: u8) {
        self.row = val & 0x0F;
    }

    pub fn read_row(&self) -> u8 {
        self.state.get(self.row as usize).copied().unwrap_or(0xFF)
    }

    fn key_set(&mut self, row: u8, col: u8, down: bool) {
        if let Some(state_row) = self.state.get_mut(row as usize) {
            if down {
                *state_row &= !(1u8 << col);
            } else {
                *state_row |= 1u8 << col;
            }
        }
    }
}

/// What a [`KeyTyper`] wants done before the next emulated frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypedKeyStep {
    Idle,
    /// Press the key that types this character, then report its release code
    /// with [`KeyTyper::pressed`].
    Press(char),
    /// Release the key pressed for the previous character.
    Release(u32),
}

/// Types queued text one character at a time, holding each key for a few
/// emulated frames. Machine-independent: the owner performs each step.
#[derive(Debug, Default)]
pub struct KeyTyper {
    queue: VecDeque<char>,
    held: Option<u32>,
    hold_frames: u32,
    gap_frames: u32,
}

impl KeyTyper {
    pub fn push_str(&mut self, text: &str) {
        self.queue.extend(text.chars());
    }

    /// Drops queued text and returns the code of a key still held, if any.
    pub fn clear(&mut self) -> Option<u32> {
        self.queue.clear();
        self.hold_frames = 0;
        self.gap_frames = 0;
        self.held.take()
    }

    /// Characters still waiting to be typed.
    pub fn queued(&self) -> String {
        self.queue.iter().collect()
    }

    /// True when nothing is queued or held.
    pub fn is_idle(&self) -> bool {
        self.queue.is_empty() && self.held.is_none()
    }

    /// Advances by one emulated frame; call once before each frame runs.
    pub fn step(&mut self) -> TypedKeyStep {
        if let Some(code) = self.held {
            if self.hold_frames > 1 {
                self.hold_frames -= 1;
                return TypedKeyStep::Idle;
            }
            self.held = None;
            self.hold_frames = 0;
            self.gap_frames = TYPED_KEY_GAP_FRAMES;
            return TypedKeyStep::Release(code);
        }
        if self.gap_frames != 0 {
            self.gap_frames -= 1;
            return TypedKeyStep::Idle;
        }
        match self.queue.pop_front() {
            Some(ch) => TypedKeyStep::Press(ch),
            None => TypedKeyStep::Idle,
        }
    }

    /// Records the release code for the character returned by the last
    /// [`TypedKeyStep::Press`].
    pub fn pressed(&mut self, code: u32) {
        self.held = Some(code);
        self.hold_frames = TYPED_KEY_HOLD_FRAMES;
    }
}

#[cfg(test)]
#[path = "key_tests.rs"]
mod tests;

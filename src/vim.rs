// Copyright 2026 Falko Strenzke, MTG AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Vim modal editing for the value editors (hex / text / date-time),
//! reached with `e`/`E` on a tree element.
//!
//! This module does **not** reimplement selection, undo or paste: it is a
//! small command interpreter that drives the *existing* [`crate::app::Editor`]
//! operations (`move_horizontal`, `move_vertical`, `home`, `end`, the
//! selection/undo machinery `EditHistory` already provides, and `paste`) —
//! see research.md R7. `v`'s selection is the very same `EditHistory` anchor
//! that Shift+motion already sets under normal bindings; a bare motion key
//! just triggers it instead of a Shift-modified one, rather than a second,
//! parallel selection mechanism living here.
//!
//! [`VimState::register`] is the editor's *own* clipboard for `x`/`y`/`d`/
//! `p`/`P`, holding content in the editor's own unit (hex digits or
//! characters — see `HexEditor::UNIT`/`TextEditor::UNIT`). It is deliberately
//! separate from the tree's `ElementBuffer` (`src/buffer.rs`) that the
//! Structure pane's own `y`/`p`/`P`/`d` fill (research.md R8): copying a
//! value inside an editor never touches what is held for pasting whole tree
//! elements, and vice versa — they are different registers for different
//! kinds of content.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::Editor;

/// Which of vim's three editing modes is active. Editors always open in
/// `Normal` under vim bindings (FR-028).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
}

/// Per-editor vim state: the active mode and this editor's own register.
/// Deliberately has no notion of *which* editor (hex/text/date-time) it is
/// attached to — that distinction is made by the caller, which always has
/// the `Editor` alongside this state and passes it into [`VimState::handle_key`].
pub struct VimState {
    pub mode: EditorMode,
    pub register: Vec<char>,
}

impl Default for VimState {
    fn default() -> Self {
        Self::new()
    }
}

impl VimState {
    pub fn new() -> Self {
        VimState { mode: EditorMode::Normal, register: Vec::new() }
    }
}

/// What the caller (`handle_edit_key` in `src/tui.rs`) should do after a key
/// has been consumed by the vim dispatch below.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VimOutcome {
    /// Stay in the editor; nothing further to do.
    Continue,
    /// Apply the edit (mirrors the existing `Enter` handling).
    Apply,
    /// Cancel the edit (mirrors the existing `Esc` handling).
    Cancel,
}

impl VimState {
    /// Handles one key. Normal and Visual mode are fully dispatched here;
    /// Insert mode only has its mode-transition keys (`Esc` back to
    /// Normal, `Enter` to apply) handled here — printable characters and
    /// Backspace/Delete in Insert mode fall through to the existing
    /// character-insertion arms already in `handle_edit_key`, unchanged by
    /// vim bindings.
    pub fn handle_key(&mut self, editor: &mut Editor, key: KeyEvent) -> VimOutcome {
        if key.code == KeyCode::Esc {
            return match self.mode {
                EditorMode::Normal => VimOutcome::Cancel,
                EditorMode::Insert | EditorMode::Visual => {
                    self.mode = EditorMode::Normal;
                    VimOutcome::Continue
                }
            };
        }
        if key.code == KeyCode::Enter {
            return VimOutcome::Apply;
        }
        match self.mode {
            EditorMode::Insert => VimOutcome::Continue,
            EditorMode::Normal => self.handle_normal(editor, key),
            EditorMode::Visual => self.handle_visual(editor, key),
        }
    }

    fn handle_normal(&mut self, editor: &mut Editor, key: KeyEvent) -> VimOutcome {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            // Terminals differ over whether a Ctrl combination arrives
            // upper or lower case, so fold it before matching (mirrors the
            // idiom `handle_edit_key`/`keymap::translate` already use).
            let folded = match key.code {
                KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
                other => other,
            };
            match folded {
                KeyCode::Char('a') => {
                    let _ = editor.adjust_number_at_cursor(1);
                }
                KeyCode::Char('x') => {
                    let _ = editor.adjust_number_at_cursor(-1);
                }
                _ => {}
            }
            return VimOutcome::Continue;
        }
        match key.code {
            KeyCode::Char('i') => self.mode = EditorMode::Insert,
            KeyCode::Char('a') => {
                editor.move_horizontal(1, false);
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('I') => {
                editor.home(false);
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('A') => {
                editor.end(false);
                self.mode = EditorMode::Insert;
            }
            KeyCode::Char('h') => editor.move_horizontal(-1, false),
            KeyCode::Char('l') => editor.move_horizontal(1, false),
            KeyCode::Char('j') => editor.move_vertical(1, false),
            KeyCode::Char('k') => editor.move_vertical(-1, false),
            KeyCode::Char('0') => editor.home(false),
            KeyCode::Char('$') => editor.end(false),
            KeyCode::Char('x') => self.delete_unit_at_cursor(editor),
            KeyCode::Char('v') => {
                // No unit buffer to select in the date/time editor; leave
                // it in Normal rather than entering a Visual mode that can
                // never select anything there.
                if !matches!(editor, Editor::DateTime(_)) {
                    editor.select_unit_at_cursor();
                    self.mode = EditorMode::Visual;
                }
            }
            KeyCode::Char('p') => self.put(editor, true),
            KeyCode::Char('P') => self.put(editor, false),
            KeyCode::Char('u') => {
                editor.undo();
            }
            _ => {}
        }
        VimOutcome::Continue
    }

    fn handle_visual(&mut self, editor: &mut Editor, key: KeyEvent) -> VimOutcome {
        match key.code {
            KeyCode::Char('h') => editor.move_horizontal(-1, true),
            KeyCode::Char('l') => editor.move_horizontal(1, true),
            KeyCode::Char('j') => editor.move_vertical(1, true),
            KeyCode::Char('k') => editor.move_vertical(-1, true),
            KeyCode::Char('0') => editor.home(true),
            KeyCode::Char('$') => editor.end(true),
            KeyCode::Char('y') => {
                self.register = editor.selection_chars().unwrap_or_default();
                editor.clear_selection();
                self.mode = EditorMode::Normal;
            }
            KeyCode::Char('d') => {
                self.register = editor.selection_chars().unwrap_or_default();
                editor.delete_selection();
                self.mode = EditorMode::Normal;
            }
            _ => {}
        }
        VimOutcome::Continue
    }

    /// `x`: select exactly the unit at the cursor and delete it into the
    /// register. A documented no-op on `DateTimeEditor` — it has no
    /// character/digit buffer to delete a "unit" from, so this must not
    /// fall through to `select_unit_at_cursor`'s no-op-but-still-called
    /// path (which would be harmless, but this makes the intent explicit).
    fn delete_unit_at_cursor(&mut self, editor: &mut Editor) {
        if matches!(editor, Editor::DateTime(_)) {
            return;
        }
        editor.select_unit_at_cursor();
        self.register = editor.selection_chars().unwrap_or_default();
        editor.delete_selection();
    }

    /// `p` (after the cursor) / `P` (before it). `Editor::paste` already
    /// inserts at the cursor, which is "before" — `P`'s semantics — so `p`
    /// alone needs to first step over the unit at/after the cursor.
    fn put(&mut self, editor: &mut Editor, after: bool) {
        if self.register.is_empty() {
            return;
        }
        if after {
            editor.move_horizontal(1, false);
        }
        let text: String = self.register.iter().collect();
        editor.paste(&text);
    }
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyEvent, KeyModifiers};

    use crate::app::{DateTimeEditor, Editor, TextFormat};

    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn hex(content: &[u8]) -> Editor {
        Editor::hex(content)
    }

    fn digits(editor: &Editor) -> String {
        let Editor::Hex(h) = editor else { panic!("not a hex editor") };
        h.digits.iter().collect()
    }

    fn cursor_of(editor: &Editor) -> usize {
        match editor {
            Editor::Hex(h) => h.cursor,
            Editor::Text(t) => t.cursor,
            Editor::DateTime(d) => d.active,
        }
    }

    #[test]
    fn opens_in_normal_mode() {
        let vim = VimState::new();
        assert_eq!(vim.mode, EditorMode::Normal);
        assert!(vim.register.is_empty());
    }

    #[test]
    fn i_a_capital_i_capital_a_enter_insert_at_the_right_place() {
        let mut editor = hex(&[0xAB, 0xCD]);
        // 'i': cursor unchanged.
        let mut vim = VimState::new();
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Char('i'))), VimOutcome::Continue);
        assert_eq!(vim.mode, EditorMode::Insert);
        assert_eq!(cursor_of(&editor), 0);

        // 'a': cursor moves right one position first (the same granularity
        // plain Right already moves by — a hex digit, not a whole octet).
        let mut vim = VimState::new();
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Char('a'))), VimOutcome::Continue);
        assert_eq!(vim.mode, EditorMode::Insert);
        assert_eq!(cursor_of(&editor), 1);

        // 'I': jumps to the start.
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('I')));
        assert_eq!(vim.mode, EditorMode::Insert);
        assert_eq!(cursor_of(&editor), 0);

        // 'A': jumps to the end.
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('A')));
        assert_eq!(vim.mode, EditorMode::Insert);
        assert_eq!(cursor_of(&editor), 4);
    }

    #[test]
    fn esc_returns_to_normal_from_insert_and_visual_and_cancels_from_normal() {
        let mut editor = hex(&[0xAB]);
        let mut vim = VimState::new();
        vim.mode = EditorMode::Insert;
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Esc)), VimOutcome::Continue);
        assert_eq!(vim.mode, EditorMode::Normal);

        vim.mode = EditorMode::Visual;
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Esc)), VimOutcome::Continue);
        assert_eq!(vim.mode, EditorMode::Normal);

        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Esc)), VimOutcome::Cancel);
    }

    #[test]
    fn enter_applies_from_normal_and_insert() {
        let mut editor = hex(&[0xAB]);
        let mut vim = VimState::new();
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Enter)), VimOutcome::Apply);
        vim.mode = EditorMode::Insert;
        assert_eq!(vim.handle_key(&mut editor, key(KeyCode::Enter)), VimOutcome::Apply);
    }

    #[test]
    fn v_l_l_y_dollar_p_duplicates_three_octets_at_the_end() {
        // AA BB CC DD — three octets (AA BB CC) get marked, copied and
        // pasted back after jumping to the end, per spec.md User Story 6's
        // documented scenario.
        let mut editor = hex(&[0xAA, 0xBB, 0xCC, 0xDD]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('v')));
        assert_eq!(vim.mode, EditorMode::Visual);
        vim.handle_key(&mut editor, key(KeyCode::Char('l')));
        vim.handle_key(&mut editor, key(KeyCode::Char('l')));
        vim.handle_key(&mut editor, key(KeyCode::Char('y')));
        assert_eq!(vim.mode, EditorMode::Normal);
        assert_eq!(vim.register.iter().collect::<String>(), "AABBCC");
        vim.handle_key(&mut editor, key(KeyCode::Char('$')));
        vim.handle_key(&mut editor, key(KeyCode::Char('p')));
        assert_eq!(digits(&editor), "AABBCCDDAABBCC");
    }

    #[test]
    fn visual_d_cuts_the_selection_into_the_register() {
        let mut editor = hex(&[0xAA, 0xBB, 0xCC]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('v')));
        vim.handle_key(&mut editor, key(KeyCode::Char('l')));
        vim.handle_key(&mut editor, key(KeyCode::Char('d')));
        assert_eq!(vim.mode, EditorMode::Normal);
        assert_eq!(vim.register.iter().collect::<String>(), "AABB");
        assert_eq!(digits(&editor), "CC");
    }

    #[test]
    fn x_deletes_one_unit_into_the_register() {
        let mut editor = hex(&[0xAA, 0xBB]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('x')));
        assert_eq!(vim.register.iter().collect::<String>(), "AA");
        assert_eq!(digits(&editor), "BB");
    }

    #[test]
    fn u_undoes_the_last_change() {
        let mut editor = hex(&[0xAA]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('x')));
        assert_eq!(digits(&editor), "");
        vim.handle_key(&mut editor, key(KeyCode::Char('u')));
        assert_eq!(digits(&editor), "AA");
    }

    #[test]
    fn ctrl_a_wraps_a_hex_octet_from_ff_to_00() {
        let mut editor = hex(&[0xFF]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, ctrl('a'));
        assert_eq!(digits(&editor), "00");
    }

    #[test]
    fn ctrl_x_wraps_a_hex_octet_from_00_to_ff() {
        let mut editor = hex(&[0x00]);
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, ctrl('x'));
        assert_eq!(digits(&editor), "FF");
    }

    #[test]
    fn ctrl_a_on_an_integer_field_increments_99_to_100() {
        let mut editor = Editor::text(TextFormat::Integer, "99".to_string());
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, ctrl('a'));
        let Editor::Text(t) = &editor else { panic!() };
        assert_eq!(t.buf.iter().collect::<String>(), "100");
    }

    #[test]
    fn ctrl_a_on_an_oid_arc() {
        let mut editor = Editor::text(TextFormat::Oid, "1.2.840.113549".to_string());
        let Editor::Text(ref mut t) = editor else { panic!() };
        t.cursor = 4; // inside the "840" arc
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, ctrl('a'));
        let Editor::Text(t) = &editor else { panic!() };
        assert_eq!(t.buf.iter().collect::<String>(), "1.2.841.113549");
    }

    #[test]
    fn datetime_field_navigation_via_h_and_l_and_x_no_op() {
        let mut editor = Editor::DateTime(DateTimeEditor {
            fields: [
                "2024".to_string(),
                "01".to_string(),
                "02".to_string(),
                "03".to_string(),
                "04".to_string(),
                "05".to_string(),
            ],
            active: 0,
            generalized: true,
            pristine: true,
        });
        let mut vim = VimState::new();
        vim.handle_key(&mut editor, key(KeyCode::Char('l')));
        assert_eq!(cursor_of(&editor), 1);
        vim.handle_key(&mut editor, key(KeyCode::Char('l')));
        assert_eq!(cursor_of(&editor), 2);
        vim.handle_key(&mut editor, key(KeyCode::Char('h')));
        assert_eq!(cursor_of(&editor), 1);

        // 'x' is a documented no-op: the fields are untouched.
        vim.handle_key(&mut editor, key(KeyCode::Char('x')));
        let Editor::DateTime(d) = &editor else { panic!() };
        assert_eq!(d.fields, ["2024", "01", "02", "03", "04", "05"]);
    }

    #[test]
    fn ctrl_c_v_z_are_no_ops_when_vim_is_active() {
        // spec.md User Story 6 scenario 11: only Ctrl+C, Ctrl+V and Ctrl+Z
        // are no-ops under vim bindings — Ctrl+X is deliberately
        // repurposed as vim's own decrement (scenario 9), mirroring real
        // vim's Ctrl-A/Ctrl-X convention, so it is exercised separately by
        // `ctrl_x_wraps_a_hex_octet_from_00_to_ff` rather than asserted
        // inert here. `handle_key`'s Normal-mode dispatch recognises
        // Ctrl+A/Ctrl+X only (T041) — everything else under Control,
        // including c/v/z, falls through its `match folded { ... _ => {} }`
        // arm untouched. This asserts that directly: simulating each
        // leaves the editor unchanged, the way `handle_edit_key`'s
        // `edit.vim.is_none()` gate (T042) keeps the old Ctrl-block from
        // ever running here too.
        for c in ['c', 'v', 'z'] {
            let mut editor = hex(&[0xAA, 0xBB]);
            let mut vim = VimState::new();
            vim.handle_key(&mut editor, ctrl(c));
            assert_eq!(digits(&editor), "AABB", "Ctrl+{c} must be a no-op under vim bindings");
            assert_eq!(vim.mode, EditorMode::Normal);
            assert!(vim.register.is_empty());
        }
    }
}

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

//! Turning a raw terminal key event into a semantic action.
//!
//! The tree pane and, from a later work package on, the value editors each
//! support two key-binding sets ([`KeyBindingSet::Normal`], modelled on
//! everyday editor conventions, and [`KeyBindingSet::Vim`]). Rather than
//! letting `src/tui.rs`'s key handlers match raw keys twice — once per set —
//! this module is the single place that knows which key means what in which
//! set. Callers ask [`translate`] "what does this key mean here", and the
//! Edit menu and help window ask [`label`] "what key would produce this
//! action" so their displayed shortcuts can never drift from what
//! `translate` actually accepts. It also gives contract C-002 ("the vim set
//! contains no Ctrl+C/V/X/Z") a home where it can be enforced by a test
//! instead of by discipline scattered across match arms in `tui.rs`.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Which key-binding convention is active.
///
/// Persisted (via [`std::fmt::Display`] / [`std::str::FromStr`]) in the
/// settings file introduced by a later work package.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum KeyBindingSet {
    #[default]
    Normal,
    Vim,
}

impl std::fmt::Display for KeyBindingSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            KeyBindingSet::Normal => "normal",
            KeyBindingSet::Vim => "vim",
        };
        f.write_str(s)
    }
}

impl std::str::FromStr for KeyBindingSet {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(KeyBindingSet::Normal),
            "vim" => Ok(KeyBindingSet::Vim),
            other => Err(format!("unknown key-binding set: {other:?}")),
        }
    }
}

/// Actions available on the tree (Structure pane, Browse mode).
///
/// Deliberately **not** covered here: `i`, `I`, `e`, `E`, `J`, `K`, `z`, `/`.
/// Those keys keep their existing, set-independent behaviour untouched by
/// this mission (FR-033), so hard-coding them in `handle_document_key`
/// remains correct — adding them to this enum would be churn with no
/// behavioural payoff.
///
/// There is deliberately no separate `Yank` variant: vim's `y` and normal's
/// Ctrl+C both mean `Copy` here (see [`translate`]). Whether a given press
/// also touches the system clipboard or stays buffer-only is `App`'s
/// decision in a later work package, keyed on the active [`KeyBindingSet`],
/// not this module's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TreeAction {
    MarkUp,
    MarkDown,
    Delete,
    Cut,
    Copy,
    PasteAfter,
    PasteBefore,
    PasteAsChild,
}

/// Actions available inside a value editor (hex/text editing, vim insert
/// and visual modes). Consumed starting with a later work package; the
/// enum is defined here now so that this file remains the single owner of
/// the whole key-binding vocabulary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditorAction {
    EnterInsertAt,
    EnterInsertAfter,
    EnterInsertLineStart,
    EnterInsertLineEnd,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Home,
    End,
    DeleteUnderCursor,
    VisualStart,
    Yank,
    DeleteSelection,
    PutAfter,
    PutBefore,
    Undo,
    Increment,
    Decrement,
    Apply,
    Cancel,
    LeaveMode,
}

/// The result of translating a raw key: either a tree action or an editor
/// action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Tree(TreeAction),
    Editor(EditorAction),
}

/// Translates a raw key event into a semantic [`Action`], per
/// `contracts/keymap.md`.
///
/// This WP only implements the Structure pane's tree-action rows; editor
/// actions are consumed starting with a later work package, so this
/// function currently only ever returns `Action::Tree(_)` or `None`.
///
/// A `context` parameter (e.g. "is a mark currently active") turned out to
/// be unnecessary here: every row of the Structure-pane table means the
/// same thing regardless of tree state, so it is intentionally omitted. A
/// caller that needs to gate an action on state (like the delete
/// confirmation's second press) does so itself using the returned
/// `TreeAction`, not by asking this function to know about that state.
pub fn translate(key: KeyEvent, set: KeyBindingSet) -> Option<Action> {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    // Mark up/down: identical in both sets.
    match key.code {
        KeyCode::Up if shift => return Some(Action::Tree(TreeAction::MarkUp)),
        KeyCode::Down if shift => return Some(Action::Tree(TreeAction::MarkDown)),
        _ => {}
    }

    // Delete: identical in both sets. `translate` only reports that this
    // key means Delete; the two-step confirmation dance is the caller's
    // job, not this function's.
    if !ctrl && !shift && key.code == KeyCode::Char('d') {
        return Some(Action::Tree(TreeAction::Delete));
    }

    match set {
        KeyBindingSet::Normal => {
            if ctrl {
                // Terminals differ over whether a Ctrl combination arrives
                // upper or lower case, so fold it before matching (mirrors
                // the idiom already used by `handle_edit_key` in tui.rs).
                let folded = match key.code {
                    KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
                    other => other,
                };
                match folded {
                    KeyCode::Char('x') => return Some(Action::Tree(TreeAction::Cut)),
                    KeyCode::Char('c') => return Some(Action::Tree(TreeAction::Copy)),
                    KeyCode::Char('v') => return Some(Action::Tree(TreeAction::PasteAfter)),
                    _ => {}
                }
            }
        }
        KeyBindingSet::Vim => {
            // Explicitly no arm here matches CONTROL + c/v/x/z: contract
            // C-002 requires the vim set to be provably free of those
            // combinations, so they must stay structurally unmatched rather
            // than matched-and-mapped-to-None.
            if !ctrl && !shift {
                match key.code {
                    KeyCode::Char('y') => return Some(Action::Tree(TreeAction::Copy)),
                    KeyCode::Char('p') => return Some(Action::Tree(TreeAction::PasteAfter)),
                    _ => {}
                }
            }
            if !ctrl && shift && key.code == KeyCode::Char('P') {
                return Some(Action::Tree(TreeAction::PasteBefore));
            }
        }
    }

    None
}

/// Returns the human-readable key text shown by the Edit menu and the help
/// window for `action` in `set`, or `None` when the action has no key in
/// that set (it is menu-only, or — for `Cut` in the vim set — simply
/// unbound).
///
/// This function's output must stay in lock-step with [`translate`]'s
/// acceptance: every `Some` here must correspond to a key `translate`
/// actually accepts for that action and set, and vice versa. The
/// `label_and_translate_agree` test below enforces this.
pub fn label(action: TreeAction, set: KeyBindingSet) -> Option<&'static str> {
    use TreeAction::*;
    match (action, set) {
        (MarkUp, _) => Some("Shift+Up"),
        (MarkDown, _) => Some("Shift+Down"),
        (Delete, _) => Some("d d"),
        (Cut, KeyBindingSet::Normal) => Some("Ctrl+X"),
        (Cut, KeyBindingSet::Vim) => None,
        (Copy, KeyBindingSet::Normal) => Some("Ctrl+C"),
        (Copy, KeyBindingSet::Vim) => Some("y"),
        (PasteAfter, KeyBindingSet::Normal) => Some("Ctrl+V"),
        (PasteAfter, KeyBindingSet::Vim) => Some("p"),
        // The normal set has no dedicated key for "paste before": pressing
        // Ctrl+V opens a before/after dialog when the selection is a first
        // sibling (contracts/keymap.md), and the dialog itself picks
        // between PasteAfter and PasteBefore. The contract's key column
        // spells this "(dialog)" rather than a key combination, so the
        // label mirrors that text verbatim; it does not correspond to a
        // single raw key event the way the other labels do (see the
        // `label_and_translate_agree` test below for how that is handled).
        (PasteBefore, KeyBindingSet::Normal) => Some("(dialog)"),
        (PasteBefore, KeyBindingSet::Vim) => Some("P"),
        (PasteAsChild, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn vim_set_has_no_ctrl_cvxz_bindings() {
        let letters = ('a'..='z').chain('A'..='Z').chain('0'..='9');
        for c in letters {
            let ev = key(KeyCode::Char(c), KeyModifiers::CONTROL);
            let result = translate(ev, KeyBindingSet::Vim);
            if matches!(c.to_ascii_lowercase(), 'c' | 'v' | 'x' | 'z') {
                assert_eq!(
                    result, None,
                    "vim set must not bind Ctrl+{c} (C-002), got {result:?}"
                );
            }
        }
    }

    #[test]
    fn both_shift_up_encodings_translate_to_mark_up() {
        let ev = key(KeyCode::Up, KeyModifiers::SHIFT);
        assert_eq!(
            translate(ev, KeyBindingSet::Normal),
            Some(Action::Tree(TreeAction::MarkUp))
        );
        assert_eq!(
            translate(ev, KeyBindingSet::Vim),
            Some(Action::Tree(TreeAction::MarkUp))
        );
    }

    #[test]
    fn shift_down_translates_to_mark_down_in_both_sets() {
        let ev = key(KeyCode::Down, KeyModifiers::SHIFT);
        assert_eq!(
            translate(ev, KeyBindingSet::Normal),
            Some(Action::Tree(TreeAction::MarkDown))
        );
        assert_eq!(
            translate(ev, KeyBindingSet::Vim),
            Some(Action::Tree(TreeAction::MarkDown))
        );
    }

    #[test]
    fn every_tree_action_has_a_key_or_is_documented_menu_only() {
        let actions = [
            TreeAction::MarkUp,
            TreeAction::MarkDown,
            TreeAction::Delete,
            TreeAction::Cut,
            TreeAction::Copy,
            TreeAction::PasteAfter,
            TreeAction::PasteBefore,
            TreeAction::PasteAsChild,
        ];
        let sets = [KeyBindingSet::Normal, KeyBindingSet::Vim];
        for &action in &actions {
            for &set in &sets {
                let result = label(action, set);
                let expected_none =
                    action == TreeAction::PasteAsChild || (action == TreeAction::Cut && set == KeyBindingSet::Vim);
                if expected_none {
                    assert_eq!(result, None, "{action:?}/{set:?} should have no key");
                } else {
                    assert!(result.is_some(), "{action:?}/{set:?} should have a key");
                }
            }
        }
    }

    /// Reconstructs a plausible [`KeyEvent`] from the key text returned by
    /// [`label`], mirroring the small vocabulary of texts `label` actually
    /// produces. Kept local and private: it is small enough that a
    /// duplicate in a later work package's tests is not worth the friction
    /// of extracting a shared helper now.
    ///
    /// `"(dialog)"` (normal-set `PasteBefore`) has no corresponding raw key
    /// event — reaching it goes through a dialog opened by `Ctrl+V`, not a
    /// single keystroke — so it is not handled here; callers must skip it.
    fn event_for_label(text: &str) -> KeyEvent {
        match text {
            "Shift+Up" => key(KeyCode::Up, KeyModifiers::SHIFT),
            "Shift+Down" => key(KeyCode::Down, KeyModifiers::SHIFT),
            "d d" => key(KeyCode::Char('d'), KeyModifiers::NONE),
            "Ctrl+X" => key(KeyCode::Char('x'), KeyModifiers::CONTROL),
            "Ctrl+C" => key(KeyCode::Char('c'), KeyModifiers::CONTROL),
            "Ctrl+V" => key(KeyCode::Char('v'), KeyModifiers::CONTROL),
            "y" => key(KeyCode::Char('y'), KeyModifiers::NONE),
            "p" => key(KeyCode::Char('p'), KeyModifiers::NONE),
            "P" => key(KeyCode::Char('P'), KeyModifiers::SHIFT),
            other => panic!("no event mapping for label text {other:?}"),
        }
    }

    #[test]
    fn label_and_translate_agree() {
        let actions = [
            TreeAction::MarkUp,
            TreeAction::MarkDown,
            TreeAction::Delete,
            TreeAction::Cut,
            TreeAction::Copy,
            TreeAction::PasteAfter,
            TreeAction::PasteBefore,
            TreeAction::PasteAsChild,
        ];
        let sets = [KeyBindingSet::Normal, KeyBindingSet::Vim];
        for &action in &actions {
            for &set in &sets {
                let Some(text) = label(action, set) else {
                    continue;
                };
                // Normal-set PasteBefore is reached via a dialog, not a
                // direct key; see `event_for_label`'s doc comment.
                if (action, set) == (TreeAction::PasteBefore, KeyBindingSet::Normal) {
                    assert_eq!(text, "(dialog)");
                    continue;
                }
                let ev = event_for_label(text);
                assert_eq!(
                    translate(ev, set),
                    Some(Action::Tree(action)),
                    "label({action:?}, {set:?}) = {text:?} but translate disagrees"
                );
            }
        }
    }

    #[test]
    fn key_binding_set_round_trips_through_display_and_from_str() {
        assert_eq!(KeyBindingSet::Normal.to_string(), "normal");
        assert_eq!(KeyBindingSet::Vim.to_string(), "vim");
        assert_eq!("normal".parse::<KeyBindingSet>().unwrap(), KeyBindingSet::Normal);
        assert_eq!("vim".parse::<KeyBindingSet>().unwrap(), KeyBindingSet::Vim);
        assert!("bogus".parse::<KeyBindingSet>().is_err());
    }

    #[test]
    fn key_binding_set_default_is_normal() {
        assert_eq!(KeyBindingSet::default(), KeyBindingSet::Normal);
    }
}

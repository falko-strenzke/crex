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

//! The in-app element buffer, and the delete/copy/cut operations that fill
//! or consume it.
//!
//! Copy, cut and (a later work package's) paste all need to move a run of
//! `Node`s around without keeping them alive as `Node`s: `Node` borrows its
//! meaning from the document tree it came from (offsets, parent links via
//! path, the specific `roots` vector it lives in), and that meaning goes
//! stale the moment the user edits the document, switches files, or the
//! marked range is deleted out from under it. Research R4 concluded the
//! only representation that survives all of that unchanged is the DER
//! encoding itself — bytes have no dependency on where they came from.
//! [`ElementBuffer`] is therefore built once, in [`ElementBuffer::from_operand`],
//! by encoding the operand's `Node`s via [`crate::ber::encode_forest`] and
//! then discarding the `Node`s; a later work package reads it back with
//! `ber::parse_forest`.
//!
//! This buffer is deliberately **not** the same thing as the system
//! clipboard, and **not** the same thing as a vim editor's own yank
//! register (`Editor::Hex`/`Editor::Text`'s per-editor register, a
//! separate concern introduced by a later work package for in-value
//! editing). The system clipboard is an external, text-based channel
//! (hex text, so it round-trips through anything); this buffer is crex's
//! own binary, in-process store, filled by every copy/cut/yank regardless
//! of key-binding set (research R12) so that paste inside crex always
//! works even where no clipboard helper is installed, and so that vim's
//! `y` — which must never touch the system clipboard (FR-025) — still has
//! somewhere to put what it yanked.

use crate::app::{node_at, node_at_mut, App, RowSource};
use crate::ber::{self, Node};
use crate::clipboard;
use crate::keymap::KeyBindingSet;
use crate::mark::Operand;

/// The DER encoding of a copied/cut/yanked run of elements, plus how many
/// elements it holds (so status lines can say "3 elements" without
/// re-parsing the bytes). Replaced whole by every copy/cut/yank — never
/// partially updated — and entirely independent of the document that
/// produced it, per the module doc comment above.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementBuffer {
    pub bytes: Vec<u8>,
    pub count: usize,
}

impl ElementBuffer {
    /// Encodes the `Node`s named by `operand` (looked up in whichever
    /// forest `operand.source` addresses) into a fresh buffer. The `Node`
    /// references live only for the duration of this call — only the
    /// encoded bytes and the count survive it, per the module doc comment's
    /// rationale for why this type holds bytes and not `Node`s.
    pub fn from_operand(app: &App, operand: &Operand) -> Self {
        let siblings = app.forest(operand.source).and_then(|roots| {
            if operand.parent.is_empty() {
                Some(roots)
            } else {
                node_at(roots, &operand.parent).map(|n| n.children.as_slice())
            }
        });
        let nodes: Vec<Node> = match siblings {
            Some(siblings) => operand
                .range
                .clone()
                .filter_map(|i| siblings.get(i).cloned())
                .collect(),
            None => Vec::new(),
        };
        let bytes = ber::encode_forest(&nodes);
        ElementBuffer { count: nodes.len(), bytes }
    }
}

impl App {
    /// The mutable node vector backing `source`'s forest, for the
    /// structural edits below (removing one or more top-level elements).
    /// Mirrors the `match` `delete_selected` used inline before this work
    /// package generalised it; the two variants with no editable forest of
    /// their own (`DecryptedPlaceholder`, which carries no forest at all,
    /// and `CmsRevealed`, always read-only) never reach here because
    /// `operand()` already refuses to build an `Operand` for either — see
    /// `mark::App::row_refusal`.
    pub(crate) fn forest_vec_mut(&mut self, source: RowSource) -> Option<&mut Vec<Node>> {
        match source {
            RowSource::Document => Some(&mut self.roots),
            RowSource::Decrypted => self.decrypted.as_mut().map(|d| &mut d.roots),
            RowSource::Pkcs12Revealed(idx) => {
                self.pkcs12.as_mut().and_then(|p| p.regions.get_mut(idx)).map(|r| &mut r.roots)
            }
            RowSource::DecryptedPlaceholder | RowSource::CmsRevealed => unreachable!(
                "operand() never builds an Operand for a read-only/placeholder row"
            ),
        }
    }

    /// The first node an `operand` names — `operand.parent` with
    /// `*operand.range.start()` appended — used for the single-element
    /// delete confirmation's "at offset N" wording, which the WP04 spec
    /// requires to name the range's first element regardless of where
    /// within a mark `self.selected` currently sits.
    fn operand_first_node(&self, operand: &Operand) -> Option<&Node> {
        let mut path = operand.parent.clone();
        path.push(*operand.range.start());
        self.forest(operand.source).and_then(|roots| node_at(roots, &path))
    }

    /// Removes every element `operand` names from its forest, from the
    /// highest sibling index to the lowest so that removing one never
    /// shifts the index of one not yet removed, then rebuilds and places
    /// the cursor per FR-008.
    ///
    /// Cursor placement works by a small trick shared with the pre-WP04
    /// `delete_selected`: `self.rebuild()` captures `self.rows.get(self.selected)`
    /// as "the row to try to reselect" *before* it rebuilds `self.rows`, and
    /// falls back to leaving `self.selected` exactly as it is (then clamped
    /// to the new `self.rows.len()`) when that row's path can no longer be
    /// found — which, for a just-deleted row, it never can. So setting
    /// `self.selected` to the flat row index the operand's *first* element
    /// currently occupies, right before removing anything, makes that
    /// fall-through land exactly on: the row that now occupies that slot
    /// (the row that followed the operand), or — if the operand ran to the
    /// end of `self.rows` — the clamp lands on the previous row (the one
    /// before the operand), or — if that previous row no longer exists
    /// either (the operand was every row remaining under its parent) — on
    /// the parent itself, which is exactly what occupies that slot once
    /// its last child is gone. No special-casing is needed for any of the
    /// three FR-008 cases; they all fall out of this one positional clamp.
    fn remove_operand(&mut self, operand: &Operand) {
        if let Some(idx) = self.rows.iter().position(|r| {
            r.source == operand.source
                && r.path.len() == operand.parent.len() + 1
                && r.path[..operand.parent.len()] == operand.parent[..]
                && r.path[operand.parent.len()] == *operand.range.start()
        }) {
            self.selected = idx;
        }
        let Some(roots) = self.forest_vec_mut(operand.source) else { return };
        if operand.parent.is_empty() {
            for i in operand.range.clone().rev() {
                if i < roots.len() {
                    roots.remove(i);
                }
            }
        } else if let Some(p) = node_at_mut(roots, &operand.parent) {
            for i in operand.range.clone().rev() {
                if i < p.children.len() {
                    p.children.remove(i);
                }
            }
        }
        self.dirty = true;
        self.rebuild();
    }

    /// Builds the buffer for `operand` and decides what the status line
    /// says, without touching the system clipboard under vim bindings
    /// (FR-025) — shared by `copy_operand` and `cut_operand` so the
    /// normal/vim/clipboard-failure wording rules exist in exactly one
    /// place. `verb` is "copied" or "cut"; the vim wording ("yanked") is
    /// independent of it, since vim's yank and crex's cut are worded the
    /// same regardless of which key produced them ("N elements cut" reads
    /// oddly for a `y` press, so cut's caller only ever passes "cut" for the
    /// vim `d`d-then-y-equivalent -- see `cut_operand`, which is the only
    /// caller that ever passes "cut").
    fn copy_operand_buffer(&self, operand: &Operand, verb: &str) -> (ElementBuffer, String) {
        let buffer = ElementBuffer::from_operand(self, operand);
        let n = buffer.count;
        let suffix = if n == 1 { "" } else { "s" };
        if self.bindings == KeyBindingSet::Vim {
            let status = if verb == "cut" {
                format!("{n} element{suffix} cut")
            } else {
                format!("{n} element{suffix} yanked")
            };
            return (buffer, status);
        }
        let hex = ber::hex_pairs(&buffer.bytes).replace(' ', "");
        let status = match clipboard::write(&hex) {
            Ok(()) => format!("{n} element{suffix} {verb} to the clipboard as hex"),
            Err(reason) => {
                format!("{n} element{suffix} {verb} — {reason} — paste within crex still works")
            }
        };
        (buffer, status)
    }

    /// `d`: delete the operand (the marked range, or just the current
    /// selection when nothing is marked). Two-step confirmation, exactly
    /// as before this work package for the single-element case — the
    /// confirmation and success wording below for `n == 1` are byte-for-byte
    /// what `delete_selected` produced prior to this work package.
    pub fn delete_operand(&mut self) {
        let operand = match self.operand() {
            Ok(op) => op,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        let n = operand.range.clone().count();
        if !self.delete_confirm {
            self.delete_confirm = true;
            let first = self.operand_first_node(&operand);
            let offset = first.map(|node| node.offset).unwrap_or_default();
            self.status = if n == 1 {
                format!(
                    "delete {} at offset {}? press d again to confirm",
                    first.map(|node| node.type_name()).unwrap_or_default(),
                    offset,
                )
            } else {
                format!("delete {n} elements at offset {offset}? press d again to confirm")
            };
            return;
        }
        self.delete_confirm = false;
        self.remove_operand(&operand);
        self.status = if self.rows.is_empty() {
            "element deleted — document is now empty ('i' inserts, 'Ctrl+S' writes)".to_string()
        } else if n == 1 {
            "element deleted — 'Ctrl+S' writes the file".to_string()
        } else {
            format!("{n} elements deleted — 'Ctrl+S' writes the file")
        };
    }

    /// Ctrl+C (normal bindings) / `y` (vim bindings): fill the element
    /// buffer, and — normal bindings only — also the system clipboard, as
    /// uppercase uninterrupted hex text (FR-009, FR-010). Vim's `y` calls
    /// this exact function too (there is no separate `Yank` action; see
    /// `keymap::TreeAction`'s doc comment) and never touches the clipboard,
    /// because `copy_operand_buffer` above reads `self.bindings` to decide.
    pub fn copy_operand(&mut self) {
        let operand = match self.operand() {
            Ok(op) => op,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        let (buffer, status) = self.copy_operand_buffer(&operand, "copied");
        self.element_buffer = Some(buffer);
        self.status = status;
    }

    /// Ctrl+X (either binding set — vim has no dedicated cut key today, see
    /// `keymap::label`): copy the operand, then remove it with no second
    /// confirmation (spec.md User Story 3, acceptance scenario 4). Makes no
    /// change at all when `operand()` itself fails — the only realistic way
    /// for "neither the clipboard nor the buffer received the data" to
    /// happen, since encoding a valid operand cannot fail (NFR-003).
    pub fn cut_operand(&mut self) {
        let operand = match self.operand() {
            Ok(op) => op,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        let (buffer, status) = self.copy_operand_buffer(&operand, "cut");
        self.element_buffer = Some(buffer);
        self.remove_operand(&operand);
        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Container;
    use std::path::PathBuf;

    fn test_app(data: &[u8]) -> App {
        let roots = ber::parse_forest(data, 0).unwrap();
        App::new(
            PathBuf::from("/nonexistent/in"),
            PathBuf::from("/nonexistent/out"),
            Container::Raw,
            roots,
            data.len(),
        )
    }

    /// `SEQUENCE { INTEGER 1, INTEGER 2, INTEGER 3, INTEGER 4, INTEGER 5 }`
    /// — one constructed parent (row 0) with five siblings (rows 1..=5),
    /// the same shape as `mark`'s own fixture, for consistency.
    fn five_integers() -> Vec<u8> {
        vec![
            0x30, 0x0F, //
            0x02, 0x01, 0x01, //
            0x02, 0x01, 0x02, //
            0x02, 0x01, 0x03, //
            0x02, 0x01, 0x04, //
            0x02, 0x01, 0x05,
        ]
    }

    #[test]
    fn range_delete_removes_all_marked_elements_and_places_cursor_correctly() {
        let data = five_integers();
        let mut app = test_app(&data);
        app.select(1); // first INTEGER
        app.mark_extend(1); // marks siblings 0..=1
        app.mark_extend(1); // marks siblings 0..=2 (three elements)
        app.delete_operand(); // first press: arms confirmation
        assert!(app.delete_confirm);
        assert_eq!(app.status, "delete 3 elements at offset 2? press d again to confirm");
        app.delete_operand(); // second press: actually deletes
        assert!(!app.delete_confirm);
        // Two INTEGERs remain (the ones that were siblings 3 and 4).
        assert_eq!(app.rows.len(), 3); // SEQUENCE + 2 remaining INTEGERs
        assert_eq!(app.status, "3 elements deleted — 'Ctrl+S' writes the file");
        // The cursor lands on the row that followed the marked range — the
        // first surviving INTEGER (originally sibling index 3). Its offset
        // is 2 (not its pre-deletion 11), because `rebuild()` re-encodes
        // and re-parses the whole document, so offsets shift once the
        // three deleted elements' bytes are gone.
        let cursor = app.selected_node().unwrap();
        assert_eq!(cursor.offset, 2);
        assert_eq!(cursor.value, vec![0x04], "cursor should land on the old INTEGER 4");
        assert!(app.dirty);
    }

    #[test]
    fn single_element_delete_wording_is_unchanged() {
        let data = five_integers();
        let mut app = test_app(&data);
        app.select(1); // first INTEGER, no mark
        assert!(app.mark.is_none());
        app.delete_operand();
        assert_eq!(app.status, "delete INTEGER at offset 2? press d again to confirm");
        app.delete_operand();
        assert_eq!(app.status, "element deleted — 'Ctrl+S' writes the file");
        assert_eq!(app.rows.len(), 5); // SEQUENCE + 4 remaining INTEGERs
    }

    #[test]
    fn deleting_the_only_element_reports_the_document_is_empty() {
        let data = [0x05, 0x00]; // a lone NULL
        let mut app = test_app(&data);
        app.select(0);
        app.delete_operand();
        app.delete_operand();
        assert_eq!(
            app.status,
            "element deleted — document is now empty ('i' inserts, 'Ctrl+S' writes)"
        );
        assert!(app.rows.is_empty());
    }

    #[test]
    fn copy_fills_clipboard_and_buffer_under_normal_bindings() {
        let data = five_integers();
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Normal;
        app.select(1);
        app.copy_operand();
        let buffer = app.element_buffer.as_ref().expect("buffer filled");
        assert_eq!(buffer.count, 1);
        assert_eq!(buffer.bytes, vec![0x02, 0x01, 0x01]);
        // The CI/sandbox test environment realistically has no clipboard
        // helper installed, so this doubles as the "copy survives a
        // clipboard failure" case: either wording is acceptable, but the
        // buffer must be filled either way.
        assert!(
            app.status == "1 element copied to the clipboard as hex"
                || app.status.contains("1 element copied —"),
            "unexpected status: {:?}",
            app.status
        );
    }

    #[test]
    fn cut_aborts_when_operand_is_invalid() {
        // An empty document: nothing is selected, so operand() fails.
        let mut app = test_app(&[]);
        assert!(app.operand().is_err());
        app.cut_operand();
        assert!(!app.dirty);
        assert!(app.element_buffer.is_none());
    }

    #[test]
    fn yank_never_touches_clipboard_under_vim_bindings() {
        let data = five_integers();
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Vim;
        app.select(1);
        app.copy_operand(); // this is vim's 'y' — see keymap::TreeAction::Copy
        let buffer = app.element_buffer.as_ref().expect("buffer filled");
        assert_eq!(buffer.count, 1);
        // There is no direct way to assert the clipboard was not touched
        // without a mocking seam in `clipboard.rs` (out of scope for this
        // WP); the vim-specific "yanked" wording is the observable proxy
        // for "the clipboard-write branch was skipped entirely".
        assert_eq!(app.status, "1 element yanked");
    }

    #[test]
    fn cut_removes_the_operand_with_no_second_confirmation() {
        let data = five_integers();
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Vim;
        app.select(1);
        app.cut_operand();
        assert!(!app.delete_confirm);
        assert_eq!(app.rows.len(), 5); // SEQUENCE + 4 remaining INTEGERs
        assert_eq!(app.status, "1 element cut");
        assert!(app.dirty);
        let buffer = app.element_buffer.as_ref().expect("buffer filled");
        assert_eq!(buffer.bytes, vec![0x02, 0x01, 0x01]);
    }
}

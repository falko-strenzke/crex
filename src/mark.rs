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

//! The tree mark: a contiguous run of siblings extended with Shift+Up/Down
//! (`keymap::TreeAction::MarkUp`/`MarkDown`).
//!
//! A mark is stored as *where* it is, not *which rows of `app.rows` it
//! currently spans*: a [`RowSource`] (which forest), the path of the
//! common parent, and a pair of sibling indices (`anchor`, `active`) within
//! that parent's children. It is deliberately never stored as a range of
//! `app.rows` indices, because `rebuild_rows()` — which runs after every
//! structural edit, filter change, or decrypt — recomputes `rows` from
//! scratch, and a row index carries no meaning across that rebuild. A
//! parent path plus sibling indices does, because the only thing that
//! actually renumbers a parent's children is inserting or removing one of
//! them, and every operation that does that clears the mark first anyway
//! (see the invariants on `App::clear_mark`'s call sites).
//!
//! `anchor` is the sibling where marking began (the first Shift+Up/Down on
//! a given selection); `active` is the sibling the most recent Shift+Up/
//! Down landed on. Extending doesn't just grow `active` away from
//! `anchor` — enough Shift+Up presses cross back past `anchor` and keep
//! marking on the *other* side, the same way a text editor's shift-
//! selection can cross back over where it started. [`Mark::range`] always
//! reports `min(anchor, active)..=max(anchor, active)`, so which end moved
//! last is only visible in `anchor`/`active` themselves, never in the
//! derived range.

use crate::app::{node_at, App, Row, RowSource};

/// A contiguous run of siblings under one parent, in one forest. See the
/// module doc comment for why it is addressed by path + sibling index
/// rather than by `app.rows` index.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mark {
    /// Which forest the mark is in (the open document, a decrypted PKCS#8
    /// value, a PKCS#12 revealed region, ...).
    pub source: RowSource,
    /// Path of the common parent; empty means the top level of `source`'s
    /// forest.
    pub parent: Vec<usize>,
    /// Sibling index where marking began.
    pub anchor: usize,
    /// Sibling index the cursor end is currently extended to. Never
    /// clamped to stay on one side of `anchor` — see the module doc
    /// comment on crossing back past it.
    pub active: usize,
}

impl Mark {
    /// The marked sibling indices, inclusive of both ends, regardless of
    /// which of `anchor`/`active` is numerically smaller.
    pub fn range(&self) -> std::ops::RangeInclusive<usize> {
        self.anchor.min(self.active)..=self.anchor.max(self.active)
    }

    /// Number of marked siblings.
    pub fn count(&self) -> usize {
        self.range().count()
    }
}

/// What a tree operation (delete/cut/copy, or a paste-as-child target
/// check) should act on: the marked range when one exists, otherwise just
/// the current selection. See [`App::operand`].
#[derive(Clone, Debug, PartialEq)]
pub struct Operand {
    pub source: RowSource,
    pub parent: Vec<usize>,
    pub range: std::ops::RangeInclusive<usize>,
}

impl App {
    /// Drop the current mark, if any.
    ///
    /// This is the single funnel every navigation method routes through —
    /// besides `mark_extend` itself, no other change to `self.selected`,
    /// `self.focus`, or `self.filter` may leave a mark standing. In
    /// `src/app.rs` that means: `select` (and therefore `move_by` and
    /// every `collapse_or_parent`/`expand_or_child` branch that calls it),
    /// `rebuild_rows` (called after every structural edit, decrypt, filter
    /// keystroke, and file open/switch — covering `toggle_expand`'s and
    /// `collapse_or_parent`'s collapse branches, which change `self.rows`
    /// without going through `select`), `toggle_focus` (leaving the
    /// document pane), and `start_filter` (entering the filter field,
    /// ahead of the first keystroke `rebuild_rows` would otherwise catch).
    pub fn clear_mark(&mut self) {
        self.mark = None;
    }

    /// The reason marking — or deriving an [`Operand`] from — `row` must be
    /// refused, shared by `mark_extend`'s start-a-mark step and
    /// `operand`'s fallback-to-selection step: both only ever want to
    /// address a row that is a real, editable node, so both apply the same
    /// rule. Reuses the exact refusal wording `delete_selected`/
    /// `start_insert` already use for the same situations, via the shared
    /// helpers those two now call as well, rather than re-deriving new
    /// text for a case that already has an established message.
    fn row_refusal(&self, row: &Row) -> Option<String> {
        if row.source == RowSource::DecryptedPlaceholder {
            return Some("decrypt the content before editing it".to_string());
        }
        if let Some(reason) = App::elided_reason(row) {
            return Some(reason);
        }
        if let Some(reason) = self.uneditable_reveal_reason(row) {
            return Some(reason);
        }
        if let Some(reason) = App::protected_root_reason(row) {
            return Some(reason);
        }
        None
    }

    /// Number of siblings `parent` (a path within `source`'s forest, empty
    /// for the top level) actually has right now — the bound `mark_extend`
    /// checks `active` against. Always the *true*, unfiltered count: a
    /// mark can never coexist with an active filter (see `mark_extend`
    /// below), so this only ever runs against the real tree.
    fn sibling_count(&self, source: RowSource, parent: &[usize]) -> usize {
        let Some(roots) = self.forest(source) else { return 0 };
        if parent.is_empty() {
            roots.len()
        } else {
            node_at(roots, parent).map(|n| n.children.len()).unwrap_or(0)
        }
    }

    /// Shift+Up (`delta = -1`) / Shift+Down (`delta = 1`) on the tree pane.
    ///
    /// The first call on a given selection starts a mark (`anchor = active
    /// =` that row's sibling index) and then immediately applies `delta` —
    /// so the very first press already marks two elements, matching the
    /// "hold Shift, tap the arrow" feel of a text editor's shift-
    /// selection. Every later call just moves `active` by `delta`, bounds-
    /// checked against the parent's real sibling count (`sibling_count`
    /// above); it is deliberately *not* clamped to stay on `anchor`'s
    /// side, so it can cross back past `anchor` and keep marking on the
    /// other side — the single easiest part of this to get subtly wrong by
    /// clamping `active` there instead.
    pub fn mark_extend(&mut self, delta: isize) {
        if !self.filter.is_empty() {
            self.status =
                "cannot mark while the tree filter is active — clear it first".to_string();
            return;
        }
        if self.mark.is_none() {
            let Some(row) = self.rows.get(self.selected).cloned() else { return };
            if let Some(reason) = self.row_refusal(&row) {
                self.status = reason;
                return;
            }
            let Some((&anchor, parent)) = row.path.split_last() else { return };
            self.mark = Some(Mark {
                source: row.source,
                parent: parent.to_vec(),
                anchor,
                active: anchor,
            });
        }
        let (source, parent, active) = {
            let mark = self.mark.as_ref().expect("just ensured Some above");
            (mark.source, mark.parent.clone(), mark.active)
        };
        let sibling_count = self.sibling_count(source, &parent);
        let new_active = active as isize + delta;
        if new_active < 0 || new_active >= sibling_count as isize {
            self.status = if delta > 0 {
                "the mark cannot extend past the parent's last element".to_string()
            } else {
                "the mark cannot extend past the parent's first element".to_string()
            };
            return;
        }
        if let Some(mark) = self.mark.as_mut() {
            mark.active = new_active as usize;
        }
    }

    /// What a tree operation should act on: the marked range if one
    /// exists, otherwise the current selection (a single-element range).
    /// See [`Operand`].
    ///
    /// Refuses (rather than silently falling back) on exactly the rows
    /// `mark_extend` refuses to start a mark on — see `row_refusal`. Does
    /// *not* separately reject an empty document: there is no selection to
    /// derive an operand from either way, and callers already handle
    /// "nothing to act on" with their own early return, so that case just
    /// falls out of `self.rows.get(self.selected)` returning `None` here.
    pub fn operand(&self) -> Result<Operand, String> {
        if let Some(mark) = &self.mark {
            return Ok(Operand {
                source: mark.source,
                parent: mark.parent.clone(),
                range: mark.range(),
            });
        }
        let row = self
            .rows
            .get(self.selected)
            .ok_or_else(|| "nothing selected".to_string())?;
        if let Some(reason) = self.row_refusal(row) {
            return Err(reason);
        }
        let Some((&last, parent)) = row.path.split_last() else {
            return Err("nothing selected".to_string());
        };
        Ok(Operand {
            source: row.source,
            parent: parent.to_vec(),
            range: last..=last,
        })
    }
}

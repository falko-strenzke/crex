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

//! The paste pipeline: closes the loop opened by `crate::buffer` (bytes
//! *out* via copy/cut/yank) and `crate::clipboard` (clipboard text read
//! *into* bytes via [`crate::clipboard::bytes_for_paste`]).
//!
//! The flow (`plan.md`'s "Key flows", in this module's own words) is:
//!
//! 1. **Resolve a source to bytes.** Either the system clipboard — read via
//!    [`crate::clipboard::read`] and interpreted via
//!    [`crate::clipboard::bytes_for_paste`] (hex, then base64, then PEM,
//!    then raw), falling back to the in-app element buffer when the
//!    clipboard is empty or unavailable (FR-017) — or the element buffer
//!    directly (vim's `p`/`P` always use this branch, and it is always raw
//!    DER already, per `crate::buffer::ElementBuffer::from_operand`'s
//!    guarantee).
//! 2. **Validate.** The resolved bytes must parse as a complete BER/DER
//!    forest via [`crate::ber::parse_forest`] with nothing left over — see
//!    [`App::paste_tree`]'s doc comment for exactly what "nothing left
//!    over" means for that function.
//! 3. **Place.** The parsed elements are spliced in before, after, or as
//!    the first child of the current selection, subject to the same
//!    region rules `App::start_insert` already enforces for the 'i'/'I'
//!    insert dialog (shared via `crate::app::sibling_insert_root_reason`
//!    and the pure `elided_reason`/`uneditable_reveal_reason` checks).
//!
//! Any failure at step 1 or 2 leaves the document completely untouched
//! (NFR-003) — no `rebuild()`, no `dirty = true`. Only step 3's actual
//! splice mutates anything, and by the time it runs the input is already
//! known-good.

use crate::app::{node_at_mut, sibling_insert_root_reason, App, Mode, Row, RowSource};
use crate::ber;
use crate::clipboard;

/// Where a pasted run of elements goes relative to the current selection.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PasteWhere {
    /// Immediately before the selected element, as its new previous
    /// sibling.
    Before,
    /// Immediately after the selected element, as its new next sibling
    /// (the common case, and the default in the placement dialog).
    After,
    /// As the selected element's new first child. Refused when the
    /// selection is primitive and non-encapsulating, exactly like the
    /// 'i'-with-child-target case of `App::start_insert`.
    AsChild,
}

/// Where `App::paste_tree` should read its bytes from.
///
/// The data model sketch in `data-model.md` allows either a borrowed
/// `&ElementBuffer` or an owned `Vec<u8>` for the `Buffer` case; a
/// borrowed form turned out awkward at the call sites this WP actually
/// has (`src/tui.rs`'s dispatch wants to read `app.element_buffer`, then
/// call a `&mut self` method on the same `app` — a borrow that already
/// does not typecheck), so this uses an owned `Vec<u8>` instead, cloned
/// from the element buffer once at the call site. `PasteSource` carries no
/// placement or validation state of its own — it only says where the
/// bytes come from.
pub enum PasteSource {
    /// Read the system clipboard, falling back to the element buffer per
    /// FR-017 when the clipboard is empty, holds no text, or no helper
    /// program is available.
    Clipboard,
    /// Use these bytes directly — already raw DER, exactly the element
    /// buffer's own encoding (or, in a test, deliberately invalid bytes to
    /// exercise the refusal path).
    Buffer(Vec<u8>),
}

/// How the pasted bytes were obtained, carried alongside them so
/// `App::paste_tree` can report — for a refusal, the parser's own message
/// prefixed by this reading — and — for success — the element count and
/// this reading, per `contracts/clipboard-payload.md`.
#[derive(Debug, PartialEq)]
enum Reading {
    /// From the element buffer, whether because the caller asked for it
    /// directly (vim `p`/`P`) or because the clipboard fell back to it
    /// (FR-017).
    Buffer,
    /// From the system clipboard, decoded via [`clipboard::bytes_for_paste`];
    /// the description is that function's own wording (e.g. "read as hex
    /// digits").
    Clipboard(String),
}

impl Reading {
    /// The prefix used ahead of a parser error message, and ahead of the
    /// success count.
    fn describe(&self) -> String {
        match self {
            Reading::Buffer => "the element buffer".to_string(),
            Reading::Clipboard(d) => d.clone(),
        }
    }
}

impl App {
    /// Resolve `source` to `(bytes, reading)`, applying the FR-017
    /// clipboard-to-buffer fallback. Returns `Err(status message)` when
    /// there is nothing to paste at all, or when the clipboard held
    /// hex-looking text with an odd digit count — a genuine refusal
    /// (per `contracts/clipboard-payload.md`'s hex-is-a-typo rule), not a
    /// silent fallback: a clipboard that plainly holds *some* text was not
    /// "empty, holding no text, or unavailable" (FR-017's exact wording),
    /// so it does not qualify for the fallback even though it failed to
    /// parse as any of the four readings.
    fn resolve_paste_source(&self, source: PasteSource) -> Result<(Vec<u8>, Reading), String> {
        match source {
            PasteSource::Buffer(bytes) => Self::resolve_buffer_bytes(bytes),
            PasteSource::Clipboard => {
                Self::resolve_clipboard(clipboard::read(), self.element_buffer.as_ref())
            }
        }
    }

    /// The `Buffer` half of `resolve_paste_source` — its own function so it
    /// can be exercised without a live `App` (only the test module needs
    /// that, but keeping it free of `&self` costs nothing).
    fn resolve_buffer_bytes(bytes: Vec<u8>) -> Result<(Vec<u8>, Reading), String> {
        if bytes.is_empty() {
            return Err("nothing to paste".to_string());
        }
        Ok((bytes, Reading::Buffer))
    }

    /// The `Clipboard` half of `resolve_paste_source`, taking the result of
    /// `clipboard::read()` as a parameter instead of calling it itself —
    /// deliberately, so the FR-017 fallback logic below can be unit tested
    /// against synthetic `Ok(vec![])` / `Err(_)` inputs without touching the
    /// real, process-global system clipboard (which several other tests in
    /// this binary also read and write concurrently, making the live
    /// clipboard's state unpredictable from any one test's point of view).
    fn resolve_clipboard(
        clip: Result<Vec<u8>, String>,
        element_buffer: Option<&crate::buffer::ElementBuffer>,
    ) -> Result<(Vec<u8>, Reading), String> {
        match clip {
            Ok(data) if !data.is_empty() => match clipboard::bytes_for_paste(&data) {
                Ok((bytes, kind)) => Ok((bytes, Reading::Clipboard(kind.describe().into_owned()))),
                Err(reason) => Err(reason),
            },
            // Empty (`Ok(vec![])`, a real but content-less clipboard) or
            // unavailable (`Err`, no helper program): both fall back to the
            // element buffer per FR-017.
            _ => match element_buffer {
                Some(buf) if !buf.bytes.is_empty() => Ok((buf.bytes.clone(), Reading::Buffer)),
                _ => Err("nothing to paste".to_string()),
            },
        }
    }

    /// Resolve `at` against the current selection into `(parent path,
    /// insertion index, forest source)`, applying every region rule
    /// `App::start_insert` enforces for the same placements — refuse a
    /// `DecryptedPlaceholder`/elided/read-only-reveal selection, refuse a
    /// sibling insertion at the top level of a decrypted PKCS#8/PKCS#12
    /// region (C-008), and refuse `AsChild` on a primitive,
    /// non-encapsulating target. An empty document accepts any `at` as the
    /// new top level, exactly like `start_insert`'s own empty-document case.
    fn paste_target(&self, at: PasteWhere) -> Result<(Vec<usize>, usize, RowSource), String> {
        if self.rows.is_empty() {
            return Ok((Vec::new(), 0, RowSource::Document));
        }
        let row = self.rows[self.selected].clone();
        if row.source == RowSource::DecryptedPlaceholder {
            return Err("decrypt the content before editing it".to_string());
        }
        if let Some(reason) = App::elided_reason(&row) {
            return Err(reason);
        }
        if let Some(reason) = self.uneditable_reveal_reason(&row) {
            return Err(reason);
        }
        let path = row.path.clone();
        match at {
            PasteWhere::AsChild => {
                let Some(node) = self.node_for_row(&row) else {
                    return Err("cannot paste here".to_string());
                };
                if !node.constructed && !node.encapsulates {
                    return Err(
                        "cannot paste a child into a primitive element (use before/after instead)"
                            .to_string(),
                    );
                }
                Ok((path, 0, row.source))
            }
            PasteWhere::Before | PasteWhere::After => {
                let (&last, parent) = path.split_last().expect("row paths are non-empty");
                if let Some(reason) = sibling_insert_root_reason(row.source, parent.is_empty()) {
                    return Err(reason);
                }
                let index = if at == PasteWhere::After { last + 1 } else { last };
                Ok((parent.to_vec(), index, row.source))
            }
        }
    }

    /// Paste `source`'s bytes at `at` relative to the current selection.
    ///
    /// Validates the resolved bytes as a complete forest via
    /// [`ber::parse_forest`] before touching anything: reading that
    /// function's loop (`src/ber.rs`, `parse_forest_depth`) confirms it
    /// already guarantees full consumption on `Ok` — each iteration parses
    /// one node from `data[pos..]`, and a node can never report consuming
    /// more bytes than that slice holds (every length check inside
    /// `parse_node`/`parse_header` is bounded by the slice it was given),
    /// so the loop can only return `Ok` once `pos` has reached exactly
    /// `data.len()`; trailing bytes that do not themselves form a valid
    /// node instead make the *next* iteration's `parse_node` call fail,
    /// which propagates out as `Err`. So there is no separate
    /// "did we consume everything" check to add here beyond calling
    /// `parse_forest` and matching on its `Result` — a mismatch between
    /// "some bytes parsed" and "the whole input was one clean forest"
    /// cannot occur for this function. (Trailing bytes that *do* happen to
    /// parse as another valid top-level element are not an error at all —
    /// that is the intended "several PEM blocks concatenated" case from
    /// the spec's edge cases, and becomes one more pasted element.)
    ///
    /// On any failure — nothing to paste, a hex-with-typo clipboard, a
    /// parse error, or a region-rule refusal — this makes **no** change to
    /// the document at all: no `rebuild()`, no `dirty = true` (NFR-003).
    /// On success: the parsed elements are spliced in, `rebuild()` runs,
    /// `dirty` is set, a collapsed target parent is expanded, the first
    /// pasted element is selected, and the status line names the count and
    /// the reading used.
    pub fn paste_tree(&mut self, source: PasteSource, at: PasteWhere) {
        if !self.file_open {
            self.status = "no file open — select one in the browser first".to_string();
            return;
        }
        let (bytes, reading) = match self.resolve_paste_source(source) {
            Ok(v) => v,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        let nodes = match ber::parse_forest(&bytes, 0) {
            Ok(nodes) => nodes,
            Err(e) => {
                self.status = format!("{}: {}", reading.describe(), e);
                return;
            }
        };
        if nodes.is_empty() {
            self.status = "nothing to paste".to_string();
            return;
        }
        let (parent, index, row_source) = match self.paste_target(at) {
            Ok(t) => t,
            Err(msg) => {
                self.status = msg;
                return;
            }
        };
        let count = nodes.len();
        let Some(roots) = self.forest_vec_mut(row_source) else {
            self.status = "cannot paste here".to_string();
            return;
        };
        if parent.is_empty() {
            for (i, node) in nodes.into_iter().enumerate() {
                roots.insert(index + i, node);
            }
        } else {
            let Some(p) = node_at_mut(roots, &parent) else {
                self.status = "cannot paste here".to_string();
                return;
            };
            for (i, node) in nodes.into_iter().enumerate() {
                p.children.insert(index + i, node);
            }
            p.expanded = true; // make the pasted elements visible, per start_insert
        }
        // A dialog may still be open (the before/after choice); return to
        // Browse now that the paste itself has happened.
        self.mode = Mode::Browse;
        self.dirty = true;
        self.rebuild();
        let mut first_path = parent;
        first_path.push(index);
        if let Some(i) =
            self.rows.iter().position(|r: &Row| r.source == row_source && r.path == first_path)
        {
            self.select(i);
        }
        let suffix = if count == 1 { "" } else { "s" };
        self.status = match reading {
            Reading::Buffer => format!("pasted {count} element{suffix} from the element buffer"),
            Reading::Clipboard(d) => format!("pasted {count} element{suffix} — {d}"),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::KeyBindingSet;
    use crate::buffer::ElementBuffer;
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

    /// `SEQUENCE { INTEGER 1, INTEGER 2, INTEGER 3 }` — a small forest with
    /// a constructed parent and three primitive siblings, enough to select
    /// a first and a non-first sibling.
    fn three_integers() -> Vec<u8> {
        vec![
            0x30, 0x09, //
            0x02, 0x01, 0x01, //
            0x02, 0x01, 0x02, //
            0x02, 0x01, 0x03,
        ]
    }

    #[test]
    fn copy_then_paste_round_trips_byte_identical() {
        let data = three_integers();
        let mut app = test_app(&data);
        app.select(1); // first INTEGER
        app.copy_operand();
        let original = ber::encode_forest(&[app.node_for_row(&app.rows[1].clone()).unwrap().clone()]);
        app.select(3); // third INTEGER — a non-first sibling
        let buf = app.element_buffer.as_ref().unwrap().bytes.clone();
        app.paste_tree(PasteSource::Buffer(buf), PasteWhere::After);
        assert!(app.dirty);
        assert_eq!(app.rows.len(), 5); // SEQUENCE + 4 INTEGERs now
        // The freshly pasted element is selected; its encoding must be
        // byte-identical to the original's.
        let pasted = app.selected_node().unwrap().clone();
        assert_eq!(ber::encode_forest(&[pasted]), original);
        assert_eq!(app.status, "pasted 1 element from the element buffer");
    }

    #[test]
    fn copy_then_paste_round_trips_a_real_certificate_extension() {
        // A structural, non-dumpasn1 stand-in for the WP's suggested
        // dumpasn1-compat check: paste a real extension copied out of a
        // real certificate into another document and confirm the DER
        // encoding survives byte-for-byte.
        let cert = std::fs::read("testdata/chain/server.der").expect("test fixture present");
        let mut source_app = test_app(&cert);
        // Row 0 is the outer Certificate SEQUENCE; find some non-root,
        // constructed row deep enough to be a realistic "extension-shaped"
        // copy without depending on exact certificate layout.
        let idx = source_app
            .rows
            .iter()
            .position(|r| r.path.len() >= 3)
            .expect("certificate has nested structure");
        source_app.select(idx);
        source_app.copy_operand();
        let buf = source_app.element_buffer.as_ref().unwrap().bytes.clone();
        let original = buf.clone();

        let dest_data = three_integers();
        let mut dest_app = test_app(&dest_data);
        dest_app.select(1);
        dest_app.paste_tree(PasteSource::Buffer(buf), PasteWhere::Before);
        assert!(dest_app.dirty);
        let pasted = dest_app.selected_node().unwrap().clone();
        assert_eq!(ber::encode_forest(&[pasted]), original);
    }

    #[test]
    fn dialog_triggers_only_on_first_sibling_under_normal_bindings() {
        // Exercises the real dispatch (`crate::tui::paste_after_key`) that
        // `src/tui.rs`'s handle_document_key routes Ctrl+V through — the
        // dialog-or-not decision lives entirely there, not in `paste_tree`,
        // so a real test of it has to go through that function.
        let data = three_integers();
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Normal;
        app.element_buffer = Some(ElementBuffer { bytes: vec![0x05, 0x00], count: 1 });

        // Non-first sibling (third INTEGER, path [3]): pastes immediately —
        // no dialog. This goes through the real system clipboard (this is
        // normal bindings' documented Ctrl+V behaviour), which is
        // process-global, shared, and concurrently written by other tests
        // in this binary — so only the property that holds regardless of
        // what the clipboard actually contained at the time is asserted
        // here (no dialog ever opens for a non-first sibling); the
        // dedicated `clipboard_empty_falls_back_to_element_buffer` and
        // `nothing_to_paste_when_clipboard_and_buffer_are_both_empty` tests
        // below cover the fallback outcome itself deterministically, via
        // `App::resolve_clipboard` directly rather than the live clipboard.
        app.select(3);
        assert!(!crate::tui::is_first_sibling(&app));
        crate::tui::paste_after_key(&mut app);
        assert!(matches!(app.mode, Mode::Browse), "no dialog for a non-first sibling");

        // First sibling (first INTEGER, path [0]): the dialog opens and
        // nothing is pasted yet.
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Normal;
        app.select(1);
        assert!(crate::tui::is_first_sibling(&app));
        crate::tui::paste_after_key(&mut app);
        assert!(matches!(app.mode, Mode::PasteWhere(_)), "first sibling opens the dialog");
        assert!(!app.dirty, "the dialog opening must not paste anything yet");
    }

    #[test]
    fn vim_paste_never_shows_dialog() {
        let data = three_integers();
        let mut app = test_app(&data);
        app.bindings = KeyBindingSet::Vim;
        app.select(1); // first sibling — would open the dialog under normal bindings
        app.element_buffer = Some(ElementBuffer { bytes: vec![0x05, 0x00], count: 1 });
        crate::tui::paste_after_key(&mut app);
        assert!(matches!(app.mode, Mode::Browse), "vim paste never opens a dialog");
        assert!(app.dirty);
        assert_eq!(app.rows.len(), 5); // SEQUENCE + 4 INTEGERs (NULL inserted)

        // Even at the first sibling, still no dialog — vim bindings simply
        // never route through the dialog-opening branch at all.
        let mut app2 = test_app(&data);
        app2.bindings = KeyBindingSet::Vim;
        app2.select(1);
        app2.element_buffer = Some(ElementBuffer { bytes: vec![0x05, 0x00], count: 1 });
        assert!(crate::tui::is_first_sibling(&app2));
        crate::tui::paste_after_key(&mut app2);
        assert!(matches!(app2.mode, Mode::Browse));
    }

    #[test]
    fn invalid_paste_leaves_document_unchanged() {
        let data = three_integers();
        let mut app = test_app(&data);
        app.select(1);
        let rows_before = app.rows.len();
        // A truncated INTEGER: tag+length claim 2 content octets, only 1 given.
        let truncated = vec![0x02, 0x02, 0x01];
        app.paste_tree(PasteSource::Buffer(truncated), PasteWhere::After);
        assert!(!app.dirty);
        assert_eq!(app.rows.len(), rows_before);
        assert!(app.status.contains("the element buffer:"), "status: {:?}", app.status);
    }

    #[test]
    fn region_rule_refuses_paste_into_decrypted_pkcs8_top_level() {
        // Same rule App::start_insert already enforces for 'i'/'I', reused
        // here via `sibling_insert_root_reason` (see its doc comment in
        // `src/app.rs`): both call sites must refuse identically, so this
        // exercises the shared function itself with the two region kinds it
        // actually distinguishes — the state paste_target's callers pass it
        // is exactly `(row.source, parent.is_empty())`.
        assert_eq!(
            crate::app::sibling_insert_root_reason(RowSource::Decrypted, true),
            Some("a decrypted PKCS#8 value must remain one top-level SEQUENCE".to_string())
        );
        assert_eq!(
            crate::app::sibling_insert_root_reason(RowSource::Pkcs12Revealed(0), true),
            Some("a decrypted PKCS#12 region must remain one top-level SEQUENCE".to_string())
        );
        // Not at the top level (a non-empty parent path): the rule does not
        // apply, matching start_insert's own "parent.is_empty()" gate.
        assert_eq!(crate::app::sibling_insert_root_reason(RowSource::Decrypted, false), None);
        assert_eq!(crate::app::sibling_insert_root_reason(RowSource::Document, true), None);

        // End-to-end through the real decrypt flow: an actual PKCS#12 file,
        // actually decrypted, selecting the revealed region's own top-level
        // row and attempting a sibling paste there.
        let raw = std::fs::read("testdata/pkcs12.der").expect("test fixture present");
        let (der, container) = crate::input::load(&raw).expect("loads");
        let roots = ber::parse_forest(&der, 0).expect("parses");
        let mut app = App::new(
            PathBuf::from("testdata/pkcs12.der"),
            PathBuf::from("testdata/pkcs12.der"),
            container,
            roots,
            der.len(),
        );
        app.start_decrypt();
        let Mode::Password(ref mut p) = app.mode else { panic!("password prompt not open") };
        for c in "asn1editor".chars() {
            p.insert_char(c);
        }
        app.submit_password();
        let region_root = app
            .rows
            .iter()
            .position(|r| matches!(r.source, RowSource::Pkcs12Revealed(_)) && r.path == [0])
            .expect("a revealed region's top-level row exists");
        app.select(region_root);
        let rows_before = app.rows.len();
        app.paste_tree(PasteSource::Buffer(vec![0x05, 0x00]), PasteWhere::After);
        assert!(!app.dirty);
        assert_eq!(app.rows.len(), rows_before);
        assert_eq!(app.status, "a decrypted PKCS#12 region must remain one top-level SEQUENCE");
    }

    // The next two tests exercise `App::resolve_clipboard` directly with a
    // synthetic `clipboard::read()` result rather than going through the
    // real, process-global system clipboard — several other tests in this
    // binary run concurrently and also read/write it for real, so its
    // actual live content at any instant is not something a single test can
    // rely on. `resolve_clipboard` takes that result as a parameter
    // precisely so this fallback logic (FR-017) can be tested deterministically.

    #[test]
    fn clipboard_empty_falls_back_to_element_buffer() {
        // Both legs of FR-017's fallback condition — no helper installed
        // (`Err`) or a helper present but the clipboard holds nothing
        // (`Ok(vec![])`) — must fall back the same way.
        let buffer = ElementBuffer { bytes: vec![0x01, 0x01, 0xFF], count: 1 };
        for clip in [Ok(Vec::new()), Err("no clipboard helper found".to_string())] {
            let (bytes, reading) = App::resolve_clipboard(clip, Some(&buffer)).expect("falls back");
            assert_eq!(bytes, buffer.bytes);
            assert!(matches!(reading, Reading::Buffer));
        }

        // End to end through `paste_tree`, to confirm the resolved bytes
        // and reading actually produce the documented status wording and a
        // real mutation — using `PasteSource::Buffer` directly (bypassing
        // the live clipboard entirely) to reach the same code path
        // `resolve_paste_source` would after `resolve_clipboard` falls back,
        // since `paste_tree` itself does not distinguish "buffer because
        // asked directly" from "buffer because the clipboard fell back".
        let data = three_integers();
        let mut app = test_app(&data);
        app.select(3);
        app.paste_tree(PasteSource::Buffer(buffer.bytes.clone()), PasteWhere::After);
        assert!(app.dirty);
        assert_eq!(app.status, "pasted 1 element from the element buffer");
    }

    #[test]
    fn nothing_to_paste_when_clipboard_and_buffer_are_both_empty() {
        assert_eq!(
            App::resolve_clipboard(Ok(Vec::new()), None),
            Err("nothing to paste".to_string())
        );
        assert_eq!(
            App::resolve_clipboard(Err("no clipboard helper found".to_string()), None),
            Err("nothing to paste".to_string())
        );
        let empty = ElementBuffer { bytes: Vec::new(), count: 0 };
        assert_eq!(
            App::resolve_clipboard(Ok(Vec::new()), Some(&empty)),
            Err("nothing to paste".to_string()),
            "an element buffer that itself holds nothing does not count as a fallback target"
        );

        // End to end: nothing at all to paste from must leave the document
        // untouched.
        let data = three_integers();
        let mut app = test_app(&data);
        app.select(3);
        app.paste_tree(PasteSource::Buffer(Vec::new()), PasteWhere::After);
        assert!(!app.dirty);
        assert_eq!(app.status, "nothing to paste");
    }

    #[test]
    fn empty_document_paste_becomes_the_new_top_level() {
        let mut app = test_app(&[]);
        assert!(app.rows.is_empty());
        app.paste_tree(PasteSource::Buffer(vec![0x05, 0x00]), PasteWhere::After);
        assert!(app.dirty);
        assert_eq!(app.rows.len(), 1);
        assert_eq!(app.status, "pasted 1 element from the element buffer");
    }

    #[test]
    fn as_child_refuses_a_primitive_non_encapsulating_target() {
        let data = three_integers();
        let mut app = test_app(&data);
        app.select(1); // an INTEGER: primitive, not encapsulating
        app.paste_tree(PasteSource::Buffer(vec![0x05, 0x00]), PasteWhere::AsChild);
        assert!(!app.dirty);
        assert!(app.status.contains("primitive"), "status: {:?}", app.status);
    }
}

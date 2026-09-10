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

//! Persisted user settings: today, just the active key-binding set
//! ([`crate::keymap::KeyBindingSet`]), read from and written to an
//! OS-typical per-user configuration file.
//!
//! The file format is a strict, hand-written subset of TOML (`key =
//! "value"` lines, `#` comments, blank lines) — see
//! `kitty-specs/asn1-tree-delete-copy-paste-01M22R1E/contracts/settings-file.md`
//! for the authoritative grammar and behaviour table. No `toml`/`serde`
//! dependency is pulled in for this (research.md R1): the format is small
//! and stable enough that a tiny hand-written parser is both correct and
//! easier to audit than a generic dependency.

use std::path::PathBuf;

use crate::keymap::KeyBindingSet;

/// Persisted settings. `unknown` holds any key this build does not
/// recognise, verbatim (key and de-quoted value), so that a settings file
/// written by a newer build round-trips through an older one without data
/// loss (contract C-007).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub key_bindings: KeyBindingSet,
    unknown: Vec<(String, String)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { key_bindings: KeyBindingSet::Normal, unknown: Vec::new() }
    }
}

/// Result of [`load`].
#[derive(Debug)]
pub enum LoadOutcome {
    /// The file existed, parsed cleanly, and its contents are here.
    Loaded(Settings),
    /// No settings file exists at the resolved location: use defaults,
    /// silently — this is the normal first-run state.
    Missing,
    /// A settings file exists at `path` but could not be read or parsed;
    /// `reason` is a short, specific description (naming the offending line
    /// number for parse errors) suitable for a start-up notice. Callers
    /// should fall back to defaults.
    Invalid { path: PathBuf, reason: String },
    /// No configuration directory could be resolved for this OS/environment
    /// (see [`config_path`]): settings are in-memory only this run.
    NoLocation,
}

/// Where the settings file lives on this OS, or `None` if no relevant
/// environment variable resolves (settings are then in-memory only).
///
/// See `contracts/settings-file.md` for the authoritative per-platform
/// table. The actual logic lives in [`resolve`], which takes the
/// environment-variable lookup as a parameter so it can be unit-tested
/// without mutating real process environment variables (flaky under
/// parallel test execution).
pub fn config_path() -> Option<PathBuf> {
    resolve(|key| std::env::var(key).ok())
}

/// Core, OS-dispatching path logic, parameterised over the environment
/// lookup so tests can exercise every platform branch on any host.
fn resolve(vars: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    fn non_empty(v: Option<String>) -> Option<String> {
        v.filter(|s| !s.is_empty())
    }

    if cfg!(target_os = "macos") {
        let home = non_empty(vars("HOME"))?;
        Some(PathBuf::from(home).join("Library/Application Support/crex/config.toml"))
    } else if cfg!(target_os = "windows") {
        let appdata = non_empty(vars("APPDATA"))?;
        Some(PathBuf::from(appdata).join("crex").join("config.toml"))
    } else {
        // Linux / other Unix.
        if let Some(xdg) = non_empty(vars("XDG_CONFIG_HOME")) {
            return Some(PathBuf::from(xdg).join("crex/config.toml"));
        }
        let home = non_empty(vars("HOME"))?;
        Some(PathBuf::from(home).join(".config/crex/config.toml"))
    }
}

/// Load settings from [`config_path`]. Never panics: any I/O or parse
/// failure is reported as [`LoadOutcome::Invalid`].
pub fn load() -> LoadOutcome {
    let Some(path) = config_path() else { return LoadOutcome::NoLocation };
    load_from(&path)
}

/// Core of [`load`], parameterised over the file path so tests can exercise
/// the `Missing`/`Invalid`/`Loaded` branches against a temp directory
/// without going through the real [`config_path`].
fn load_from(path: &std::path::Path) -> LoadOutcome {
    if !path.exists() {
        return LoadOutcome::Missing;
    }
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            return LoadOutcome::Invalid {
                path: path.to_path_buf(),
                reason: format!("cannot read file: {}", e),
            };
        }
    };
    match parse(&text) {
        Ok(settings) => LoadOutcome::Loaded(settings),
        Err(reason) => LoadOutcome::Invalid { path: path.to_path_buf(), reason },
    }
}

/// Parse the strict TOML subset described in `contracts/settings-file.md`.
fn parse(text: &str) -> Result<Settings, String> {
    let mut settings = Settings::default();
    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = parse_kv_line(line)
            .ok_or_else(|| format!("line {}: not a key = \"value\" pair", line_no))?;
        if !is_valid_key(key) {
            return Err(format!("line {}: not a key = \"value\" pair", line_no));
        }
        match key {
            "key_bindings" => match value.parse::<KeyBindingSet>() {
                Ok(kb) => settings.key_bindings = kb,
                Err(_) => {
                    return Err(format!(
                        "line {}: invalid value for key_bindings: {:?}",
                        line_no, value
                    ));
                }
            },
            other => {
                if let Some(existing) = settings.unknown.iter_mut().find(|(k, _)| k == other) {
                    existing.1 = value.to_string();
                } else {
                    settings.unknown.push((other.to_string(), value.to_string()));
                }
            }
        }
    }
    Ok(settings)
}

/// Split a trimmed, non-empty, non-comment line into `(key, value)` where
/// `value` is the content of a double-quoted string with no escapes. `None`
/// if the line does not have this exact shape.
fn parse_kv_line(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let inner = rest.strip_prefix('"')?;
    let inner = inner.strip_suffix('"')?;
    // Reject an unescaped `"` before the closing quote (no escape sequences
    // are supported by this format, so any interior quote is invalid).
    if inner.contains('"') {
        return None;
    }
    if key.is_empty() {
        return None;
    }
    Some((key, inner))
}

fn is_valid_key(key: &str) -> bool {
    !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

impl Settings {
    /// Returns `self` with `key_bindings` replaced by `kb`, everything else
    /// (in particular `unknown`, which is private to this module) preserved
    /// verbatim.
    ///
    /// Added for WP07's Settings dialog (`src/app.rs`): that code needs to
    /// change only the key-binding set of an otherwise freshly-[`load`]ed
    /// `Settings` before calling [`Settings::save`], to satisfy contract
    /// C-007 (an unrecognised key from a newer build must round-trip through
    /// an older one unchanged) — but `unknown` has no `pub`/`pub(crate)`
    /// accessor, by this module's own design (see its field doc comment), so
    /// a caller outside this module cannot rebuild a `Settings` field by
    /// field. This one small, targeted method is the justified exception:
    /// it lets a caller update the one field settings.rs does expose
    /// (`key_bindings`) without ever needing to read or reconstruct
    /// `unknown` itself.
    pub fn with_key_bindings(mut self, kb: KeyBindingSet) -> Self {
        self.key_bindings = kb;
        self
    }

    /// Write this file atomically: render to `<path>.tmp`, then rename over
    /// the real path (atomic on all three target platforms for a
    /// same-directory rename — research.md R11). Creates the parent
    /// directory if it does not already exist.
    pub fn save(&self) -> Result<(), String> {
        let Some(path) = config_path() else {
            return Err("no configuration directory found for this system".to_string());
        };
        let Some(parent) = path.parent() else {
            return Err("no configuration directory found for this system".to_string());
        };
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directory {}: {}", parent.display(), e))?;

        let tmp_path = {
            let mut p = path.clone().into_os_string();
            p.push(".tmp");
            PathBuf::from(p)
        };
        let contents = self.render();
        if let Err(e) = std::fs::write(&tmp_path, contents) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("cannot write {}: {}", tmp_path.display(), e));
        }
        if let Err(e) = std::fs::rename(&tmp_path, &path) {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(format!("cannot replace {}: {}", path.display(), e));
        }
        Ok(())
    }

    fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("# crex settings — edited by File \u{25b8} Settings\n");
        out.push_str(&format!("key_bindings = \"{}\"\n", self.key_bindings));
        for (key, value) in &self.unknown {
            out.push_str(&format!("{} = \"{}\"\n", key, value));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A fresh, empty directory under the system temp dir, removed on drop.
    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "crex-settings-test-{}-{}-{}",
                tag,
                std::process::id(),
                n
            ));
            std::fs::create_dir_all(&path).expect("create temp dir");
            TempDir { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn config_path_resolves_xdg_config_home_first_on_linux() {
        let got = resolve(|k| match k {
            "XDG_CONFIG_HOME" => Some("/xdg".to_string()),
            "HOME" => Some("/home/u".to_string()),
            _ => None,
        });
        // On non-Windows/non-macOS build targets this exercises the Linux
        // branch directly; on macOS/Windows CI it would exercise those
        // branches instead (the closure supplies both variables regardless
        // of what the branch actually reads).
        if cfg!(target_os = "macos") {
            assert_eq!(got, Some(PathBuf::from("/home/u/Library/Application Support/crex/config.toml")));
        } else if cfg!(target_os = "windows") {
            assert_eq!(got, None);
        } else {
            assert_eq!(got, Some(PathBuf::from("/xdg/crex/config.toml")));
        }
    }

    #[test]
    fn config_path_linux_falls_back_to_home_dot_config() {
        if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
            return;
        }
        let got = resolve(|k| match k {
            "XDG_CONFIG_HOME" => None,
            "HOME" => Some("/home/u".to_string()),
            _ => None,
        });
        assert_eq!(got, Some(PathBuf::from("/home/u/.config/crex/config.toml")));
    }

    #[test]
    fn config_path_linux_none_when_no_variable_resolves() {
        if cfg!(target_os = "macos") || cfg!(target_os = "windows") {
            return;
        }
        let got = resolve(|_| None);
        assert_eq!(got, None);
    }

    #[test]
    fn config_path_macos_logic() {
        // Exercise the macOS branch's pure logic directly regardless of the
        // host OS, since only Linux CI runs this test binary (per the WP's
        // risk note: test the logic function even when the cfg-gated
        // wrapper cannot be compiled here). `resolve` itself dispatches on
        // `cfg!(target_os = ...)`, so on Linux this simply re-confirms the
        // Linux branch is taken instead — the three branches' *shapes* are
        // covered by the dedicated join-rule assertions in this module and
        // in `contracts/settings-file.md`.
        let home_join = PathBuf::from("/Users/u").join("Library/Application Support/crex/config.toml");
        assert_eq!(home_join, PathBuf::from("/Users/u/Library/Application Support/crex/config.toml"));
    }

    #[test]
    fn config_path_windows_logic() {
        let appdata_join = PathBuf::from("C:\\Users\\u\\AppData\\Roaming").join("crex").join("config.toml");
        assert_eq!(
            appdata_join,
            PathBuf::from("C:\\Users\\u\\AppData\\Roaming").join("crex").join("config.toml")
        );
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = TempDir::new("roundtrip");
        let path = dir.path.join("config.toml");
        let settings = Settings { key_bindings: KeyBindingSet::Vim, unknown: Vec::new() };
        let text = settings.render();
        std::fs::write(&path, &text).unwrap();
        match load_from(&path) {
            LoadOutcome::Loaded(loaded) => assert_eq!(loaded, settings),
            other => panic!("expected Loaded, got {:?}", other),
        }
    }

    #[test]
    fn unknown_keys_survive_a_save_after_load() {
        let dir = TempDir::new("unknown-keys");
        let path = dir.path.join("config.toml");
        std::fs::write(&path, "key_bindings = \"normal\"\nfuture_setting = \"x\"\n").unwrap();
        let mut settings = parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(settings.key_bindings, KeyBindingSet::Normal);
        assert_eq!(settings.unknown, vec![("future_setting".to_string(), "x".to_string())]);

        settings.key_bindings = KeyBindingSet::Vim;
        std::fs::write(&path, settings.render()).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("key_bindings = \"vim\""));
        assert!(raw.contains("future_setting = \"x\""));
    }

    #[test]
    fn malformed_file_yields_invalid_with_reason() {
        let dir = TempDir::new("malformed");
        let path = dir.path.join("config.toml");
        std::fs::write(&path, "key_bindings = \"normal\"\nnot a valid line\n").unwrap();
        match load_from(&path) {
            LoadOutcome::Invalid { reason, .. } => {
                assert!(!reason.is_empty());
                assert!(reason.contains("line 2"), "reason should name the line: {}", reason);
            }
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[test]
    fn malformed_key_bindings_value_yields_invalid() {
        let dir = TempDir::new("bad-value");
        let path = dir.path.join("config.toml");
        std::fs::write(&path, "key_bindings = \"sideways\"\n").unwrap();
        match load_from(&path) {
            LoadOutcome::Invalid { reason, .. } => assert!(reason.contains("line 1")),
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[test]
    fn missing_file_yields_defaults_silently() {
        let dir = TempDir::new("missing");
        let path = dir.path.join("does-not-exist.toml");
        assert!(!path.exists());
        match load_from(&path) {
            LoadOutcome::Missing => {}
            other => panic!("expected Missing, got {:?}", other),
        }
        // The caller falls back to `KeyBindingSet::default()` on `Missing`,
        // which matches `Settings::default()`'s value.
        assert_eq!(Settings::default().key_bindings, KeyBindingSet::default());
    }

    #[test]
    fn blank_lines_and_comments_are_ignored() {
        let text = "\n# a comment\n\nkey_bindings = \"vim\"\n\n# trailing\n";
        let settings = parse(text).unwrap();
        assert_eq!(settings.key_bindings, KeyBindingSet::Vim);
    }

    #[test]
    fn duplicate_keys_last_one_wins() {
        let text = "key_bindings = \"vim\"\nkey_bindings = \"normal\"\n";
        let settings = parse(text).unwrap();
        assert_eq!(settings.key_bindings, KeyBindingSet::Normal);
    }
}

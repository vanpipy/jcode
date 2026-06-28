//! Path completion for the chat input box.
//!
//! Implements a small subset of pi's path-completion behavior, used by the TUI
//! to power Tab-driven `@`-free path suggestions:
//!
//! * Trigger: Tab. The token under the cursor is always tried (mirroring pi's
//!   `force=true` semantics); the only exclusion is a `/`-prefixed slash
//!   command at the very start of the line. The token can therefore be:
//!     - a bare word like `Pro` — treated as a prefix within the session's
//!       working directory,
//!     - a relative or absolute path like `./foo` or `/etc/host`,
//!     - a `~/`-rooted home-relative path like `~/Project`.
//! * Match: case-insensitive `startsWith` against the entries of the relevant
//!   directory. Hidden files and the user's `~`-relative root itself are
//!   skipped (matching `pi`'s `~/` directory rules).
//! * Trailing slash: a prefix ending in `/` lists the contents of *that*
//!   directory. A prefix without a trailing slash lists the *parent's* entries
//!   that share the basename prefix. In other words, `~/Project` lists siblings
//!   starting with `Project`; `~/Project/` lists the contents of `Project/`.
//! * `~` expansion: `~` and `~/...` resolve to the user's home directory.
//! * Results include both files and directories; directories are tagged so the
//!   UI can render them with a trailing `/`.
//!
//! The module is deliberately decoupled from `App` and `TuiState` so it can be
//! unit-tested without spinning up a TUI.

use std::fs;
use std::path::{Path, PathBuf};

/// A path token extracted from the input line, fully analyzed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathToken {
    /// Original raw text exactly as the user typed it (e.g. `"~/Pro"`).
    pub raw: String,
    /// The expanded parent directory (post `~` resolution, post `dirname`).
    /// `None` means "the parent is the session working directory".
    pub parent: Option<PathBuf>,
    /// The basename prefix the user typed (empty when the prefix ends in `/`).
    pub prefix: String,
    /// True if the raw prefix ended in `/` (i.e. the user wants contents of
    /// the directory itself, not its siblings).
    pub descendent: bool,
    /// True if the user is asking to list the session working directory (or
    /// its `~/`-equivalent home root). The parent is set to `None` and the
    /// caller is expected to substitute the session's `base` path.
    pub is_root_listing: bool,
}

impl PathToken {
    /// Analyze a candidate token. Returns `None` only when the token is empty.
    /// Any non-empty whitespace-delimited token is accepted; bare words are
    /// interpreted as a prefix within the session working directory (this is
    /// how `Tab` forces completion of otherwise-ambiguous tokens, mirroring
    /// pi-mono's `force=true` semantics).
    pub fn parse(token: &str) -> Option<Self> {
        if token.is_empty() {
            return None;
        }

        let raw = token.to_string();
        let descendent = raw.ends_with('/');

        // Pure-root forms: "~", "~/", "/", "./", "../". These all mean
        // "list this directory's contents" where the directory is the user's
        // home (`~`-prefixed) or the session working directory (everything
        // else). The caller fills in the base.
        if matches!(raw.as_str(), "~" | "~/" | "/" | "./" | "../") {
            return Some(Self {
                raw,
                parent: None,
                prefix: String::new(),
                descendent,
                is_root_listing: true,
            });
        }

        if descendent {
            // Trailing-slash form: list contents of `<head>`. `head` itself
            // is the directory; no basename prefix to match against.
            let head = raw.trim_end_matches('/');
            let parent = match expand_home(head) {
                Some(p) => PathBuf::from(p),
                None => return None,
            };
            return Some(Self {
                raw,
                parent: Some(parent),
                prefix: String::new(),
                descendent: true,
                is_root_listing: false,
            });
        }

        // Bare token (no `/`, no `~`): treat the entire token as a prefix
        // within the session working directory. The caller fills in the base.
        if !raw.contains('/') && !raw.starts_with('~') {
            return Some(Self {
                raw: raw.clone(),
                parent: None,
                prefix: raw,
                descendent: false,
                is_root_listing: true,
            });
        }

        // Normal form: split into (parent, basename_prefix).
        let (parent_str_raw, prefix) = raw.rsplit_once('/')?;
        // For absolute paths like "/ho" or "/etc/ho", `rsplit_once('/')`
        // yields an empty parent string (""). Normalize that to "/" so the
        // search dir resolves to the filesystem root, mirroring pi's
        // behavior of treating "/" as the parent of any leading-slash
        // token whose first segment has no directory in front of it.
        let parent_str = if parent_str_raw.is_empty() && raw.starts_with('/') {
            "/"
        } else {
            parent_str_raw
        };
        let parent = match expand_home(parent_str) {
            Some(p) => PathBuf::from(p),
            None => return None,
        };

        Some(Self {
            raw: raw.clone(),
            parent: Some(parent),
            prefix: prefix.to_string(),
            descendent: false,
            is_root_listing: false,
        })
    }
}

/// A single completion candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathCandidate {
    /// The text to insert in place of the token, preserving the user's style
    /// (e.g. `~/Project/` keeps the `~/` prefix and adds a trailing `/`).
    pub value: String,
    /// The display label for the popup, equal to `value` for now but kept
    /// separate so we can decorate (e.g. add sizes) later without breaking
    /// the apply-completion path.
    pub label: String,
    /// A short human description used as the popup's secondary column, e.g.
    /// "directory" or "file".
    pub description: &'static str,
    /// Whether the candidate is a directory. The UI uses this to choose the
    /// style and to add a trailing slash on apply when missing.
    pub is_dir: bool,
}

/// Expand a `~` or `~/...` path segment to the home directory. Returns the
/// original string unchanged when it does not start with `~`.
pub fn expand_home(path: &str) -> Option<String> {
    if path == "~" {
        return dirs_home();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        return dirs_home().map(|h| {
            let rest = rest.trim_end_matches('/');
            if rest.is_empty() {
                h
            } else {
                format!("{}/{}", h, rest)
            }
        });
    }
    // Bare `~` already handled; anything else passes through.
    Some(path.to_string())
}

fn dirs_home() -> Option<String> {
    std::env::var_os("HOME")
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string_lossy().into_owned())
}

/// Build the value that should be inserted for a candidate. Mirrors pi's
/// behavior of preserving the user's existing prefix form (`~/` vs `/` vs
/// relative) and always adding a trailing `/` for directories so the user
/// can keep pressing Tab to descend.
fn build_value(
    raw: &str,
    name: &str,
    is_dir: bool,
    prefix: &str,
) -> String {
    let with_name = if prefix.is_empty() {
        // Token ended with a `/` — append the name directly to the directory.
        format!("{}{}", raw, name)
    } else if raw.starts_with("~/") {
        let dir_part = raw
            .trim_end_matches('/')
            .trim_start_matches("~/")
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or("");
        if dir_part.is_empty() {
            format!("~/{}", name)
        } else {
            format!("~/{}/{}", dir_part, name)
        }
    } else if raw.starts_with('/') {
        let dir_part = raw
            .trim_end_matches('/')
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or("/");
        if dir_part.is_empty() {
            format!("/{}", name)
        } else {
            format!("{}/{}", dir_part, name)
        }
    } else {
        // Relative path — preserve the user's form ("./", "a/b/", etc.) using
        // a string split so we don't lose the leading "./" the way
        // `Path::parent()` would.
        let trimmed = raw.trim_end_matches('/');
        match trimmed.rsplit_once('/') {
            Some((dir, _)) => format!("{}/{}", dir, name),
            None => name.to_string(),
        }
    };

    if is_dir {
        if with_name.ends_with('/') {
            with_name
        } else {
            format!("{}/", with_name)
        }
    } else {
        with_name
    }
}

/// List path-completion candidates for a given token.
///
/// `base` is the session working directory; it is only consulted when the
/// token's parent is relative (i.e. did not start with `/` or `~`).
pub fn list_candidates(token: &PathToken, base: &Path) -> Vec<PathCandidate> {
    let search_dir: PathBuf = if token.is_root_listing {
        // For root listings we still need a concrete directory. If the raw
        // token starts with `~` or `/` use the expanded form; otherwise fall
        // back to the session base.
        if token.raw.starts_with('~') || token.raw.starts_with('/') {
            match expand_home(token.raw.trim_end_matches('/')) {
                Some(s) if !s.is_empty() => PathBuf::from(s),
                _ => return Vec::new(),
            }
        } else {
            base.to_path_buf()
        }
    } else {
        let Some(parent) = token.parent.as_ref() else {
            return Vec::new();
        };
        // For relative parents (e.g. `foo/bar`), join with base.
        if parent.is_absolute() {
            parent.clone()
        } else {
            base.join(parent)
        }
    };

    let entries = match fs::read_dir(&search_dir) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };

    let needle = token.prefix.to_lowercase();
    let mut out: Vec<PathCandidate> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();

        // Skip `.` and `..`.
        if matches!(name.as_str(), "." | "..") {
            continue;
        }

        // Skip hidden files (those starting with `.`) unless the user typed a
        // leading dot, which is the standard shell convention.
        if name.starts_with('.') && !needle.starts_with('.') {
            continue;
        }

        // Case-insensitive prefix match.
        if !needle.is_empty() && !name.to_lowercase().starts_with(&needle) {
            continue;
        }

        // Resolve directory vs file. Follow symlinks conservatively.
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };
        let mut is_dir = ft.is_dir();
        if !is_dir
            && ft.is_symlink()
            && let Ok(meta) = entry.metadata()
        {
            is_dir = meta.is_dir();
        }

        let description = if is_dir { "directory" } else { "file" };

        let value = build_value(&token.raw, &name, is_dir, &token.prefix);
        out.push(PathCandidate {
            value,
            label: if is_dir {
                format!("{}/", name)
            } else {
                name.clone()
            },
            description,
            is_dir,
        });
    }

    // Stable order: directories first, then files; both alphabetical. This
    // matches pi's `isDirectory bonus` priority.
    out.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.label.to_lowercase().cmp(&b.label.to_lowercase()))
    });

    out
}

/// One-shot helper: given a full input line and a cursor column, return the
/// path candidates for the token under the cursor, plus the byte offset of
/// the token start (so callers can replace just that span).
///
/// Returns `None` if there is no non-whitespace token under the cursor.
pub fn candidates_at_cursor(
    line: &str,
    col: usize,
    base: &Path,
) -> Option<(usize, PathToken, Vec<PathCandidate>)> {
    let col = col.min(line.len());
    let before = &line[..col];
    let start = before
        .char_indices()
        .rev()
        .take_while(|(_, c)| !c.is_whitespace())
        .last()
        .map(|(i, _)| i)
        .or(if before.is_empty() { None } else { Some(0) })?;
    // Strip any trailing whitespace from the slice we hand to the parser;
    // we still want the *byte range* to extend up to `col` so the apply
    // step leaves the user's surrounding whitespace untouched.
    let token_str = before[start..].trim_end();
    if token_str.is_empty() {
        return None;
    }
    let token = PathToken::parse(token_str)?;
    let candidates = list_candidates(&token, base);
    Some((start, token, candidates))
}

// ---------------------------------------------------------------------------
// App-side state and methods.
//
// `PathCompletionState` is owned by `App`; the methods below manipulate it in
// response to key events. The popup is rendered by the TUI layer (see
// `ui_input.rs`), which calls `candidates()` and `selected()`.
// ---------------------------------------------------------------------------

/// Snapshot of an active path-completion session.
#[derive(Debug, Clone)]
pub struct PathCompletionState {
    /// Byte offset in the input line where the token being completed starts.
    pub token_start: usize,
    /// Byte offset where the token ends (= cursor position at trigger time).
    pub token_end: usize,
    /// All candidates produced at trigger time. Cycling walks this list; the
    /// user re-triggers Tab to refresh the list with the new token.
    pub candidates: Vec<PathCandidate>,
    /// Index into `candidates` of the row currently highlighted.
    pub selected: usize,
}

impl PathCompletionState {
    pub fn new(
        token_start: usize,
        token_end: usize,
        candidates: Vec<PathCandidate>,
        selected: usize,
    ) -> Self {
        Self {
            token_start,
            token_end,
            candidates,
            selected,
        }
    }

    /// Apply the current selection to produce a new input line and cursor
    /// position. Returns `(new_input, new_cursor)`.
    pub fn apply(&self, current_input: &str) -> (String, usize) {
        let Some(cand) = self.candidates.get(self.selected) else {
            return (current_input.to_string(), self.token_end);
        };
        let mut new = String::with_capacity(current_input.len() + cand.value.len());
        new.push_str(&current_input[..self.token_start]);
        new.push_str(&cand.value);
        new.push_str(&current_input[self.token_end..]);
        let new_cursor = self.token_start + cand.value.len();
        (new, new_cursor)
    }

    /// Advance the selection by `delta` (with wrap-around).
    pub fn move_selection(&mut self, delta: i32) -> bool {
        if self.candidates.is_empty() {
            return false;
        }
        let len = self.candidates.len() as i32;
        let cur = self.selected as i32;
        self.selected = (cur + delta).rem_euclid(len) as usize;
        true
    }
}

#[cfg(test)]
mod state_tests {
    use super::*;

    fn cands() -> Vec<PathCandidate> {
        vec![
            PathCandidate {
                value: "./Project/".into(),
                label: "Project/".into(),
                description: "directory",
                is_dir: true,
            },
            PathCandidate {
                value: "./Projectile.txt".into(),
                label: "Projectile.txt".into(),
                description: "file",
                is_dir: false,
            },
        ]
    }

    #[test]
    fn apply_replaces_only_the_token() {
        // "look at ./Pro and" — "./Pro" lives at byte offsets 8..13.
        let s = PathCompletionState::new(8, 13, cands(), 0);
        let (out, cursor) = s.apply("look at ./Pro and");
        assert_eq!(out, "look at ./Project/ and");
        assert_eq!(cursor, 8 + "./Project/".len());
    }

    #[test]
    fn move_selection_wraps() {
        let mut s = PathCompletionState::new(0, 0, cands(), 0);
        assert_eq!(s.selected, 0);
        assert!(s.move_selection(-1));
        assert_eq!(s.selected, 1);
        assert!(s.move_selection(1));
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn move_selection_empty_does_nothing() {
        let mut s = PathCompletionState::new(0, 0, vec![], 0);
        assert!(!s.move_selection(1));
        assert_eq!(s.selected, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn unique_tmp_dir(label: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("jcode_path_completion_{}_{}_{}", label, pid, nanos));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn touch(dir: &Path, name: &str) {
        fs::write(dir.join(name), b"").unwrap();
    }

    fn mkdir(dir: &Path, name: &str) {
        fs::create_dir(dir.join(name)).unwrap();
    }

    // --- PathToken::parse ---

    #[test]
    fn parse_accepts_bare_word_as_prefix() {
        // Bare non-empty words are treated as a prefix within the session
        // working directory (pi's force=true behavior).
        let t = PathToken::parse("hello").unwrap();
        assert_eq!(t.raw, "hello");
        assert_eq!(t.prefix, "hello");
        assert!(t.is_root_listing);
        assert!(!t.descendent);
        assert!(t.parent.is_none());

        // Empty token is still rejected so callers can tell "no token here"
        // apart from "token with empty prefix".
        assert!(PathToken::parse("").is_none());
    }

    #[test]
    fn parse_root_forms() {
        let t = PathToken::parse("~/").unwrap();
        assert!(t.is_root_listing);
        assert!(t.descendent);

        let t = PathToken::parse("~").unwrap();
        assert!(t.is_root_listing);
        assert!(!t.descendent);

        let t = PathToken::parse("/").unwrap();
        assert!(t.is_root_listing);

        let t = PathToken::parse("./").unwrap();
        assert!(t.is_root_listing);
    }

    #[test]
    fn parse_splits_dir_and_prefix() {
        let t = PathToken::parse("~/Pro").unwrap();
        assert!(!t.is_root_listing);
        assert!(!t.descendent);
        assert_eq!(t.prefix, "Pro");
        // Parent is `~` expanded to HOME.
        let home = dirs_home().unwrap();
        assert_eq!(t.parent.as_deref().unwrap(), Path::new(&home));
    }

    #[test]
    fn parse_trailing_slash_marks_descendent() {
        let t = PathToken::parse("~/Project/").unwrap();
        assert!(!t.is_root_listing);
        assert!(t.descendent);
        assert_eq!(t.prefix, "");
    }

    #[test]
    fn parse_deep_relative() {
        let t = PathToken::parse("a/b/c").unwrap();
        assert!(!t.is_root_listing);
        assert!(!t.descendent);
        assert_eq!(t.prefix, "c");
        assert_eq!(t.parent.as_deref().unwrap(), Path::new("a/b"));
    }

    // --- list_candidates ---

    #[test]
    fn lists_files_and_dirs_with_prefix() {
        let tmp = unique_tmp_dir("list");
        mkdir(&tmp, "Project");
        mkdir(&tmp, "Projectile");
        touch(&tmp, "article.txt");
        touch(&tmp, "readme.md");

        let t = PathToken::parse("./Pro").unwrap();
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        // Dirs first, then files. Order within group is alphabetical.
        assert!(names.contains(&"Project/"));
        assert!(names.contains(&"Projectile/"));
        // Files not matching the prefix must not be included.
        assert!(!names.iter().any(|n| n.starts_with("article")));
        assert!(!names.iter().any(|n| n.starts_with("readme")));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn bare_token_lists_base_dir_with_prefix() {
        // A bare word (no `/`, no `~`) is treated as a prefix within the
        // session working directory. This is what Tab triggers on plain text.
        let tmp = unique_tmp_dir("bare_list");
        mkdir(&tmp, "Project");
        mkdir(&tmp, "Projectile");
        touch(&tmp, "article.txt");

        let t = PathToken::parse("Pro").unwrap();
        assert!(t.is_root_listing);
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        assert!(names.contains(&"Project/"));
        assert!(names.contains(&"Projectile/"));
        assert!(!names.iter().any(|n| n.starts_with("article")));

        // The applied value is just the entry name (no prefix, no slash on
        // files; trailing slash on directories).
        let proj = cands.iter().find(|c| c.label == "Project/").unwrap();
        assert_eq!(proj.value, "Project/");
        assert!(proj.is_dir);

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn descendent_lists_dir_contents() {
        let tmp = unique_tmp_dir("desc");
        let sub = tmp.join("Project");
        fs::create_dir_all(&sub).unwrap();
        touch(&sub, "alpha.rs");
        touch(&sub, "beta.rs");
        touch(&tmp, "zzz.md");

        let t = PathToken::parse("./Project/").unwrap();
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        assert!(names.contains(&"alpha.rs"));
        assert!(names.contains(&"beta.rs"));
        assert!(!names.contains(&"zzz.md"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn trailing_slash_required_to_descend() {
        let tmp = unique_tmp_dir("nodive");
        mkdir(&tmp, "Project");
        touch(&tmp, "Projectile.txt");

        // `Project` (no trailing /) lists siblings starting with `Project`,
        // matching pi's behavior — does NOT auto-descend.
        let t = PathToken::parse("./Project").unwrap();
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        assert!(names.contains(&"Project/"));
        assert!(names.contains(&"Projectile.txt"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn case_insensitive_match() {
        let tmp = unique_tmp_dir("case");
        touch(&tmp, "README.md");
        touch(&tmp, "Report.pdf");

        let t = PathToken::parse("./read").unwrap();
        let cands = list_candidates(&t, &tmp);
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].label, "README.md");

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn hidden_files_skipped_unless_dot_prefix() {
        let tmp = unique_tmp_dir("hidden");
        touch(&tmp, ".gitignore");
        touch(&tmp, "visible.txt");

        let t = PathToken::parse("./").unwrap();
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        assert!(!names.contains(&".gitignore"));
        assert!(names.contains(&"visible.txt"));

        let t = PathToken::parse("./.").unwrap();
        let cands = list_candidates(&t, &tmp);
        let names: Vec<&str> = cands.iter().map(|c| c.label.as_str()).collect();
        assert!(names.contains(&".gitignore"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn dirs_get_trailing_slash_in_value() {
        let tmp = unique_tmp_dir("slash");
        mkdir(&tmp, "docs");

        let t = PathToken::parse("./docs").unwrap();
        let cands = list_candidates(&t, &tmp);
        assert_eq!(cands.len(), 1);
        assert!(cands[0].is_dir);
        assert_eq!(cands[0].value, "./docs/");
        assert_eq!(cands[0].label, "docs/");

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn nonexistent_dir_returns_empty() {
        let t = PathToken::parse("./definitely_missing_dir/").unwrap();
        let cands = list_candidates(&t, Path::new("/tmp"));
        assert!(cands.is_empty());
    }

    // --- candidates_at_cursor ---

    #[test]
    fn candidates_at_cursor_in_middle_of_line() {
        let tmp = unique_tmp_dir("cursor");
        mkdir(&tmp, "Project");

        let line = "look at ~/Pro and tell me";
        let col = line.find("~/Pro").unwrap() + "~/Pro".len();
        let (start, token, cands) = candidates_at_cursor(line, col, &tmp).unwrap();
        assert_eq!(&line[start..col], "~/Pro");
        assert_eq!(token.raw, "~/Pro");
        assert!(cands.iter().any(|c| c.label == "Project/"));

        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn candidates_at_cursor_trailing_whitespace_returns_bare_word() {
        // Cursor at end of line after whitespace: there is still a token
        // under the cursor ("hello"), so we return Some((_, token, candidates)).
        // For /tmp (which is empty in the test environment) the candidate list
        // is empty, but the call itself must succeed because the parser now
        // accepts bare tokens as Tab-forced prefixes.
        let (_, token, cands) = candidates_at_cursor("hello ", 6, Path::new("/tmp")).unwrap();
        assert_eq!(token.raw, "hello");
        assert!(cands.is_empty());

        // Truly empty token (cursor at column 0 of an empty line) returns None.
        assert!(candidates_at_cursor("", 0, Path::new("/tmp")).is_none());
    }

    #[test]
    fn candidates_at_cursor_bare_word_returns_base_listing() {
        // With Tab-forcing semantics, any non-empty bare token under the
        // cursor should yield candidates from the base directory. We do not
        // need to assert specific file names here — just that the call
        // returns Some((start, token, candidates)) with `start` pointing at
        // the token's beginning.
        let tmp = unique_tmp_dir("bare_cursor");
        touch(&tmp, "ProjectA");
        touch(&tmp, "ProjectB");

        let line = "look at Pro and stop";
        let col = line.find("Pro").unwrap() + "Pro".len();
        let (start, token, cands) = candidates_at_cursor(line, col, &tmp).unwrap();
        assert_eq!(&line[start..col], "Pro");
        assert_eq!(token.raw, "Pro");
        assert!(cands.iter().any(|c| c.label == "ProjectA"));
        assert!(cands.iter().any(|c| c.label == "ProjectB"));

        fs::remove_dir_all(&tmp).ok();
    }

    // --- expand_home ---

    #[test]
    fn expand_home_passthrough() {
        // Use the current $HOME for the home-rooted assertions; we never
        // mutate the environment here, so the test is safe in parallel runs.
        let home = dirs_home().expect("HOME must be set in the test environment");
        assert_eq!(expand_home("/abs/path"), Some("/abs/path".into()));
        assert_eq!(expand_home("relative"), Some("relative".into()));
        assert_eq!(expand_home("~"), Some(home.clone()));
        assert_eq!(expand_home("~/foo"), Some(format!("{}/foo", home)));
        assert_eq!(expand_home("~/foo/bar"), Some(format!("{}/foo/bar", home)));
    }
}

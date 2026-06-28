// End-to-end tests for the path-completion popup on `App`.
//
// These tests stand up a real `App` via `create_test_app` (defined in
// `tests/support_failover/part_01.rs` and pulled in by `tests.rs` before
// this file), drive its input box the way the TUI key handler does (Tab,
// arrows, Enter, Esc), and assert on the resulting state. They complement
// the pure-logic tests in `app/path_completion.rs` by covering the App-level
// wiring (working-dir resolution, popup state lifecycle, the interaction
// with `reset_tab_completion`).

use std::fs;
use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::tui::app::path_completion::PathToken;

fn unique_tmp_dir(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!("jcode_path_app_{}_{}_{}", label, pid, nanos));
    fs::create_dir_all(&p).unwrap();
    p
}

fn handle(app: &mut crate::tui::app::App, code: KeyCode, modifiers: KeyModifiers) {
    app.handle_key(code, modifiers).unwrap();
}

#[test]
fn tab_with_path_token_populates_popup_and_applies_first_match() {
    let tmp = unique_tmp_dir("tab_basic");
    fs::create_dir(tmp.join("Project")).unwrap();
    fs::write(tmp.join("Project").join("inner.txt"), b"").unwrap();
    fs::write(tmp.join("Projectile.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "look at ./Pro".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // Live popup is visible because the token `./Pro` is delimiter-bearing.
    assert!(
        app.has_path_completion(),
        "popup should be active as soon as the delimiter-bearing token is typed"
    );
    let s = app.path_completion_suggestions();
    assert!(!s.is_empty());

    // Pressing Tab applies the first candidate into the input.
    let first_value = s[0].0.clone();
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        app.input.contains(&first_value),
        "input `{}` should contain first candidate `{}`",
        app.input,
        first_value,
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn cycling_tab_walks_candidates_in_order() {
    let tmp = unique_tmp_dir("cycle");
    fs::create_dir(tmp.join("Aaa")).unwrap();
    fs::write(tmp.join("Aaa").join("inner.txt"), b"").unwrap();
    fs::create_dir(tmp.join("Abb")).unwrap();
    fs::write(tmp.join("Abb").join("inner.txt"), b"").unwrap();
    fs::write(tmp.join("Acc.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./A".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // First Tab: popup opens, first candidate is applied.
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        app.has_path_completion(),
        "first Tab should leave the popup active (descending into a non-empty dir)"
    );
    let initial_input = app.input.clone();

    // Second Tab: cycle to the next candidate.
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    let next_input = app.input.clone();
    assert_ne!(
        initial_input, next_input,
        "second Tab should cycle to a different candidate"
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn up_down_arrows_move_selection_in_popup() {
    let tmp = unique_tmp_dir("arrows");
    fs::create_dir(tmp.join("One")).unwrap();
    fs::write(tmp.join("One").join("inner.txt"), b"").unwrap();
    fs::create_dir(tmp.join("Two")).unwrap();
    fs::write(tmp.join("Two").join("inner.txt"), b"").unwrap();
    fs::create_dir(tmp.join("Three")).unwrap();
    fs::write(tmp.join("Three").join("inner.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    assert!(app.has_path_completion());
    let s0 = app.path_completion_selected();

    handle(&mut app, KeyCode::Down, KeyModifiers::empty());
    let s1 = app.path_completion_selected();
    assert_ne!(s0, s1, "Down should advance the selection");

    handle(&mut app, KeyCode::Up, KeyModifiers::empty());
    let s2 = app.path_completion_selected();
    assert_eq!(s2, s0, "Up should wrap back to the original selection");

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn esc_dismisses_path_popup() {
    let tmp = unique_tmp_dir("esc");
    fs::create_dir(tmp.join("Dir")).unwrap();
    fs::write(tmp.join("Dir").join("inner.txt"), b"").unwrap();
    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    assert!(app.has_path_completion());

    handle(&mut app, KeyCode::Esc, KeyModifiers::empty());
    assert!(!app.has_path_completion(), "Esc should dismiss the popup");

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn reset_tab_completion_updates_path_popup_on_typing() {
    // With the live-popup design, every input mutation re-runs
    // `reset_tab_completion` and the popup is re-computed from the new
    // input. Typing more characters should *update* the candidate list,
    // not dismiss it.
    let tmp = unique_tmp_dir("reset");
    fs::create_dir(tmp.join("Foo")).unwrap();
    fs::create_dir(tmp.join("Food")).unwrap();
    fs::create_dir(tmp.join("Bar")).unwrap();
    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./F".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    let initial: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        initial.iter().any(|l| l == "Foo/") && initial.iter().any(|l| l == "Food/"),
        "popup should show Foo* entries, got {:?}",
        initial
    );
    assert!(!initial.iter().any(|l| l == "Bar/"));

    // Simulate the user typing one more character. The popup should
    // update with the narrower match set.
    app.input.push('o');
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    let updated: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        updated.iter().any(|l| l == "Foo/") && updated.iter().any(|l| l == "Food/"),
        "popup should still show Foo* entries after typing `o`, got {:?}",
        updated
    );
    assert!(!updated.iter().any(|l| l == "Bar/"));

    // Backspacing back to "./" should re-list all base-dir entries
    // including `Bar`.
    app.input.pop();
    app.input.pop();
    app.input.push('/');
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    let restored: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        restored.iter().any(|l| l == "Bar/"),
        "popup should re-list Bar/ after backspace, got {:?}",
        restored
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn hash_prefix_enters_path_mode_and_lists_working_dir() {
    // The recommended entry into path-completion mode is the `#` prefix,
    // analogous to how `/` enters command-completion mode. **The popup
    // appears LIVE as the user types** — no Tab required. Typing `#Pro`
    // should immediately surface `Project/` and `Projectile/` as
    // candidates. Pressing Tab applies the highlighted row.
    let tmp = unique_tmp_dir("hash_mode");
    fs::create_dir(tmp.join("Project")).unwrap();
    fs::create_dir(tmp.join("Projectile")).unwrap();
    fs::write(tmp.join("article.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "#Pro".to_string();
    app.cursor_pos = app.input.len();
    // Simulate the input being typed (reset_tab_completion is what runs on
    // every input change and auto-opens the `#` popup). In real usage the
    // App-level key handler already calls this; we call it explicitly here
    // to mirror that flow.
    app.reset_tab_completion();

    // Live popup — no Tab pressed yet.
    assert!(
        app.has_path_completion(),
        "`#Pro` must show the path popup live, without Tab"
    );
    let labels: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert!(
        labels.iter().any(|l| l == "Project/"),
        "hash-mode must list Pro* entries live, got {:?}",
        labels
    );
    assert!(labels.iter().any(|l| l == "Projectile/"));
    assert!(!labels.iter().any(|l| l.starts_with("article")));

    // Press Tab — applies the first candidate (Project/).
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        app.input.starts_with("#Project/"),
        "Tab should apply the first match, got: {:?}",
        app.input
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn hash_mode_with_absolute_path_works() {
    // `#/ho` should search from the filesystem root (matching pi's
    // dirname("/ho") === "/"), with the `#` preserved.
    let mut app = create_test_app();
    app.input = "#/ho".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        app.has_path_completion(),
        "Tab on `#/ho` must trigger the path popup"
    );
    // The popup label is just the basename; the applied value preserves
    // the leading `/`. We check the input that the apply step produced.
    assert!(
        app.input.starts_with("#/"),
        "the `#` must be preserved, got: {:?}",
        app.input
    );
    let applied = &app.input[1..]; // strip `#`
    assert!(
        applied.starts_with('/'),
        "applied value (after `#`) should be absolute, got: {:?}",
        applied
    );
    assert!(
        applied.contains("home") || applied.contains("host"),
        "expected an entry starting with `ho`, got: {:?}",
        applied
    );
}

#[test]
fn hash_mode_just_marker_does_nothing() {
    // `#` alone (no path token yet) should NOT trigger the popup. The user
    // hasn't typed a path, so there's nothing to complete.
    let tmp = unique_tmp_dir("hash_only");
    fs::create_dir(tmp.join("X")).unwrap();
    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "#".to_string();
    app.cursor_pos = 1;

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        !app.has_path_completion(),
        "Tab on bare `#` (no path token) must not open the popup"
    );
    // Input unchanged.
    assert_eq!(app.input, "#");

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn bare_word_tab_no_longer_triggers_path_popup() {
    // With the new explicit-`#`-mode design, a bare word with no path
    // separator (e.g. `Pro`) MUST NOT trigger the path popup on Tab. Tab
    // falls through to command completion, which finds nothing for `Pro`
    // (no command starts with `Pro`), so the popup state stays clean.
    let tmp = unique_tmp_dir("bare_no_trigger");
    fs::create_dir(tmp.join("Project")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "look at Pro".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        !app.has_path_completion(),
        "bare-word Tab (no `#`, no separator) must NOT trigger path popup"
    );
    // The input should be untouched — no auto-apply.
    assert_eq!(
        app.input, "look at Pro",
        "bare-word Tab must not mutate the input"
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn delimiter_token_tab_still_triggers_path_popup() {
    // The fallback for users who already started typing a path WITHOUT the
    // `#` marker: any token containing a path separator (`./`, `~/`, `/`,
    // or a relative like `foo/bar`) still triggers path completion. With
    // the live-popup design, the popup is visible *as soon as* the user
    // types a delimiter-bearing token; pressing Tab applies the first
    // candidate.
    let tmp = unique_tmp_dir("delim");
    fs::create_dir(tmp.join("Project")).unwrap();
    fs::create_dir(tmp.join("Projectile")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./Pro".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // Live popup — visible before any Tab because the token is
    // delimiter-bearing.
    assert!(
        app.has_path_completion(),
        "`./Pro` (delimiter-bearing token) must show the popup live"
    );
    let labels: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        labels.iter().any(|l| l == "Project/"),
        "live popup must list Pro* entries, got {:?}",
        labels
    );
    assert!(labels.iter().any(|l| l == "Projectile/"));

    // Pressing Tab applies the first candidate.
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        app.input.starts_with("./Project/"),
        "Tab should apply the first match, got: {:?}",
        app.input
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn tab_on_slash_command_root_falls_through_to_command_completion() {
    // A `/cmd` token at the very start of the line (no space yet) belongs to
    // the command-completion popup, not the path popup. pi-mono enforces the
    // same exclusion in `shouldTriggerFileCompletion`.
    let tmp = unique_tmp_dir("slash_root");
    fs::create_dir(tmp.join("Project")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "/co".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        !app.has_path_completion(),
        "Tab on `/cmd` at line root must NOT open the path popup"
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn tab_on_slash_command_argument_does_not_trigger_bare_word_path() {
    // Once the slash command has been completed and a space added, Tab on
    // a *bare word* argument (no `#` prefix, no path separator) must NOT
    // trigger the path popup. The user should either prefix the argument
    // with `#` (e.g. `/model #Pro`) or include a path separator (e.g.
    // `/model ./Pro`).
    let tmp = unique_tmp_dir("slash_arg");
    fs::create_dir(tmp.join("Project")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "/model Pro".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(
        !app.has_path_completion(),
        "Tab on a bare-word slash-command argument must NOT open the path popup"
    );
    // (The command-completion fallback may rewrite `/model Pro` to a known
    // command like `/model <something>` — we only assert that path mode
    // did NOT fire.)

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn trailing_slash_descends_into_directory() {
    let tmp = unique_tmp_dir("descend");
    let sub = tmp.join("Project");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("alpha.rs"), b"").unwrap();
    fs::write(sub.join("beta.rs"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./Project/".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // The trailing slash means "list contents of Project/" — the popup
    // shows the descendents live, without Tab.
    assert!(app.has_path_completion());
    let labels: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert!(
        labels.iter().any(|l| l == "alpha.rs"),
        "descending into Project/ should list alpha.rs, got {:?}",
        labels,
    );
    assert!(
        labels.iter().any(|l| l == "beta.rs"),
        "descending into Project/ should list beta.rs, got {:?}",
        labels,
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn path_popup_actually_renders_into_terminal() {
    // Visual regression guard: the popup state must show up in the rendered
    // frame, not just in the App's internal state. This is the assertion that
    // would have caught any "state updates but UI doesn't redraw" bug.
    //
    // Uses `#Pro` to enter path mode explicitly (the recommended way).
    let _lock = scroll_render_test_lock();
    let tmp = unique_tmp_dir("render");
    fs::create_dir(tmp.join("Project")).unwrap();
    fs::write(tmp.join("Project").join("inner.txt"), b"").unwrap();
    fs::write(tmp.join("Projectile.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "#Pro".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // Live popup is visible because the user is in `#` mode.
    assert!(
        app.has_path_completion(),
        "`#Pro` should show the path popup live"
    );

    // Render and grep the buffer for our labels. We don't pin the exact
    // color since that depends on the renderer's theme, but the text must
    // be visible — otherwise the user sees nothing on Tab.
    let backend = ratatui::backend::TestBackend::new(120, 20);
    let mut terminal = ratatui::Terminal::new(backend).expect("test terminal");
    terminal
        .draw(|f| crate::tui::ui::draw(f, &app))
        .expect("draw");
    let rendered = buffer_to_text(&terminal);

    assert!(
        rendered.contains("Project/"),
        "rendered buffer should show Project/ from the path popup, got:\n{rendered}",
    );
    assert!(
        rendered.contains("Projectile.txt"),
        "rendered buffer should show Projectile.txt from the path popup, got:\n{rendered}",
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn chinese_prefix_then_relative_path_tab_works() {
    // User-reported scenario: `查看路径 ./ho` followed by Tab. The presence of
    // Chinese characters BEFORE the path token must not interfere with path
    // detection. The token under the cursor is `./ho`, which should resolve
    // against the working directory's entries.
    let tmp = unique_tmp_dir("zh_rel");
    fs::create_dir(tmp.join("home")).unwrap();
    fs::create_dir(tmp.join("hot")).unwrap();
    fs::write(tmp.join("hello.txt"), b"").unwrap();
    fs::write(tmp.join("README"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "查看路径 ./ho".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // Live popup — the Chinese prefix and trailing whitespace don't
    // interfere with delimiter-bearing token detection.
    let suggestions: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(label, _)| label)
        .collect();
    assert!(
        app.has_path_completion(),
        "`./ho` after `查看路径 ` must show the path popup live, suggestions were: {:?}",
        suggestions
    );
    // `ho` should match entries starting with `ho` (case-insensitive).
    assert!(
        suggestions.iter().any(|l| l == "home/"),
        "expected home/ in suggestions, got: {:?}",
        suggestions
    );
    assert!(
        suggestions.iter().any(|l| l == "hot/"),
        "expected hot/ in suggestions, got: {:?}",
        suggestions
    );
    assert!(
        !suggestions.iter().any(|l| l.contains("README")),
        "non-matching README must be filtered out, got: {:?}",
        suggestions
    );

    // Pressing Tab applies the first candidate, preserving the Chinese prefix.
    let first_label = suggestions[0].clone();
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    // The apply step rewrites `./ho` to the first match. The exact applied
    // text may be `./home/` (relative form) or `home/` depending on how the
    // candidate was constructed — assert both forms are acceptable.
    assert!(
        app.input.contains(&first_label) || app.input.contains(&format!("./{}", first_label)),
        "applying the first candidate should rewrite `./ho` to include `{}`, got: {:?}",
        first_label,
        app.input
    );
    assert!(
        app.input.starts_with("查看路径 "),
        "Chinese prefix must be preserved verbatim, got: {:?}",
        app.input
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn absolute_path_searches_from_filesystem_root() {
    // A leading-slash token whose first segment has no directory in front
    // (e.g. `/ho`) must search the *filesystem root* `/`, NOT the session
    // working directory. This mirrors pi's behavior: `dirname("/ho") === "/"`
    // and `basename("/ho") === "ho"`, so the search dir is `/` and the
    // needle is `ho`.
    let token = PathToken::parse("/ho").expect("/ho must parse");
    assert_eq!(
        token.parent.as_deref(),
        Some(std::path::Path::new("/")),
        "/ho must resolve its parent to the filesystem root, got: {:?}",
        token.parent
    );
    assert_eq!(token.prefix, "ho");
    assert!(!token.is_root_listing);

    // The "list contents" form: `/tmp/` should also resolve parent to `/tmp`
    // (absolute), and prefix to "".
    let token = PathToken::parse("/tmp/").expect("/tmp/ must parse");
    assert_eq!(
        token.parent.as_deref(),
        Some(std::path::Path::new("/tmp")),
        "/tmp/ must keep its absolute parent, got: {:?}",
        token.parent
    );
    assert_eq!(token.prefix, "");
    assert!(token.descendent);
    assert!(!token.is_root_listing);

    // And the multi-segment absolute form: `/etc/hos` should look in /etc.
    let token = PathToken::parse("/etc/hos").expect("/etc/hos must parse");
    assert_eq!(
        token.parent.as_deref(),
        Some(std::path::Path::new("/etc")),
        "/etc/hos must resolve its parent to /etc, got: {:?}",
        token.parent
    );
    assert_eq!(token.prefix, "hos");
}

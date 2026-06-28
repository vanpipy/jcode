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
    fs::write(tmp.join("Projectile.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "look at ./Pro".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(app.has_path_completion(), "popup should be active");
    let s = app.path_completion_suggestions();
    assert!(!s.is_empty());
    // The first candidate should be applied into the input.
    let first_value = s[0].0.clone();
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
    fs::create_dir(tmp.join("Abb")).unwrap();
    fs::write(tmp.join("Acc.txt"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./A".to_string();
    app.cursor_pos = app.input.len();

    // First Tab: popup opens, first candidate is applied.
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(app.has_path_completion());
    // The applied value replaces just the path token (without the `./` prefix
    // since the parser strips the prefix marker).
    let first_value = app.path_completion_suggestions()[0].0.clone();
    assert!(first_value.starts_with("A"));
    assert!(app.input.ends_with(&first_value) || app.input.contains(&first_value));
    let initial_selected = app.path_completion_selected();

    // Second Tab: cycle to the next candidate.
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    let next_selected = app.path_completion_selected();
    assert_ne!(initial_selected, next_selected);

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn up_down_arrows_move_selection_in_popup() {
    let tmp = unique_tmp_dir("arrows");
    fs::create_dir(tmp.join("One")).unwrap();
    fs::create_dir(tmp.join("Two")).unwrap();
    fs::create_dir(tmp.join("Three")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
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
    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(app.has_path_completion());

    handle(&mut app, KeyCode::Esc, KeyModifiers::empty());
    assert!(!app.has_path_completion(), "Esc should dismiss the popup");

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn reset_tab_completion_also_clears_path_state() {
    let tmp = unique_tmp_dir("reset");
    fs::create_dir(tmp.join("Foo")).unwrap();
    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "./".to_string();
    app.cursor_pos = app.input.len();

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert!(app.has_path_completion());

    // The input layer calls reset_tab_completion() after every keystroke that
    // mutates the buffer. The reset must also drop the path popup, otherwise
    // stale candidates would be applied to a freshly-edited input.
    app.reset_tab_completion();
    assert!(
        !app.has_path_completion(),
        "reset_tab_completion must also clear the path popup"
    );

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

    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
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

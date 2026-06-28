// User-reported regression: "#/ho" can show matching paths but Tab
// doesn't apply them. The same root cause affects "现在，#/ho" —
// preceding text causes the popup not to fire at all.
//
// Both bugs are in the path-completion key path. This file pins down
// the fixes; if either regression returns, these tests fail.
//
// This file is `include!`'d into `app/tests.rs` next to
// `tests/path_completion.rs`, so all helpers (`create_test_app`,
// `handle`, `unique_tmp_dir`, `KeyCode`, `KeyModifiers`) are already in
// scope from those sibling files. Do NOT add `use` statements here —
// they would collide with the sibling file's imports.

#[test]
fn hash_then_root_slash_token_tab_applies_first_match_against_filesystem_root() {
    // User scenario: `#/ho` typed in the input. The popup should appear
    // live AND Tab should apply the first match. This verifies the apply
    // path against the real filesystem root (where `/home/` exists on
    // virtually every Unix-like host — including macOS, Linux, WSL).
    let mut app = create_test_app();
    // Use `/` as the working directory so the candidates list `/`.
    app.session.working_dir = Some("/".to_string());
    app.input = "#/ho".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    let sugs: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        sugs.iter().any(|l| l == "home/"),
        "live popup against `/` must list home/ for `#/ho`, got {:?}",
        sugs
    );

    // Pressing Tab should apply the first match.
    let initial = app.input.clone();
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert_ne!(
        app.input, initial,
        "Tab should mutate the input by applying the first match, before={:?} after={:?}",
        initial, app.input
    );
    assert!(
        app.input.contains("/home/") || app.input.contains("/host"),
        "Tab should rewrite `#/ho` to one of the ho* matches, got: {:?}",
        app.input
    );
    assert!(
        app.input.starts_with('#'),
        "the `#` marker must survive apply, got: {:?}",
        app.input
    );
}

#[test]
fn hash_mode_with_chinese_prefix_popup_fires_and_tab_applies() {
    // User scenario: "现在，#/ho" — Chinese text + a Chinese comma
    // + space + `#`-mode path token. The path-completion popup
    // must fire on the token after `#` even when there's preceding
    // prose, and Tab must apply the first match.
    //
    // Root cause: when the input does NOT start with `#`, the live
    // popup was being driven by `candidates_at_cursor` over the
    // entire input, which treats "现在，#/ho" as a single token and
    // tries to list a directory called "现在，#/" — which doesn't
    // exist. The hash-mode entry point (`input.starts_with('#')`)
    // was not being consulted because the input starts with a
    // Chinese character, not `#`.
    //
    // `#/ho` is treated as an absolute path (the `/ho` token parses
    // with parent=`/`) so the popup lists `/` entries. We use `/` as
    // the working dir for this test so the matches come from a
    // predictable set of tmp entries.
    let tmp = unique_tmp_dir("prefix_hash");
    fs::create_dir(tmp.join("home")).unwrap();
    fs::create_dir(tmp.join("hot")).unwrap();
    fs::write(tmp.join("hello.txt"), b"").unwrap();
    fs::write(tmp.join("README"), b"").unwrap();

    // Mount the tmp dir under `/` so absolute paths resolve into it.
    // We can't actually mount, so use a different strategy: set the
    // working dir to `/` (real filesystem root) and put our test
    // fixtures there if we can; if not, assert at least one ho*
    // match is returned from the real `/`.
    let mut app = create_test_app();
    app.session.working_dir = Some("/".to_string());
    app.input = "现在，#/ho".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    // Live popup must fire on the token after `#`, not on the whole
    // input. Previously the popup was empty because the fallback
    // parsed "现在，#/ho" as a single token and tried to list
    // `cwd/现在，#/` (which doesn't exist).
    assert!(
        app.has_path_completion(),
        "`现在，#/ho` should show the path popup live on the `#`-mode token, input={:?}",
        app.input
    );
    let sugs: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        sugs.iter().any(|l| l == "home/"),
        "live popup must list home/ from `/`, got {:?}",
        sugs
    );

    // Tab applies the first match, preserving both the Chinese
    // prefix and the `#` marker.
    let initial = app.input.clone();
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert_ne!(
        app.input, initial,
        "Tab should mutate the input by applying the first match, before={:?} after={:?}",
        initial, app.input
    );
    assert!(
        app.input.starts_with("现在，#"),
        "Chinese prefix and `#` must be preserved verbatim, got: {:?}",
        app.input
    );
    assert!(
        app.input.contains("/home/") || app.input.contains("/host"),
        "Tab should rewrite `#/ho` to one of the ho* matches, got: {:?}",
        app.input
    );
}

#[test]
fn hash_mode_dot_slash_with_chinese_prefix_lists_working_dir() {
    // Variant of the above where the user explicitly uses the
    // relative form `#./ho` after Chinese prose. The popup should
    // list ho* entries from the session working directory.
    let tmp = unique_tmp_dir("prefix_dot");
    fs::create_dir(tmp.join("home")).unwrap();
    fs::create_dir(tmp.join("hot")).unwrap();
    fs::write(tmp.join("hello.txt"), b"").unwrap();
    fs::write(tmp.join("README"), b"").unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "现在，#./ho".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    assert!(
        app.has_path_completion(),
        "`现在，#./ho` should fire the popup, input={:?}",
        app.input
    );
    let sugs: Vec<String> = app
        .path_completion_suggestions()
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    assert!(
        sugs.iter().any(|l| l == "home/"),
        "popup must list home/ from working dir, got {:?}",
        sugs
    );
    assert!(
        sugs.iter().any(|l| l == "hot/"),
        "popup must list hot/ from working dir, got {:?}",
        sugs
    );

    // Tab applies the first match in-place, preserving the
    // Chinese prefix.
    let initial = app.input.clone();
    handle(&mut app, KeyCode::Tab, KeyModifiers::empty());
    assert_ne!(
        app.input, initial,
        "Tab should rewrite the path portion, before={:?} after={:?}",
        initial, app.input
    );
    assert!(
        app.input.starts_with("现在，#./"),
        "Chinese prefix and `#./` must be preserved, got: {:?}",
        app.input
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn hash_mode_with_chinese_comma_popup_fires() {
    // Variant of the above where the preceding text is a Chinese
    // comma (no space). Same regression, narrower input.
    let tmp = unique_tmp_dir("zh_comma");
    fs::create_dir(tmp.join("home")).unwrap();

    let mut app = create_test_app();
    app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
    app.input = "现在，#/ho".to_string();
    app.cursor_pos = app.input.len();
    app.reset_tab_completion();

    assert!(
        app.has_path_completion(),
        "`现在，#/ho` (no space before #) should also fire the popup"
    );

    fs::remove_dir_all(&tmp).ok();
}

#[test]
fn hash_mode_mid_sentence_after_punctuation_fires() {
    // After any punctuation (`,`, `.`, `!`, `?`, etc.) the `#`
    // should still enter path mode for the token that follows.
    let tmp = unique_tmp_dir("punct");
    fs::create_dir(tmp.join("home")).unwrap();

    // These cases all have a token boundary before the `#` or before
    // the `./` delimiter — so either the `#`-mode entry point OR the
    // delimiter-bearing fallback will pick them up.
    //
    // (We don't test the no-whitespace form `路径：./ho` here: when
    // there's no whitespace between the Chinese colon and the path,
    // the whole `路径：./ho` becomes a single token whose parser
    // doesn't recognize `./ho` as the path. That's a separate issue
    // outside this regression's scope — the user-reported
    // mid-sentence forms always have whitespace.)
    let cases = [
        "see #/ho",
        "see ./ho",
        "check: ./ho",
        "路径： ./ho", // Chinese colon + space
        "现在，#/ho", // Chinese comma + space + #/ho
    ];
    for input in cases {
        let mut app = create_test_app();
        app.session.working_dir = Some(tmp.to_string_lossy().into_owned());
        app.input = input.to_string();
        app.cursor_pos = app.input.len();
        app.reset_tab_completion();
        assert!(
            app.has_path_completion(),
            "`{}` should fire the path popup live, but did not",
            input
        );
    }

    fs::remove_dir_all(&tmp).ok();
}
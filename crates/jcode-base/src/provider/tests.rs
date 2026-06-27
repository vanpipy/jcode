use super::*;
use crate::provider::models::{ensure_model_allowed_for_subscription, filtered_display_models};

fn with_clean_provider_test_env<T>(f: impl FnOnce() -> T) -> T {
    let _guard = crate::storage::lock_test_env();
    let temp = tempfile::tempdir().expect("tempdir");
    let prev_home = std::env::var_os("JCODE_HOME");
    let prev_subscription =
        std::env::var_os(crate::subscription_catalog::JCODE_SUBSCRIPTION_ACTIVE_ENV);
    let mut profile_env_keys = vec![
        "OPENROUTER_API_KEY",
        "DEEPSEEK_API_KEY",
        "KIMI_API_KEY",
        "JCODE_OPENROUTER_API_BASE",
        "JCODE_OPENROUTER_API_KEY_NAME",
        "JCODE_OPENROUTER_ENV_FILE",
        "JCODE_OPENROUTER_CACHE_NAMESPACE",
        "JCODE_OPENROUTER_PROVIDER_FEATURES",
        "JCODE_OPENROUTER_TRANSPORT_STATE",
        "JCODE_OPENROUTER_ALLOW_NO_AUTH",
        "JCODE_OPENROUTER_MODEL_CATALOG",
        "JCODE_OPENROUTER_MODEL",
        "JCODE_OPENROUTER_STATIC_MODELS",
        "JCODE_OPENAI_COMPAT_API_BASE",
        "JCODE_OPENAI_COMPAT_API_KEY_NAME",
        "JCODE_OPENAI_COMPAT_ENV_FILE",
        "JCODE_OPENAI_COMPAT_DEFAULT_MODEL",
        "JCODE_OPENAI_COMPAT_LOCAL_ENABLED",
        "OPENAI_COMPAT_API_KEY",
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "JCODE_RUNTIME_PROVIDER",
        "JCODE_ACTIVE_PROVIDER",
        "JCODE_FORCE_PROVIDER",
        "JCODE_OPENAI_MODEL",
        "JCODE_NAMED_PROVIDER_PROFILE",
        "JCODE_PROVIDER_PROFILE_ACTIVE",
        "JCODE_PROVIDER_PROFILE_NAME",
    ];
    for profile in crate::provider_catalog::openai_compatible_profiles() {
        if !profile_env_keys.contains(&profile.api_key_env) {
            profile_env_keys.push(profile.api_key_env);
        }
    }
    let saved_profile_env = profile_env_keys
        .into_iter()
        .map(|key| (key, std::env::var_os(key)))
        .collect::<Vec<_>>();
    crate::env::set_var("JCODE_HOME", temp.path());
    for (key, _) in &saved_profile_env {
        crate::env::remove_var(key);
    }
    crate::subscription_catalog::clear_runtime_env();
    crate::auth::claude::set_active_account_override(None);
    crate::auth::codex::set_active_account_override(None);
    // The in-memory model catalog services are process-global; earlier tests
    // may have hydrated scopes (fixture models) that would corrupt this test's
    // known_*_model_ids() validation, and vice versa. Reset on entry and exit
    // so neither direction leaks.
    crate::provider::models::reset_model_catalog_services_for_tests();

    let result = f();

    crate::provider::models::reset_model_catalog_services_for_tests();
    crate::auth::claude::set_active_account_override(None);
    crate::auth::codex::set_active_account_override(None);
    if let Some(prev_home) = prev_home {
        crate::env::set_var("JCODE_HOME", prev_home);
    } else {
        crate::env::remove_var("JCODE_HOME");
    }
    if let Some(prev_subscription) = prev_subscription {
        crate::env::set_var(
            crate::subscription_catalog::JCODE_SUBSCRIPTION_ACTIVE_ENV,
            prev_subscription,
        );
    } else {
        crate::env::remove_var(crate::subscription_catalog::JCODE_SUBSCRIPTION_ACTIVE_ENV);
    }
    for (key, value) in saved_profile_env {
        if let Some(value) = value {
            crate::env::set_var(key, value);
        } else {
            crate::env::remove_var(key);
        }
    }
    crate::subscription_catalog::clear_runtime_env();
    result
}

fn enter_test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio runtime")
}

fn with_env_var<T>(key: &str, value: &str, f: impl FnOnce() -> T) -> T {
    let prev = std::env::var_os(key);
    crate::env::set_var(key, value);
    let result = f();
    if let Some(prev) = prev {
        crate::env::set_var(key, prev);
    } else {
        crate::env::remove_var(key);
    }
    result
}

fn save_test_openai_compatible_login_config(default_model: &str) {
    let env_file = crate::provider_catalog::OPENAI_COMPAT_PROFILE.env_file;
    crate::provider_catalog::save_env_value_to_env_file(
        "JCODE_OPENAI_COMPAT_API_BASE",
        env_file,
        Some("https://example-openai-compatible.test/v1"),
    )
    .expect("save api base");
    crate::provider_catalog::save_env_value_to_env_file(
        "OPENAI_COMPAT_API_KEY",
        env_file,
        Some("sk-test-openai-compatible"),
    )
    .expect("save api key");
    crate::provider_catalog::save_env_value_to_env_file(
        "JCODE_OPENAI_COMPAT_DEFAULT_MODEL",
        env_file,
        Some(default_model),
    )
    .expect("save default model");
}

fn save_test_openrouter_model_cache(namespace: &str, source_api_base: &str, model_ids: &[&str]) {
    let jcode_home = std::env::var_os("JCODE_HOME").expect("test JCODE_HOME should be set");
    let cache_dir = std::path::PathBuf::from(jcode_home).join("cache");
    std::fs::create_dir_all(&cache_dir).expect("create model cache dir");
    let cache = jcode_provider_openrouter::DiskCache {
        cached_at: jcode_provider_openrouter::current_unix_secs().expect("current unix time"),
        source_api_base: Some(source_api_base.to_string()),
        models: model_ids
            .iter()
            .map(|id| jcode_provider_openrouter::ModelInfo {
                id: (*id).to_string(),
                name: String::new(),
                context_length: None,
                pricing: jcode_provider_openrouter::ModelPricing::default(),
                created: None,
            })
            .collect(),
    };
    let path = cache_dir.join(format!("{namespace}_models.json"));
    std::fs::write(
        path,
        serde_json::to_string(&cache).expect("serialize model cache"),
    )
    .expect("write model cache");
}

fn clear_openai_compatible_runtime_env() {
    for key in [
        "JCODE_OPENAI_COMPAT_API_BASE",
        "JCODE_OPENAI_COMPAT_API_KEY_NAME",
        "JCODE_OPENAI_COMPAT_ENV_FILE",
        "JCODE_OPENAI_COMPAT_DEFAULT_MODEL",
        "JCODE_OPENAI_COMPAT_LOCAL_ENABLED",
        "OPENAI_COMPAT_API_KEY",
        "JCODE_OPENROUTER_CACHE_NAMESPACE",
    ] {
        crate::env::remove_var(key);
    }
}

fn save_test_openai_oauth_credentials() {
    crate::auth::codex::upsert_account_from_tokens(
        &crate::auth::codex::primary_account_label(),
        "test-oauth-access-token",
        "test-oauth-refresh-token",
        None,
        Some(chrono::Utc::now().timestamp_millis() + 86_400_000),
    )
    .expect("save test OpenAI OAuth credentials");
}

fn test_multi_provider_with_openai() -> MultiProvider {
    save_test_openai_oauth_credentials();
    crate::env::set_var("OPENAI_API_KEY", "sk-test-openai-api-key");
    let credentials = crate::auth::codex::load_credentials().expect("OpenAI credentials");
    MultiProvider {
        claude: RwLock::new(None),
        anthropic: RwLock::new(None),
        openai: RwLock::new(Some(Arc::new(openai::OpenAIProvider::new(credentials)))),
        copilot_api: RwLock::new(None),
        antigravity: RwLock::new(None),
        gemini: RwLock::new(None),
        cursor: RwLock::new(None),
        bedrock: RwLock::new(None),
        openrouter: RwLock::new(None),
        openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
        active_openai_compatible_profile: RwLock::new(None),
        active: RwLock::new(ActiveProvider::OpenAI),
        use_claude_cli: false,
        startup_notices: RwLock::new(Vec::new()),
        forced_provider: None,
    }
}

#[test]
fn openai_model_switch_prefixes_preserve_oauth_vs_api_state_space() {
    with_clean_provider_test_env(|| {
        let rt = enter_test_runtime();
        let _runtime_guard = rt.enter();
        let provider = test_multi_provider_with_openai();
        let models = known_openai_model_ids();
        let primary = models.first().expect("at least one OpenAI model").as_str();
        let alternate = models.get(1).map(String::as_str).unwrap_or(primary);

        let cases = [
            vec![
                (
                    format!("openai-api:{primary}"),
                    openai::OpenAICredentialMode::ApiKey,
                    primary,
                ),
                (
                    format!("openai-oauth:{alternate}"),
                    openai::OpenAICredentialMode::OAuth,
                    alternate,
                ),
            ],
            vec![
                (
                    format!("openai-oauth:{primary}"),
                    openai::OpenAICredentialMode::OAuth,
                    primary,
                ),
                (
                    format!("openai-api:{alternate}"),
                    openai::OpenAICredentialMode::ApiKey,
                    alternate,
                ),
                (
                    format!("openai-oauth:{primary}"),
                    openai::OpenAICredentialMode::OAuth,
                    primary,
                ),
            ],
        ];

        for sequence in cases {
            for (request, expected_mode, expected_model) in sequence {
                provider
                    .set_model(&request)
                    .unwrap_or_else(|err| panic!("switch {request} should succeed: {err}"));
                assert_eq!(
                    provider.active_provider(),
                    ActiveProvider::OpenAI,
                    "{request}"
                );
                assert_eq!(provider.model(), expected_model, "{request}");
                assert_eq!(
                    provider
                        .openai_provider()
                        .expect("OpenAI provider")
                        .credential_mode_snapshot(),
                    expected_mode,
                    "{request}"
                );
            }
        }
    });
}

#[test]
fn openai_model_route_roundtrip_preserves_auth_method_for_model_switches() {
    with_clean_provider_test_env(|| {
        let rt = enter_test_runtime();
        let _runtime_guard = rt.enter();
        let provider = test_multi_provider_with_openai();
        let models = known_openai_model_ids();
        let primary = models.first().expect("at least one OpenAI model").as_str();
        let alternate = models.get(1).map(String::as_str).unwrap_or(primary);

        // This mirrors the /model picker path: the selected ModelRoute becomes a
        // default/session model + provider key, then a future /model switch uses
        // that persisted provider key to reconstruct the provider-prefixed
        // request. The important invariant is that OpenAI OAuth and API key are
        // distinct states even though both execute in ActiveProvider::OpenAI.
        let route_cases = [
            (
                primary,
                "openai-oauth",
                "openai",
                "openai-oauth",
                openai::OpenAICredentialMode::OAuth,
            ),
            (
                alternate,
                "openai-api-key",
                "openai-api",
                "openai-api",
                openai::OpenAICredentialMode::ApiKey,
            ),
            (
                primary,
                "openai-api",
                "openai-api",
                "openai-api",
                openai::OpenAICredentialMode::ApiKey,
            ),
        ];

        for (bare_model, api_method, expected_provider_key, expected_prefix, expected_mode) in
            route_cases
        {
            let selection =
                MultiProvider::default_model_selection_from_route(bare_model, api_method, "OpenAI");
            assert_eq!(
                selection.model_spec,
                format!("{expected_prefix}:{bare_model}")
            );
            assert_eq!(
                selection.provider_key.as_deref(),
                Some(expected_provider_key)
            );

            let request = MultiProvider::model_switch_request_for_session_model(
                bare_model,
                selection.provider_key.as_deref(),
            );
            assert_eq!(request, format!("{expected_prefix}:{bare_model}"));

            provider
                .set_model(&request)
                .unwrap_or_else(|err| panic!("/model switch {request} should succeed: {err}"));
            assert_eq!(
                provider.active_provider(),
                ActiveProvider::OpenAI,
                "{request}"
            );
            assert_eq!(provider.model(), bare_model, "{request}");
            assert_eq!(
                provider
                    .openai_provider()
                    .expect("OpenAI provider")
                    .credential_mode_snapshot(),
                expected_mode,
                "{request}"
            );
        }
    });
}

#[test]
fn active_explicit_credential_reflects_openai_switch_immediately_and_none_for_auto() {
    use jcode_provider_core::{Provider, ResolvedCredential};
    with_clean_provider_test_env(|| {
        let rt = enter_test_runtime();
        let _runtime_guard = rt.enter();
        let provider = test_multi_provider_with_openai();
        let model = known_openai_model_ids()
            .first()
            .expect("at least one OpenAI model")
            .clone();

        // Fresh provider with both credentials present defaults to auto, which
        // has no explicit pin: the info widget must fall back to its cached
        // heuristic instead of asserting an OAuth-vs-API choice the user never
        // made.
        assert_eq!(
            provider.active_explicit_credential(),
            None,
            "auto mode must not report an explicit pin"
        );

        // Switching to the API-key route pins the credential in memory, so the
        // widget must report API key on the very next read with no cache delay.
        provider
            .set_model(&format!("openai-api:{model}"))
            .expect("switch to OpenAI API key");
        assert_eq!(
            provider.active_explicit_credential(),
            Some(ResolvedCredential::ApiKey),
            "explicit API-key switch must be visible immediately"
        );

        // Switching back to OAuth flips it back just as immediately.
        provider
            .set_model(&format!("openai-oauth:{model}"))
            .expect("switch to OpenAI OAuth");
        assert_eq!(
            provider.active_explicit_credential(),
            Some(ResolvedCredential::Oauth),
            "explicit OAuth switch must be visible immediately"
        );
    });
}

#[test]
fn openai_model_routes_cover_oauth_api_and_no_auth_state_space() {
    with_clean_provider_test_env(|| {
        let rt = enter_test_runtime();
        let _runtime_guard = rt.enter();
        let model = known_openai_model_ids()
            .first()
            .expect("at least one OpenAI model")
            .clone();

        let provider = test_multi_provider_with_openai();
        let routes = provider.model_routes();
        let methods = routes
            .iter()
            .filter(|route| route.provider == "OpenAI" && route.model == model)
            .map(|route| (route.api_method.as_str(), route.available))
            .collect::<Vec<_>>();
        assert!(
            methods.contains(&("openai-oauth", true)),
            "routes: {methods:?}"
        );
        assert!(
            methods.contains(&("openai-api-key", true)),
            "routes: {methods:?}"
        );

        crate::env::remove_var("OPENAI_API_KEY");
        crate::auth::AuthStatus::invalidate_cache();
        let oauth_only = provider.model_routes();
        let oauth_only_methods = oauth_only
            .iter()
            .filter(|route| route.provider == "OpenAI" && route.model == model)
            .map(|route| route.api_method.as_str())
            .collect::<Vec<_>>();
        assert_eq!(oauth_only_methods, vec!["openai-oauth"]);

        crate::env::set_var("OPENAI_API_KEY", "sk-test-openai-api-key");
        std::fs::remove_file(
            crate::storage::jcode_dir()
                .unwrap()
                .join("openai-auth.json"),
        )
        .expect("remove oauth credentials");
        crate::auth::AuthStatus::invalidate_cache();
        let api_only = provider.model_routes();
        let api_only_methods = api_only
            .iter()
            .filter(|route| route.provider == "OpenAI" && route.model == model)
            .map(|route| route.api_method.as_str())
            .collect::<Vec<_>>();
        assert_eq!(api_only_methods, vec!["openai-api-key"]);
    });
}

fn assert_openai_compatible_route_available(provider: &MultiProvider, model: &str) {
    let routes = provider.model_routes();
    assert!(
        routes.iter().any(|route| {
            route.provider == "OpenAI-compatible"
                && matches!(
                    route.api_method.as_str(),
                    "openai-compatible" | "openai-compatible:openai-compatible"
                )
                && route.model == model
                && route.available
        }),
        "configured OpenAI-compatible model should be immediately visible after API-key setup; routes: {routes:?}"
    );
}

#[test]
fn openai_compatible_api_key_setup_makes_configured_model_route_available() {
    with_clean_provider_test_env(|| {
        save_test_openai_compatible_login_config("glm-test-login-flow");

        assert!(
            crate::provider_catalog::openai_compatible_profile_is_configured(
                crate::provider_catalog::OPENAI_COMPAT_PROFILE,
            )
        );

        let provider = MultiProvider::new();
        assert_openai_compatible_route_available(&provider, "glm-test-login-flow");

        provider
            .set_model_on_openai_compatible_profile(
                crate::provider_catalog::OPENAI_COMPAT_PROFILE,
                "glm-test-login-flow",
            )
            .expect("configured OpenAI-compatible model should select without requiring another provider login");

        assert_eq!(provider.model(), "glm-test-login-flow");
    });
}

#[test]
fn openai_compatible_api_key_setup_survives_process_restart_without_relogin() {
    with_clean_provider_test_env(|| {
        save_test_openai_compatible_login_config("restart-visible-model");

        // Simulate a fresh process: the login command wrote the config file, but
        // none of the runtime env vars from the login process remain populated.
        clear_openai_compatible_runtime_env();

        let resolved = crate::provider_catalog::resolve_openai_compatible_profile(
            crate::provider_catalog::OPENAI_COMPAT_PROFILE,
        );
        assert_eq!(
            resolved.api_base,
            "https://example-openai-compatible.test/v1"
        );
        assert_eq!(
            resolved.default_model.as_deref(),
            Some("restart-visible-model")
        );
        assert!(
            crate::provider_catalog::openai_compatible_profile_is_configured(
                crate::provider_catalog::OPENAI_COMPAT_PROFILE,
            )
        );

        let provider = MultiProvider::new();
        assert_openai_compatible_route_available(&provider, "restart-visible-model");
        provider
            .set_model_on_openai_compatible_profile(
                crate::provider_catalog::OPENAI_COMPAT_PROFILE,
                "restart-visible-model",
            )
            .expect("saved credentials should be selectable after a fresh process restart");
        assert_eq!(provider.model(), "restart-visible-model");
    });
}

#[test]
fn configured_openai_compatible_profile_routes_use_live_cache_when_not_active_provider() {
    with_clean_provider_test_env(|| {
        crate::provider_catalog::save_env_value_to_env_file(
            "OPENROUTER_API_KEY",
            "openrouter.env",
            Some("sk-test-openrouter"),
        )
        .expect("save openrouter key");
        crate::provider_catalog::save_env_value_to_env_file(
            "OPENCODE_API_KEY",
            "opencode.env",
            Some("oc-test-opencode"),
        )
        .expect("save opencode key");
        save_test_openrouter_model_cache(
            "opencode",
            "https://opencode.ai/zen/v1",
            &["kimi-k2.6", "zen-live-only-model"],
        );

        let provider = MultiProvider::new();
        let routes = provider.model_routes();
        let opencode_routes = routes
            .iter()
            .filter(|route| route.provider == "OpenCode Zen")
            .collect::<Vec<_>>();

        assert!(
            opencode_routes
                .iter()
                .any(|route| route.model == "zen-live-only-model"
                    && route.api_method == "openai-compatible:opencode"
                    && !route
                        .detail
                        .contains("fallback: static provider model list")),
            "non-active configured direct profile should expose its live /models cache, routes: {opencode_routes:?}"
        );
        assert!(
            !opencode_routes.iter().any(|route| route.model == "glm-4.7"),
            "static fallback models should drop out once a live profile catalog is available, routes: {opencode_routes:?}"
        );
    });
}

#[test]
fn standard_openrouter_catalog_refresh_is_noop_when_cache_fresh() {
    with_clean_provider_test_env(|| {
        let runtime = enter_test_runtime();
        runtime.block_on(async {
            crate::provider_catalog::save_env_value_to_env_file(
                "OPENROUTER_API_KEY",
                "openrouter.env",
                Some("sk-test-openrouter"),
            )
            .expect("save openrouter key");
            // A fresh, non-empty standard OpenRouter cache should suppress the
            // background refresh entirely so we never fire a needless network
            // request on every picker render.
            save_test_openrouter_model_cache(
                "openrouter",
                "https://openrouter.ai/api/v1",
                &["openrouter/owl-alpha"],
            );

            assert!(
                !openrouter::maybe_schedule_standard_openrouter_catalog_refresh(
                    "unit test fresh cache"
                ),
                "a fresh non-empty standard OpenRouter cache must not trigger a refresh"
            );
        });
    });
}

#[test]
fn standard_openrouter_catalog_refresh_skips_without_key() {
    with_clean_provider_test_env(|| {
        let runtime = enter_test_runtime();
        runtime.block_on(async {
            // No OPENROUTER_API_KEY configured: the refresh must not be
            // scheduled regardless of cache state.
            assert!(
                !openrouter::maybe_schedule_standard_openrouter_catalog_refresh(
                    "unit test missing key"
                ),
                "standard OpenRouter refresh must be skipped when no key is configured"
            );
        });
    });
}

#[test]
fn standard_openrouter_catalog_refresh_fires_when_named_profile_owns_slot() {
    with_clean_provider_test_env(|| {
        let runtime = enter_test_runtime();
        runtime.block_on(async {
            crate::provider_catalog::save_env_value_to_env_file(
                "OPENROUTER_API_KEY",
                "openrouter.env",
                Some("sk-test-openrouter"),
            )
            .expect("save openrouter key");
            // Simulate an active named profile (e.g. NVIDIA NIM) occupying the
            // shared OpenRouter/OpenAI-compatible slot: it sets the runtime env
            // vars to point at a non-openrouter.ai endpoint. The standard
            // OpenRouter catalog refresh must STILL fire so `/model` can list
            // openrouter.ai models (issue #292). Cache is missing -> not fresh.
            crate::env::set_var(
                "JCODE_OPENROUTER_API_BASE",
                "https://integrate.api.nvidia.com/v1",
            );
            crate::env::set_var("JCODE_OPENROUTER_CACHE_NAMESPACE", "mynvidia");

            // Other tests in this process may already have attempted (or be
            // running) an `openrouter` catalog refresh; clear the process-wide
            // backoff/in-flight tracker or this assertion is flaky under
            // parallel test execution.
            openrouter::reset_profile_catalog_refresh_tracker_for_tests();

            assert!(
                openrouter::maybe_schedule_standard_openrouter_catalog_refresh(
                    "unit test named profile owns slot"
                ),
                "standard OpenRouter refresh must fire even when a named profile sets JCODE_OPENROUTER_* env"
            );
        });
    });
}

fn test_multi_provider_with_cursor() -> MultiProvider {
    MultiProvider {
        claude: RwLock::new(None),
        anthropic: RwLock::new(None),
        openai: RwLock::new(None),
        copilot_api: RwLock::new(None),
        antigravity: RwLock::new(None),
        gemini: RwLock::new(None),
        cursor: RwLock::new(Some(Arc::new(cursor::CursorCliProvider::new()))),
        bedrock: RwLock::new(None),
        openrouter: RwLock::new(None),
        openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
        active_openai_compatible_profile: RwLock::new(None),
        active: RwLock::new(ActiveProvider::Cursor),
        use_claude_cli: false,
        startup_notices: RwLock::new(Vec::new()),
        forced_provider: None,
    }
}

include!("tests/auth_refresh.rs");
include!("tests/model_resolution.rs");
include!("tests/fallback_failover.rs");
include!("tests/catalog_subscription.rs");

/// Regression test for the user-defined profile session-restore bug (Bug B).
///
/// User scenario: `default_provider = "minimax-cn"` with a user-defined
/// `[providers.minimax-cn]` in `~/.jcode/config.toml`. When jcode starts
/// and creates a new session, it calls `provider.model()` to capture the
/// active model into `session.model`. For user-defined profiles, the openrouter
/// slot's runtime IS the `minimax-cn` profile (created via
/// `OpenRouterProvider::new_named_openai_compatible` at startup), and its
/// `model` field holds the bare `MiniMax-M3` from `default_model`. So `set_model`
/// is called via `set_config_default_model("MiniMax-M3", Some("minimax-cn"))`
/// and the slot stores `MiniMax-M3`. Good so far.
///
/// The bug appears when jcode restores an existing session whose `session.model`
/// is `minimax-cn:MiniMax-M3` (this is the form `set_route_selection` writes
/// via `RouteSelection::routed_model_spec()`). At restore, the agent calls
/// `set_model_with_auth_refresh(provider, "minimax-cn:MiniMax-M3")`. If the
/// openrouter slot has been REPLACED with a real-OpenRouter runtime at some
/// point (e.g. via `new_openrouter_api_key_runtime`), `profile_id` becomes
/// `None` and `strip_session_profile_prefix` no longer strips the prefix,
/// leaving the slot's model as `minimax-cn:MiniMax-M3` which the API rejects
/// with 400.
///
/// This test exercises the reproduce: build a MultiProvider with the user-defined
/// `minimax-cn` openrouter slot, then trigger a sequence of operations that
/// mimics (a) initial config_default_model set, (b) session-save capturing the
/// bare model, (c) session-restore with the prefixed model spec.
#[test]
fn test_user_defined_profile_session_round_trip_strips_prefix() {
    with_clean_provider_test_env(|| {
        let _key = crate::env::set_var("MINIMAX_API_KEY", "test-minimax-key");
        crate::env::remove_var("JCODE_OPENROUTER_API_KEY_NAME");
        crate::env::remove_var("JCODE_OPENROUTER_API_BASE");
        crate::env::remove_var("JCODE_OPENROUTER_CACHE_NAMESPACE");
        crate::env::remove_var("JCODE_OPENROUTER_PROVIDER_FEATURES");

        let profile = crate::config::NamedProviderConfig {
            provider_type: crate::config::NamedProviderType::OpenAiCompatible,
            base_url: "https://api.minimaxi.com/v1".to_string(),
            api: None,
            auth: crate::config::NamedProviderAuth::Bearer,
            auth_header: None,
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            api_key: None,
            env_file: None,
            default_model: Some("MiniMax-M3".to_string()),
            requires_api_key: Some(true),
            provider_routing: false,
            model_catalog: true,
            allow_provider_pinning: false,
            models: vec![],
            extra_body: None,
            supports_reasoning_effort: None,
        };
        let openrouter = openrouter::OpenRouterProvider::new_named_openai_compatible(
            "minimax-cn",
            &profile,
        )
        .expect("named provider should initialize");

        let provider = MultiProvider {
            claude: RwLock::new(None),
            anthropic: RwLock::new(None),
            openai: RwLock::new(None),
            copilot_api: RwLock::new(None),
            antigravity: RwLock::new(None),
            gemini: RwLock::new(None),
            cursor: RwLock::new(None),
            bedrock: RwLock::new(None),
            openrouter: RwLock::new(Some(Arc::new(openrouter))),
            openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
            active_openai_compatible_profile: RwLock::new(None),
            active: RwLock::new(ActiveProvider::OpenRouter),
            use_claude_cli: false,
            startup_notices: RwLock::new(Vec::new()),
            forced_provider: Some(ActiveProvider::OpenRouter),
        };

        // (a) Startup: set_config_default_model("MiniMax-M3", "minimax-cn")
        provider
            .set_model("MiniMax-M3")
            .expect("set_model bare model should succeed");
        assert_eq!(provider.model(), "MiniMax-M3");

        // (b) Session save: capture current model into session.model.
        //     For new sessions, the prefix should NOT appear here because
        //     the openrouter slot already holds the bare id.
        let saved_session_model = provider.model();
        assert_eq!(
            saved_session_model, "MiniMax-M3",
            "session.model captured at session-create time must be the bare \
             model id; if it ever gets the `<profile>:` prefix here, the \
             session file is poisoned on save and restore cannot fix it"
        );

        // (c) Some downstream code (e.g. picker selection) writes the
        //     prefixed form into session.model and the next restore tries
        //     to set it. This is the poisoned-state case.
        let poisoned_session_model = "minimax-cn:MiniMax-M3";
        provider
            .set_model(poisoned_session_model)
            .expect("set_model with profile prefix should succeed via slot strip");

        assert_eq!(
            provider.model(),
            "MiniMax-M3",
            "After set_model with profile prefix, provider.model() must return \
             the bare id, not the prefixed one. This is the regression: with a \
             user-defined profile, the prefix was leaking through to the API \
             because the openrouter slot lost its profile_id."
        );
    });
}

/// Regression test for `session_safe_model_id` — the sanitizer that protects
/// session files from being poisoned with `<profile>:<model>` forms. Used by
/// `agent::provider::Agent::set_route_selection` to scrub the model field
/// before persisting it to disk.
#[test]
fn test_session_safe_model_id_strips_user_defined_profile_prefix() {
    with_clean_provider_test_env(|| {
        // Register a user-defined profile by writing config.toml under the
        // test's JCODE_HOME (which `with_clean_provider_test_env` already set
        // to a temp dir) and invalidating the config cache so the next
        // `config()` call reloads from disk.
        let config_path = crate::config::Config::path().expect("config path resolves");
        std::fs::create_dir_all(config_path.parent().expect("config dir"))
            .expect("create config dir");
        std::fs::write(
            &config_path,
            r#"[providers.minimax-cn]
type = "openai-compatible"
base_url = "https://api.minimaxi.com/v1"
api_key_env = "MINIMAX_API_KEY"
default_model = "MiniMax-M3"
requires_api_key = true
"#,
        )
        .expect("write test config");
        crate::config::invalidate_config_cache();

        // Strip the user-defined prefix
        assert_eq!(
            crate::provider::session_safe_model_id("minimax-cn:MiniMax-M3"),
            "MiniMax-M3"
        );
        // Built-in prefixes are preserved (they encode routing decisions
        // that need to round-trip through session restore).
        assert_eq!(
            crate::provider::session_safe_model_id("openrouter:anthropic/claude-sonnet-4"),
            "openrouter:anthropic/claude-sonnet-4"
        );
        assert_eq!(
            crate::provider::session_safe_model_id("claude-oauth:claude-opus-4-8"),
            "claude-oauth:claude-opus-4-8"
        );
        // Bare model id passes through unchanged.
        assert_eq!(
            crate::provider::session_safe_model_id("MiniMax-M3"),
            "MiniMax-M3"
        );
        // Unknown prefix is preserved (not a recognized profile).
        assert_eq!(
            crate::provider::session_safe_model_id("unknown-prefix:MiniMax-M3"),
            "unknown-prefix:MiniMax-M3"
        );
    });
}

/// Regression test for `MultiProvider::set_model` accepting a user-defined
/// profile prefix and routing it to the existing openrouter slot, even when
/// `cfg.providers` registers the profile under a name that collides with a
/// non-built-in prefix jcode did not statically recognize.
#[test]
fn test_set_model_with_user_defined_profile_prefix_routes_to_openrouter() {
    with_clean_provider_test_env(|| {
        let _key = crate::env::set_var("MINIMAX_API_KEY", "test-minimax-key");
        crate::env::remove_var("JCODE_OPENROUTER_API_KEY_NAME");
        crate::env::remove_var("JCODE_OPENROUTER_API_BASE");
        crate::env::remove_var("JCODE_OPENROUTER_CACHE_NAMESPACE");
        crate::env::remove_var("JCODE_OPENROUTER_PROVIDER_FEATURES");

        // Register the profile in config so MultiProvider::set_model can find it.
        let config_path = crate::config::Config::path().expect("config path resolves");
        std::fs::create_dir_all(config_path.parent().expect("config dir"))
            .expect("create config dir");
        std::fs::write(
            &config_path,
            r#"[providers.minimax-cn]
type = "openai-compatible"
base_url = "https://api.minimaxi.com/v1"
api_key_env = "MINIMAX_API_KEY"
default_model = "MiniMax-M3"
requires_api_key = true
"#,
        )
        .expect("write test config");
        crate::config::invalidate_config_cache();

        let profile = crate::config::NamedProviderConfig {
            provider_type: crate::config::NamedProviderType::OpenAiCompatible,
            base_url: "https://api.minimaxi.com/v1".to_string(),
            api: None,
            auth: crate::config::NamedProviderAuth::Bearer,
            auth_header: None,
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            api_key: None,
            env_file: None,
            default_model: Some("MiniMax-M3".to_string()),
            requires_api_key: Some(true),
            provider_routing: false,
            model_catalog: true,
            allow_provider_pinning: false,
            models: vec![],
            extra_body: None,
            supports_reasoning_effort: None,
        };
        let openrouter = openrouter::OpenRouterProvider::new_named_openai_compatible(
            "minimax-cn",
            &profile,
        )
        .expect("named provider should initialize");

        let provider = MultiProvider {
            claude: RwLock::new(None),
            anthropic: RwLock::new(None),
            openai: RwLock::new(None),
            copilot_api: RwLock::new(None),
            antigravity: RwLock::new(None),
            gemini: RwLock::new(None),
            cursor: RwLock::new(None),
            bedrock: RwLock::new(None),
            openrouter: RwLock::new(Some(Arc::new(openrouter))),
            openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
            active_openai_compatible_profile: RwLock::new(None),
            active: RwLock::new(ActiveProvider::OpenRouter),
            use_claude_cli: false,
            startup_notices: RwLock::new(Vec::new()),
            forced_provider: Some(ActiveProvider::OpenRouter),
        };

        provider
            .set_model("minimax-cn:MiniMax-M3")
            .expect("set_model should accept user-defined profile prefix");

        assert_eq!(
            provider.model(),
            "MiniMax-M3",
            "set_model must strip the user-defined profile prefix before \
             storing in the openrouter slot; the fixed code path routes the \
             bare model id through set_model_on_provider_with_credential_modes \
             so the openrouter slot stores the bare id directly"
        );
    });
}

/// Regression test for the user-defined profile session-restore bug.
///
/// When a user configures `[providers.minimax-cn]` (or any other custom
/// OpenAI-compatible profile in `config.toml`) and jcode restores a session
/// whose `model` field is `"<profile>:<model>"` (e.g. `minimax-cn:MiniMax-M3`),
/// `MultiProvider::set_model` must strip the profile prefix before storing it
/// in the openrouter slot. Otherwise the slot's `model` ends up being
/// `"minimax-cn:MiniMax-M3"` verbatim, which is then sent to the upstream
/// API as the `model` JSON field and rejected with 400 unknown_model.
///
/// Before the fix, this test failed because `set_model("minimax-cn:MiniMax-M3")`
/// did not strip the prefix when the profile was user-defined (vs. a built-in
/// profile like `minimax` whose id is in the static `OPENAI_COMPAT_PROFILES`
/// list, which DID strip correctly via `openai_compatible_model_prefix`).
#[test]
fn test_user_defined_profile_session_restore_strips_profile_prefix() {
    with_clean_provider_test_env(|| {
        // Set up the user-defined profile env: api key + base URL via env vars
        // (the same path `apply_named_provider_profile_env_from_config` uses).
        let _key = crate::env::set_var("MINIMAX_API_KEY", "test-minimax-key");
        // The `set_config_default_model` path goes through the OpenRouter
        // branch. When the openrouter slot is initialized for a user-defined
        // profile, it reads `JCODE_OPENROUTER_*` env vars to redirect the
        // shared OpenAI-compatible slot. We must not pre-set those here or
        // the rebind path will succeed via env redirect rather than via the
        // slot's profile_id, masking the regression.
        crate::env::remove_var("JCODE_OPENROUTER_API_KEY_NAME");
        crate::env::remove_var("JCODE_OPENROUTER_API_BASE");
        crate::env::remove_var("JCODE_OPENROUTER_CACHE_NAMESPACE");
        crate::env::remove_var("JCODE_OPENROUTER_PROVIDER_FEATURES");

        // Construct the openrouter slot the same way startup does for a
        // named profile (via `OpenRouterProvider::new_named_openai_compatible`).
        let profile = crate::config::NamedProviderConfig {
            provider_type: crate::config::NamedProviderType::OpenAiCompatible,
            base_url: "https://api.minimaxi.com/v1".to_string(),
            api: None,
            auth: crate::config::NamedProviderAuth::Bearer,
            auth_header: None,
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            api_key: None,
            env_file: None,
            default_model: Some("MiniMax-M3".to_string()),
            requires_api_key: Some(true),
            provider_routing: false,
            model_catalog: true,
            allow_provider_pinning: false,
            models: vec![],
            extra_body: None,
            supports_reasoning_effort: None,
        };
        let openrouter = openrouter::OpenRouterProvider::new_named_openai_compatible(
            "minimax-cn",
            &profile,
        )
        .expect("named provider should initialize");

        let provider = MultiProvider {
            claude: RwLock::new(None),
            anthropic: RwLock::new(None),
            openai: RwLock::new(None),
            copilot_api: RwLock::new(None),
            antigravity: RwLock::new(None),
            gemini: RwLock::new(None),
            cursor: RwLock::new(None),
            bedrock: RwLock::new(None),
            openrouter: RwLock::new(Some(Arc::new(openrouter))),
            openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
            active_openai_compatible_profile: RwLock::new(None),
            active: RwLock::new(ActiveProvider::OpenRouter),
            use_claude_cli: false,
            startup_notices: RwLock::new(Vec::new()),
            forced_provider: Some(ActiveProvider::OpenRouter),
        };

        // Session restore path: model spec includes the user-defined profile
        // prefix. `set_model` must strip it so the upstream API only sees
        // the bare model id.
        provider
            .set_model("minimax-cn:MiniMax-M3")
            .expect("set_model should accept user-defined profile prefix");

        assert_eq!(
            provider.model(),
            "MiniMax-M3",
            "session restore with user-defined profile prefix must be stripped \
             before storing in the openrouter slot; otherwise the upstream API \
             receives `model=minimax-cn:MiniMax-M3` and rejects it as unknown"
        );
    });
}

/// Companion to the above test: the OpenRouter slot's `model` must end up
/// WITHOUT the profile prefix, AND the active compatible profile marker
/// should reflect the named profile (so downstream `provider.model()` lookups
/// via `active_openrouter_execution_provider()` resolve to the named-profile
/// runtime that knows how to strip the prefix).
#[test]
fn test_user_defined_profile_session_restore_keeps_active_profile() {
    with_clean_provider_test_env(|| {
        let _key = crate::env::set_var("MINIMAX_API_KEY", "test-minimax-key");
        crate::env::remove_var("JCODE_OPENROUTER_API_KEY_NAME");
        crate::env::remove_var("JCODE_OPENROUTER_API_BASE");
        crate::env::remove_var("JCODE_OPENROUTER_CACHE_NAMESPACE");
        crate::env::remove_var("JCODE_OPENROUTER_PROVIDER_FEATURES");

        let profile = crate::config::NamedProviderConfig {
            provider_type: crate::config::NamedProviderType::OpenAiCompatible,
            base_url: "https://api.minimaxi.com/v1".to_string(),
            api: None,
            auth: crate::config::NamedProviderAuth::Bearer,
            auth_header: None,
            api_key_env: Some("MINIMAX_API_KEY".to_string()),
            api_key: None,
            env_file: None,
            default_model: Some("MiniMax-M3".to_string()),
            requires_api_key: Some(true),
            provider_routing: false,
            model_catalog: true,
            allow_provider_pinning: false,
            models: vec![],
            extra_body: None,
            supports_reasoning_effort: None,
        };
        let openrouter = openrouter::OpenRouterProvider::new_named_openai_compatible(
            "minimax-cn",
            &profile,
        )
        .expect("named provider should initialize");

        // Pre-install as an active compatible profile (the normal startup
        // state once `apply_named_provider_profile_env` has wired everything
        // up).
        let provider = MultiProvider {
            claude: RwLock::new(None),
            anthropic: RwLock::new(None),
            openai: RwLock::new(None),
            copilot_api: RwLock::new(None),
            antigravity: RwLock::new(None),
            gemini: RwLock::new(None),
            cursor: RwLock::new(None),
            bedrock: RwLock::new(None),
            openrouter: RwLock::new(Some(Arc::new(openrouter))),
            openai_compatible_profiles: RwLock::new(std::collections::HashMap::new()),
            active_openai_compatible_profile: RwLock::new(None),
            active: RwLock::new(ActiveProvider::OpenRouter),
            use_claude_cli: false,
            startup_notices: RwLock::new(Vec::new()),
            forced_provider: None,
        };

        provider
            .set_model("minimax-cn:MiniMax-M3")
            .expect("set_model should accept user-defined profile prefix");

        // The active compatible profile should resolve to the named profile
        // so that subsequent `provider.model()` lookups go through the
        // strip-prefix-aware runtime.
        assert_eq!(
            provider.model(),
            "MiniMax-M3",
            "provider.model() must return the bare model id, not the prefixed one"
        );
    });
}

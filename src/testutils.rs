use crate::Cache;
use tempfile::{tempdir, TempDir};

/// Creates a Cache instance in a randomly-named temporary directory so
/// that subsequent test runs are isolated from one another
pub fn create_test_cache() -> (Cache, TempDir) {
    let binding = tempdir().expect("Failed to create temp dir");
    let temp_dir = binding.path();
    let cache = Cache::builder()
        .with_data_dir(temp_dir)
        .build()
        .expect("Failed to create test cache");
    (cache, binding)
}

/// Points Firefox profile discovery at the bundled test fixtures in
/// `test_data/firefox_profile`, allowing `firefox::Browser::new()` to resolve
/// a profile without a real Firefox installation.
///
/// This sets the `TEST_FIREFOX_PROFILE_DIR` environment variable exactly once
/// (via [`std::sync::Once`]) and never unsets it. Doing so keeps the test
/// suite deterministic regardless of test ordering or thread count, and lets
/// both the unit tests and the integration test binaries run via plain
/// `cargo test` with no external configuration.
pub fn init_firefox_test_env() {
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("test_data/firefox_profile");
        std::env::set_var("TEST_FIREFOX_PROFILE_DIR", dir);
    });
}

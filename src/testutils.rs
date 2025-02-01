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

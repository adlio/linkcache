use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use linkcache::*;
use tempfile::*;

/// Creates a Cache instance in a randomly-named temporary directory so
/// that subsequent test runs are isolated from one another
fn test_cache_instance() -> (Cache, TempDir) {
    let binding = tempdir().expect("Failed to create temp dir");
    let dir = binding.path();
    std::fs::set_permissions(binding.path(), Permissions::from_mode(0o755))
        .expect("Failed to set directory permissions");
    let file_path = dir.join("test.sqlite");
    println!("Using cache file: {:?}", file_path);
    let cache = Cache::new(file_path).expect("Failed to create test cache");
    (cache, binding)
}

#[test]
fn test_indexing_chrome_bookmarks() -> Result<()> {
    let (mut cache, _temp_dir) = test_cache_instance();

    cache.add(Link {
        title: "Visual Studio Code".to_string(),
        url: "https://code.visualstudio.com".to_string(),
        ..Default::default()
    })?;
    cache.add(Link {
        title: "Sublime Text".to_string(),
        url: "https://www.sublimetext.com".to_string(),
        ..Default::default()
    })?;
    let results = &cache.search("VS Code").expect("Search failed");
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Visual Studio Code");

    // TODO This browser location should be a custom path with a fixed set of
    // data in the Bookmarks file for predictable testing.
    let browser = chrome::Browser::new().expect("Failed to instantiate browser");
    browser
        .cache_bookmarks(&mut cache)
        .expect("Failed to cache bookmarks");
    Ok(())
}

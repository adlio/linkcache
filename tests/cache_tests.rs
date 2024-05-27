use linkcache::*;
use tempfile::*;

/// Creates a Cache instance in a randomly-named temporary directory so
/// that subsequent test runs are isolated from one another
fn test_cache_instance() -> Cache {
    let binding = tempdir().expect("Failed to create temp dir");
    CacheBuilder::new()
        .with_root_dir(binding.into_path())
        .build()
        .expect("Failed to build cache instance")
}

#[test]
fn test_indexing_chrome_bookmarks() {
    let mut cache = test_cache_instance();

    assert!(cache
        .add(Link {
            title: "Visual Studio Code".to_string(),
            url: "https://code.visualstudio.com".to_string(),
            ..Default::default()
        })
        .is_ok());
    assert!(cache
        .add(Link {
            title: "Sublime Text".to_string(),
            url: "https://www.sublimetext.com".to_string(),
            ..Default::default()
        })
        .is_ok());
    cache.commit().expect("Failed to commit");
    let results = &cache.search("Viz Studio").expect("Search failed");
    assert!(!results.is_empty());
    assert_eq!(results[0].title, "Visual Studio Code");

    // TODO This browser location should be a custom path with a fixed set of
    // data in the Bookmarks file for predictable testing.
    let browser = chrome::Browser::new().expect("Failed to instantiate browser");
    browser
        .cache_bookmarks(&mut cache)
        .expect("Failed to cache bookmarks");
}

use linkcache::chrome;
use linkcache::testutils::create_test_cache;
use linkcache::{Link, Result};

#[test]
fn test_indexing_chrome_bookmarks() -> Result<()> {
    let (mut cache, _temp_dir) = create_test_cache();

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

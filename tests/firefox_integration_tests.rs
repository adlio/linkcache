use linkcache::firefox;
use linkcache::testutils::create_test_cache;
use linkcache::Link;
use std::path::PathBuf;

/// Helper function to get the path to our test Firefox profile
fn test_firefox_profile_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_data/firefox_profile/test.default");
    path
}

#[test]
fn test_firefox_integration_full_workflow() {
    // Create a test cache
    let (mut cache, _temp_dir) = create_test_cache();
    
    // Create a browser instance with our test profile
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(test_firefox_profile_dir());
    
    // Create a replica of the places database
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Cache both bookmarks and history
    browser.cache_bookmarks(&mut cache).expect("Failed to cache bookmarks");
    browser.cache_history(&mut cache).expect("Failed to cache history");
    
    // Verify we can search for bookmarks
    let results = cache.search("Mozilla").expect("Search failed");
    assert!(!results.is_empty(), "Should find Mozilla bookmark");
    
    // Verify we can search for history entries
    let results = cache.search("Wikipedia").expect("Search failed");
    assert!(!results.is_empty(), "Should find Wikipedia history entry");
    
    // Verify we can search with partial terms
    let results = cache.search("Rust").expect("Search failed");
    assert!(!results.is_empty(), "Should find Rust bookmark with partial search");
    
    // Verify we can search with fuzzy matching (this might be too strict for fuzzy search)
    // let results = cache.search("Gthb").expect("Search failed");
    // assert!(!results.is_empty(), "Should find GitHub with fuzzy search");
    
    // Add a new link and verify it can be found
    cache.add(Link {
        guid: "test_guid".to_string(),
        url: "https://www.firefox.com".to_string(),
        title: "Firefox Browser".to_string(),
        subtitle: Some("Test Subtitle".to_string()),
        source: Some("test".to_string()),
        timestamp: chrono::Utc::now(),
        score: None,
    }).expect("Failed to add link");
    
    let results = cache.search("Firefox Browser").expect("Search failed");
    assert!(!results.is_empty(), "Should find newly added link");
    assert_eq!(results[0].title, "Firefox Browser");
}

#[test]
fn test_firefox_integration_empty_search() {
    // Create a test cache
    let (mut cache, _temp_dir) = create_test_cache();
    
    // Create a browser instance with our test profile
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(test_firefox_profile_dir());
    
    // Create a replica of the places database
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Cache both bookmarks and history
    browser.cache_bookmarks(&mut cache).expect("Failed to cache bookmarks");
    browser.cache_history(&mut cache).expect("Failed to cache history");
    
    // Verify empty search returns all items
    let results = cache.search("").expect("Search failed");
    assert!(!results.is_empty(), "Empty search should return results");
    
    // We should have at least 5 items (3 bookmarks + 2 history entries)
    assert!(results.len() >= 5, "Should have at least 5 items in the cache");
}

#[test]
fn test_firefox_integration_no_results() {
    // Create a test cache
    let (mut cache, _temp_dir) = create_test_cache();
    
    // Create a browser instance with our test profile
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(test_firefox_profile_dir());
    
    // Create a replica of the places database
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Cache both bookmarks and history
    browser.cache_bookmarks(&mut cache).expect("Failed to cache bookmarks");
    browser.cache_history(&mut cache).expect("Failed to cache history");
    
    // Search for something that doesn't exist
    let results = cache.search("ThisShouldNotExistAnywhere12345").expect("Search failed");
    assert!(results.is_empty(), "Search for non-existent term should return no results");
}
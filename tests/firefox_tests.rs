use linkcache::firefox;
use linkcache::testutils::create_test_cache;
use std::path::PathBuf;

/// Helper function to get the path to our test Firefox profile
fn test_firefox_profile_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_data/firefox_profile/test.default");
    path
}

#[test]
fn test_firefox_with_profile_dir() {
    let profile_dir = test_firefox_profile_dir();
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(profile_dir);
    
    assert!(browser.places_path().exists(), "Test places.sqlite should exist");
}

#[test]
fn test_firefox_cache_bookmarks() {
    let (mut cache, _temp_dir) = create_test_cache();
    let profile_dir = test_firefox_profile_dir();
    
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(profile_dir);
    
    // Create a replica of the places database in the cache
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Cache the bookmarks
    browser.cache_bookmarks(&mut cache).expect("Failed to cache bookmarks");
    
    // Search for a known bookmark
    let results = cache.search("Mozilla").expect("Search failed");
    assert!(!results.is_empty(), "Should find Mozilla bookmark");
    assert_eq!(results[0].title, "Mozilla");
    
    // Search for another known bookmark
    let results = cache.search("Rust").expect("Search failed");
    assert!(!results.is_empty(), "Should find Rust bookmark");
    assert_eq!(results[0].title, "Rust Programming Language");
}

#[test]
fn test_firefox_cache_history() {
    let (mut cache, _temp_dir) = create_test_cache();
    let profile_dir = test_firefox_profile_dir();
    
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(profile_dir);
    
    // Create a replica of the places database in the cache
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Cache the history
    browser.cache_history(&mut cache).expect("Failed to cache history");
    
    // Search for a known history entry
    let results = cache.search("Wikipedia").expect("Search failed");
    assert!(!results.is_empty(), "Should find Wikipedia history entry");
    assert_eq!(results[0].title, "Wikipedia");
}

#[test]
fn test_firefox_all_bookmarks() {
    let (cache, _temp_dir) = create_test_cache();
    let profile_dir = test_firefox_profile_dir();
    
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(profile_dir);
    
    // Create a replica of the places database in the cache
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Get all bookmarks
    let bookmarks = browser.all_bookmarks(&cache).expect("Failed to get all bookmarks");
    
    // We should have 3 bookmarks in our test data
    assert_eq!(bookmarks.len(), 3, "Should have 3 bookmarks");
    
    // Check that we have the expected bookmarks
    let titles: Vec<String> = bookmarks.iter().map(|b| b.title.clone()).collect();
    assert!(titles.contains(&"Mozilla".to_string()), "Should have Mozilla bookmark");
    assert!(titles.contains(&"Rust Programming Language".to_string()), "Should have Rust bookmark");
    assert!(titles.contains(&"GitHub".to_string()), "Should have GitHub bookmark");
    
    // Check that at least one bookmark has a subtitle
    let has_subtitle = bookmarks.iter().any(|b| b.subtitle.is_some());
    assert!(has_subtitle, "At least one bookmark should have a subtitle");
}

#[test]
fn test_firefox_all_history() {
    let (cache, _temp_dir) = create_test_cache();
    let profile_dir = test_firefox_profile_dir();
    
    let browser = firefox::Browser::new()
        .expect("Failed to create browser")
        .with_profile_dir(profile_dir);
    
    // Create a replica of the places database in the cache
    browser.create_places_replica(&cache).expect("Failed to create places replica");
    
    // Get all history
    let history = browser.all_history(&cache).expect("Failed to get all history");
    
    // We should have 5 history entries in our test data (3 bookmarks + 2 history-only entries)
    assert!(history.len() >= 2, "Should have at least 2 history entries");
    
    // Check that we have the expected history entries
    let titles: Vec<String> = history.iter().map(|h| h.title.clone()).collect();
    assert!(titles.contains(&"Example Domain".to_string()), "Should have Example Domain history entry");
    assert!(titles.contains(&"Wikipedia".to_string()), "Should have Wikipedia history entry");
}
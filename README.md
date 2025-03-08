# Linkcache

A utility for caching and searching browser links (bookmarks and history) from different browsers.

## Features

- Cache browser bookmarks and history from Firefox, Chrome, and Arc browsers
- Search across all cached links with fuzzy matching
- Integration with Alfred workflows for quick access to browser history and bookmarks

## Testing

The project includes comprehensive tests for Firefox functionality:

### Unit Tests

Located in `src/firefox.rs` under the `tests` module:
- Tests for core Firefox functionality
- Tests for profile directory detection
- Tests for places database operations

### Integration Tests

Located in `tests/firefox_tests.rs`:
- Tests for Firefox profile handling
- Tests for caching bookmarks and history
- Tests for searching cached links

### End-to-End Tests

Located in `tests/firefox_integration_tests.rs`:
- Tests for the full workflow from Firefox data extraction to searching
- Tests for empty searches and no results

## Running Tests

To run all tests:

```bash
cargo test
```

To run only Firefox-related tests:

```bash
cargo test firefox
```

## Test Data

The project includes mock Firefox profile data in `test_data/firefox_profile/` for testing purposes. This allows testing without requiring an actual Firefox installation.
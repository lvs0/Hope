// Integration tests for hope-hal
// Run with: cargo test --test '*'

// Note: The actual unit tests are in src/lib.rs
// This file is for integration tests that require compilation
// of the full library.

#[test]
fn hal_version() {
    // Verify version is accessible
    let version = env!("CARGO_PKG_VERSION");
    assert!(version.starts_with("0.1"));
}

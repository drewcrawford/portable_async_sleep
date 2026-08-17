# Changelog

All notable changes to this project are documented here.

## Unreleased

### Fixed

- Cancelled native sleeps now leave the scheduler promptly instead of holding
  memory until their original deadlines.
- Extremely long native sleeps no longer panic when their duration exceeds the
  platform's `Instant` range.
- WebAssembly sleeps now round fractional milliseconds up, so short sleeps do
  not finish early.
- Long WebAssembly durations no longer wrap through JavaScript's signed 32-bit
  timeout limit and fire immediately.
- WebAssembly timeout callbacks now clean up after themselves instead of being
  permanently forgotten. Tiny timers, tidy heap.
- The test suite no longer depends on a transitive dependency combination that
  fails to compile.

### Changed

- The native scheduler now uses indexed, ordered timer storage, making timer
  insertion and cancellation scale more gracefully under load.
- Unused WebAssembly runtime dependencies were removed, and minimum dependency
  versions now account for Rust 2024 edition support.
- The public API is unchanged: `async_sleep(Duration)` remains the whole story.

## 0.1.1 - 2025-08-28

### Added

- Added browser WebAssembly support while keeping the returned future `Send`.
- Expanded concurrent-sleep coverage, documentation, and CI checks.

## 0.1.0 - 2025-06-08

### Added

- Initial release of runtime-independent asynchronous sleep for native targets.

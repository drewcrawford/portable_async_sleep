# Investigation

## Original problem

Find and fix correctness and security bugs in the `portable_async_sleep` crate.

## Changes made

* Replaced the native `Vec`/full-sort timer queue with indexed `BTreeMap` and
  `HashMap` storage.
* Added cancellation notifications so dropped futures promptly remove their
  native timers and release their continuation allocations.
* Split extremely long native sleeps into representable `Instant` chunks,
  preventing `Instant::now() + duration` overflow panics.
* Rounded fractional WASM milliseconds upward and split long JavaScript
  timeouts, preventing early completion and signed `i32` wraparound.
* Replaced permanently forgotten WASM closures with self-deallocating one-shot
  callbacks; bounded chunks let callbacks from cancelled futures run promptly.
* Removed unused runtime dependencies and the broken `test_executors`/
  `wasm_thread` test stack, replacing it with native `futures` and direct
  `wasm-bindgen-test` coverage.
* Raised WASM dependency minima to the first tested edition-2024-compatible
  release family.

## Verified

* Native unit tests pass on the current toolchain and the declared Rust 1.85.1
  MSRV (5 tests each).
* Browser WASM tests pass in headless Firefox (10 tests), including `Send`,
  cancellation, sub-millisecond, and long-duration regression coverage.
* Native and WASM Clippy passes with all warnings denied.
* All five doctests pass and the publish file list excludes this investigation
  log.
* Both native and WASM library builds pass with Cargo's minimum-version
  resolver; WASM also passes with current compatible dependencies.
* Direct OSV queries returned no advisories for the four resolved runtime
  dependencies (`continue`, `atomic-waker`, `wasm-bindgen`, and `js-sys`).

## Ideas

No unresolved investigation ideas remain.

## Steps taken

1. Inspected the complete crate, CI configuration, manifest, platform backends,
   and recent history.
2. Ran the baseline suite and reproduced a compile failure in a transitive
   dev-dependency.
3. Built and read the `continue` and `wasm-bindgen` dependency documentation to
   verify cancellation, sender-drop, and closure-lifetime behavior.
4. Audited native timer insertion, expiration, cancellation, and overflow
   paths; implemented indexed scheduling and cancellation-aware cleanup.
5. Audited WASM duration conversion and callback ownership; implemented
   ceiling conversion, bounded chunks, and one-shot callback reclamation.
6. Reworked tests and dependency scopes, then ran native, WASM browser, Clippy,
   rustdoc, MSRV, package-content, and minimum-version checks.
7. Queried the OSV vulnerability database for every resolved runtime
   dependency.

## Notes

`cargo-audit` was not installed, so the advisory check used OSV's API directly.

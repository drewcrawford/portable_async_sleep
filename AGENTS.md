# portable_async_sleep

## Development commands

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps
```

The crate supports native Rust targets and `wasm32-unknown-unknown`. Keep the
public API portable and preserve the minimum Rust version declared in
`Cargo.toml` when changing dependencies or implementation details.

The crate-level documentation in `src/lib.rs` is the source for `README.md`.
When changing that documentation, update the README with the same content and
use the repository-relative asset path only in the README.

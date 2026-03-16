# Aletheia — Verifiable Proof Layer for Coding Agents

## Project Structure
- **Rust core library**: `crates/aletheia-core/` — pure computation, zero IO
- **Rust CLI binary**: `crates/aletheia-cli/` — stdin, files, HTML export
- **Design specs**: `docs/superpowers/specs/`
- **CI**: `.github/workflows/ci.yml`

## Build & Test
- `cargo test --all` — run all tests
- `cargo build --release` — production build
- `cargo clippy --all -- -D warnings` — lint (zero warnings policy)
- `cargo fmt --check` — format check

## Code Standards

### Rust
- No `unwrap()` in library code — propagate with `?`
- All public APIs have doc comments (`///`)
- Use `thiserror` for error types
- Cross-platform: `std::path::PathBuf` always, never hardcoded path separators
- Use `dirs::config_dir()` for platform-specific paths
- Byte arrays displayed as hex (`hex` crate)

### Cryptography
- SHA-256 only via `sha2` crate
- Ed25519 only via `ed25519-dalek` crate
- Hash chains: `prev_hash` of first entry = `[0u8; 32]`
- Merkle tree: odd leaf count → duplicate last leaf
- All serialized packs include version field
- Canonical JSON for hashing (deterministic)

### Testing
- Unit tests in each module (`#[cfg(test)]`)
- Integration tests in `tests/`
- Test on Windows + Linux + macOS in CI
- Crypto tests must include tamper-detection cases

## Key Invariants (Never Break)
1. Evidence packs serialize/deserialize without loss
2. Hash chains are append-only and immutable after creation
3. Verification is deterministic: same input → same result
4. CLI exit codes: 0=success, 1=verification fail, 2=usage error, 3=IO error

## Product Positioning
- This is a **proof layer**, not a governance platform
- Never pitch as "agent control plane" or "MCP governance"
- Always pitch as "verifiable proof", "signed receipts", "tamper-evident evidence"

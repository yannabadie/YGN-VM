---
paths:
  - "crates/**/*.rs"
---

# Rust Cryptographic Implementation Rules

- SHA-256 via `sha2` crate only — no custom hashing
- Ed25519 via `ed25519-dalek` only — no alternative signing libs
- No `unwrap()` on crypto operations — propagate errors
- All byte arrays (hashes, signatures, keys) use fixed-size arrays, not Vec<u8>
- Hex encoding via `hex` crate for display/serialization
- Test vectors must include tamper-detection (modify input, verify failure)
- Canonical JSON serialization for hashing (serde_json::to_vec)
- Cross-platform: no platform-specific crypto APIs

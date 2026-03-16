# Aletheia Proof MVP Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI that captures coding-agent events via stdin, chains them cryptographically, seals them into signed evidence packs, verifies integrity, and exports human-readable reports.

**Architecture:** Cargo workspace with 2 crates: `aletheia-core` (pure library, zero IO) and `aletheia-cli` (binary with clap, stdin, file IO). Core handles all crypto (SHA-256 hash chain, Merkle tree, Ed25519 signing/verification). CLI handles user interaction and file operations.

**Tech Stack:** Rust 1.85+, sha2, ed25519-dalek, serde/serde_json, clap, dirs, uuid, hex, thiserror

**Spec:** `docs/superpowers/specs/2026-03-16-aletheia-proof-mvp-design.md`

---

## File Structure

```
Cargo.toml                              # Workspace root
crates/
  aletheia-core/
    Cargo.toml
    src/
      lib.rs                            # Public re-exports
      error.rs                          # AletheiaError enum (thiserror)
      event.rs                          # Event, EventKind, EventContext structs
      chain.rs                          # HashChain: append events, link prev_hash
      merkle.rs                         # compute_merkle_root()
      signing.rs                        # Ed25519 keygen, sign, verify
      receipt.rs                        # Receipt struct, creation from Event
      pack.rs                           # EvidencePack, PackMetadata, PackSignature
      verify.rs                         # verify_pack() → VerificationResult
  aletheia-cli/
    Cargo.toml
    src/
      main.rs                           # Clap app, subcommand dispatch
      cmd_keygen.rs                     # aletheia keygen
      cmd_capture.rs                    # aletheia capture (stdin → session)
      cmd_seal.rs                       # aletheia seal (session → pack)
      cmd_verify.rs                     # aletheia verify (pack → result)
      cmd_export.rs                     # aletheia export (pack → html/json/md)
      paths.rs                          # Cross-platform path resolution
      templates/
        report.html                     # Embedded HTML template
tests/
  e2e.rs                                # End-to-end integration tests
.github/
  workflows/
    ci.yml                              # CI: fmt, clippy, test on 3 OSes
```

---

## Chunk 1: Workspace + Core Foundations

### Task 1: Initialize Cargo Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/aletheia-core/Cargo.toml`
- Create: `crates/aletheia-core/src/lib.rs`
- Create: `crates/aletheia-cli/Cargo.toml`
- Create: `crates/aletheia-cli/src/main.rs`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = ["crates/aletheia-core", "crates/aletheia-cli"]
```

- [ ] **Step 2: Create aletheia-core Cargo.toml**

```toml
[package]
name = "aletheia-core"
version = "0.1.0"
edition = "2021"
description = "Cryptographic evidence packs for coding-agent compliance"
license = "MIT"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
ed25519-dalek = { version = "2", features = ["rand_core"] }
rand = "0.8"
uuid = { version = "1", features = ["v7"] }
hex = "0.4"
thiserror = "2"
```

- [ ] **Step 3: Create aletheia-core/src/lib.rs (stub)**

```rust
pub mod error;
pub mod event;
```

- [ ] **Step 4: Create aletheia-cli Cargo.toml**

```toml
[package]
name = "aletheia-cli"
version = "0.1.0"
edition = "2021"
description = "CLI for Aletheia cryptographic evidence packs"
license = "MIT"

[[bin]]
name = "aletheia"
path = "src/main.rs"

[dependencies]
aletheia-core = { path = "../aletheia-core" }
clap = { version = "4", features = ["derive"] }
dirs = "6"
chrono = { version = "0.4", features = ["serde"] }
tokio = { version = "1", features = ["full"] }
serde_json = "1"
hex = "0.4"
```

- [ ] **Step 5: Create aletheia-cli/src/main.rs (stub)**

```rust
fn main() {
    println!("aletheia v0.1.0");
}
```

- [ ] **Step 6: Verify workspace compiles**

Run: `cargo build --workspace`
Expected: Compiles successfully with no errors.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml crates/
git commit -m "feat: initialize Cargo workspace with core and cli crates"
```

---

### Task 2: Error Types

**Files:**
- Create: `crates/aletheia-core/src/error.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write error.rs**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AletheiaError {
    #[error("hash chain broken at receipt {index}: expected {expected}, got {actual}")]
    BrokenChain {
        index: u64,
        expected: String,
        actual: String,
    },

    #[error("merkle root mismatch: expected {expected}, got {actual}")]
    MerkleRootMismatch { expected: String, actual: String },

    #[error("chain head mismatch: expected {expected}, got {actual}")]
    ChainHeadMismatch { expected: String, actual: String },

    #[error("event hash mismatch at receipt {index}: expected {expected}, got {actual}")]
    EventHashMismatch {
        index: u64,
        expected: String,
        actual: String,
    },

    #[error("invalid signature from signer {signer}")]
    InvalidSignature { signer: String },

    #[error("signing error: {0}")]
    SigningError(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("empty evidence pack")]
    EmptyPack,
}

pub type Result<T> = std::result::Result<T, AletheiaError>;
```

- [ ] **Step 2: Verify compiles**

Run: `cargo build -p aletheia-core`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-core/src/error.rs
git commit -m "feat(core): add AletheiaError types"
```

---

### Task 3: Event Module

**Files:**
- Create: `crates/aletheia-core/src/event.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing test**

Add to `crates/aletheia-core/src/event.rs`:

```rust
use serde::{Deserialize, Serialize};

// Structs will go here

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_serialization_roundtrip() {
        let event = Event::new(
            EventKind::ToolUse,
            "claude-code".to_string(),
            EventContext::new("session-1".to_string()),
            serde_json::json!({"command": "cargo test"}),
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: Event = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.source, deserialized.source);
        assert_eq!(event.payload, deserialized.payload);
    }

    #[test]
    fn event_from_json_line_valid() {
        let line = r#"{"kind":"shell_exec","source":"ci","payload":{"cmd":"make"}}"#;
        let event = Event::from_json_line(line, "test-session").unwrap();
        assert_eq!(event.source, "ci");
        assert!(matches!(event.kind, EventKind::ShellExec));
    }

    #[test]
    fn event_from_json_line_minimal() {
        let line = r#"{"payload":{"text":"hello"}}"#;
        let event = Event::from_json_line(line, "test-session").unwrap();
        assert!(matches!(event.kind, EventKind::Custom));
        assert_eq!(event.source, "manual");
    }

    #[test]
    fn event_from_plain_text() {
        let event = Event::from_plain_text("some log line", "test-session");
        assert!(matches!(event.kind, EventKind::Custom));
        assert_eq!(event.payload, serde_json::json!({"text": "some log line"}));
    }

    #[test]
    fn event_context_with_all_fields() {
        let ctx = EventContext {
            session_id: "s1".to_string(),
            repo: Some("myrepo".to_string()),
            branch: Some("main".to_string()),
            pr_number: Some(42),
            tool: Some("claude-code".to_string()),
            policy: Some("default".to_string()),
            result: Some("success".to_string()),
        };
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(json.contains("myrepo"));
        assert!(json.contains("policy"));
    }

    #[test]
    fn event_context_skips_none_fields() {
        let ctx = EventContext::new("s1".to_string());
        let json = serde_json::to_string(&ctx).unwrap();
        assert!(!json.contains("repo"));
        assert!(!json.contains("policy"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p aletheia-core -- event`
Expected: FAIL — `Event` struct not defined yet.

- [ ] **Step 3: Write the implementation**

Complete `crates/aletheia-core/src/event.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub timestamp: u64,
    pub kind: EventKind,
    pub source: String,
    pub context: EventContext,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ToolUse,
    FileEdit,
    ShellExec,
    PrAction,
    TestRun,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventContext {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
}

impl EventContext {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            repo: None,
            branch: None,
            pr_number: None,
            tool: None,
            policy: None,
            result: None,
        }
    }
}

impl Event {
    pub fn new(
        kind: EventKind,
        source: String,
        context: EventContext,
        payload: serde_json::Value,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            id: Uuid::now_v7().to_string(),
            timestamp: now,
            kind,
            source,
            context,
            payload,
        }
    }

    /// Parse a JSON line into an Event. Missing fields get defaults.
    pub fn from_json_line(line: &str, session_id: &str) -> std::result::Result<Self, serde_json::Error> {
        let line = line.trim_end_matches('\r');

        #[derive(Deserialize)]
        struct PartialEvent {
            #[serde(default)]
            id: Option<String>,
            #[serde(default)]
            timestamp: Option<u64>,
            #[serde(default = "default_kind")]
            kind: EventKind,
            #[serde(default = "default_source")]
            source: String,
            #[serde(default)]
            context: Option<EventContext>,
            pub payload: serde_json::Value,
        }

        fn default_kind() -> EventKind {
            EventKind::Custom
        }
        fn default_source() -> String {
            "manual".to_string()
        }

        let partial: PartialEvent = serde_json::from_str(line)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Ok(Event {
            id: partial.id.unwrap_or_else(|| Uuid::now_v7().to_string()),
            timestamp: partial.timestamp.unwrap_or(now),
            kind: partial.kind,
            source: partial.source,
            context: partial.context.unwrap_or_else(|| EventContext::new(session_id.to_string())),
            payload: partial.payload,
        })
    }

    /// Wrap plain text as a Custom event.
    pub fn from_plain_text(text: &str, session_id: &str) -> Self {
        let text = text.trim_end_matches('\r');
        Event::new(
            EventKind::Custom,
            "manual".to_string(),
            EventContext::new(session_id.to_string()),
            serde_json::json!({"text": text}),
        )
    }
}
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod error;
pub mod event;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p aletheia-core -- event`
Expected: All 6 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-core/src/event.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add Event, EventKind, EventContext with JSONL parsing"
```

---

## Chunk 2: Core Crypto — Chain, Merkle, Signing

### Task 4: Hash Chain

**Files:**
- Create: `crates/aletheia-core/src/chain.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod chain;`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, EventContext, EventKind};

    fn make_event(msg: &str) -> Event {
        Event::new(
            EventKind::Custom,
            "test".to_string(),
            EventContext::new("test-session".to_string()),
            serde_json::json!({"msg": msg}),
        )
    }

    #[test]
    fn empty_chain() {
        let chain = HashChain::new();
        assert_eq!(chain.len(), 0);
        assert_eq!(chain.head(), [0u8; 32]);
    }

    #[test]
    fn single_event_chain() {
        let mut chain = HashChain::new();
        let receipt = chain.append(make_event("first"));
        assert_eq!(receipt.sequence, 0);
        assert_eq!(receipt.prev_hash, [0u8; 32]);
        assert_ne!(receipt.hash, [0u8; 32]);
    }

    #[test]
    fn chain_links_prev_hash() {
        let mut chain = HashChain::new();
        let r0 = chain.append(make_event("first"));
        let r1 = chain.append(make_event("second"));
        assert_eq!(r1.prev_hash, r0.hash);
        assert_eq!(chain.head(), r1.hash);
    }

    #[test]
    fn hash_is_deterministic() {
        let event = Event {
            id: "fixed-id".to_string(),
            timestamp: 1000,
            kind: EventKind::Custom,
            source: "test".to_string(),
            context: EventContext::new("s".to_string()),
            payload: serde_json::json!({"x": 1}),
        };
        let hash1 = compute_event_hash(&event);
        let hash2 = compute_event_hash(&event);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn different_events_different_hashes() {
        let e1 = Event {
            id: "id-1".to_string(),
            timestamp: 1000,
            kind: EventKind::Custom,
            source: "test".to_string(),
            context: EventContext::new("s".to_string()),
            payload: serde_json::json!({"x": 1}),
        };
        let e2 = Event {
            id: "id-2".to_string(),
            timestamp: 1000,
            kind: EventKind::Custom,
            source: "test".to_string(),
            context: EventContext::new("s".to_string()),
            payload: serde_json::json!({"x": 2}),
        };
        assert_ne!(compute_event_hash(&e1), compute_event_hash(&e2));
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p aletheia-core -- chain`
Expected: FAIL — `HashChain` not defined.

- [ ] **Step 3: Write implementation**

```rust
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use crate::event::Event;

/// SHA-256 hash of the canonical JSON serialization of an event.
pub fn compute_event_hash(event: &Event) -> [u8; 32] {
    let bytes = serde_json::to_vec(event).expect("Event serialization cannot fail");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher.finalize().into()
}

/// A receipt: an event sealed with its hash and chain link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub event: Event,
    #[serde(with = "hex_bytes_32")]
    pub hash: [u8; 32],
    #[serde(with = "hex_bytes_32")]
    pub prev_hash: [u8; 32],
    pub sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<[u8; 64]>,
}

/// Append-only hash chain of receipts.
pub struct HashChain {
    receipts: Vec<Receipt>,
    head: [u8; 32],
}

impl HashChain {
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
            head: [0u8; 32],
        }
    }

    pub fn append(&mut self, event: Event) -> Receipt {
        let hash = compute_event_hash(&event);
        let receipt = Receipt {
            event,
            hash,
            prev_hash: self.head,
            sequence: self.receipts.len() as u64,
            signature: None,
        };
        self.head = hash;
        self.receipts.push(receipt.clone());
        receipt
    }

    pub fn head(&self) -> [u8; 32] {
        self.head
    }

    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    pub fn into_receipts(self) -> Vec<Receipt> {
        self.receipts
    }
}

/// Serde helper for [u8; 32] as hex string.
mod hex_bytes_32 {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))?;
        Ok(arr)
    }
}

pub use hex_bytes_32 as serde_hex_32;
```

- [ ] **Step 4: Update lib.rs**

```rust
pub mod error;
pub mod event;
pub mod chain;
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p aletheia-core -- chain`
Expected: All 5 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-core/src/chain.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add HashChain with SHA-256 event hashing and prev_hash linking"
```

---

### Task 5: Merkle Tree

**Files:**
- Create: `crates/aletheia-core/src/merkle.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod merkle;`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_leaves_returns_zero() {
        assert_eq!(compute_merkle_root(&[]), [0u8; 32]);
    }

    #[test]
    fn single_leaf() {
        let leaf = [1u8; 32];
        let root = compute_merkle_root(&[leaf]);
        // Single leaf: hash(leaf || leaf) since it's duplicated
        let mut hasher = Sha256::new();
        hasher.update(leaf);
        hasher.update(leaf);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(root, expected);
    }

    #[test]
    fn two_leaves() {
        let a = [1u8; 32];
        let b = [2u8; 32];
        let root = compute_merkle_root(&[a, b]);
        let mut hasher = Sha256::new();
        hasher.update(a);
        hasher.update(b);
        let expected: [u8; 32] = hasher.finalize().into();
        assert_eq!(root, expected);
    }

    #[test]
    fn deterministic() {
        let leaves: Vec<[u8; 32]> = (0..5).map(|i| [i as u8; 32]).collect();
        let r1 = compute_merkle_root(&leaves);
        let r2 = compute_merkle_root(&leaves);
        assert_eq!(r1, r2);
    }

    #[test]
    fn modification_changes_root() {
        let mut leaves: Vec<[u8; 32]> = (0..4).map(|i| [i as u8; 32]).collect();
        let r1 = compute_merkle_root(&leaves);
        leaves[2] = [99u8; 32];
        let r2 = compute_merkle_root(&leaves);
        assert_ne!(r1, r2);
    }

    #[test]
    fn odd_leaf_count() {
        let leaves: Vec<[u8; 32]> = (0..3).map(|i| [i as u8; 32]).collect();
        let root = compute_merkle_root(&leaves);
        assert_ne!(root, [0u8; 32]);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p aletheia-core -- merkle`
Expected: FAIL.

- [ ] **Step 3: Write implementation**

```rust
use sha2::{Digest, Sha256};

/// Compute the Merkle root of a set of 32-byte leaf hashes.
///
/// - Empty input returns `[0u8; 32]`.
/// - Odd leaf counts: last leaf is duplicated.
/// - Parent = SHA-256(left || right).
pub fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }

    let mut layer: Vec<[u8; 32]> = leaves.to_vec();

    while layer.len() > 1 {
        if layer.len() % 2 != 0 {
            let last = *layer.last().unwrap();
            layer.push(last);
        }

        let mut next = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(pair[0]);
            hasher.update(pair[1]);
            next.push(hasher.finalize().into());
        }
        layer = next;
    }

    layer[0]
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Run: `cargo test -p aletheia-core -- merkle`
Expected: All 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/merkle.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add Merkle tree root computation"
```

---

### Task 6: Ed25519 Signing

**Files:**
- Create: `crates/aletheia-core/src/signing.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod signing;`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keygen_produces_valid_pair() {
        let (signing, verifying) = generate_keypair();
        assert_eq!(signing.len(), 32);
        assert_eq!(verifying.len(), 32);
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let (sk_bytes, vk_bytes) = generate_keypair();
        let message = [42u8; 32];
        let sig = sign(&sk_bytes, &message).unwrap();
        assert!(verify(&vk_bytes, &message, &sig).is_ok());
    }

    #[test]
    fn tampered_message_fails_verification() {
        let (sk_bytes, vk_bytes) = generate_keypair();
        let message = [42u8; 32];
        let sig = sign(&sk_bytes, &message).unwrap();
        let tampered = [43u8; 32];
        assert!(verify(&vk_bytes, &tampered, &sig).is_err());
    }

    #[test]
    fn tampered_signature_fails_verification() {
        let (sk_bytes, vk_bytes) = generate_keypair();
        let message = [42u8; 32];
        let mut sig = sign(&sk_bytes, &message).unwrap();
        sig[0] ^= 0xFF;
        assert!(verify(&vk_bytes, &message, &sig).is_err());
    }

    #[test]
    fn wrong_key_fails_verification() {
        let (sk_bytes, _) = generate_keypair();
        let (_, other_vk) = generate_keypair();
        let message = [42u8; 32];
        let sig = sign(&sk_bytes, &message).unwrap();
        assert!(verify(&other_vk, &message, &sig).is_err());
    }

    #[test]
    fn hex_roundtrip() {
        let (sk, vk) = generate_keypair();
        let sk_hex = hex::encode(sk);
        let vk_hex = hex::encode(vk);
        let sk_back: [u8; 32] = hex::decode(&sk_hex).unwrap().try_into().unwrap();
        let vk_back: [u8; 32] = hex::decode(&vk_hex).unwrap().try_into().unwrap();
        assert_eq!(sk, sk_back);
        assert_eq!(vk, vk_back);
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p aletheia-core -- signing`
Expected: FAIL.

- [ ] **Step 3: Write implementation**

```rust
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use crate::error::AletheiaError;

/// Generate a new Ed25519 keypair. Returns (signing_key_bytes, verifying_key_bytes).
pub fn generate_keypair() -> ([u8; 32], [u8; 32]) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    (signing_key.to_bytes(), verifying_key.to_bytes())
}

/// Sign a 32-byte message with an Ed25519 signing key.
pub fn sign(signing_key_bytes: &[u8; 32], message: &[u8; 32]) -> crate::error::Result<[u8; 64]> {
    let signing_key = SigningKey::from_bytes(signing_key_bytes);
    let signature = signing_key.sign(message);
    Ok(signature.to_bytes())
}

/// Verify an Ed25519 signature on a 32-byte message.
pub fn verify(
    verifying_key_bytes: &[u8; 32],
    message: &[u8; 32],
    signature_bytes: &[u8; 64],
) -> crate::error::Result<()> {
    let verifying_key = VerifyingKey::from_bytes(verifying_key_bytes)
        .map_err(|e| AletheiaError::SigningError(e.to_string()))?;
    let signature = ed25519_dalek::Signature::from_bytes(signature_bytes);
    verifying_key
        .verify(message, &signature)
        .map_err(|_| AletheiaError::InvalidSignature {
            signer: hex::encode(verifying_key_bytes),
        })
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Run: `cargo test -p aletheia-core -- signing`
Expected: All 6 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/signing.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add Ed25519 keygen, sign, verify"
```

---

## Chunk 3: Core Assembly — Pack + Verify

### Task 7: Evidence Pack

**Files:**
- Create: `crates/aletheia-core/src/pack.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod pack;`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::HashChain;
    use crate::event::{Event, EventContext, EventKind};

    fn make_chain(n: usize) -> HashChain {
        let mut chain = HashChain::new();
        for i in 0..n {
            chain.append(Event::new(
                EventKind::Custom,
                "test".to_string(),
                EventContext::new("s1".to_string()),
                serde_json::json!({"i": i}),
            ));
        }
        chain
    }

    #[test]
    fn pack_from_chain_unsigned() {
        let chain = make_chain(3);
        let pack = EvidencePack::from_chain(chain, None);
        assert_eq!(pack.version, "1.0");
        assert_eq!(pack.receipts.len(), 3);
        assert_ne!(pack.merkle_root, [0u8; 32]);
        assert!(pack.signatures.is_empty());
    }

    #[test]
    fn pack_from_chain_signed() {
        let chain = make_chain(3);
        let (sk, _vk) = crate::signing::generate_keypair();
        let pack = EvidencePack::from_chain(chain, Some(&sk));
        assert_eq!(pack.signatures.len(), 1);
    }

    #[test]
    fn pack_serialization_roundtrip() {
        let chain = make_chain(2);
        let pack = EvidencePack::from_chain(chain, None);
        let json = serde_json::to_string_pretty(&pack).unwrap();
        let deserialized: EvidencePack = serde_json::from_str(&json).unwrap();
        assert_eq!(pack.merkle_root, deserialized.merkle_root);
        assert_eq!(pack.receipts.len(), deserialized.receipts.len());
    }

    #[test]
    fn pack_metadata_correct() {
        let chain = make_chain(5);
        let pack = EvidencePack::from_chain(chain, None);
        assert_eq!(pack.metadata.event_count, 5);
    }
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cargo test -p aletheia-core -- pack`
Expected: FAIL.

- [ ] **Step 3: Write implementation**

```rust
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chain::{serde_hex_32, HashChain, Receipt};
use crate::merkle::compute_merkle_root;
use crate::signing;

pub const PACK_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub version: String,
    pub session_id: String,
    pub created_at: u64,
    pub sealed_at: u64,
    pub metadata: PackMetadata,
    pub receipts: Vec<Receipt>,
    #[serde(with = "serde_hex_32")]
    pub merkle_root: [u8; 32],
    #[serde(with = "serde_hex_32")]
    pub chain_head: [u8; 32],
    pub signatures: Vec<PackSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_source: Option<String>,
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackSignature {
    pub signer: String,
    pub signature: String, // hex-encoded 64 bytes
    pub signed_at: u64,
}

impl EvidencePack {
    /// Create a sealed evidence pack from a hash chain.
    /// Optionally sign with an Ed25519 signing key (32 bytes).
    pub fn from_chain(chain: HashChain, signing_key: Option<&[u8; 32]>) -> Self {
        let receipts = chain.receipts().to_vec();
        let chain_head = chain.head();

        let leaves: Vec<[u8; 32]> = receipts.iter().map(|r| r.hash).collect();
        let merkle_root = compute_merkle_root(&leaves);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let created_at = receipts
            .first()
            .map(|r| r.event.timestamp)
            .unwrap_or(now);

        let session_id = receipts
            .first()
            .map(|r| r.event.context.session_id.clone())
            .unwrap_or_default();

        let metadata = PackMetadata {
            repo: receipts.first().and_then(|r| r.event.context.repo.clone()),
            branch: receipts.first().and_then(|r| r.event.context.branch.clone()),
            pr_number: receipts.first().and_then(|r| r.event.context.pr_number),
            agent_source: receipts.first().map(|r| r.event.source.clone()),
            event_count: receipts.len(),
        };

        let mut signatures = Vec::new();
        if let Some(sk) = signing_key {
            if let Ok(sig) = signing::sign(sk, &merkle_root) {
                let vk_bytes = ed25519_dalek::SigningKey::from_bytes(sk)
                    .verifying_key()
                    .to_bytes();
                signatures.push(PackSignature {
                    signer: hex::encode(vk_bytes),
                    signature: hex::encode(sig),
                    signed_at: now,
                });
            }
        }

        Self {
            version: PACK_VERSION.to_string(),
            session_id,
            created_at,
            sealed_at: now,
            metadata,
            receipts,
            merkle_root,
            chain_head,
            signatures,
        }
    }
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Run: `cargo test -p aletheia-core -- pack`
Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/pack.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add EvidencePack assembly from HashChain with optional signing"
```

---

### Task 8: Verification

**Files:**
- Create: `crates/aletheia-core/src/verify.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod verify;`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::HashChain;
    use crate::event::{Event, EventContext, EventKind};
    use crate::signing::generate_keypair;

    fn make_signed_pack() -> (EvidencePack, [u8; 32]) {
        let mut chain = HashChain::new();
        for i in 0..3 {
            chain.append(Event::new(
                EventKind::Custom,
                "test".to_string(),
                EventContext::new("s1".to_string()),
                serde_json::json!({"i": i}),
            ));
        }
        let (sk, vk) = generate_keypair();
        let pack = EvidencePack::from_chain(chain, Some(&sk));
        (pack, vk)
    }

    #[test]
    fn valid_pack_verifies() {
        let (pack, vk) = make_signed_pack();
        let result = verify_pack(&pack, Some(&vk));
        assert!(result.is_ok());
        let v = result.unwrap();
        assert!(v.chain_ok);
        assert!(v.merkle_ok);
        assert!(v.chain_head_ok);
        assert_eq!(v.signatures_valid, 1);
        assert_eq!(v.signatures_total, 1);
        assert_eq!(v.receipt_count, 3);
    }

    #[test]
    fn unsigned_pack_verifies_without_key() {
        let mut chain = HashChain::new();
        chain.append(Event::new(
            EventKind::Custom,
            "test".to_string(),
            EventContext::new("s1".to_string()),
            serde_json::json!({}),
        ));
        let pack = EvidencePack::from_chain(chain, None);
        let result = verify_pack(&pack, None);
        assert!(result.is_ok());
    }

    #[test]
    fn tampered_event_fails() {
        let (mut pack, vk) = make_signed_pack();
        pack.receipts[1].event.payload = serde_json::json!({"tampered": true});
        let result = verify_pack(&pack, Some(&vk));
        assert!(result.is_err());
    }

    #[test]
    fn tampered_chain_fails() {
        let (mut pack, vk) = make_signed_pack();
        pack.receipts[1].prev_hash = [0u8; 32];
        let result = verify_pack(&pack, Some(&vk));
        assert!(result.is_err());
    }

    #[test]
    fn tampered_merkle_root_fails() {
        let (mut pack, vk) = make_signed_pack();
        pack.merkle_root = [99u8; 32];
        let result = verify_pack(&pack, Some(&vk));
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests, verify failure**

Run: `cargo test -p aletheia-core -- verify`
Expected: FAIL.

- [ ] **Step 3: Write implementation**

```rust
use crate::chain::compute_event_hash;
use crate::error::AletheiaError;
use crate::merkle::compute_merkle_root;
use crate::pack::EvidencePack;
use crate::signing;

#[derive(Debug)]
pub struct VerificationResult {
    pub receipt_count: usize,
    pub chain_ok: bool,
    pub merkle_ok: bool,
    pub chain_head_ok: bool,
    pub signatures_valid: usize,
    pub signatures_total: usize,
}

/// Verify the full integrity of an evidence pack.
///
/// Steps:
/// 1. Recompute each event hash, compare with stored hash.
/// 2. Verify prev_hash chain linking.
/// 3. Recompute Merkle root, compare with stored root.
/// 4. Verify chain_head matches last receipt hash.
/// 5. Verify signatures (if verifying_key provided).
pub fn verify_pack(
    pack: &EvidencePack,
    verifying_key: Option<&[u8; 32]>,
) -> crate::error::Result<VerificationResult> {
    let receipts = &pack.receipts;

    // Step 1 + 2: verify event hashes and chain linking
    for (i, receipt) in receipts.iter().enumerate() {
        let computed_hash = compute_event_hash(&receipt.event);
        if computed_hash != receipt.hash {
            return Err(AletheiaError::EventHashMismatch {
                index: receipt.sequence,
                expected: hex::encode(receipt.hash),
                actual: hex::encode(computed_hash),
            });
        }

        let expected_prev = if i == 0 {
            [0u8; 32]
        } else {
            receipts[i - 1].hash
        };

        if receipt.prev_hash != expected_prev {
            return Err(AletheiaError::BrokenChain {
                index: receipt.sequence,
                expected: hex::encode(expected_prev),
                actual: hex::encode(receipt.prev_hash),
            });
        }
    }

    // Step 3: verify Merkle root
    let leaves: Vec<[u8; 32]> = receipts.iter().map(|r| r.hash).collect();
    let computed_root = compute_merkle_root(&leaves);
    if computed_root != pack.merkle_root {
        return Err(AletheiaError::MerkleRootMismatch {
            expected: hex::encode(pack.merkle_root),
            actual: hex::encode(computed_root),
        });
    }

    // Step 4: verify chain head
    let computed_head = receipts.last().map(|r| r.hash).unwrap_or([0u8; 32]);
    if computed_head != pack.chain_head {
        return Err(AletheiaError::ChainHeadMismatch {
            expected: hex::encode(pack.chain_head),
            actual: hex::encode(computed_head),
        });
    }

    // Step 5: verify signatures
    let mut signatures_valid = 0;
    let signatures_total = pack.signatures.len();

    if let Some(vk) = verifying_key {
        for sig_entry in &pack.signatures {
            let sig_bytes: [u8; 64] = hex::decode(&sig_entry.signature)
                .map_err(|e| AletheiaError::SigningError(e.to_string()))?
                .try_into()
                .map_err(|_| AletheiaError::SigningError("invalid signature length".into()))?;
            signing::verify(vk, &pack.merkle_root, &sig_bytes)?;
            signatures_valid += 1;
        }
    }

    Ok(VerificationResult {
        receipt_count: receipts.len(),
        chain_ok: true,
        merkle_ok: true,
        chain_head_ok: true,
        signatures_valid,
        signatures_total,
    })
}
```

- [ ] **Step 4: Update lib.rs, run tests**

Run: `cargo test -p aletheia-core -- verify`
Expected: All 5 tests PASS.

- [ ] **Step 5: Run all core tests**

Run: `cargo test -p aletheia-core`
Expected: All tests PASS (event + chain + merkle + signing + pack + verify).

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-core/src/verify.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add full evidence pack verification pipeline"
```

---

## Chunk 4: CLI — Main, Keygen, Capture

### Task 9: CLI Main + Paths

**Files:**
- Modify: `crates/aletheia-cli/src/main.rs`
- Create: `crates/aletheia-cli/src/paths.rs`
- Create: `crates/aletheia-cli/src/cmd_keygen.rs` (stub)
- Create: `crates/aletheia-cli/src/cmd_capture.rs` (stub)
- Create: `crates/aletheia-cli/src/cmd_seal.rs` (stub)
- Create: `crates/aletheia-cli/src/cmd_verify.rs` (stub)
- Create: `crates/aletheia-cli/src/cmd_export.rs` (stub)

- [ ] **Step 1: Write paths.rs**

```rust
use std::path::PathBuf;

/// Returns the Aletheia config directory (cross-platform).
/// Windows: %APPDATA%\aletheia
/// Linux: ~/.config/aletheia
/// macOS: ~/Library/Application Support/aletheia
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aletheia")
}

pub fn keys_dir() -> PathBuf {
    config_dir().join("keys")
}

pub fn sessions_dir() -> PathBuf {
    config_dir().join("sessions")
}

pub fn session_dir(name: &str) -> PathBuf {
    sessions_dir().join(name)
}
```

- [ ] **Step 2: Write main.rs with clap**

```rust
mod cmd_capture;
mod cmd_export;
mod cmd_keygen;
mod cmd_seal;
mod cmd_verify;
mod paths;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aletheia", version, about = "Cryptographic evidence packs for coding-agent compliance")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate an Ed25519 keypair
    Keygen {
        /// Output directory for keys
        #[arg(long)]
        output: Option<String>,
        /// Key name (default: "default")
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// Capture events from stdin into a session
    Capture {
        /// Session name
        #[arg(long)]
        session: String,
        /// Event source identifier
        #[arg(long, default_value = "manual")]
        source: String,
        /// Context key=value pairs (comma-separated)
        #[arg(long)]
        context: Option<String>,
    },
    /// Seal a session into a signed evidence pack
    Seal {
        /// Session name
        #[arg(long)]
        session: String,
        /// Path to signing key (.sec file)
        #[arg(long)]
        key: Option<String>,
        /// Output path for the evidence pack
        #[arg(long)]
        output: Option<String>,
    },
    /// Verify an evidence pack's integrity
    Verify {
        /// Path to .aletheia.json pack file
        pack: String,
        /// Path to verifying key (.pub file)
        #[arg(long)]
        key: Option<String>,
    },
    /// Export an evidence pack as HTML, JSON, or Markdown
    Export {
        /// Output format
        #[arg(long)]
        format: String,
        /// Path to .aletheia.json pack file
        pack: String,
        /// Output path (default: stdout)
        #[arg(long)]
        output: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Keygen { output, name } => cmd_keygen::run(output, name),
        Commands::Capture { session, source, context } => cmd_capture::run(session, source, context),
        Commands::Seal { session, key, output } => cmd_seal::run(session, key, output),
        Commands::Verify { pack, key } => cmd_verify::run(pack, key),
        Commands::Export { format, pack, output } => cmd_export::run(format, pack, output),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(match &e {
            _ if e.to_string().contains("integrity")
                || e.to_string().contains("mismatch")
                || e.to_string().contains("signature") => 1,
            _ => 3,
        });
    }
}
```

- [ ] **Step 3: Create stub command files**

Each stub file (`cmd_keygen.rs`, `cmd_capture.rs`, `cmd_seal.rs`, `cmd_verify.rs`, `cmd_export.rs`):

```rust
pub fn run(/* params */) -> Result<(), Box<dyn std::error::Error>> {
    todo!()
}
```

(Match the signatures from main.rs for each command.)

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p aletheia-cli`
Expected: Compiles. (Commands will panic at runtime since they're `todo!()`)

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-cli/
git commit -m "feat(cli): add clap CLI structure with 5 subcommands"
```

---

### Task 10: Keygen Command

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_keygen.rs`

- [ ] **Step 1: Write implementation**

```rust
use aletheia_core::signing::generate_keypair;
use std::fs;
use std::path::PathBuf;

pub fn run(output: Option<String>, name: String) -> Result<(), Box<dyn std::error::Error>> {
    let dir = output
        .map(PathBuf::from)
        .unwrap_or_else(|| crate::paths::keys_dir());

    fs::create_dir_all(&dir)?;

    let (sk, vk) = generate_keypair();

    let sk_path = dir.join(format!("{name}.sec"));
    let vk_path = dir.join(format!("{name}.pub"));

    fs::write(&sk_path, hex::encode(sk))?;
    fs::write(&vk_path, hex::encode(vk))?;

    eprintln!("Keypair generated:");
    eprintln!("  Secret: {}", sk_path.display());
    eprintln!("  Public: {}", vk_path.display());

    Ok(())
}
```

- [ ] **Step 2: Test manually**

Run: `cargo run -p aletheia-cli -- keygen --output /tmp/aletheia-test-keys --name test`
Expected: Two files created, hex-encoded, 64 chars each.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-cli/src/cmd_keygen.rs
git commit -m "feat(cli): implement keygen command"
```

---

### Task 11: Capture Command

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_capture.rs`

- [ ] **Step 1: Write implementation**

```rust
use aletheia_core::chain::HashChain;
use aletheia_core::event::Event;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;

pub fn run(
    session: String,
    source: String,
    context: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_dir = crate::paths::session_dir(&session);
    fs::create_dir_all(&session_dir)?;

    let receipts_path = session_dir.join("receipts.jsonl");
    let metadata_path = session_dir.join("metadata.json");

    // Parse optional context
    let (repo, branch, pr_number) = parse_context(context.as_deref());

    // Save metadata
    let metadata = serde_json::json!({
        "session_id": session,
        "source": source,
        "repo": repo,
        "branch": branch,
        "pr_number": pr_number,
    });
    fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;

    // Read stdin line by line
    let stdin = io::stdin();
    let mut chain = HashChain::new();
    let mut output = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&receipts_path)?;

    let mut count = 0u64;
    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let event = if line.starts_with('{') {
            match Event::from_json_line(&line, &session) {
                Ok(mut e) => {
                    if source != "manual" {
                        e.source = source.clone();
                    }
                    if let Some(ref r) = repo {
                        e.context.repo = Some(r.clone());
                    }
                    if let Some(ref b) = branch {
                        e.context.branch = Some(b.clone());
                    }
                    if let Some(pr) = pr_number {
                        e.context.pr_number = Some(pr);
                    }
                    e
                }
                Err(_) => Event::from_plain_text(&line, &session),
            }
        } else {
            Event::from_plain_text(&line, &session)
        };

        let receipt = chain.append(event);
        let receipt_json = serde_json::to_string(&receipt)?;

        use std::io::Write;
        writeln!(output, "{receipt_json}")?;
        count += 1;
    }

    eprintln!("Captured {count} events into session '{session}'");
    eprintln!("  Receipts: {}", receipts_path.display());

    Ok(())
}

fn parse_context(ctx: Option<&str>) -> (Option<String>, Option<String>, Option<u64>) {
    let mut repo = None;
    let mut branch = None;
    let mut pr_number = None;

    if let Some(ctx_str) = ctx {
        for pair in ctx_str.split(',') {
            if let Some((k, v)) = pair.split_once('=') {
                match k.trim() {
                    "repo" => repo = Some(v.trim().to_string()),
                    "branch" => branch = Some(v.trim().to_string()),
                    "pr" => pr_number = v.trim().parse().ok(),
                    _ => {}
                }
            }
        }
    }

    (repo, branch, pr_number)
}
```

- [ ] **Step 2: Test manually**

Run: `echo '{"payload":{"cmd":"cargo test"}}' | cargo run -p aletheia-cli -- capture --session test-1 --source claude-code`
Expected: Session directory created, receipts.jsonl has 1 line.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-cli/src/cmd_capture.rs
git commit -m "feat(cli): implement capture command (stdin → session)"
```

---

## Chunk 5: CLI — Seal, Verify, Export

### Task 12: Seal Command

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_seal.rs`

- [ ] **Step 1: Write implementation**

```rust
use aletheia_core::chain::{HashChain, Receipt};
use aletheia_core::pack::EvidencePack;
use std::fs;
use std::io::BufRead;
use std::path::PathBuf;

pub fn run(
    session: String,
    key: Option<String>,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_dir = crate::paths::session_dir(&session);
    let receipts_path = session_dir.join("receipts.jsonl");

    if !receipts_path.exists() {
        return Err(format!("Session '{}' not found at {}", session, receipts_path.display()).into());
    }

    // Load receipts
    let file = fs::File::open(&receipts_path)?;
    let reader = std::io::BufReader::new(file);
    let mut chain = HashChain::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let receipt: Receipt = serde_json::from_str(&line)?;
        // Re-append the event to rebuild the chain
        chain.append(receipt.event);
    }

    if chain.is_empty() {
        return Err("Session has no events".into());
    }

    // Load signing key if provided
    let signing_key = if let Some(key_path) = key {
        let hex_str = fs::read_to_string(&key_path)?.trim().to_string();
        let bytes: [u8; 32] = hex::decode(&hex_str)?
            .try_into()
            .map_err(|_| "Invalid signing key: expected 32 bytes")?;
        Some(bytes)
    } else {
        None
    };

    // Create pack
    let pack = EvidencePack::from_chain(chain, signing_key.as_ref());

    // Write output
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(format!("{session}.aletheia.json")));

    let json = serde_json::to_string_pretty(&pack)?;
    fs::write(&output_path, &json)?;

    eprintln!("Evidence pack sealed:");
    eprintln!("  File: {}", output_path.display());
    eprintln!("  Events: {}", pack.metadata.event_count);
    eprintln!("  Merkle root: {}", hex::encode(pack.merkle_root));
    eprintln!("  Signatures: {}", pack.signatures.len());

    Ok(())
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/aletheia-cli/src/cmd_seal.rs
git commit -m "feat(cli): implement seal command (session → signed evidence pack)"
```

---

### Task 13: Verify Command

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_verify.rs`

- [ ] **Step 1: Write implementation**

```rust
use aletheia_core::pack::EvidencePack;
use aletheia_core::verify::verify_pack;
use std::fs;

pub fn run(pack_path: String, key: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let json = fs::read_to_string(&pack_path)?;
    let pack: EvidencePack = serde_json::from_str(&json)?;

    let verifying_key = if let Some(key_path) = key {
        let hex_str = fs::read_to_string(&key_path)?.trim().to_string();
        let bytes: [u8; 32] = hex::decode(&hex_str)?
            .try_into()
            .map_err(|_| "Invalid verifying key: expected 32 bytes")?;
        Some(bytes)
    } else {
        None
    };

    match verify_pack(&pack, verifying_key.as_ref()) {
        Ok(result) => {
            let output = serde_json::json!({
                "status": "verified",
                "receipt_count": result.receipt_count,
                "chain_ok": result.chain_ok,
                "merkle_ok": result.merkle_ok,
                "chain_head_ok": result.chain_head_ok,
                "signatures_valid": result.signatures_valid,
                "signatures_total": result.signatures_total,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            Ok(())
        }
        Err(e) => {
            let output = serde_json::json!({
                "status": "failed",
                "error": e.to_string(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
            std::process::exit(1);
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/aletheia-cli/src/cmd_verify.rs
git commit -m "feat(cli): implement verify command"
```

---

### Task 14: Export Command (JSON + Markdown + HTML)

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_export.rs`
- Create: `crates/aletheia-cli/src/templates/report.html`

- [ ] **Step 1: Create HTML template**

Create `crates/aletheia-cli/src/templates/report.html` — a standalone HTML template with placeholders:

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Aletheia Evidence Report — {{SESSION_ID}}</title>
<style>
  :root { --bg: #fff; --fg: #1a1a1a; --accent: #2563eb; --green: #16a34a; --red: #dc2626; --border: #e5e7eb; --code-bg: #f3f4f6; }
  @media (prefers-color-scheme: dark) { :root { --bg: #111; --fg: #e5e5e5; --accent: #60a5fa; --green: #4ade80; --red: #f87171; --border: #333; --code-bg: #1e1e1e; } }
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: var(--bg); color: var(--fg); max-width: 900px; margin: 0 auto; padding: 2rem; line-height: 1.6; }
  h1, h2, h3 { margin: 1.5rem 0 0.5rem; }
  .badge { display: inline-block; padding: 0.25rem 0.75rem; border-radius: 4px; font-weight: bold; font-size: 0.9rem; }
  .badge-ok { background: var(--green); color: #fff; }
  .badge-fail { background: var(--red); color: #fff; }
  table { width: 100%; border-collapse: collapse; margin: 1rem 0; }
  th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--border); }
  th { font-weight: 600; }
  code { font-family: "Fira Code", "Cascadia Code", monospace; font-size: 0.85rem; background: var(--code-bg); padding: 0.15rem 0.3rem; border-radius: 3px; }
  .hash { font-family: monospace; font-size: 0.8rem; word-break: break-all; color: var(--accent); }
  details { margin: 1rem 0; }
  summary { cursor: pointer; font-weight: 600; }
  pre { background: var(--code-bg); padding: 1rem; border-radius: 6px; overflow-x: auto; font-size: 0.85rem; }
  .timeline-item { padding: 0.5rem 0; border-left: 2px solid var(--accent); padding-left: 1rem; margin-left: 0.5rem; }
  .timeline-time { font-size: 0.8rem; color: #888; }
  footer { margin-top: 3rem; padding-top: 1rem; border-top: 1px solid var(--border); font-size: 0.8rem; color: #888; }
  @media print { body { max-width: 100%; } }
</style>
</head>
<body>
<h1>Aletheia — Evidence Report</h1>
<p><strong>Session:</strong> {{SESSION_ID}} | <strong>Sealed:</strong> {{SEALED_AT}} | <span class="badge {{STATUS_CLASS}}">{{STATUS_TEXT}}</span></p>

<h2>Summary</h2>
<table>
  <tr><th>Events</th><td>{{EVENT_COUNT}}</td></tr>
  <tr><th>Chain integrity</th><td>{{CHAIN_STATUS}}</td></tr>
  <tr><th>Merkle root</th><td><code class="hash">{{MERKLE_ROOT}}</code></td></tr>
  <tr><th>Signatures</th><td>{{SIGNATURES_STATUS}}</td></tr>
</table>

<h2>Timeline</h2>
{{TIMELINE}}

<h2>Hash Chain</h2>
{{HASH_CHAIN}}

<h2>Signatures</h2>
{{SIGNATURES}}

<details>
<summary>Raw Evidence Pack (JSON)</summary>
<pre>{{RAW_JSON}}</pre>
</details>

<script type="application/json" id="evidence-data">{{RAW_JSON_UNESCAPED}}</script>

<footer>Generated by Aletheia v{{VERSION}}</footer>
</body>
</html>
```

- [ ] **Step 2: Write cmd_export.rs**

```rust
use aletheia_core::pack::EvidencePack;
use aletheia_core::verify::verify_pack;
use std::fs;
use std::io::Write;

pub fn run(
    format: String,
    pack_path: String,
    output: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = fs::read_to_string(&pack_path)?;
    let pack: EvidencePack = serde_json::from_str(&json)?;

    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&pack)?,
        "markdown" | "md" => render_markdown(&pack),
        "html" => render_html(&pack, &json)?,
        other => return Err(format!("Unknown format: {other}. Use html, json, or markdown.").into()),
    };

    match output {
        Some(path) => fs::write(&path, &content)?,
        None => print!("{content}"),
    }

    Ok(())
}

fn render_markdown(pack: &EvidencePack) -> String {
    let verified = verify_pack(pack, None).is_ok();
    let status = if verified { "VERIFIED" } else { "FAILED" };

    let mut md = String::new();
    md.push_str(&format!("# Aletheia Evidence Report\n\n"));
    md.push_str(&format!("**Session:** {} | **Status:** {}\n\n", pack.session_id, status));
    md.push_str("## Summary\n\n");
    md.push_str(&format!("| Field | Value |\n|-------|-------|\n"));
    md.push_str(&format!("| Events | {} |\n", pack.metadata.event_count));
    md.push_str(&format!("| Merkle root | `{}` |\n", hex::encode(pack.merkle_root)));
    md.push_str(&format!("| Signatures | {} |\n", pack.signatures.len()));
    md.push_str(&format!("| Version | {} |\n\n", pack.version));

    md.push_str("## Timeline\n\n");
    for r in &pack.receipts {
        let kind = serde_json::to_string(&r.event.kind).unwrap_or_default();
        md.push_str(&format!(
            "- **#{}** `{}` [{}] — {}\n",
            r.sequence,
            r.event.source,
            kind.trim_matches('"'),
            truncate_payload(&r.event.payload, 80),
        ));
    }

    md.push_str("\n## Hash Chain\n\n");
    md.push_str("```\n");
    for r in &pack.receipts {
        md.push_str(&format!(
            "#{}: {} ← {}\n",
            r.sequence,
            &hex::encode(r.hash)[..16],
            &hex::encode(r.prev_hash)[..16],
        ));
    }
    md.push_str("```\n\n");
    md.push_str(&format!("*Generated by Aletheia v{}*\n", pack.version));

    md
}

fn render_html(pack: &EvidencePack, raw_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let template = include_str!("templates/report.html");
    let verified = verify_pack(pack, None).is_ok();

    let sealed_at = chrono::DateTime::from_timestamp_millis(pack.sealed_at as i64)
        .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| pack.sealed_at.to_string());

    // Build timeline HTML
    let mut timeline = String::new();
    for r in &pack.receipts {
        let kind = serde_json::to_string(&r.event.kind).unwrap_or_default();
        timeline.push_str(&format!(
            r#"<div class="timeline-item"><span class="timeline-time">#{}</span> <code>{}</code> [{}] — {}</div>"#,
            r.sequence,
            html_escape(&r.event.source),
            html_escape(kind.trim_matches('"')),
            html_escape(&truncate_payload(&r.event.payload, 100)),
        ));
        timeline.push('\n');
    }

    // Build hash chain HTML
    let mut chain_html = String::from("<table><tr><th>#</th><th>Hash</th><th>Prev Hash</th></tr>");
    for r in &pack.receipts {
        chain_html.push_str(&format!(
            "<tr><td>{}</td><td><code class=\"hash\">{}</code></td><td><code class=\"hash\">{}</code></td></tr>",
            r.sequence,
            hex::encode(r.hash),
            hex::encode(r.prev_hash),
        ));
    }
    chain_html.push_str("</table>");

    // Build signatures HTML
    let mut sigs_html = String::new();
    if pack.signatures.is_empty() {
        sigs_html.push_str("<p>No signatures (unsigned pack).</p>");
    } else {
        sigs_html.push_str("<table><tr><th>Signer</th><th>Signed At</th></tr>");
        for s in &pack.signatures {
            let ts = chrono::DateTime::from_timestamp_millis(s.signed_at as i64)
                .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| s.signed_at.to_string());
            sigs_html.push_str(&format!(
                "<tr><td><code class=\"hash\">{}</code></td><td>{}</td></tr>",
                &s.signer, ts,
            ));
        }
        sigs_html.push_str("</table>");
    }

    let escaped_json = html_escape(raw_json);

    let html = template
        .replace("{{SESSION_ID}}", &html_escape(&pack.session_id))
        .replace("{{SEALED_AT}}", &sealed_at)
        .replace("{{STATUS_CLASS}}", if verified { "badge-ok" } else { "badge-fail" })
        .replace("{{STATUS_TEXT}}", if verified { "VERIFIED" } else { "INTEGRITY FAILURE" })
        .replace("{{EVENT_COUNT}}", &pack.metadata.event_count.to_string())
        .replace("{{CHAIN_STATUS}}", if verified { "OK" } else { "BROKEN" })
        .replace("{{MERKLE_ROOT}}", &hex::encode(pack.merkle_root))
        .replace("{{SIGNATURES_STATUS}}", &format!("{}", pack.signatures.len()))
        .replace("{{TIMELINE}}", &timeline)
        .replace("{{HASH_CHAIN}}", &chain_html)
        .replace("{{SIGNATURES}}", &sigs_html)
        .replace("{{RAW_JSON}}", &escaped_json)
        .replace("{{RAW_JSON_UNESCAPED}}", raw_json)
        .replace("{{VERSION}}", &pack.version);

    Ok(html)
}

fn truncate_payload(value: &serde_json::Value, max_len: usize) -> String {
    let s = value.to_string();
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

- [ ] **Step 3: Add chrono to Cargo.toml if not present, verify compile**

Run: `cargo build -p aletheia-cli`
Expected: Compiles.

- [ ] **Step 4: Test full pipeline manually**

```bash
echo '{"payload":{"cmd":"cargo test"},"kind":"shell_exec","source":"claude"}
{"payload":{"file":"src/main.rs","diff":"+fn hello()"},"kind":"file_edit","source":"claude"}
{"payload":{"result":"all tests pass"},"kind":"test_run","source":"ci"}' | cargo run -p aletheia-cli -- capture --session demo-1 --source claude-code

cargo run -p aletheia-cli -- seal --session demo-1

cargo run -p aletheia-cli -- verify demo-1.aletheia.json

cargo run -p aletheia-cli -- export --format html demo-1.aletheia.json --output demo-report.html

cargo run -p aletheia-cli -- export --format markdown demo-1.aletheia.json

cargo run -p aletheia-cli -- export --format json demo-1.aletheia.json
```

Expected: Full pipeline works. HTML report is openable in a browser. JSON and Markdown output to stdout.

- [ ] **Step 5: Test with signing**

```bash
cargo run -p aletheia-cli -- keygen --name demo

cargo run -p aletheia-cli -- seal --session demo-1 --key <config_dir>/aletheia/keys/demo.sec --output demo-signed.aletheia.json

cargo run -p aletheia-cli -- verify demo-signed.aletheia.json --key <config_dir>/aletheia/keys/demo.pub
```

Expected: Signed pack with 1 signature, verification passes.

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-cli/src/cmd_export.rs crates/aletheia-cli/src/cmd_seal.rs crates/aletheia-cli/src/cmd_verify.rs crates/aletheia-cli/src/templates/
git commit -m "feat(cli): implement seal, verify, and export commands with HTML/Markdown/JSON output"
```

---

## Chunk 6: Integration Tests + CI

### Task 15: End-to-End Integration Tests

**Files:**
- Create: `tests/e2e.rs`

- [ ] **Step 1: Write e2e tests**

```rust
use std::process::Command;

fn cargo_bin() -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "aletheia-cli", "--"]);
    cmd
}

#[test]
fn full_pipeline_unsigned() {
    let tmp = std::env::temp_dir().join("aletheia-test-unsigned");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    let session_dir = tmp.join("sessions").join("e2e-test");
    let pack_path = tmp.join("test.aletheia.json");

    // Capture
    let capture = Command::new("cargo")
        .args(["run", "-p", "aletheia-cli", "--", "capture", "--session", "e2e-test"])
        .env("XDG_CONFIG_HOME", &tmp)
        .stdin(std::process::Stdio::piped())
        .spawn();

    // Use a simpler approach: write to stdin
    let mut child = Command::new("cargo")
        .args(["run", "-p", "aletheia-cli", "--", "capture", "--session", "e2e-test"])
        .env("XDG_CONFIG_HOME", &tmp)
        .stdin(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, r#"{{"payload":{{"cmd":"test"}}}}"#).unwrap();
        writeln!(stdin, r#"{{"payload":{{"cmd":"build"}}}}"#).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "Capture failed: {}", String::from_utf8_lossy(&output.stderr));

    // Seal
    let seal = Command::new("cargo")
        .args([
            "run", "-p", "aletheia-cli", "--",
            "seal", "--session", "e2e-test",
            "--output", pack_path.to_str().unwrap(),
        ])
        .env("XDG_CONFIG_HOME", &tmp)
        .output()
        .unwrap();
    assert!(seal.status.success(), "Seal failed: {}", String::from_utf8_lossy(&seal.stderr));
    assert!(pack_path.exists());

    // Verify
    let verify = Command::new("cargo")
        .args(["run", "-p", "aletheia-cli", "--", "verify", pack_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(verify.status.success(), "Verify failed");
    let stdout = String::from_utf8_lossy(&verify.stdout);
    assert!(stdout.contains("verified"));

    // Export JSON
    let export = Command::new("cargo")
        .args([
            "run", "-p", "aletheia-cli", "--",
            "export", "--format", "json", pack_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(export.status.success());
    let json: serde_json::Value = serde_json::from_slice(&export.stdout).unwrap();
    assert_eq!(json["version"], "1.0");

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp);
}
```

- [ ] **Step 2: Run integration tests**

Run: `cargo test --test e2e`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/
git commit -m "test: add end-to-end integration tests for full CLI pipeline"
```

---

### Task 16: GitHub Actions CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write CI workflow**

```yaml
name: CI

on:
  push:
    branches: [master, main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  check:
    name: Check (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --all -- -D warnings

      - name: Test
        run: cargo test --all

      - name: Build release
        run: cargo build --release

      - name: Upload binary
        if: matrix.os == 'ubuntu-latest'
        uses: actions/upload-artifact@v4
        with:
          name: aletheia-${{ matrix.os }}
          path: target/release/aletheia
```

- [ ] **Step 2: Commit**

```bash
git add .github/
git commit -m "ci: add GitHub Actions workflow for fmt, clippy, test on 3 OSes"
```

---

### Task 17: Final Quality Pass

- [ ] **Step 1: Run full test suite**

Run: `cargo test --all`
Expected: All tests pass.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all -- -D warnings`
Expected: Zero warnings.

- [ ] **Step 3: Run fmt**

Run: `cargo fmt --all`
Expected: Clean.

- [ ] **Step 4: Push to GitHub**

```bash
git push origin master
```

- [ ] **Step 5: Verify CI passes**

Run: `gh run list --limit 1`
Expected: CI workflow triggered and passing on all 3 OSes.

---

## Task Summary

| Task | Component | Estimated Steps |
|------|-----------|----------------|
| 1 | Workspace setup | 7 |
| 2 | Error types | 3 |
| 3 | Event module | 6 |
| 4 | Hash chain | 6 |
| 5 | Merkle tree | 5 |
| 6 | Ed25519 signing | 5 |
| 7 | Evidence pack | 5 |
| 8 | Verification | 6 |
| 9 | CLI main + paths | 5 |
| 10 | Keygen command | 3 |
| 11 | Capture command | 3 |
| 12 | Seal command | 2 |
| 13 | Verify command | 2 |
| 14 | Export command | 6 |
| 15 | E2E tests | 3 |
| 16 | CI workflow | 2 |
| 17 | Quality pass | 5 |
| **Total** | | **72 steps** |

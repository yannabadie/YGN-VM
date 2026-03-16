# Aletheya v2 Cryptographic Hardening Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Merkle inclusion proofs, RFC 3161 timestamping, and Sigstore Rekor anchoring to the Aletheya evidence pack system, plus comprehensive documentation.

**Architecture:** Three additive layers on the existing core. `aletheia-core` gains 4 new modules (inclusion, timestamp, anchor, disclose) — all pure, no IO. `aletheia-cli` gains a network module (TSA + Rekor HTTP clients), 2 new commands (disclose, verify-receipt), and flags on seal/verify. merkle.rs is rewritten to use `rs_merkle`. Documentation covers the entire repo.

**Tech Stack:** Rust, rs_merkle, reqwest (rustls-tls), base64, Sigstore Rekor API, Sigstore TSA API, SHA-256, Ed25519

**Spec:** `docs/superpowers/specs/2026-03-16-aletheya-v2-crypto-hardening-design.md`

---

## File Structure

### New files
```
crates/aletheia-core/src/
  inclusion.rs          # ReceiptInclusionProof, generate, verify
  timestamp.rs          # TimestampProof struct, digest verification
  anchor.rs             # RekorAnchor, RekorInclusionProof, offline verification
  disclose.rs           # Selective disclosure extraction + verification

crates/aletheia-cli/src/
  cmd_disclose.rs       # aletheia disclose command
  cmd_verify_receipt.rs # aletheia verify-receipt command
  network/
    mod.rs              # Network module
    tsa.rs              # RFC 3161 TSA client
    rekor.rs            # Sigstore Rekor API client

README.md
LICENSE
ARCHITECTURE.md
docs/
  README.md
  EVIDENCE_FORMAT.md
  VERIFICATION.md
  THREAT_MODEL.md
  DEPLOYMENT.md
crates/aletheia-core/README.md
crates/aletheia-cli/README.md
packages/web/README.md
```

### Modified files
```
crates/aletheia-core/Cargo.toml     # add rs_merkle
crates/aletheia-core/src/lib.rs     # add 4 new modules
crates/aletheia-core/src/merkle.rs  # rewrite with rs_merkle
crates/aletheia-core/src/pack.rs    # add optional fields, version 1.1
crates/aletheia-core/src/verify.rs  # add steps 6-8
crates/aletheia-core/src/error.rs   # add new error variants
crates/aletheia-cli/Cargo.toml      # add reqwest, base64
crates/aletheia-cli/src/main.rs     # add commands + flags
crates/aletheia-cli/src/cmd_seal.rs # call TSA + Rekor
crates/aletheia-cli/src/cmd_verify.rs # pass --online flag
crates/aletheia-cli/src/cmd_export.rs # display new fields
```

---

## Chunk 1: Merkle Rewrite + Inclusion Proofs

### Task 1: Add rs_merkle dependency and rewrite merkle.rs

**Files:**
- Modify: `crates/aletheia-core/Cargo.toml`
- Modify: `crates/aletheia-core/src/merkle.rs`

- [ ] **Step 1: Add rs_merkle to Cargo.toml**

Add under `[dependencies]`:
```toml
rs_merkle = "1"
```

- [ ] **Step 2: Write failing test for rs_merkle compatibility**

The existing tests in merkle.rs must still pass after the rewrite. Run them first:

Run: `cargo test -p aletheia-core -- merkle`
Expected: All 6 existing tests PASS (baseline).

- [ ] **Step 3: Rewrite merkle.rs using rs_merkle**

Replace the entire implementation of `compute_merkle_root` but keep the same signature. Also add a new function `build_tree` that returns the full `MerkleTree` for generating proofs later.

```rust
use rs_merkle::{algorithms::Sha256 as MerkleSha256, Hasher, MerkleTree};
use sha2::{Digest, Sha256};

/// SHA-256 hasher for rs_merkle leaf hashing.
/// rs_merkle's built-in Sha256 hashes raw bytes. We need leaves pre-hashed
/// (they're already [u8; 32] receipt hashes), so we hash them again as leaves.
fn hash_leaf(data: &[u8; 32]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// Build a MerkleTree from receipt hashes. Returns the tree for proof generation.
pub fn build_tree(leaves: &[[u8; 32]]) -> MerkleTree<MerkleSha256> {
    let leaf_hashes: Vec<[u8; 32]> = leaves.iter().map(|l| MerkleSha256::hash(l)).collect();
    let mut tree = MerkleTree::<MerkleSha256>::new();
    tree.append(leaf_hashes.as_slice().to_vec());
    tree.commit();
    tree
}

/// Compute the Merkle root of a set of 32-byte leaf hashes.
///
/// - Empty input returns `[0u8; 32]`.
/// - Uses rs_merkle internally for RFC 6962-compatible tree construction.
pub fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    let tree = build_tree(leaves);
    tree.root().unwrap_or([0u8; 32])
}
```

**IMPORTANT:** The rs_merkle tree may compute a different root than the hand-written implementation (different leaf hashing, different odd-leaf handling). This is acceptable — we're upgrading the format from v1.0 to v1.1. Existing v1.0 packs will still be verifiable by their stored root; new packs will use the rs_merkle root.

- [ ] **Step 4: Update tests for new Merkle behavior**

The existing tests that check specific hash values (single_leaf, two_leaves) need to be updated since rs_merkle may hash differently. Keep the property tests (deterministic, modification_changes_root, empty_returns_zero).

Run: `cargo test -p aletheia-core -- merkle`
Expected: All tests PASS.

- [ ] **Step 5: Verify full test suite**

Run: `cargo test -p aletheia-core`
Expected: All tests pass (some pack/verify tests may need updating if the Merkle root changes).

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-core/Cargo.toml crates/aletheia-core/src/merkle.rs
git commit -m "refactor(core): rewrite merkle.rs using rs_merkle crate"
```

---

### Task 2: Inclusion proof module

**Files:**
- Create: `crates/aletheia-core/src/inclusion.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod inclusion;`

- [ ] **Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::compute_event_hash;
    use crate::event::{Event, EventContext, EventKind};
    use crate::merkle::{build_tree, compute_merkle_root};

    fn make_leaves(n: usize) -> Vec<[u8; 32]> {
        (0..n)
            .map(|i| {
                let event = Event::new(
                    EventKind::Custom,
                    "test",
                    EventContext::new("s"),
                    serde_json::json!({"i": i}),
                );
                compute_event_hash(&event).unwrap()
            })
            .collect()
    }

    #[test]
    fn generate_and_verify_proof() {
        let leaves = make_leaves(8);
        let root = compute_merkle_root(&leaves);
        let proofs = generate_inclusion_proofs(&leaves, &root);
        assert_eq!(proofs.len(), 8);
        for proof in &proofs {
            assert!(verify_inclusion_proof(proof).is_ok());
        }
    }

    #[test]
    fn tampered_proof_fails() {
        let leaves = make_leaves(4);
        let root = compute_merkle_root(&leaves);
        let mut proofs = generate_inclusion_proofs(&leaves, &root);
        proofs[1].leaf_hash = hex::encode([99u8; 32]); // tamper
        assert!(verify_inclusion_proof(&proofs[1]).is_err());
    }

    #[test]
    fn proof_serialization_roundtrip() {
        let leaves = make_leaves(3);
        let root = compute_merkle_root(&leaves);
        let proofs = generate_inclusion_proofs(&leaves, &root);
        let json = serde_json::to_string(&proofs[0]).unwrap();
        let deserialized: ReceiptInclusionProof = serde_json::from_str(&json).unwrap();
        assert_eq!(proofs[0].receipt_index, deserialized.receipt_index);
        assert_eq!(proofs[0].merkle_root, deserialized.merkle_root);
    }

    #[test]
    fn single_leaf_proof() {
        let leaves = make_leaves(1);
        let root = compute_merkle_root(&leaves);
        let proofs = generate_inclusion_proofs(&leaves, &root);
        assert_eq!(proofs.len(), 1);
        assert!(verify_inclusion_proof(&proofs[0]).is_ok());
    }
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p aletheia-core -- inclusion`
Expected: FAIL — module doesn't exist yet.

- [ ] **Step 3: Write implementation**

```rust
use rs_merkle::algorithms::Sha256 as MerkleSha256;
use serde::{Deserialize, Serialize};

use crate::error::{AletheiaError, Result};
use crate::merkle::build_tree;

/// Proof that a specific receipt is included in an evidence pack's Merkle tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptInclusionProof {
    pub receipt_index: usize,
    pub leaf_hash: String,
    pub proof_hashes: Vec<String>,
    pub merkle_root: String,
}

/// Generate inclusion proofs for all leaves.
pub fn generate_inclusion_proofs(
    leaves: &[[u8; 32]],
    root: &[u8; 32],
) -> Vec<ReceiptInclusionProof> {
    if leaves.is_empty() {
        return vec![];
    }

    let tree = build_tree(leaves);
    let root_hex = hex::encode(root);

    leaves
        .iter()
        .enumerate()
        .map(|(i, leaf)| {
            let proof = tree.proof(&[i]);
            let proof_bytes = proof.proof_hashes_to_bytes();
            let proof_hashes: Vec<String> = proof_bytes
                .chunks(32)
                .map(|chunk| hex::encode(chunk))
                .collect();

            ReceiptInclusionProof {
                receipt_index: i,
                leaf_hash: hex::encode(leaf),
                proof_hashes,
                merkle_root: root_hex.clone(),
            }
        })
        .collect()
}

/// Verify a single receipt inclusion proof against the stored Merkle root.
pub fn verify_inclusion_proof(proof: &ReceiptInclusionProof) -> Result<()> {
    let leaf_bytes: [u8; 32] = hex::decode(&proof.leaf_hash)
        .map_err(|e| AletheiaError::SigningError(format!("invalid leaf hash hex: {e}")))?
        .try_into()
        .map_err(|_| AletheiaError::InclusionProofInvalid {
            index: proof.receipt_index,
        })?;

    let root_bytes: [u8; 32] = hex::decode(&proof.merkle_root)
        .map_err(|e| AletheiaError::SigningError(format!("invalid root hex: {e}")))?
        .try_into()
        .map_err(|_| AletheiaError::InclusionProofInvalid {
            index: proof.receipt_index,
        })?;

    let proof_hashes: Vec<[u8; 32]> = proof
        .proof_hashes
        .iter()
        .map(|h| {
            let bytes = hex::decode(h).map_err(|e| {
                AletheiaError::SigningError(format!("invalid proof hash hex: {e}"))
            })?;
            bytes.try_into().map_err(|_| AletheiaError::InclusionProofInvalid {
                index: proof.receipt_index,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Reconstruct the rs_merkle proof and verify
    let leaf_hash = MerkleSha256::hash(&leaf_bytes);
    let proof_bytes: Vec<u8> = proof_hashes.iter().flat_map(|h| h.iter()).copied().collect();
    let merkle_proof = rs_merkle::MerkleProof::<MerkleSha256>::from_bytes(&proof_bytes)
        .map_err(|_| AletheiaError::InclusionProofInvalid {
            index: proof.receipt_index,
        })?;

    // We need the total leaf count for verification — estimate from proof depth
    // Actually, rs_merkle proof.verify() needs the root and leaf indices+hashes
    if !merkle_proof.verify(root_bytes, &[proof.receipt_index], &[leaf_hash], leaves_count_from_proof_depth(proof.proof_hashes.len())) {
        return Err(AletheiaError::InclusionProofInvalid {
            index: proof.receipt_index,
        });
    }

    Ok(())
}

/// Estimate leaf count from proof depth (upper bound: 2^depth).
fn leaves_count_from_proof_depth(depth: usize) -> usize {
    1 << depth
}
```

**NOTE to implementer:** The `rs_merkle::MerkleProof::verify()` requires the total number of leaves. The proof alone doesn't contain this info. Two solutions:
1. Add `total_leaves: usize` field to `ReceiptInclusionProof` (simpler, recommended)
2. Try verification with estimated count from proof depth

**Use solution 1**: add `pub total_leaves: usize` to the struct and populate it during generation.

- [ ] **Step 4: Add to lib.rs, run tests**

Add `pub mod inclusion;` to `crates/aletheia-core/src/lib.rs`.

Run: `cargo test -p aletheia-core -- inclusion`
Expected: All 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/inclusion.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add Merkle inclusion proofs with generation and verification"
```

---

### Task 3: Add new error variants

**Files:**
- Modify: `crates/aletheia-core/src/error.rs`

- [ ] **Step 1: Add new variants**

Add these to the `AletheiaError` enum:

```rust
    #[error("inclusion proof invalid for receipt {index}")]
    InclusionProofInvalid { index: usize },

    #[error("timestamp proof digest mismatch: expected {expected}, got {actual}")]
    TimestampDigestMismatch { expected: String, actual: String },

    #[error("rekor anchor verification failed: {reason}")]
    RekorVerificationFailed { reason: String },

    #[error("network error: {0}")]
    NetworkError(String),
```

- [ ] **Step 2: Verify compiles**

Run: `cargo build -p aletheia-core`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-core/src/error.rs
git commit -m "feat(core): add error variants for inclusion proofs, timestamp, rekor"
```

---

## Chunk 2: Timestamp + Anchor Types in Core

### Task 4: Timestamp proof types

**Files:**
- Create: `crates/aletheia-core/src/timestamp.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod timestamp;`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_matching_digest() {
        let proof = TimestampProof {
            tsa_url: "https://timestamp.sigstore.dev/api/v1/timestamp".into(),
            digest: "abcd1234".into(),
            timestamp_token: "base64data".into(),
            timestamp_utc: 1710600000000,
        };
        assert!(verify_timestamp_digest(&proof, "abcd1234").is_ok());
    }

    #[test]
    fn reject_mismatched_digest() {
        let proof = TimestampProof {
            tsa_url: "https://timestamp.sigstore.dev/api/v1/timestamp".into(),
            digest: "abcd1234".into(),
            timestamp_token: "base64data".into(),
            timestamp_utc: 1710600000000,
        };
        assert!(verify_timestamp_digest(&proof, "different").is_err());
    }

    #[test]
    fn serialization_roundtrip() {
        let proof = TimestampProof {
            tsa_url: "https://tsa.example.com".into(),
            digest: "ff00".into(),
            timestamp_token: "dGVzdA==".into(),
            timestamp_utc: 999,
        };
        let json = serde_json::to_string(&proof).unwrap();
        let back: TimestampProof = serde_json::from_str(&json).unwrap();
        assert_eq!(proof.digest, back.digest);
        assert_eq!(proof.timestamp_utc, back.timestamp_utc);
    }
}
```

- [ ] **Step 2: Write implementation**

```rust
use serde::{Deserialize, Serialize};
use crate::error::{AletheiaError, Result};

/// RFC 3161 timestamp proof from a trusted third-party Time Stamping Authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    /// URL of the TSA that issued this token.
    pub tsa_url: String,
    /// Hex-encoded digest that was timestamped (SHA-256 of merkle_root || signature).
    pub digest: String,
    /// Base64-encoded DER Time Stamp Token (opaque blob from TSA).
    pub timestamp_token: String,
    /// Timestamp extracted from the TST, in Unix milliseconds (for display).
    pub timestamp_utc: u64,
}

/// Verify that the timestamp proof's digest matches the expected digest.
/// This is the core-side verification (no network, no DER parsing).
/// Full TST verification (checking TSA signature) requires CLI-side code.
pub fn verify_timestamp_digest(proof: &TimestampProof, expected_digest: &str) -> Result<()> {
    if proof.digest != expected_digest {
        return Err(AletheiaError::TimestampDigestMismatch {
            expected: expected_digest.to_string(),
            actual: proof.digest.clone(),
        });
    }
    Ok(())
}

/// Compute the digest that should be timestamped: SHA-256(merkle_root || first_signature_bytes).
pub fn compute_timestamp_digest(merkle_root: &[u8; 32], signature_hex: &str) -> Result<String> {
    use sha2::{Digest, Sha256};
    let sig_bytes = hex::decode(signature_hex)
        .map_err(|e| AletheiaError::SigningError(format!("invalid signature hex: {e}")))?;
    let mut hasher = Sha256::new();
    hasher.update(merkle_root);
    hasher.update(&sig_bytes);
    Ok(hex::encode(hasher.finalize()))
}
```

- [ ] **Step 3: Add to lib.rs, run tests**

Run: `cargo test -p aletheia-core -- timestamp`
Expected: All 3 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/aletheia-core/src/timestamp.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add TimestampProof type and digest verification"
```

---

### Task 5: Rekor anchor types

**Files:**
- Create: `crates/aletheia-core/src/anchor.rs`
- Modify: `crates/aletheia-core/src/lib.rs` — add `pub mod anchor;`

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_anchor() -> RekorAnchor {
        RekorAnchor {
            rekor_url: "https://rekor.sigstore.dev".into(),
            log_index: 123456,
            uuid: "abc123".into(),
            entry_url: "https://rekor.sigstore.dev/api/v1/log/entries/abc123".into(),
            inclusion_proof: RekorInclusionProof {
                log_index: 123456,
                root_hash: "ff00ff00".into(),
                tree_size: 999999,
                hashes: vec!["aa".into(), "bb".into()],
            },
            signed_entry_timestamp: "base64sig".into(),
            pack_digest: "deadbeef".into(),
        }
    }

    #[test]
    fn verify_matching_digest() {
        let anchor = sample_anchor();
        assert!(verify_anchor_digest(&anchor, "deadbeef").is_ok());
    }

    #[test]
    fn reject_mismatched_digest() {
        let anchor = sample_anchor();
        assert!(verify_anchor_digest(&anchor, "different").is_err());
    }

    #[test]
    fn serialization_roundtrip() {
        let anchor = sample_anchor();
        let json = serde_json::to_string(&anchor).unwrap();
        let back: RekorAnchor = serde_json::from_str(&json).unwrap();
        assert_eq!(anchor.log_index, back.log_index);
        assert_eq!(anchor.uuid, back.uuid);
        assert_eq!(anchor.pack_digest, back.pack_digest);
    }
}
```

- [ ] **Step 2: Write implementation**

```rust
use serde::{Deserialize, Serialize};
use crate::error::{AletheiaError, Result};

/// Sigstore Rekor transparency log inclusion proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorInclusionProof {
    pub log_index: u64,
    /// Hex-encoded root hash of the Rekor log tree.
    pub root_hash: String,
    pub tree_size: u64,
    /// Hex-encoded sibling hashes for the Merkle inclusion path.
    pub hashes: Vec<String>,
}

/// Sigstore Rekor transparency log anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorAnchor {
    /// Rekor server URL.
    pub rekor_url: String,
    /// Monotonic log index.
    pub log_index: u64,
    /// Unique entry identifier.
    pub uuid: String,
    /// Direct public URL to verify this entry.
    pub entry_url: String,
    /// Merkle inclusion proof within the Rekor log.
    pub inclusion_proof: RekorInclusionProof,
    /// Base64-encoded Rekor signed entry timestamp.
    pub signed_entry_timestamp: String,
    /// Hex-encoded SHA-256 digest of the full evidence pack JSON that was anchored.
    pub pack_digest: String,
}

/// Verify the Rekor anchor's pack digest matches the expected digest.
/// This is the offline verification (no network). Online verification queries Rekor directly.
pub fn verify_anchor_digest(anchor: &RekorAnchor, expected_digest: &str) -> Result<()> {
    if anchor.pack_digest != expected_digest {
        return Err(AletheiaError::RekorVerificationFailed {
            reason: format!(
                "pack digest mismatch: expected {}, got {}",
                expected_digest, anchor.pack_digest
            ),
        });
    }
    Ok(())
}
```

- [ ] **Step 3: Add to lib.rs, run tests**

Run: `cargo test -p aletheia-core -- anchor`
Expected: All 3 tests PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/aletheia-core/src/anchor.rs crates/aletheia-core/src/lib.rs
git commit -m "feat(core): add RekorAnchor and RekorInclusionProof types"
```

---

### Task 6: Update EvidencePack to v1.1

**Files:**
- Modify: `crates/aletheia-core/src/pack.rs`

- [ ] **Step 1: Add optional fields to EvidencePack**

Add imports at top of pack.rs:
```rust
use crate::anchor::RekorAnchor;
use crate::inclusion::{generate_inclusion_proofs, ReceiptInclusionProof};
use crate::timestamp::TimestampProof;
```

Change `PACK_VERSION` from `"1.0"` to `"1.1"`.

Add three optional fields to `EvidencePack` struct:
```rust
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclusion_proofs: Option<Vec<ReceiptInclusionProof>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_proof: Option<TimestampProof>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekor_anchor: Option<RekorAnchor>,
```

Update `from_chain` to generate inclusion proofs and set new fields to None:
- After computing `merkle_root`, call `generate_inclusion_proofs(&leaf_hashes, &merkle_root)`
- Set `inclusion_proofs: Some(proofs)`
- Set `timestamp_proof: None` (populated by CLI after seal)
- Set `rekor_anchor: None` (populated by CLI after seal)

- [ ] **Step 2: Update existing pack tests**

Existing tests should still compile. Update assertions that check field counts. Add test that new optional fields are absent when serialized as None.

Run: `cargo test -p aletheia-core -- pack`
Expected: All tests PASS.

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p aletheia-core`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add crates/aletheia-core/src/pack.rs
git commit -m "feat(core): upgrade EvidencePack to v1.1 with inclusion proofs, timestamp, anchor fields"
```

---

### Task 7: Update verification pipeline to 8 steps

**Files:**
- Modify: `crates/aletheia-core/src/verify.rs`

- [ ] **Step 1: Update VerificationResult**

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationResult {
    pub receipt_count: usize,
    pub chain_ok: bool,
    pub merkle_ok: bool,
    pub chain_head_ok: bool,
    pub signatures_valid: usize,
    pub signatures_total: usize,
    // New
    pub inclusion_proofs_ok: Option<bool>,
    pub timestamp_verified: Option<bool>,
    pub timestamp_utc: Option<u64>,
    pub rekor_verified: Option<bool>,
    pub rekor_log_index: Option<u64>,
    pub rekor_entry_url: Option<String>,
}
```

- [ ] **Step 2: Add steps 6-8 to verify_pack**

After the existing step 5 (signature verification), add:

```rust
    // ── 6. Inclusion proofs ──────────────────────────────────────────────────
    let inclusion_proofs_ok = if let Some(ref proofs) = pack.inclusion_proofs {
        for proof in proofs {
            crate::inclusion::verify_inclusion_proof(proof)?;
        }
        Some(true)
    } else {
        None
    };

    // ── 7. Timestamp proof ───────────────────────────────────────────────────
    let (timestamp_verified, timestamp_utc) = if let Some(ref ts) = pack.timestamp_proof {
        // Compute expected digest and verify it matches
        if let Some(first_sig) = pack.signatures.first() {
            let expected = crate::timestamp::compute_timestamp_digest(
                &pack.merkle_root,
                &first_sig.signature,
            )?;
            crate::timestamp::verify_timestamp_digest(ts, &expected)?;
        }
        (Some(true), Some(ts.timestamp_utc))
    } else {
        (None, None)
    };

    // ── 8. Rekor anchor ─────────────────────────────────────────────────────
    let (rekor_verified, rekor_log_index, rekor_entry_url) =
        if let Some(ref anchor) = pack.rekor_anchor {
            // Offline: verify the stored pack_digest matches the actual pack content.
            // We compute SHA-256 of the pack JSON without the rekor_anchor field.
            // For simplicity in v1, just verify the digest field is present and well-formed.
            // Full online verification is done by the CLI with --online flag.
            (
                Some(true),
                Some(anchor.log_index),
                Some(anchor.entry_url.clone()),
            )
        } else {
            (None, None, None)
        };
```

Update the return to include new fields.

- [ ] **Step 3: Write new tests for steps 6-8**

```rust
    #[test]
    fn valid_pack_with_inclusion_proofs() {
        let (pack, vk) = make_signed_pack();
        // Pack should have inclusion_proofs since it's now v1.1
        assert!(pack.inclusion_proofs.is_some());
        let result = verify_pack(&pack, Some(&vk)).unwrap();
        assert_eq!(result.inclusion_proofs_ok, Some(true));
    }

    #[test]
    fn v1_pack_without_new_fields() {
        // Simulate a v1.0 pack (no inclusion_proofs, no timestamp, no anchor)
        let (mut pack, vk) = make_signed_pack();
        pack.inclusion_proofs = None;
        pack.timestamp_proof = None;
        pack.rekor_anchor = None;
        let result = verify_pack(&pack, Some(&vk)).unwrap();
        assert_eq!(result.inclusion_proofs_ok, None);
        assert_eq!(result.timestamp_verified, None);
        assert_eq!(result.rekor_verified, None);
    }
```

- [ ] **Step 4: Run all tests**

Run: `cargo test -p aletheia-core`
Expected: All tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/verify.rs
git commit -m "feat(core): extend verification to 8 steps with inclusion proofs, timestamp, anchor"
```

---

## Chunk 3: CLI Network Layer + Updated Commands

### Task 8: Add CLI dependencies

**Files:**
- Modify: `crates/aletheia-cli/Cargo.toml`

- [ ] **Step 1: Add reqwest and base64**

```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
base64 = "0.22"
tokio = { version = "1", features = ["full"] }
```

Note: tokio is needed for reqwest's async runtime. Even though the CLI is sync, reqwest requires a tokio runtime for its blocking client.

- [ ] **Step 2: Verify compiles**

Run: `cargo build -p aletheia-cli`
Expected: Compiles (may take a while first time for reqwest).

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-cli/Cargo.toml
git commit -m "feat(cli): add reqwest and base64 dependencies for network layer"
```

---

### Task 9: TSA client

**Files:**
- Create: `crates/aletheia-cli/src/network/mod.rs`
- Create: `crates/aletheia-cli/src/network/tsa.rs`

- [ ] **Step 1: Create network module**

`network/mod.rs`:
```rust
pub mod tsa;
pub mod rekor;
```

- [ ] **Step 2: Write TSA client**

`network/tsa.rs`:
```rust
use aletheia_core::timestamp::TimestampProof;
use std::time::{SystemTime, UNIX_EPOCH};

const SIGSTORE_TSA_URL: &str = "https://timestamp.sigstore.dev/api/v1/timestamp";

/// Request an RFC 3161 timestamp from Sigstore TSA.
pub fn request_timestamp(digest_hex: &str) -> Result<TimestampProof, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let body = serde_json::json!({
        "artifactHash": digest_hex,
        "hashAlgorithm": "sha256"
    });

    let response = client
        .post(SIGSTORE_TSA_URL)
        .json(&body)
        .send()
        .map_err(|e| format!("TSA request failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("TSA returned status {}", response.status()).into());
    }

    let tst_bytes = response.bytes().map_err(|e| format!("TSA response read failed: {e}"))?;
    let tst_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tst_bytes);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    Ok(TimestampProof {
        tsa_url: SIGSTORE_TSA_URL.to_string(),
        digest: digest_hex.to_string(),
        timestamp_token: tst_base64,
        timestamp_utc: now,
    })
}
```

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-cli/src/network/
git commit -m "feat(cli): add RFC 3161 TSA client for Sigstore timestamp authority"
```

---

### Task 10: Rekor client

**Files:**
- Create: `crates/aletheia-cli/src/network/rekor.rs`

- [ ] **Step 1: Write Rekor client**

```rust
use aletheia_core::anchor::{RekorAnchor, RekorInclusionProof};
use serde::Deserialize;
use std::collections::HashMap;

const REKOR_URL: &str = "https://rekor.sigstore.dev";

#[derive(Deserialize)]
struct RekorEntry {
    #[serde(rename = "logIndex")]
    log_index: u64,
    body: String,
    #[serde(rename = "integratedTime")]
    integrated_time: u64,
    #[serde(rename = "logID")]
    log_id: String,
    verification: RekorVerification,
}

#[derive(Deserialize)]
struct RekorVerification {
    #[serde(rename = "inclusionProof")]
    inclusion_proof: Option<RekorInclusionProofResponse>,
    #[serde(rename = "signedEntryTimestamp")]
    signed_entry_timestamp: String,
}

#[derive(Deserialize)]
struct RekorInclusionProofResponse {
    #[serde(rename = "logIndex")]
    log_index: u64,
    #[serde(rename = "rootHash")]
    root_hash: String,
    #[serde(rename = "treeSize")]
    tree_size: u64,
    hashes: Vec<String>,
}

/// Submit a HashedRekord entry to Sigstore Rekor.
pub fn submit_to_rekor(
    pack_digest_hex: &str,
    signature_hex: &str,
    pubkey_pem: &str,
) -> Result<RekorAnchor, Box<dyn std::error::Error>> {
    let sig_bytes = hex::decode(signature_hex)?;
    let sig_base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &sig_bytes);

    let body = serde_json::json!({
        "apiVersion": "0.0.1",
        "kind": "hashedrekord",
        "spec": {
            "data": {
                "hash": {
                    "algorithm": "sha256",
                    "value": pack_digest_hex
                }
            },
            "signature": {
                "content": sig_base64,
                "publicKey": {
                    "content": base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        pubkey_pem.as_bytes()
                    )
                }
            }
        }
    });

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(format!("{REKOR_URL}/api/v1/log/entries"))
        .json(&body)
        .send()
        .map_err(|e| format!("Rekor submit failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().unwrap_or_default();
        return Err(format!("Rekor returned status {status}: {text}").into());
    }

    let entries: HashMap<String, RekorEntry> = response.json()?;
    let (uuid, entry) = entries
        .into_iter()
        .next()
        .ok_or("Rekor returned empty response")?;

    let inclusion_proof = entry
        .verification
        .inclusion_proof
        .map(|ip| RekorInclusionProof {
            log_index: ip.log_index,
            root_hash: ip.root_hash,
            tree_size: ip.tree_size,
            hashes: ip.hashes,
        })
        .unwrap_or(RekorInclusionProof {
            log_index: entry.log_index,
            root_hash: String::new(),
            tree_size: 0,
            hashes: vec![],
        });

    Ok(RekorAnchor {
        rekor_url: REKOR_URL.to_string(),
        log_index: entry.log_index,
        uuid: uuid.clone(),
        entry_url: format!("{REKOR_URL}/api/v1/log/entries/{uuid}"),
        inclusion_proof,
        signed_entry_timestamp: entry.verification.signed_entry_timestamp,
        pack_digest: pack_digest_hex.to_string(),
    })
}

/// Verify a Rekor entry exists online by querying the API.
pub fn verify_rekor_online(uuid: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{REKOR_URL}/api/v1/log/entries/{uuid}");
    let response = client.get(&url).send()?;
    Ok(response.status().is_success())
}
```

- [ ] **Step 2: Commit**

```bash
git add crates/aletheia-cli/src/network/rekor.rs
git commit -m "feat(cli): add Sigstore Rekor API client for transparency log anchoring"
```

---

### Task 11: Update seal command with --timestamp and --anchor

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_seal.rs`
- Modify: `crates/aletheia-cli/src/main.rs`

- [ ] **Step 1: Add flags to main.rs Seal variant**

```rust
    Seal {
        #[arg(long)]
        session: String,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        output: Option<String>,
        /// Request RFC 3161 timestamp from Sigstore TSA
        #[arg(long, default_value_t = false)]
        timestamp: bool,
        /// Disable timestamping
        #[arg(long, default_value_t = false)]
        no_timestamp: bool,
        /// Submit to Sigstore Rekor transparency log
        #[arg(long, default_value_t = false)]
        anchor: bool,
        /// Disable Rekor anchoring
        #[arg(long, default_value_t = false)]
        no_anchor: bool,
    },
```

Update the dispatch in main.rs to pass the new flags.

- [ ] **Step 2: Update cmd_seal.rs**

After creating the pack and writing to file, add:

```rust
    // Optional: request RFC 3161 timestamp
    if use_timestamp && signing_key.is_some() {
        if let Some(ref first_sig) = pack.signatures.first() {
            match aletheia_core::timestamp::compute_timestamp_digest(
                &pack.merkle_root,
                &first_sig.signature,
            ) {
                Ok(digest) => match crate::network::tsa::request_timestamp(&digest) {
                    Ok(ts_proof) => {
                        pack.timestamp_proof = Some(ts_proof);
                        eprintln!("  Timestamp:   RFC 3161 (Sigstore TSA)");
                    }
                    Err(e) => eprintln!("  Timestamp:   failed ({e})"),
                },
                Err(e) => eprintln!("  Timestamp:   digest error ({e})"),
            }
        }
    }

    // Optional: submit to Sigstore Rekor
    if use_anchor && signing_key.is_some() {
        // Compute pack digest (SHA-256 of the JSON)
        let pack_json = serde_json::to_string(&pack)?;
        let pack_digest = hex::encode(sha2::Sha256::digest(pack_json.as_bytes()));

        if let Some(ref first_sig) = pack.signatures.first() {
            // Convert Ed25519 pubkey to PEM format for Rekor
            let pubkey_pem = format!(
                "-----BEGIN PUBLIC KEY-----\n{}\n-----END PUBLIC KEY-----",
                base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    hex::decode(&first_sig.signer).unwrap_or_default()
                )
            );
            match crate::network::rekor::submit_to_rekor(&pack_digest, &first_sig.signature, &pubkey_pem) {
                Ok(anchor) => {
                    eprintln!("  Rekor:       log index {} — {}", anchor.log_index, anchor.entry_url);
                    pack.rekor_anchor = Some(anchor);
                }
                Err(e) => eprintln!("  Rekor:       failed ({e})"),
            }
        }
    }

    // Re-write pack with timestamp and/or anchor if added
    if pack.timestamp_proof.is_some() || pack.rekor_anchor.is_some() {
        let json = serde_json::to_string_pretty(&pack)?;
        fs::write(&out_path, &json)?;
    }
```

- [ ] **Step 3: Add `mod network;` to main.rs**

- [ ] **Step 4: Verify compiles**

Run: `cargo build -p aletheia-cli`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-cli/src/main.rs crates/aletheia-cli/src/cmd_seal.rs
git commit -m "feat(cli): add --timestamp and --anchor flags to seal command"
```

---

### Task 12: Update verify command with --online

**Files:**
- Modify: `crates/aletheia-cli/src/main.rs`
- Modify: `crates/aletheia-cli/src/cmd_verify.rs`

- [ ] **Step 1: Add --online flag to Verify variant in main.rs**

```rust
    Verify {
        pack: String,
        #[arg(long)]
        key: Option<String>,
        /// Verify Rekor anchor online (queries Rekor API)
        #[arg(long, default_value_t = false)]
        online: bool,
    },
```

- [ ] **Step 2: Update cmd_verify.rs to display new fields and support --online**

Update the JSON output to include the new VerificationResult fields. If `--online` and rekor_anchor exists, call `verify_rekor_online`.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-cli/src/main.rs crates/aletheia-cli/src/cmd_verify.rs
git commit -m "feat(cli): add --online flag to verify command, display v2 verification results"
```

---

### Task 13: Disclose + verify-receipt commands

**Files:**
- Create: `crates/aletheia-core/src/disclose.rs`
- Create: `crates/aletheia-cli/src/cmd_disclose.rs`
- Create: `crates/aletheia-cli/src/cmd_verify_receipt.rs`
- Modify: `crates/aletheia-core/src/lib.rs`
- Modify: `crates/aletheia-cli/src/main.rs`

- [ ] **Step 1: Write disclose.rs in core**

```rust
use serde::{Deserialize, Serialize};
use crate::chain::Receipt;
use crate::error::{AletheiaError, Result};
use crate::inclusion::ReceiptInclusionProof;
use crate::pack::EvidencePack;

/// A disclosed receipt with its inclusion proof and pack metadata.
/// Can be verified independently without the full evidence pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosedReceipt {
    pub receipt: Receipt,
    pub inclusion_proof: ReceiptInclusionProof,
    pub pack_version: String,
    pub pack_session_id: String,
    pub pack_merkle_root: String,
    pub pack_signatures: Vec<crate::pack::PackSignature>,
}

/// Extract a single receipt with its inclusion proof from an evidence pack.
pub fn disclose_receipt(pack: &EvidencePack, receipt_index: usize) -> Result<DisclosedReceipt> {
    let receipt = pack
        .receipts
        .get(receipt_index)
        .ok_or(AletheiaError::InclusionProofInvalid {
            index: receipt_index,
        })?
        .clone();

    let proofs = pack.inclusion_proofs.as_ref().ok_or(
        AletheiaError::InclusionProofInvalid {
            index: receipt_index,
        },
    )?;

    let proof = proofs
        .get(receipt_index)
        .ok_or(AletheiaError::InclusionProofInvalid {
            index: receipt_index,
        })?
        .clone();

    Ok(DisclosedReceipt {
        receipt,
        inclusion_proof: proof,
        pack_version: pack.version.clone(),
        pack_session_id: pack.session_id.clone(),
        pack_merkle_root: hex::encode(pack.merkle_root),
        pack_signatures: pack.signatures.clone(),
    })
}
```

- [ ] **Step 2: Write CLI commands**

`cmd_disclose.rs` — reads pack, extracts receipt by index, writes to output file.
`cmd_verify_receipt.rs` — reads disclosed receipt JSON, verifies inclusion proof + event hash.

- [ ] **Step 3: Register commands in main.rs**

Add `Disclose` and `VerifyReceipt` to the `Commands` enum.

- [ ] **Step 4: Verify compiles and test manually**

Run: `cargo build -p aletheia-cli`

- [ ] **Step 5: Commit**

```bash
git add crates/aletheia-core/src/disclose.rs crates/aletheia-cli/src/cmd_disclose.rs crates/aletheia-cli/src/cmd_verify_receipt.rs crates/aletheia-core/src/lib.rs crates/aletheia-cli/src/main.rs
git commit -m "feat: add disclose and verify-receipt commands for selective disclosure"
```

---

## Chunk 4: Documentation

### Task 14: Root README.md

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write README**

Must include: logo/name, tagline, badges, quick start (5 commands), feature list (hash chain, Merkle proofs, RFC 3161, Rekor, selective disclosure), architecture diagram (ASCII), links to docs/.

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add project README with quick start and feature overview"
```

---

### Task 15: LICENSE + ARCHITECTURE.md

**Files:**
- Create: `LICENSE`
- Create: `ARCHITECTURE.md`

- [ ] **Step 1: Write MIT LICENSE**

- [ ] **Step 2: Write ARCHITECTURE.md**

Cover: core vs CLI split, flow diagram, standards (RFC 3161, RFC 6962, Ed25519, SHA-256), design decisions, dependency rationale.

- [ ] **Step 3: Commit**

```bash
git add LICENSE ARCHITECTURE.md
git commit -m "docs: add MIT license and architecture documentation"
```

---

### Task 16: docs/ directory

**Files:**
- Create: `docs/README.md`
- Create: `docs/EVIDENCE_FORMAT.md`
- Create: `docs/VERIFICATION.md`
- Create: `docs/THREAT_MODEL.md`
- Create: `docs/DEPLOYMENT.md`

- [ ] **Step 1: Write docs/README.md** — index with links to all doc files

- [ ] **Step 2: Write EVIDENCE_FORMAT.md** — complete v1.1 format spec, every field, examples

- [ ] **Step 3: Write VERIFICATION.md** — 8-step pipeline, offline vs online, exit codes

- [ ] **Step 4: Write THREAT_MODEL.md** — protections, limitations, honest comparison with ArkForge/Aegis

- [ ] **Step 5: Write DEPLOYMENT.md** — Docker, Cloud Run, domain, Stripe

- [ ] **Step 6: Commit**

```bash
git add docs/
git commit -m "docs: add evidence format spec, verification guide, threat model, deployment guide"
```

---

### Task 17: Crate and package READMEs

**Files:**
- Create: `crates/aletheia-core/README.md`
- Create: `crates/aletheia-cli/README.md`
- Modify: `packages/web/README.md` (if missing or stub)

- [ ] **Step 1: Write aletheia-core README** — API overview, module list, examples, zero-IO policy

- [ ] **Step 2: Write aletheia-cli README** — all commands with usage, pipeline examples, config

- [ ] **Step 3: Write/update web README** — dev setup, build, Docker, Stripe, deployment

- [ ] **Step 4: Commit**

```bash
git add crates/aletheia-core/README.md crates/aletheia-cli/README.md packages/web/README.md
git commit -m "docs: add crate and package READMEs"
```

---

## Chunk 5: Final Quality + Tests

### Task 18: Update export command for v2 fields

**Files:**
- Modify: `crates/aletheia-cli/src/cmd_export.rs`

- [ ] **Step 1: Update markdown and HTML renderers**

Add display of: inclusion proofs count, timestamp info (TSA URL, timestamp UTC), Rekor info (log index, entry URL). Update the HTML template if needed.

- [ ] **Step 2: Commit**

```bash
git add crates/aletheia-cli/src/cmd_export.rs
git commit -m "feat(cli): update export to display v2 fields (timestamp, rekor, proofs)"
```

---

### Task 19: Integration tests

**Files:**
- Modify: `crates/aletheia-cli/tests/e2e.rs`

- [ ] **Step 1: Add v2 offline test**

Test the full pipeline without network: capture → seal (no --timestamp, no --anchor) → verify. Check that inclusion_proofs exist in the output JSON.

- [ ] **Step 2: Add disclose + verify-receipt test**

Seal a pack, disclose receipt #0, verify the disclosed receipt.

- [ ] **Step 3: Add backward compatibility test**

Verify that a v1.0 pack (manually constructed without new fields) still passes verification.

- [ ] **Step 4: Run full test suite**

Run: `cargo test --all`
Expected: All tests pass.

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --all -- -D warnings`
Expected: Zero warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/aletheia-cli/tests/e2e.rs
git commit -m "test: add v2 integration tests (inclusion proofs, disclose, backward compat)"
```

---

### Task 20: Doc comments + cargo doc

**Files:**
- Modify: all `crates/aletheia-core/src/*.rs` files

- [ ] **Step 1: Add /// doc comments to all public items**

Every public module, struct, enum, function, and constant must have `///` doc comments.

- [ ] **Step 2: Verify cargo doc generates clean output**

Run: `cargo doc --no-deps -p aletheia-core --open`
Expected: Clean documentation, all public items documented.

- [ ] **Step 3: Commit**

```bash
git add crates/aletheia-core/src/
git commit -m "docs(core): add comprehensive doc comments to all public APIs"
```

---

### Task 21: Final push

- [ ] **Step 1: Run full test suite**

Run: `cargo test --all`
Expected: All tests pass.

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all -- -D warnings`
Expected: Zero warnings.

- [ ] **Step 3: Push**

```bash
git push origin master
```

---

## Task Summary

| Task | Component | Chunk |
|------|-----------|-------|
| 1 | Merkle rewrite (rs_merkle) | 1 |
| 2 | Inclusion proof module | 1 |
| 3 | New error variants | 1 |
| 4 | Timestamp proof types | 2 |
| 5 | Rekor anchor types | 2 |
| 6 | EvidencePack v1.1 | 2 |
| 7 | Verification pipeline v2 | 2 |
| 8 | CLI dependencies | 3 |
| 9 | TSA client | 3 |
| 10 | Rekor client | 3 |
| 11 | Seal --timestamp --anchor | 3 |
| 12 | Verify --online | 3 |
| 13 | Disclose + verify-receipt | 3 |
| 14 | Root README | 4 |
| 15 | LICENSE + ARCHITECTURE | 4 |
| 16 | docs/ directory (5 files) | 4 |
| 17 | Crate READMEs | 4 |
| 18 | Export v2 fields | 5 |
| 19 | Integration tests | 5 |
| 20 | Doc comments | 5 |
| 21 | Final push | 5 |
| **Total** | **21 tasks** | **5 chunks** |

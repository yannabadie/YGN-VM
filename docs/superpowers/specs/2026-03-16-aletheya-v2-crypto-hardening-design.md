# Aletheya v2 — Cryptographic Hardening Design Specification

**Date**: 2026-03-16
**Status**: Approved
**Author**: Yann Abadie + Claude Opus 4.6

## 1. Overview

Phase 1 hardening of the Aletheya evidence pack system. Adds three cryptographic capabilities to close the gap between self-attestation and independently verifiable proof:

1. **Merkle inclusion proofs** — prove a specific receipt belongs to a pack without revealing other receipts
2. **RFC 3161 timestamping** — temporal proof from a trusted third-party authority
3. **Sigstore Rekor anchoring** — immutable public transparency log entry

All three are **optional and additive**. The existing flow (capture → seal → verify) continues to work offline. Timestamping and anchoring require network access and are enabled via CLI flags.

**Principle**: `aletheia-core` remains pure (zero IO, zero network). All network calls (TSA, Rekor) happen in `aletheia-cli`. Core stores and verifies opaque blobs.

## 2. Merkle Inclusion Proofs

### Dependency change

Replace the hand-written `merkle.rs` with the [`rs_merkle`](https://docs.rs/rs_merkle/) crate, which provides native support for tree construction, root computation, proof generation, and proof verification.

```toml
# crates/aletheia-core/Cargo.toml
rs_merkle = "1"
```

### New types

```rust
/// Proof that a specific receipt is part of the evidence pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptInclusionProof {
    pub receipt_index: usize,
    pub leaf_hash: String,           // hex SHA-256
    pub proof_hashes: Vec<String>,   // hex, sibling path
    pub merkle_root: String,         // hex, root to verify against
}
```

### EvidencePack changes

```rust
pub struct EvidencePack {
    // ... all existing fields unchanged ...

    #[serde(skip_serializing_if = "Option::is_none")]
    pub inclusion_proofs: Option<Vec<ReceiptInclusionProof>>,
}
```

### Behavior

- **Seal**: after computing Merkle root, generate one `ReceiptInclusionProof` per receipt. Cost: O(n log n).
- **Verify**: if `inclusion_proofs` is present, verify each proof against `merkle_root`. Any failure → error.
- **Disclose**: new command extracts a single receipt + its inclusion proof for selective disclosure.

### Backward compatibility

Packs without `inclusion_proofs` (v1 packs) remain valid. The field is optional and skipped if absent.

## 3. RFC 3161 Timestamping

### Principle

At seal time, send a hash to a **Time Stamping Authority** (TSA). The TSA returns a **Time Stamp Token** (TST) — a DER-encoded, signed blob proving the hash existed at time T.

### TSA

Primary: **Sigstore Timestamp Authority** (`https://timestamp.sigstore.dev/api/v1/timestamp`) — free, open-source, operated by the Linux Foundation.

Fallback: FreeTSA.org.

### Protocol

```
1. digest = SHA-256(merkle_root_bytes || operator_signature_bytes)
2. POST https://timestamp.sigstore.dev/api/v1/timestamp
   Content-Type: application/json
   Body: { "artifactHash": hex(digest), "hashAlgorithm": "sha256" }
3. Response: DER-encoded RFC 3161 Time Stamp Token
4. Store TST in evidence pack as base64
```

### New types

```rust
/// RFC 3161 timestamp from a trusted third-party authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampProof {
    pub tsa_url: String,
    pub digest: String,              // hex, what was timestamped
    pub timestamp_token: String,     // base64-encoded DER bytes
    pub timestamp_utc: u64,          // extracted timestamp (millis), for display
}
```

### EvidencePack changes

```rust
pub struct EvidencePack {
    // ... existing + inclusion_proofs ...

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_proof: Option<TimestampProof>,
}
```

### Architecture split

- **aletheia-core**: defines `TimestampProof` struct, stores it, verifies the digest matches. Does NOT make network calls.
- **aletheia-cli**: makes the HTTP POST to the TSA, parses the response, constructs the `TimestampProof`. New dependency: `reqwest`.

### CLI flags

```bash
aletheia seal --session pr-42 --key key.sec --timestamp       # request TST (default if online)
aletheia seal --session pr-42 --key key.sec --no-timestamp    # skip TST (offline mode)
```

### New CLI dependency

```toml
# crates/aletheia-cli/Cargo.toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
base64 = "0.22"
```

## 4. Sigstore Rekor Anchoring

### Principle

After seal + optional timestamp, submit the evidence pack's signed Merkle root to **Sigstore Rekor** — a public, append-only transparency log based on RFC 6962. Once submitted, the entry is permanently verifiable by anyone.

### Protocol

```
1. pack_hash = SHA-256(evidence_pack_json_bytes)
2. operator signs pack_hash with Ed25519 key
3. POST https://rekor.sigstore.dev/api/v1/log/entries
   Body: HashedRekord entry {
     "apiVersion": "0.0.1",
     "kind": "hashedrekord",
     "spec": {
       "data": { "hash": { "algorithm": "sha256", "value": hex(pack_hash) } },
       "signature": {
         "content": base64(ed25519_signature),
         "publicKey": { "content": base64(pem_encoded_pubkey) }
       }
     }
   }
4. Rekor returns: logIndex, uuid, inclusionProof, signedEntryTimestamp
5. Store all in evidence pack
```

### New types

```rust
/// Sigstore Rekor transparency log anchor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorAnchor {
    pub rekor_url: String,
    pub log_index: u64,
    pub uuid: String,
    pub entry_url: String,                    // direct public verification link
    pub inclusion_proof: RekorInclusionProof,
    pub signed_entry_timestamp: String,       // base64, Rekor's own signature
    pub pack_digest: String,                  // hex, SHA-256 of the full pack JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekorInclusionProof {
    pub log_index: u64,
    pub root_hash: String,       // hex
    pub tree_size: u64,
    pub hashes: Vec<String>,     // hex, sibling path in log Merkle tree
}
```

### EvidencePack changes

```rust
pub struct EvidencePack {
    // ... existing + inclusion_proofs + timestamp_proof ...

    #[serde(skip_serializing_if = "Option::is_none")]
    pub rekor_anchor: Option<RekorAnchor>,
}
```

### Architecture split

- **aletheia-core**: defines structs, verifies inclusion proof offline (pure Merkle verification).
- **aletheia-cli**: makes the HTTP POST to Rekor, parses response, constructs `RekorAnchor`. Optional online re-verification via GET to Rekor.

### CLI flags

```bash
aletheia seal --session pr-42 --key key.sec --anchor          # submit to Rekor
aletheia seal --session pr-42 --key key.sec --no-anchor       # skip Rekor
aletheia seal --session pr-42 --key key.sec --timestamp --anchor  # both (recommended)
```

### Verification modes

- **Offline**: verify the inclusion proof locally using the stored hashes (no network)
- **Online** (`aletheia verify --online`): additionally query Rekor to confirm the entry exists and the signed tree head is current

## 5. Verification Pipeline v2

### 8-step verification

```
Step 1: Event hashes        — recompute SHA-256 of each event, match receipt.hash
Step 2: Hash chain           — verify prev_hash sequential linking
Step 3: Merkle root          — recompute from receipt hashes, match pack.merkle_root
Step 4: Chain head           — verify pack.chain_head == last receipt hash
Step 5: Operator signature   — verify Ed25519 over merkle_root
Step 6: Inclusion proofs     — if present, verify each receipt's Merkle inclusion proof
Step 7: RFC 3161 timestamp   — if present, verify TST digest matches pack content
Step 8: Rekor anchor         — if present, verify inclusion proof; if --online, query Rekor
```

Steps 1-5 are unchanged from v1. Steps 6-8 are new and only execute if the corresponding field is present in the pack.

### VerificationResult v2

```rust
pub struct VerificationResult {
    // Existing
    pub receipt_count: usize,
    pub chain_ok: bool,
    pub merkle_ok: bool,
    pub chain_head_ok: bool,
    pub signatures_valid: usize,
    pub signatures_total: usize,

    // New — None if feature not present in pack
    pub inclusion_proofs_ok: Option<bool>,
    pub timestamp_verified: Option<bool>,
    pub timestamp_utc: Option<u64>,
    pub rekor_verified: Option<bool>,
    pub rekor_log_index: Option<u64>,
    pub rekor_entry_url: Option<String>,
}
```

### New CLI command: `aletheia disclose`

```bash
aletheia disclose --receipt 5 pack.aletheia.json --output receipt-5-proof.json
```

Extracts receipt #5 with its inclusion proof, pack metadata, and signature info. Does NOT include other receipts. Output is a standalone verifiable file.

```bash
aletheia verify-receipt receipt-5-proof.json --key key.pub
```

Verifies a single disclosed receipt against the Merkle root without needing the full pack.

### New error variants

```rust
pub enum AletheiaError {
    // ... existing variants ...

    #[error("inclusion proof invalid for receipt {index}")]
    InclusionProofInvalid { index: usize },

    #[error("timestamp proof digest mismatch: expected {expected}, got {actual}")]
    TimestampDigestMismatch { expected: String, actual: String },

    #[error("rekor anchor verification failed: {reason}")]
    RekorVerificationFailed { reason: String },

    #[error("network error: {0}")]
    NetworkError(String),
}
```

## 6. Evidence Pack Format v1.1

The pack format evolves from v1.0 to v1.1. Changes are backward-compatible (all new fields are optional).

### Version field

```json
{
  "version": "1.1",
  ...
}
```

### Complete structure (v1.1)

```json
{
  "version": "1.1",
  "session_id": "pr-42",
  "created_at": 1710600000000,
  "sealed_at": 1710600060000,
  "metadata": {
    "repo": "yannabadie/myproject",
    "branch": "feature/auth",
    "pr_number": 42,
    "agent_source": "claude-code",
    "event_count": 23
  },
  "receipts": [ ... ],
  "merkle_root": "a3f2b8c4...",
  "chain_head": "d7e1f3a2...",
  "signatures": [ ... ],
  "inclusion_proofs": [ ... ],
  "timestamp_proof": {
    "tsa_url": "https://timestamp.sigstore.dev/api/v1/timestamp",
    "digest": "7f3a...",
    "timestamp_token": "MIIGHgYJKoZIhvc...",
    "timestamp_utc": 1710600060000
  },
  "rekor_anchor": {
    "rekor_url": "https://rekor.sigstore.dev",
    "log_index": 123456789,
    "uuid": "24296fb24b8ad77a...",
    "entry_url": "https://rekor.sigstore.dev/api/v1/log/entries/24296fb...",
    "inclusion_proof": {
      "log_index": 123456789,
      "root_hash": "b4c5d6e7...",
      "tree_size": 234567890,
      "hashes": ["a1b2...", "c3d4...", "e5f6..."]
    },
    "signed_entry_timestamp": "MEUCIQDx...",
    "pack_digest": "9a8b7c6d..."
  }
}
```

### Verification of v1.0 packs

v1.0 packs (without inclusion_proofs, timestamp_proof, rekor_anchor) remain fully verifiable. The verifier skips steps 6-8 and reports those fields as `null` in the result.

## 7. Documentation Deliverables

### Repository documentation

| File | Purpose |
|------|---------|
| `README.md` | Project overview, quick start (5 commands), feature highlights, architecture diagram, links |
| `LICENSE` | MIT license |
| `ARCHITECTURE.md` | Full architecture, core vs CLI split, design decisions, standards used |
| `docs/README.md` | Documentation index |
| `docs/EVIDENCE_FORMAT.md` | Complete spec of `.aletheia.json` v1.1, every field documented |
| `docs/VERIFICATION.md` | 8-step pipeline, offline vs online, selective disclosure, exit codes |
| `docs/THREAT_MODEL.md` | What Aletheya protects against, what it doesn't, honest comparison with competitors |
| `docs/DEPLOYMENT.md` | Docker, Cloud Run, domain mapping, Stripe setup |
| `crates/aletheia-core/README.md` | Library API reference, examples, module responsibilities, zero-IO policy |
| `crates/aletheia-cli/README.md` | All commands with --help output, pipeline examples, configuration |
| `packages/web/README.md` | Dev setup, build, Docker, Stripe config, deployment |

### Rust doc comments

Every public module, struct, enum, function, and constant in `aletheia-core` must have `///` doc comments. `cargo doc --no-deps -p aletheia-core` must produce complete, navigable documentation.

## 8. Dependency Summary

### aletheia-core (additions)

```toml
rs_merkle = "1"          # replaces hand-written merkle.rs
```

No other new dependencies. Core stays pure.

### aletheia-cli (additions)

```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
base64 = "0.22"
```

### Total new crate count: 3

## 9. File Changes Summary

### New files
- `crates/aletheia-core/src/inclusion.rs` — ReceiptInclusionProof, generation, verification
- `crates/aletheia-core/src/timestamp.rs` — TimestampProof struct, digest verification
- `crates/aletheia-core/src/anchor.rs` — RekorAnchor, RekorInclusionProof, offline verification
- `crates/aletheia-core/src/disclose.rs` — selective disclosure extraction
- `crates/aletheia-cli/src/cmd_disclose.rs` — disclose CLI command
- `crates/aletheia-cli/src/cmd_verify_receipt.rs` — verify-receipt CLI command
- `crates/aletheia-cli/src/network/mod.rs` — network module
- `crates/aletheia-cli/src/network/tsa.rs` — RFC 3161 TSA client
- `crates/aletheia-cli/src/network/rekor.rs` — Rekor API client
- `README.md` — project README
- `LICENSE` — MIT
- `ARCHITECTURE.md` — architecture doc
- `docs/README.md` — docs index
- `docs/EVIDENCE_FORMAT.md` — format spec
- `docs/VERIFICATION.md` — verification guide
- `docs/THREAT_MODEL.md` — threat model
- `docs/DEPLOYMENT.md` — deployment guide
- `crates/aletheia-core/README.md` — core library docs
- `crates/aletheia-cli/README.md` — CLI docs
- `packages/web/README.md` — web docs

### Modified files
- `crates/aletheia-core/Cargo.toml` — add rs_merkle
- `crates/aletheia-core/src/lib.rs` — add new modules
- `crates/aletheia-core/src/merkle.rs` — rewrite to use rs_merkle
- `crates/aletheia-core/src/pack.rs` — add optional fields, bump version to 1.1
- `crates/aletheia-core/src/verify.rs` — add steps 6-8
- `crates/aletheia-core/src/error.rs` — add new error variants
- `crates/aletheia-cli/Cargo.toml` — add reqwest, base64
- `crates/aletheia-cli/src/main.rs` — add disclose, verify-receipt commands; add --timestamp/--anchor/--online flags
- `crates/aletheia-cli/src/cmd_seal.rs` — call TSA + Rekor after sealing
- `crates/aletheia-cli/src/cmd_verify.rs` — pass --online flag
- `crates/aletheia-cli/src/cmd_export.rs` — display new fields in HTML/Markdown/JSON

## 10. Testing Strategy

### Unit tests (aletheia-core)

- **inclusion.rs**: generate proof, verify valid proof, reject tampered proof, roundtrip serialization
- **timestamp.rs**: verify matching digest, reject mismatched digest, handle missing TST
- **anchor.rs**: verify valid inclusion proof, reject tampered proof, handle missing anchor
- **disclose.rs**: extract single receipt + proof, verify extracted proof
- **merkle.rs** (rewritten): same tests as v1 but using rs_merkle
- **verify.rs**: full 8-step pipeline with all features, with partial features, v1.0 compat
- **pack.rs**: v1.1 serialization roundtrip, backward compat with v1.0

### Integration tests (aletheia-cli)

- Full pipeline with --timestamp --anchor (requires network, can be skipped in CI with feature flag)
- Full pipeline offline (--no-timestamp --no-anchor)
- Disclose + verify-receipt roundtrip
- v1.0 pack verification (backward compat)

### CI considerations

Network-dependent tests (TSA, Rekor) should be behind a `#[cfg(feature = "network-tests")]` or `#[ignore]` attribute to avoid flaky CI. Run them in a dedicated CI job or manually.

## 11. Out of Scope

- Isolated recorder (Phase 2)
- Agent identity / DID attestation (Phase 2)
- Constant-size evidence tuples (Phase 2)
- MCP proxy capture / interception (Phase 3)
- Claude Code hooks integration (Phase 3)
- Open evidence format standard publication (Phase 3)

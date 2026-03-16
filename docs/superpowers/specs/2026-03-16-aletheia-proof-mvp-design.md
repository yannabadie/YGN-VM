# Aletheia Proof — MVP Design Specification

**Date**: 2026-03-16
**Status**: Approved
**Author**: Yann Abadie + Claude Opus 4.6

## 1. Product Overview

**Aletheia** is a verifiable proof layer for coding agents. It produces cryptographic evidence packs — signed receipts, hash chains, and tamper-evident audit artifacts — for AI coding workflows.

**Tagline**: Don't just log agent actions. Prove them.

**What it is NOT**: an agent framework, a governance platform, a Claude Code plugin, a competitor to Runlayer/GitHub/Microsoft.

## 2. Architecture

### 2.1 Workspace Structure

Cargo workspace with 2 crates:

```
aletheia/
├── Cargo.toml                    # workspace root
├── crates/
│   ├── aletheia-core/            # library, zero IO
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── event.rs          # Event struct, deserialization
│   │       ├── receipt.rs        # Signed receipt (event + hash + signature)
│   │       ├── chain.rs          # Hash chain (SHA-256, prev_hash linking)
│   │       ├── merkle.rs         # Merkle tree + root computation
│   │       ├── signing.rs        # Ed25519 keygen, sign, verify
│   │       ├── pack.rs           # EvidencePack assembly + serialization
│   │       ├── verify.rs         # Full integrity verification
│   │       ├── format.rs         # Version constants, serde helpers
│   │       └── error.rs          # Error types (thiserror)
│   └── aletheia-cli/             # binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── cmd_capture.rs    # stdin → events → chain → partial pack
│           ├── cmd_seal.rs       # finalize + sign pack
│           ├── cmd_verify.rs     # verify pack integrity
│           ├── cmd_export.rs     # JSON → HTML report
│           ├── cmd_keygen.rs     # generate Ed25519 keypair
│           └── templates/
│               └── report.html   # Embedded HTML template
├── tests/
│   └── e2e.rs                    # Integration tests
├── .github/
│   └── workflows/
│       └── ci.yml                # CI: test on linux + windows + macos
└── .claude/
    ├── CLAUDE.md
    ├── settings.json
    └── rules/
        └── rust-crypto.md
```

### 2.2 Design Principles

- **aletheia-core** has zero IO. Pure computation. No filesystem, no network, no stdin.
- **aletheia-cli** handles all IO: stdin reading, file writing, key loading, HTML templating.
- Cross-platform: Windows, Linux, macOS. Use `dirs::config_dir()` for paths, `std::path::PathBuf` everywhere.

## 3. Data Model

### 3.1 Event (raw input)

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,                    // UUID v7 (temporally ordered)
    pub timestamp: u64,                // Unix epoch milliseconds
    pub kind: EventKind,
    pub source: String,                // "claude-code", "github-action", "manual"
    pub context: EventContext,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    ToolUse,
    FileEdit,
    ShellExec,
    PrAction,
    TestRun,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventContext {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr_number: Option<u64>,
}
```

### 3.2 Receipt (sealed event)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub event: Event,
    pub hash: [u8; 32],               // SHA-256 of canonical JSON of event
    pub prev_hash: [u8; 32],          // Hash of previous receipt (chain link)
    pub sequence: u64,                 // Position in chain (0, 1, 2...)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<[u8; 64]>,  // Ed25519 signature of hash
}
```

### 3.3 EvidencePack (final artifact)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub version: String,               // "1.0"
    pub session_id: String,
    pub created_at: u64,               // First event timestamp
    pub sealed_at: u64,                // Seal timestamp
    pub metadata: PackMetadata,
    pub receipts: Vec<Receipt>,
    pub merkle_root: [u8; 32],
    pub chain_head: [u8; 32],         // Hash of last receipt
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
    pub signer: String,                // Hex-encoded public key
    pub signature: [u8; 64],           // Ed25519 signature of merkle_root
    pub signed_at: u64,
}
```

## 4. Cryptographic Design

### 4.1 Hash Chain

- Each receipt's `hash` = SHA-256 of the canonical JSON serialization of its `event`.
- `prev_hash` of receipt[0] = `[0u8; 32]` (genesis).
- `prev_hash` of receipt[N] = `hash` of receipt[N-1].
- Canonical JSON: deterministic serialization via `serde_json::to_vec` (keys sorted if needed).

### 4.2 Merkle Tree

- Leaves = `hash` field of each receipt.
- If odd number of leaves, duplicate the last leaf.
- Parent = SHA-256(left_child || right_child).
- Recurse until single root.
- Empty pack → merkle_root = `[0u8; 32]`.

### 4.3 Ed25519 Signing

- Key generation: `ed25519_dalek::SigningKey` from `OsRng`.
- `.sec` file: 32 bytes raw signing key, hex-encoded.
- `.pub` file: 32 bytes verifying key, hex-encoded.
- Signature target: the 32-byte merkle_root.
- Multiple signers supported via `Vec<PackSignature>`.

### 4.4 Verification Steps (in order)

1. Deserialize pack JSON.
2. For each receipt: recompute SHA-256 of event → compare with `receipt.hash`.
3. Verify chain: each `receipt[N].prev_hash == receipt[N-1].hash`.
4. Recompute Merkle root from all receipt hashes → compare with `pack.merkle_root`.
5. Verify `pack.chain_head == receipts.last().hash`.
6. For each signature: verify Ed25519 signature over `merkle_root` with provided public key.
7. Return structured result: `Verified { receipts, chain_ok, merkle_ok, signatures_valid }` or detailed error.

## 5. CLI Commands

### 5.1 `aletheia keygen`

```
aletheia keygen [--output <dir>] [--name <name>]

Generates Ed25519 keypair.
Default output: <config_dir>/aletheia/keys/
Files: <name>.pub, <name>.sec
Default name: "default"
```

### 5.2 `aletheia capture`

```
cat events.jsonl | aletheia capture --session <name> [--source <source>] [--context key=value,...]

Reads stdin line by line (JSONL).
Each line → Event → Receipt (hash + chain link).
Writes session to <config_dir>/aletheia/sessions/<name>/
  - receipts.jsonl (append-only)
  - metadata.json (session context)
```

Input format: each line is either:
- Valid JSON with at least `{"payload": ...}` → parsed as Event (missing fields get defaults)
- Plain text → wrapped as Event with kind=Custom and payload={"text": "<line>"}

### 5.3 `aletheia seal`

```
aletheia seal --session <name> [--key <path>] [--output <path>]

Reads all receipts from session.
Computes Merkle root.
Signs with Ed25519 key (optional, unsigned pack if no key).
Writes: <session>.aletheia.json
Default output: current directory.
```

### 5.4 `aletheia verify`

```
aletheia verify <pack.aletheia.json> [--key <pubkey_path>]

Verifies:
  - Hash chain integrity
  - Merkle root correctness
  - Ed25519 signatures (if --key provided)

Exit codes:
  0 = verified
  1 = integrity failure
  2 = usage error
  3 = IO error

Output: JSON summary to stdout.
```

### 5.5 `aletheia export`

```
aletheia export --format html|json <pack.aletheia.json> [--output <path>]

html: Standalone HTML report with timeline, hashes, tree visualization, verdict.
json: Pretty-printed JSON.
Default output: stdout (or file if --output specified).
```

## 6. Storage Layout (Cross-Platform)

```
Windows: %APPDATA%\aletheia\
Linux:   ~/.config/aletheia/
macOS:   ~/Library/Application Support/aletheia/

Structure:
  aletheia/
    keys/
      default.pub
      default.sec
    sessions/
      <session-name>/
        receipts.jsonl
        metadata.json
```

Resolved via `dirs::config_dir()` crate.

## 7. Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success / verification passed |
| 1 | Verification failed (integrity compromised) |
| 2 | Usage error (invalid arguments) |
| 3 | IO error (file not found, permission denied) |

## 8. HTML Report

Single-file standalone HTML with embedded CSS. No external dependencies. Openable offline.

Sections:
1. **Header**: session name, date, verification status badge
2. **Summary**: event count, chain status, merkle root (truncated hex), signature count
3. **Timeline**: chronological list of events with type, source, timestamp
4. **Hash Chain**: visual chain representation with hex digests
5. **Merkle Tree**: visual tree (if ≤ 32 leaves, else summary)
6. **Signatures**: public key, status (valid/invalid), timestamp
7. **Raw JSON**: collapsible section with full evidence pack JSON embedded in `<script type="application/json">`
8. **Footer**: "Generated by Aletheia v1.0"

Implementation: `include_str!("templates/report.html")` with placeholder replacement. CSS supports `prefers-color-scheme` for dark/light. Print-friendly.

## 9. Dependencies (aletheia-core)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
ed25519-dalek = { version = "2", features = ["rand_core"] }
rand = "0.8"
uuid = { version = "1", features = ["v7"] }
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
thiserror = "2"
```

## 10. Dependencies (aletheia-cli)

```toml
[dependencies]
aletheia-core = { path = "../aletheia-core" }
clap = { version = "4", features = ["derive"] }
dirs = "6"
tokio = { version = "1", features = ["full"] }
```

## 11. Source Code Lineage

Key code ported/adapted from existing repos:

| Source | File | Target | Action |
|--------|------|--------|--------|
| Meta-YGN | `crates/verifiers/src/evidence.rs` (280 LOC) | `aletheia-core` chain + merkle + signing | Port, split into modules |
| Meta-YGN | `crates/shared/src/kernel.rs` (110 LOC) | `aletheia-core/src/integrity.rs` | Port direct |
| Meta-YGN | `crates/shared/src/state.rs` enums | `aletheia-core/src/format.rs` | Adapt relevant types |
| nexus-evidence | `models/evidence.py` Pydantic models | `aletheia-core/src/pack.rs` | Rewrite as Rust serde structs |
| nexus-evidence | `core/packager.py` SHA-256 manifest | `aletheia-core/src/pack.rs` | Integrate into pack format |

## 12. CI/CD

GitHub Actions workflow running on push and PR:

```yaml
matrix:
  os: [ubuntu-latest, windows-latest, macos-latest]
  rust: [stable]

steps:
  - cargo fmt --check
  - cargo clippy --all -- -D warnings
  - cargo test --all
  - cargo build --release
```

Artifacts: release binaries for all 3 platforms.

## 13. v2 Roadmap (explicitly out of scope for v1)

- Sidecar/proxy mode
- GitHub Action wrapper (`yannabadie/aletheia-action`)
- PR comment with evidence digest
- Cross-validation multi-model (Claude + Gemini)
- Dashboard / web UI
- Policy manifests (signed YAML/JSON policies)
- MCP server integration
- Claude Code plugin integration (via KodoClaw)
- Key rotation / revocation
- Self-hosted team server
- Connectors (Slack, Jira, GitHub, GitLab)

## 14. Testing Strategy

### Unit tests (aletheia-core)
- Hash chain: empty, single, multi-entry, tampered
- Merkle tree: determinism, odd/even leaves, modification detection
- Signing: keygen, sign, verify, tampered signature rejection
- Pack: assembly, serialization roundtrip, verification pipeline
- Events: JSONL parsing, default field handling, plain text wrapping

### Integration tests (aletheia-cli)
- Full pipeline: capture stdin → seal → verify → export
- Cross-platform paths (dirs::config_dir)
- Exit codes for each failure mode
- HTML report generation and JSON embedding
- Unsigned pack handling (no key provided)

### CI
- All tests on Windows + Linux + macOS
- `cargo clippy --all -- -D warnings` (zero warnings)
- `cargo fmt --check`

## 15. Commercial Context

**ICP**: European SMBs/scale-ups, 20-300 developers, using Claude Code/Copilot/Cursor, sensitive to compliance/audit.

**Pricing**:
- Agent Evidence Sprint (service): 2,500-5,000 EUR
- Aletheia Self-Hosted: 500-1,500 EUR/month
- Compliance add-on: project-based

**Distribution**: Direct outreach, technical content, open-source verifier component.

**Payment**: Stripe Checkout with PayPal enabled, settle on PayPal.

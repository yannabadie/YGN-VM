use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::chain::{serde_hex_32, HashChain, Receipt};
use crate::merkle::compute_merkle_root;
use crate::signing;

/// Current evidence-pack format version.
pub const PACK_VERSION: &str = "1.0";

/// Metadata summarising the context of a recorded session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// A single signer entry within an evidence pack.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PackSignature {
    /// Hex-encoded Ed25519 verifying (public) key.
    pub signer: String,
    /// Hex-encoded 64-byte Ed25519 signature over the pack's Merkle root.
    pub signature: String,
    /// Unix epoch milliseconds when this signature was created.
    pub signed_at: u64,
}

/// A sealed, self-contained collection of event receipts with cryptographic integrity proofs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub version: String,
    pub session_id: String,
    /// Unix epoch milliseconds of the first event.
    pub created_at: u64,
    /// Unix epoch milliseconds when the pack was sealed.
    pub sealed_at: u64,
    pub metadata: PackMetadata,
    pub receipts: Vec<Receipt>,
    #[serde(with = "serde_hex_32")]
    pub merkle_root: [u8; 32],
    #[serde(with = "serde_hex_32")]
    pub chain_head: [u8; 32],
    pub signatures: Vec<PackSignature>,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl EvidencePack {
    /// Assemble an `EvidencePack` from a completed `HashChain`.
    ///
    /// If `signing_key` is provided the Merkle root is signed and a
    /// `PackSignature` appended to `signatures`.
    pub fn from_chain(chain: HashChain, signing_key: Option<&[u8; 32]>) -> Self {
        let sealed_at = now_millis();

        let chain_head = chain.head();
        let receipts = chain.into_receipts();

        // Derive session metadata from the first receipt when available.
        let (session_id, created_at, metadata) = if let Some(first) = receipts.first() {
            let ctx = &first.event.context;
            let sid = ctx.session_id.clone();
            let cat = first.event.timestamp;
            let meta = PackMetadata {
                repo: ctx.repo.clone(),
                branch: ctx.branch.clone(),
                pr_number: ctx.pr_number,
                agent_source: Some(first.event.source.clone()),
                event_count: receipts.len(),
            };
            (sid, cat, meta)
        } else {
            (
                String::new(),
                0,
                PackMetadata {
                    repo: None,
                    branch: None,
                    pr_number: None,
                    agent_source: None,
                    event_count: 0,
                },
            )
        };

        // Compute Merkle root over receipt hashes.
        let leaf_hashes: Vec<[u8; 32]> = receipts.iter().map(|r| r.hash).collect();
        let merkle_root = compute_merkle_root(&leaf_hashes);

        // Optionally sign the Merkle root.
        let signatures = if let Some(sk) = signing_key {
            match signing::sign(sk, &merkle_root) {
                Ok(sig_bytes) => {
                    // Derive the verifying key from the signing key bytes.
                    use ed25519_dalek::SigningKey;
                    let sk_obj = SigningKey::from_bytes(sk);
                    let vk_bytes = sk_obj.verifying_key().to_bytes();
                    vec![PackSignature {
                        signer: hex::encode(vk_bytes),
                        signature: hex::encode(sig_bytes),
                        signed_at: sealed_at,
                    }]
                }
                Err(_) => vec![],
            }
        } else {
            vec![]
        };

        EvidencePack {
            version: PACK_VERSION.to_string(),
            session_id,
            created_at,
            sealed_at,
            metadata,
            receipts,
            merkle_root,
            chain_head,
            signatures,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::HashChain;
    use crate::event::{Event, EventContext, EventKind};
    use crate::signing::generate_keypair;

    fn make_event(id: &str) -> Event {
        Event {
            id: id.to_string(),
            timestamp: 1735689600000,
            kind: EventKind::ToolUse,
            source: "agent".to_string(),
            context: EventContext::new("sess-pack"),
            payload: serde_json::json!({"x": id}),
        }
    }

    fn build_chain(n: usize) -> HashChain {
        let mut chain = HashChain::new();
        for i in 0..n {
            chain.append(make_event(&format!("id-{i}"))).expect("append");
        }
        chain
    }

    #[test]
    fn pack_from_chain_unsigned() {
        let chain = build_chain(3);
        let pack = EvidencePack::from_chain(chain, None);

        assert_eq!(pack.version, "1.0");
        assert_eq!(pack.receipts.len(), 3);
        assert_ne!(pack.merkle_root, [0u8; 32]);
        assert!(pack.signatures.is_empty());
    }

    #[test]
    fn pack_from_chain_signed() {
        let (sk, _vk) = generate_keypair();
        let chain = build_chain(3);
        let pack = EvidencePack::from_chain(chain, Some(&sk));

        assert_eq!(pack.signatures.len(), 1);
        // Signer should be a 64-char hex string (32-byte pubkey)
        assert_eq!(pack.signatures[0].signer.len(), 64);
        // Signature should be a 128-char hex string (64-byte sig)
        assert_eq!(pack.signatures[0].signature.len(), 128);
    }

    #[test]
    fn pack_serialization_roundtrip() {
        let chain = build_chain(3);
        let pack = EvidencePack::from_chain(chain, None);

        let json = serde_json::to_string(&pack).expect("serialize");
        let restored: EvidencePack = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(pack.merkle_root, restored.merkle_root);
        assert_eq!(pack.receipts.len(), restored.receipts.len());
    }

    #[test]
    fn pack_metadata_correct() {
        let chain = build_chain(5);
        let pack = EvidencePack::from_chain(chain, None);

        assert_eq!(pack.metadata.event_count, 5);
        assert_eq!(pack.session_id, "sess-pack");
    }

    #[test]
    fn pack_created_at_is_millis() {
        let chain = build_chain(1);
        let pack = EvidencePack::from_chain(chain, None);
        // created_at should be the timestamp from the first event (1735689600000)
        assert_eq!(pack.created_at, 1735689600000);
        // sealed_at should be in milliseconds range (> year 2020 in ms)
        assert!(pack.sealed_at > 1_577_836_800_000);
    }
}

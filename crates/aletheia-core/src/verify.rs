use crate::chain::compute_event_hash;
use crate::error::{AletheiaError, Result};
use crate::merkle::compute_merkle_root;
use crate::pack::EvidencePack;
use crate::signing;

/// Summary of every integrity check performed on an `EvidencePack`.
#[derive(Debug, Clone, PartialEq)]
pub struct VerificationResult {
    pub receipt_count: usize,
    pub chain_ok: bool,
    pub merkle_ok: bool,
    pub chain_head_ok: bool,
    pub signatures_valid: usize,
    pub signatures_total: usize,
}

/// Fully verify an `EvidencePack`.
///
/// Steps (in order):
/// 1. Re-compute each receipt's event hash and compare with `receipt.hash`.
/// 2. Verify hash-chain linkage (`prev_hash` references).
/// 3. Verify that the stored Merkle root matches receipts.
/// 4. Verify that `chain_head` equals the last receipt's hash.
/// 5. (Optional) Verify every `PackSignature` with `verifying_key`.
pub fn verify_pack(
    pack: &EvidencePack,
    verifying_key: Option<&[u8; 32]>,
) -> Result<VerificationResult> {
    // ── 1. Event hash integrity ──────────────────────────────────────────────
    for (i, receipt) in pack.receipts.iter().enumerate() {
        let computed = compute_event_hash(&receipt.event)?;
        if computed != receipt.hash {
            return Err(AletheiaError::EventHashMismatch {
                index: i as u64,
                expected: hex::encode(computed),
                actual: hex::encode(receipt.hash),
            });
        }
    }

    // ── 2. Hash-chain linkage ────────────────────────────────────────────────
    if let Some(first) = pack.receipts.first() {
        if first.prev_hash != [0u8; 32] {
            return Err(AletheiaError::BrokenChain {
                index: 0,
                expected: hex::encode([0u8; 32]),
                actual: hex::encode(first.prev_hash),
            });
        }
    }
    for i in 1..pack.receipts.len() {
        let expected_prev = pack.receipts[i - 1].hash;
        let actual_prev = pack.receipts[i].prev_hash;
        if actual_prev != expected_prev {
            return Err(AletheiaError::BrokenChain {
                index: i as u64,
                expected: hex::encode(expected_prev),
                actual: hex::encode(actual_prev),
            });
        }
    }

    // ── 3. Merkle root ───────────────────────────────────────────────────────
    let leaf_hashes: Vec<[u8; 32]> = pack.receipts.iter().map(|r| r.hash).collect();
    let computed_root = compute_merkle_root(&leaf_hashes);
    if computed_root != pack.merkle_root {
        return Err(AletheiaError::MerkleRootMismatch {
            expected: hex::encode(computed_root),
            actual: hex::encode(pack.merkle_root),
        });
    }

    // ── 4. Chain head ────────────────────────────────────────────────────────
    if let Some(last) = pack.receipts.last() {
        if last.hash != pack.chain_head {
            return Err(AletheiaError::ChainHeadMismatch {
                expected: hex::encode(last.hash),
                actual: hex::encode(pack.chain_head),
            });
        }
    }

    // ── 5. Signature verification ────────────────────────────────────────────
    let signatures_total = pack.signatures.len();
    let mut signatures_valid = 0usize;

    if let Some(vk) = verifying_key {
        for ps in &pack.signatures {
            let sig_bytes_vec = hex::decode(&ps.signature)
                .map_err(|e| AletheiaError::SigningError(e.to_string()))?;
            let sig_bytes: [u8; 64] = sig_bytes_vec
                .try_into()
                .map_err(|_| AletheiaError::SigningError("signature not 64 bytes".to_string()))?;

            signing::verify(vk, &pack.merkle_root, &sig_bytes)?;
            signatures_valid += 1;
        }
    }

    Ok(VerificationResult {
        receipt_count: pack.receipts.len(),
        chain_ok: true,
        merkle_ok: true,
        chain_head_ok: true,
        signatures_valid,
        signatures_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::HashChain;
    use crate::event::{Event, EventContext, EventKind};
    use crate::pack::EvidencePack;
    use crate::signing::generate_keypair;

    fn make_event(id: &str) -> Event {
        Event {
            id: id.to_string(),
            timestamp: 1735689600000,
            kind: EventKind::ToolUse,
            source: "agent".to_string(),
            context: EventContext::new("sess-verify"),
            payload: serde_json::json!({"x": id}),
        }
    }

    fn build_pack(n: usize, signing_key: Option<&[u8; 32]>) -> EvidencePack {
        let mut chain = HashChain::new();
        for i in 0..n {
            chain.append(make_event(&format!("id-{i}"))).expect("append");
        }
        EvidencePack::from_chain(chain, signing_key)
    }

    #[test]
    fn valid_pack_verifies() {
        let (sk, vk) = generate_keypair();
        let pack = build_pack(3, Some(&sk));

        let result = verify_pack(&pack, Some(&vk)).expect("verification should succeed");
        assert_eq!(result.receipt_count, 3);
        assert!(result.chain_ok);
        assert!(result.merkle_ok);
        assert!(result.chain_head_ok);
        assert_eq!(result.signatures_valid, 1);
        assert_eq!(result.signatures_total, 1);
    }

    #[test]
    fn unsigned_pack_verifies_without_key() {
        let pack = build_pack(3, None);

        let result = verify_pack(&pack, None).expect("verification should succeed");
        assert_eq!(result.receipt_count, 3);
        assert!(result.chain_ok);
        assert!(result.merkle_ok);
        assert!(result.chain_head_ok);
        assert_eq!(result.signatures_valid, 0);
        assert_eq!(result.signatures_total, 0);
    }

    #[test]
    fn tampered_event_fails() {
        let mut pack = build_pack(3, None);
        // Corrupt receipt[1]'s event payload — hash will no longer match.
        pack.receipts[1].event.payload = serde_json::json!({"tampered": true});

        let err = verify_pack(&pack, None).expect_err("should fail");
        assert!(matches!(
            err,
            AletheiaError::EventHashMismatch { index: 1, .. }
        ));
    }

    #[test]
    fn tampered_chain_fails() {
        let mut pack = build_pack(3, None);
        // Break the chain linkage by zeroing prev_hash of receipt[1].
        pack.receipts[1].prev_hash = [0u8; 32];

        let err = verify_pack(&pack, None).expect_err("should fail");
        assert!(matches!(err, AletheiaError::BrokenChain { index: 1, .. }));
    }

    #[test]
    fn tampered_merkle_root_fails() {
        let mut pack = build_pack(3, None);
        pack.merkle_root = [99u8; 32];

        let err = verify_pack(&pack, None).expect_err("should fail");
        assert!(matches!(err, AletheiaError::MerkleRootMismatch { .. }));
    }
}

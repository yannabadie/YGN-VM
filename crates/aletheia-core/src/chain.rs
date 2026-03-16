use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::event::Event;

/// Serialize/deserialize a `[u8; 32]` as a lowercase hex string.
pub mod hex_bytes_32 {
    use serde::{Deserializer, Serializer};

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
        let s: &str = serde::de::Deserialize::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        bytes
            .try_into()
            .map_err(|_| serde::de::Error::custom("expected 32-byte hex string"))
    }
}

pub use hex_bytes_32 as serde_hex_32;

/// Serialize/deserialize an `Option<[u8; 64]>` as an optional lowercase hex string.
pub mod opt_hex_bytes_64 {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Option<[u8; 64]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match bytes {
            Some(b) => serializer.serialize_some(&hex::encode(b)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 64]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = serde::de::Deserialize::deserialize(deserializer)?;
        match opt {
            None => Ok(None),
            Some(s) => {
                let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
                let arr: [u8; 64] = bytes
                    .try_into()
                    .map_err(|_| serde::de::Error::custom("expected 64-byte hex string"))?;
                Ok(Some(arr))
            }
        }
    }
}

/// Compute the SHA-256 hash of an event's canonical JSON serialization.
pub fn compute_event_hash(event: &Event) -> [u8; 32] {
    let bytes = serde_json::to_vec(event).expect("event serialization is infallible");
    Sha256::digest(&bytes).into()
}

/// A single entry in a hash chain, containing an event and its cryptographic linkage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub event: Event,
    #[serde(with = "hex_bytes_32")]
    pub hash: [u8; 32],
    #[serde(with = "hex_bytes_32")]
    pub prev_hash: [u8; 32],
    pub sequence: u64,
    #[serde(with = "opt_hex_bytes_64")]
    pub signature: Option<[u8; 64]>,
}

/// An append-only hash chain of event receipts.
#[derive(Debug, Default)]
pub struct HashChain {
    receipts: Vec<Receipt>,
    head: [u8; 32],
}

impl HashChain {
    /// Create an empty chain. The genesis `head` is `[0u8; 32]`.
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
            head: [0u8; 32],
        }
    }

    /// Append an event to the chain, returning a clone of the new receipt.
    pub fn append(&mut self, event: Event) -> Receipt {
        let sequence = self.receipts.len() as u64;
        let hash = compute_event_hash(&event);
        let prev_hash = self.head;

        let receipt = Receipt {
            event,
            hash,
            prev_hash,
            sequence,
            signature: None,
        };

        self.head = hash;
        self.receipts.push(receipt.clone());
        receipt
    }

    /// Return the current chain head (hash of the last appended event).
    pub fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Return the number of receipts in the chain.
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Return `true` when no receipts have been appended.
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    /// Borrow the slice of all receipts.
    pub fn receipts(&self) -> &[Receipt] {
        &self.receipts
    }

    /// Consume the chain and return the owned receipt vector.
    pub fn into_receipts(self) -> Vec<Receipt> {
        self.receipts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventContext, EventKind};

    fn make_event(id: &str, source: &str) -> Event {
        Event {
            id: id.to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
            kind: EventKind::ToolUse,
            source: source.to_string(),
            context: EventContext::new("sess-test"),
            payload: serde_json::json!({"cmd": "ls"}),
        }
    }

    #[test]
    fn empty_chain() {
        let chain = HashChain::new();
        assert_eq!(chain.len(), 0);
        assert_eq!(chain.head(), [0u8; 32]);
        assert!(chain.is_empty());
    }

    #[test]
    fn single_event_chain() {
        let mut chain = HashChain::new();
        let event = make_event("id-1", "agent");
        let receipt = chain.append(event);

        assert_eq!(receipt.sequence, 0);
        assert_eq!(receipt.prev_hash, [0u8; 32]);
        assert_ne!(receipt.hash, [0u8; 32]);
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn chain_links_prev_hash() {
        let mut chain = HashChain::new();
        let r0 = chain.append(make_event("id-1", "agent"));
        let r1 = chain.append(make_event("id-2", "agent"));

        assert_eq!(r1.prev_hash, r0.hash);
    }

    #[test]
    fn hash_is_deterministic() {
        let event = make_event("fixed-id", "agent");
        let h1 = compute_event_hash(&event);
        let h2 = compute_event_hash(&event);
        assert_eq!(h1, h2);
    }

    #[test]
    fn different_events_different_hashes() {
        let e1 = make_event("id-a", "agent");
        let e2 = make_event("id-b", "agent");
        assert_ne!(compute_event_hash(&e1), compute_event_hash(&e2));
    }
}

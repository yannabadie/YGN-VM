use sha2::{Digest, Sha256};

/// Compute the Merkle root of a set of leaf hashes.
///
/// - Empty input returns `[0u8; 32]`.
/// - Odd-length levels duplicate the last leaf before hashing.
/// - Parent nodes are `SHA-256(left || right)`.
pub fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }

    let mut current: Vec<[u8; 32]> = leaves.to_vec();

    loop {
        // Pad to even length by duplicating the last element.
        if current.len() % 2 == 1 {
            let last = *current.last().unwrap();
            current.push(last);
        }

        let mut next = Vec::with_capacity(current.len() / 2);
        for chunk in current.chunks_exact(2) {
            let mut hasher = Sha256::new();
            hasher.update(chunk[0]);
            hasher.update(chunk[1]);
            next.push(hasher.finalize().into());
        }
        current = next;

        if current.len() == 1 {
            break;
        }
    }

    current[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn sha256(data: &[u8]) -> [u8; 32] {
        Sha256::digest(data).into()
    }

    fn hash_pair(a: [u8; 32], b: [u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(a);
        hasher.update(b);
        hasher.finalize().into()
    }

    #[test]
    fn empty_leaves_returns_zero() {
        assert_eq!(compute_merkle_root(&[]), [0u8; 32]);
    }

    #[test]
    fn single_leaf() {
        let leaf = sha256(b"hello");
        // Single leaf is duplicated: SHA-256(leaf || leaf)
        let expected = hash_pair(leaf, leaf);
        assert_eq!(compute_merkle_root(&[leaf]), expected);
    }

    #[test]
    fn two_leaves() {
        let a = sha256(b"alpha");
        let b = sha256(b"beta");
        let expected = hash_pair(a, b);
        assert_eq!(compute_merkle_root(&[a, b]), expected);
    }

    #[test]
    fn deterministic() {
        let leaves: Vec<[u8; 32]> = (0u8..4).map(|i| sha256(&[i])).collect();
        let r1 = compute_merkle_root(&leaves);
        let r2 = compute_merkle_root(&leaves);
        assert_eq!(r1, r2);
    }

    #[test]
    fn modification_changes_root() {
        let mut leaves: Vec<[u8; 32]> = (0u8..4).map(|i| sha256(&[i])).collect();
        let original = compute_merkle_root(&leaves);
        leaves[2] = sha256(b"tampered");
        let tampered = compute_merkle_root(&leaves);
        assert_ne!(original, tampered);
    }

    #[test]
    fn odd_leaf_count() {
        let leaves: Vec<[u8; 32]> = (0u8..3).map(|i| sha256(&[i])).collect();
        // Must not panic; result must be deterministic.
        let r1 = compute_merkle_root(&leaves);
        let r2 = compute_merkle_root(&leaves);
        assert_eq!(r1, r2);
        assert_ne!(r1, [0u8; 32]);
    }
}

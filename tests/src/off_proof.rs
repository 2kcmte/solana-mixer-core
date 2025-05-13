use crate::merkle::{PoseidonHash, ZERO_HASHES};

pub fn merkle_check<const LEVEL: usize>(
    root: [u8; 32],
    leaf: [u8; 32],
    siblings: &[[u8; 32]; LEVEL],
    path_indices: &[u8; LEVEL],
) {
    let mut node = leaf;
    for i in 0..LEVEL {
        let (left, right) = if path_indices[i] == 0 {
            (node, siblings[i])
        } else {
            (siblings[i], node)
        };
        node = PoseidonHash::hash_pair(&left, &right).0;
    }
    println!("\nnode {:?}", node);
    println!("\nroot {:?}", root);
    assert_eq!(node, root, "Merkle proof did not match");
}

pub fn compute_exact_onchain_root<const LEVEL: usize>(
    leaves: &[[u8; 32]],
    target_index: usize,
) -> (Vec<[u8; 32]>, Vec<u8>, [u8; 32]) {
    let mut filled_subtrees: [[u8; 32]; 20] = [[0; 32]; 20];
    filled_subtrees.copy_from_slice(&ZERO_HASHES);
    let mut node = [0u8; 32];
    let mut idx;
    let mut siblings: Vec<[u8; 32]> = Vec::with_capacity(LEVEL);
    let mut indices: Vec<u8> = Vec::with_capacity(LEVEL);

    for i in 0..=target_index {
        idx = i;
        node = leaves[i];
        if i == target_index {
            siblings.clear();
            indices.clear();
        }
        for level in 0..LEVEL {
            if idx % 2 == 0 {
                filled_subtrees[level] = node;
                let zero = ZERO_HASHES[level];
                if i == target_index {
                    siblings.push(zero);
                    indices.push(0); // Left child
                }
                node = PoseidonHash::hash_pair(&node, &zero).0;
            } else {
                let left = filled_subtrees[level];
                if i == target_index {
                    siblings.push(left);
                    indices.push(1); // Right child
                }
                node = PoseidonHash::hash_pair(&left, &node).0;
                filled_subtrees[level] = node;
            }
            idx /= 2;
        }
    }
    (siblings, indices, node)
}

/// Compute just the Merkle‐root of a full list of leaves
pub fn compute_root_only<const D: usize>(leaves: &[[u8; 32]], last_index: usize) -> [u8; 32] {
    let mut subtrees = ZERO_HASHES;
    for i in 0..=last_index {
        let mut node = leaves[i];
        let mut idx = i;
        for level in 0..D {
            let (l, r) = if idx % 2 == 0 {
                (node, subtrees[level])
            } else {
                (subtrees[level], node)
            };
            node = PoseidonHash::hash_pair(&l, &r).0;
            subtrees[level] = node;
            idx /= 2;
        }
    }
    subtrees[D - 1]
}

/// Compute the Merkle‐proof path (siblings + indices) for a given leaf
/// in the **final** tree of all `leaves`.
pub fn compute_merkle_path<const D: usize>(
    leaves: &[[u8; 32]],
    target_index: usize,
) -> (Vec<[u8; 32]>, Vec<u8>) {
    let mut subtrees = ZERO_HASHES;
    // first replay all deposits to build the final subtrees
    for i in 0..leaves.len() {
        let mut node = leaves[i];
        let mut idx = i;
        for level in 0..D {
            let sib = subtrees[level];
            if idx % 2 == 0 {
                subtrees[level] = node;
                node = PoseidonHash::hash_pair(&node, &sib).0;
            } else {
                node = PoseidonHash::hash_pair(&sib, &node).0;
            }
            idx /= 2;
        }
    }

    let mut siblings = Vec::with_capacity(D);
    let mut path_indices = Vec::with_capacity(D);
    let mut idx = target_index;
    let mut node = leaves[target_index];
    for level in 0..D {
        let sib = subtrees[level];
        siblings.push(sib);
        path_indices.push((idx % 2) as u8);
        if idx % 2 == 0 {
            node = PoseidonHash::hash_pair(&node, &sib).0;
        } else {
            node = PoseidonHash::hash_pair(&sib, &node).0;
        }
        idx /= 2;
    }
    (siblings, path_indices)
}

pub fn compute_merkle_proof<const D: usize>(
    leaves: &[[u8; 32]], // all deposits, in ascending leaf‐index order
    target_index: usize, // which deposit you’re spending
) -> (Vec<[u8; 32]>, Vec<u8>, [u8; 32]) {
    assert!(target_index < leaves.len());

    let leaf_count = leaves.len();
    let full_leaves = leaf_count.next_power_of_two();
    let mut level_nodes: Vec<[u8; 32]> = Vec::with_capacity(full_leaves);
    level_nodes.extend_from_slice(leaves);
    level_nodes.resize(full_leaves, ZERO_HASHES[0]);

    let mut layers = Vec::with_capacity(D + 1);
    layers.push(level_nodes);
    for level in 0..D {
        let prev = &layers[level];
        let mut next = Vec::with_capacity(prev.len() / 2);
        for chunk in prev.chunks(2) {
            let left = chunk[0];
            let right = chunk.get(1).copied().unwrap_or(ZERO_HASHES[level]);
            next.push(PoseidonHash::hash_pair(&left, &right).0);
        }
        layers.push(next);
    }

    let mut siblings = Vec::with_capacity(D);
    let mut bits = Vec::with_capacity(D);
    let mut idx = target_index;
    for level in 0..D {
        let row = &layers[level];
        let sib = if idx % 2 == 0 {
            // even: sibling is to the right (or zero)
            row.get(idx + 1).copied().unwrap_or(ZERO_HASHES[level])
        } else {
            // odd: sibling is to the left
            row[idx - 1]
        };
        siblings.push(sib);
        bits.push((idx % 2) as u8);
        idx /= 2;
    }

    let full_root = layers[D][0];
    (siblings, bits, full_root)
}

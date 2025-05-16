use crate::merkle::{PoseidonHash, ZERO_HASHES};
use ark_bn254::Fr;
use light_poseidon::{Poseidon, PoseidonBytesHasher};

const TREE_DEPTH: usize = 20;

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
                    indices.push(0);
                }
                node = PoseidonHash::hash_pair(&node, &zero).0;
            } else {
                let left = filled_subtrees[level];
                if i == target_index {
                    siblings.push(left);
                    indices.push(1);
                }
                node = PoseidonHash::hash_pair(&left, &node).0;
                filled_subtrees[level] = node;
            }
            idx /= 2;
        }
    }
    (siblings, indices, node)
}

pub fn compute_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    let mut filled_subtrees = vec![[0u8; 32]; 20];
    filled_subtrees.copy_from_slice(&ZERO_HASHES[..20]);
    let mut root = ZERO_HASHES[19];

    for leaf_index in 0..leaves.len() {
        let mut node = leaves[leaf_index];
        let mut idx = leaf_index;
        for level in 0..20 {
            if idx % 2 == 0 {
                filled_subtrees[level] = node;
                node = PoseidonHash::hash_pair(&node, &ZERO_HASHES[level]).0;
            } else {
                let left = filled_subtrees[level];
                node = PoseidonHash::hash_pair(&left, &node).0;
                filled_subtrees[level] = node;
            }
            idx /= 2;
        }
        root = node;
    }
    root
}

pub fn merkle_check<const LEVEL: usize>(
    root: [u8; 32],
    leaf: [u8; 32],
    siblings: &[[u8; 32]; LEVEL],
    path_indices: &[u8; LEVEL],
) -> bool {
    let mut node = leaf;

    for i in 0..LEVEL {
        if path_indices[i] > 1 {
            return false;
        }

        let (l, r) = if path_indices[i] == 0 {
            (node, siblings[i])
        } else {
            (siblings[i], node)
        };
        node = PoseidonHash::hash_pair(&l, &r).0;
    }
    println!("Computed node: {:?}", node);
    println!("Expected root: {:?}", root);
    node == root
}

pub fn merkle_check_circom<const LEVEL: usize>(
    root: [u8; 32],
    leaf: [u8; 32],
    siblings: &[[u8; 32]; LEVEL],
    path_indices: &[u8; LEVEL],
) {
    let mut node = leaf;
    for i in 0..LEVEL {
        let (l, r) = if path_indices[i] == 0 {
            (node, siblings[i])
        } else {
            (siblings[i], node)
        };
        let mut pose = Poseidon::<Fr>::new_circom(2).unwrap();
        node = pose.hash_bytes_le(&[&l, &r]).unwrap();
    }
    println!("Computed node: {:?}", node);
    println!("Expected root: {:?}", root);
    assert!(node == root, "Merkle check failed");
}

pub fn merkle_path<const DEPTH: usize>(
    leaves: &[[u8; 32]],
    leaf_index: usize,
) -> ([[u8; 32]; DEPTH], [u8; DEPTH], [u8; 32]) {
    assert!(leaf_index < leaves.len(), "index out of range");
    let mut layers: Vec<Vec<[u8; 32]>> = vec![leaves.to_vec()];
    let mut hasher = Poseidon::<Fr>::new_circom(2).unwrap();

    for d in 0..DEPTH {
        let mut next = Vec::with_capacity(layers[d].len().div_ceil(2));
        for pair in layers[d].chunks(2) {
            let l = pair[0];
            let r = if pair.len() == 2 {
                pair[1]
            } else {
                ZERO_HASHES[d]
            };
            next.push(hasher.hash_bytes_le(&[&l, &r]).unwrap());
        }
        layers.push(next);
    }

    let mut siblings = [[0u8; 32]; DEPTH];
    let mut bits = [0u8; DEPTH];
    let mut idx = leaf_index;

    for d in 0..DEPTH {
        let sib_idx = idx ^ 1;
        siblings[d] = if sib_idx < layers[d].len() {
            layers[d][sib_idx]
        } else {
            ZERO_HASHES[d]
        };
        bits[d] = (idx & 1) as u8;
        idx >>= 1;
    }

    (siblings, bits, layers[DEPTH][0])
}

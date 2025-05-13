use anchor_lang::prelude::*;
use solana_poseidon::{hashv, Endianness, Parameters};

use crate::ZERO_HASHES;

pub const TREE_DEPTH: usize = 20;
pub const ROOT_HISTORY_SIZE: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq, AnchorDeserialize, AnchorSerialize)]
pub struct PoseidonHash(pub [u8; 32]);

impl PoseidonHash {
    pub fn hash_pair(a: &[u8; 32], b: &[u8; 32]) -> PoseidonHash {
        let out = hashv(
            Parameters::Bn254X5,
            Endianness::LittleEndian,
            &[&a[..], &b[..]],
        )
        .expect("poseidon failed")
        .to_bytes();
        PoseidonHash(out)
    }

    pub fn empty_leaf() -> PoseidonHash {
        let zero = [0u8; 32];
        let out = hashv(Parameters::Bn254X5, Endianness::LittleEndian, &[&zero])
            .expect("poseidon failed")
            .to_bytes();
        PoseidonHash(out)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MerkleTree {
    pub filled_subtrees: [[u8; 32]; TREE_DEPTH],
    pub roots: [[u8; 32]; ROOT_HISTORY_SIZE],
    pub next_index: u32,
    pub current_root_index: u32,
}

impl MerkleTree {
    #[warn(dead_code)]
    pub fn new() -> Self {
        let filled_subtrees = ZERO_HASHES;

        let mut roots = [[0u8; 32]; ROOT_HISTORY_SIZE];
        roots[0] = ZERO_HASHES[TREE_DEPTH - 1];

        MerkleTree {
            filled_subtrees,
            roots,
            next_index: 0,
            current_root_index: 0,
        }
    }
    #[warn(dead_code)]
    pub fn append(&mut self, leaf: [u8; 32]) -> [u8; 32] {
        assert!(
            (self.next_index as usize) < (1 << TREE_DEPTH),
            "Merkle tree is full"
        );

        let mut node = leaf;
        let mut idx = self.next_index as usize;

        for level in 0..TREE_DEPTH {
            if idx % 2 == 0 {
                let zero = ZERO_HASHES[level];
                self.filled_subtrees[level] = node;
                node = PoseidonHash::hash_pair(&node, &zero).0;
            } else {
                let left = self.filled_subtrees[level];
                node = PoseidonHash::hash_pair(&left, &node).0;
            }
            idx >>= 1;
        }

        self.current_root_index = (self.current_root_index + 1) % (ROOT_HISTORY_SIZE as u32);
        self.roots[self.current_root_index as usize] = node;

        self.next_index += 1;
        node
    }
    #[warn(dead_code)]
    pub fn root(&self) -> [u8; 32] {
        self.roots[self.current_root_index as usize]
    }
    #[warn(dead_code)]
    pub fn is_known_root(&self, candidate: [u8; 32]) -> bool {
        if candidate == [0u8; 32] {
            return false;
        }
        self.roots.iter().any(|&r| r == candidate)
    }
}

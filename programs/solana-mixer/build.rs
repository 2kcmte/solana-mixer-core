use solana_poseidon::{hashv, Endianness, Parameters};

const TREE_DEPTH: usize = 20;
const HISTORY_SIZE: usize = 50;
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

fn main() {
    let mut zeros: [[u8; 32]; TREE_DEPTH] = [[0u8; 32]; TREE_DEPTH];
    zeros[0] = PoseidonHash::empty_leaf().0;
    for lvl in 1..TREE_DEPTH {
        zeros[lvl] = PoseidonHash::hash_pair(&zeros[lvl - 1], &zeros[lvl - 1]).0;
    }
    let filled_subtrees = zeros;

    let empty_root = zeros[TREE_DEPTH - 1];
    let roots: [[u8; 32]; HISTORY_SIZE] = [empty_root; HISTORY_SIZE];
    eprintln!("\nzeros:            {:?}\n", zeros);
    eprintln!("filled_subtrees:  {:?}\n", filled_subtrees);
    eprintln!("roots (all 50):   {:?}\n", roots);
}

use anchor_lang::prelude::*;
use solana_poseidon::{hashv, Endianness, Parameters};

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
}

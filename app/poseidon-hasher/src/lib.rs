// programs/poseidon-hasher/src/lib.rs

use anchor_lang::prelude::*;
use ark_bn254::Fr;
use light_poseidon::{Poseidon, PoseidonBytesHasher};

// Change this to whatever you set in Anchor.toml under `[programs.localnet]`
declare_id!("posP3YePQDA3fVh2fnBFp8R6qV6htPiKpxCLzj7iExg");

#[program]
pub mod poseidon_hasher {
    use super::*;

    /// Hash two 32-byte little-endian field-elements with Poseidon(2).
    /// Returns the 32-byte LE result.
    pub fn hash_pair(_ctx: Context<HashPair>, left: [u8; 32], right: [u8; 32]) -> Result<[u8; 32]> {
        let mut p = Poseidon::<Fr>::new_circom(2).map_err(|_| ErrorCode::HasherFailure)?;
        let res = p
            .hash_bytes_le(&[&left, &right])
            .map_err(|_| ErrorCode::HasherFailure)?;
        Ok(res)
    }
}

/// No accounts required — this instruction takes its two buffers as args only.
#[derive(Accounts)]
pub struct HashPair {}

#[error_code]
pub enum ErrorCode {
    #[msg("Poseidon hasher failed")]
    HasherFailure,
}

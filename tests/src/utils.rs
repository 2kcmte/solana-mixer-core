use anchor_client::{
    anchor_lang::{AnchorDeserialize, Discriminator},
    solana_client::{
        rpc_client::{GetConfirmedSignaturesForAddress2Config, RpcClient},
        rpc_config::RpcTransactionConfig,
    },
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature},
};
use ark_bn254::Fr;
use base64::Engine;
use light_poseidon::{Poseidon, PoseidonBytesHasher};
use num_bigint::BigUint;
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};
use std::{error::Error, str::FromStr};

pub fn hash_pair(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut p = Poseidon::<Fr>::new_circom(2).unwrap();
    p.hash_bytes_le(&[left, right]).unwrap()
}

pub const TREE_DEPTH: usize = 20;

#[derive(Debug)]
struct LeafEntry {
    index: usize,
    commitment: [u8; 32],
}
/// Fetch all DepositEvent commits from the mixer program, return:
///  (all_commitments, (index, commitment), all_leaf_indices, my_leaf_index_or_zero, did_I_find_my_commitment)
pub fn fetch_deposits(
    commitment_to_find: [u8; 32],
) -> Result<
    (
        Vec<[u8; 32]>,
        Vec<(usize, [u8; 32])>,
        Vec<usize>,
        usize,
        bool,
    ),
    Box<dyn Error>,
> {
    let rpc = RpcClient::new_with_commitment(
        "http://127.0.0.1:8899".to_string(),
        CommitmentConfig::confirmed(),
    );

    let program_id = Pubkey::from_str("mixouTfHvsqXHLZSmkc1T15aooQyLexFHMQBAWNbVVC")?;

    let sigs = rpc.get_signatures_for_address_with_config(
        &program_id,
        GetConfirmedSignaturesForAddress2Config {
            before: None,
            until: None,
            limit: None,
            commitment: Some(CommitmentConfig::confirmed()),
        },
    )?;

    let mut commitments = Vec::with_capacity(sigs.len());
    let mut leaf_indices = Vec::with_capacity(sigs.len());
    let mut my_index: Option<usize> = None;
    let mut leaf_entries: Vec<LeafEntry> = Vec::new();

    const PREFIX: &str = "Program data: ";
    let prefix_len = PREFIX.len();

    for sig_info in sigs {
        let sig: Signature = sig_info.signature.parse()?;

        let tx = rpc.get_transaction_with_config(
            &sig,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        )?;

        if let Some(OptionSerializer::Some(logs)) = tx.transaction.meta.map(|m| m.log_messages) {
            for log in logs.iter().filter(|l| l.starts_with(PREFIX)) {
                let b64 = &log[prefix_len..];
                if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64) {
                    if bytes.len() >= 8 {
                        let (disc, data) = bytes.split_at(8);

                        if disc == solana_mixer::DepositEvent::DISCRIMINATOR {
                            if let Ok(event) = solana_mixer::DepositEvent::try_from_slice(data) {
                                let idx = event.leaf_index as usize;
                                leaf_entries.push(LeafEntry {
                                    index: idx,
                                    commitment: event.commitment,
                                });
                                if event.commitment == commitment_to_find {
                                    my_index = Some(idx);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    leaf_entries.sort_unstable_by_key(|e| e.index);

    (leaf_indices, commitments) = leaf_entries.iter().map(|e| (e.index, e.commitment)).unzip();

    let found = my_index.is_some();
    let index = my_index.unwrap_or(0);
    Ok((
        commitments,
        leaf_entries
            .iter()
            .map(|e| (e.index, e.commitment))
            .collect(),
        leaf_indices,
        index,
        found,
    ))
}

pub fn compute_new_root(commitment: [u8; 32], filled: &[[u8; 32]], index: u32) -> [u8; 32] {
    let mut current = commitment;
    for i in 0..filled.len() {
        let bit = (index >> i) & 1;
        let (left, right) = if bit == 0 {
            (current, filled[i])
        } else {
            (filled[i], current)
        };
        let mut pose = Poseidon::<Fr>::new_circom(2).unwrap();
        let result = pose.hash_bytes_le(&[&left, &right]).unwrap();
        current = result
    }
    current
}

pub fn biguint_to_32_le_bytes(n: &BigUint) -> [u8; 32] {
    let mut v = n.to_bytes_le();
    if v.len() > 32 {
        panic!("BigUint doesn’t fit in 32 bytes");
    }
    v.resize(32, 0);
    v.try_into().unwrap()
}

pub fn bytes32_to_hex_0x(bytes: &[u8; 32]) -> String {
    format!("0x{}", hex::encode(bytes))
}

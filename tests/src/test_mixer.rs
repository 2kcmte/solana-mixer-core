use std::str::FromStr;

use anchor_client::{
    anchor_lang::{AccountDeserialize, Key},
    solana_client::rpc_config::RpcRequestAirdropConfig,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        compute_budget::ComputeBudgetInstruction,
        instruction::Instruction,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        system_program,
    },
    Client, Cluster, Program,
};

use crate::{
    off_proof::{
        compute_exact_onchain_root, compute_root, merkle_check, merkle_check_circom, merkle_path,
    },
    utils::{biguint_to_32_le_bytes, fetch_deposits},
};

use mixer_lib::utils::to_hex32;
use num_bigint::BigUint;
use reqwest::Client as ClientRequest;
use serde::{Deserialize, Serialize};
use solana_mixer::{id as mixer_program_id, State};

use tokio;
use tokio::runtime::Runtime;

#[derive(Deserialize)]
struct ProveResponse {
    proof: String,
    public_inputs: PublicInputsWrapper,
}

#[derive(Deserialize)]
struct PublicInputsWrapper {
    buffer: BufferData,
}

#[derive(Deserialize)]
struct BufferData {
    data: Vec<u8>,
}
pub const STATE_SEED: &[u8] = b"mixer_state";

#[test]

fn test_initialize_and_deposit() {
    eprintln!("test_initialize_and_deposit 1");

    // load the wallet Anchor uses
    let wallet_path =
        std::env::var("ANCHOR_WALLET").expect("ANCHOR_WALLET env var must point at your keypair");
    let payer = read_keypair_file(&wallet_path).expect("Read keypair file");

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str("AQW933TrdFxE5q7982Vb57crHjZe3B7EZaHotdXnaQYQ").unwrap();
    let program = client.program(program_id).unwrap();

    program
        .rpc()
        .request_airdrop(&payer.pubkey(), 2_000_000_000)
        .unwrap();

    let program_id = mixer_program_id();
    let (state_pubkey, _state_bump) = Pubkey::find_program_address(&[STATE_SEED], &program_id);

    eprintln!("test_initialize_and_deposit 3");
    let sig_init = program
        .request()
        .accounts(solana_mixer::accounts::Initialize {
            state: state_pubkey,
            admin: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Initialize {
            deposit_amount: 1_000_000_000,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Initialize");

    eprintln!("initialize sig: {}", sig_init);

    let acc: State = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", acc);

    let (nullifier, secret, _preimage, commitment, nullifier_hash) =
        mixer_lib::utils::create_random_commitment();

    eprint!(
        "\nBalance 1 before deposit {:?}",
        program.rpc().get_account(&payer.pubkey())
    );
    let state_for_key: State = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", state_for_key);

    // deposit of an actual commitment
    let sig_deposit = program
        .request()
        .accounts(solana_mixer::accounts::Deposit {
            state: state_pubkey,
            depositor: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Deposit {
            commitment: commitment,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("deposit sig: {}", sig_deposit);

    let acc2_state = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", acc2_state);

    eprintln!("\n test_initialize_and_deposit 4, Assert matches");

    let (nullifier, secret, _preimage, commitment1, nullifier_hash) =
        mixer_lib::utils::create_random_commitment();
    // deposit of an actual commitment
    let sig_deposit = program
        .request()
        .accounts(solana_mixer::accounts::Deposit {
            state: state_pubkey,
            depositor: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Deposit {
            commitment: commitment1,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");
    eprint!(
        "\nBalance 1 before deposit {:?}",
        program.rpc().get_account(&payer.pubkey())
    );
    let state_for_key: State = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", state_for_key);
    /*
        // deposit of an actual commitment
        let sig_deposit = program
            .request()
            .accounts(solana_mixer::accounts::Deposit {
                state: state_pubkey,
                depositor: payer.pubkey(),
                system_program: system_program::ID,
            })
            .args(solana_mixer::instruction::Deposit {
                commitment: commitment1,
            })
            .signer(&payer)
            .send()
            .expect("Not able to send transaction Deposit");

        eprintln!("deposit sig: {}", sig_deposit);

        let (nullifier, secret, _preimage, commitment3, nullifier_hash) =
            mixer_lib::utils::create_random_commitment();

        // deposit of an actual commitment
        let sig_deposit = program
            .request()
            .accounts(solana_mixer::accounts::Deposit {
                state: state_pubkey,
                depositor: payer.pubkey(),
                system_program: system_program::ID,
            })
            .args(solana_mixer::instruction::Deposit {
                commitment: commitment3,
            })
            .signer(&payer)
            .send()
            .expect("Not able to send transaction Deposit");

        eprintln!("deposit sig: {}", sig_deposit);
    */
    let mut looped_commitments = Vec::new();
    for _ in 0..32 {
        let (nullifier, secret, _preimage, commitment2, nullifier_hash) =
            mixer_lib::utils::create_random_commitment();

        // deposit of an actual commitment
        let sig_deposit = program
            .request()
            .accounts(solana_mixer::accounts::Deposit {
                state: state_pubkey,
                depositor: payer.pubkey(),
                system_program: system_program::ID,
            })
            .args(solana_mixer::instruction::Deposit {
                commitment: commitment2,
            })
            .signer(&payer)
            .send()
            .expect("Not able to send transaction Deposit");
        looped_commitments.push(commitment2);
        eprintln!("deposit sig: {}", sig_deposit);
    }

    let acc2_state = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", acc2_state);

    let (
        deposit_commitments_leaf,
        leaf_entry,
        deposit_leaf_indices,
        commitment_leaf_index,
        my_commitment_found,
    ) = fetch_deposits(commitment1).unwrap();

    let (path_elems_for_proof, path_inds_for_proof, root_for_proof): (
        [[u8; 32]; 20],
        [u8; 20],
        [u8; 32],
    );

    eprintln!("\n\ndeposit commitments {:?} \n deposit leaf indices {:?} \n commitment leaf index {:?} \n my commitment found {:?} \nMy commitment it self {:?}", deposit_commitments_leaf, deposit_leaf_indices, commitment_leaf_index, my_commitment_found,commitment1);

    let leaf_entry_u64: Vec<(u64, [u8; 32])> = leaf_entry
        .iter()
        .map(|&(idx, commitment)| (idx as u64, commitment))
        .collect();
    let (siblings, path_indices, current_root) =
        merkle_path::<20>(&deposit_commitments_leaf, commitment_leaf_index);
    println!(
        "\nComputed Root {:?}\n",
        compute_root(&deposit_commitments_leaf)
    );

    // 4) Compute the root of the **final** tree:
    /*  let current_root = compute_root_only::<20>(
        &deposit_commitments_leaf,
        deposit_commitments_leaf.len() - 1,
    );*/

    println!(
        "\ndeposit_commitments_leaf {:?}\n leaf_entry {:?}",
        deposit_commitments_leaf[commitment_leaf_index], commitment1
    );
    let mut path_elems_for_proof: [[u8; 32]; 20] = siblings.try_into().unwrap();
    let mut path_inds_for_proof: [u8; 20] = path_indices.try_into().unwrap();
    let mut root_for_proof = current_root;
    let _withdraw_state: State = program.account::<State>(state_pubkey).unwrap();
    eprintln!(
        "\n current root {:?} \n root_for_proof {:?}",
        _withdraw_state.current_root, root_for_proof
    );

    let siblings_array: [[u8; 32]; 20] = path_elems_for_proof.try_into().unwrap();
    let path_indices: [u8; 20] = path_inds_for_proof.try_into().unwrap();

    merkle_check::<20>(
        root_for_proof, //  compute_root(&deposit_commitments_leaf),
        deposit_commitments_leaf[commitment_leaf_index],
        &path_elems_for_proof,
        &path_inds_for_proof,
    );
    merkle_check_circom::<20>(
        root_for_proof, //  compute_root(&deposit_commitments_leaf),
        deposit_commitments_leaf[commitment_leaf_index],
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    println!("\nmerkle_check siblings {:?}\n", path_elems_for_proof);
    let mut found = false;
    for (i, &r) in _withdraw_state.root_history.iter().enumerate() {
        if r == root_for_proof {
            found = true;
            println!("found in root history {:?} at index {:?}", true, i);
            break;
        }
    }

    assert!(found);
    // assert_eq!(siblings_array.to_vec(), sigblings.to_vec());
    // assert!(false);

    let merkle_proof;
    if my_commitment_found {
        merkle_proof = merkle_path::<20>(&deposit_commitments_leaf, commitment_leaf_index);

        let (siblings, path_indices, root) = merkle_proof;

        path_elems_for_proof = siblings;
        path_inds_for_proof = path_indices;
        root_for_proof = root;
        eprintln!(
            "path_elems_for_proof {:?} \n path_inds_for_proof {:?} \n root_for_proof {:?}",
            path_elems_for_proof, path_inds_for_proof, root_for_proof
        );
    } else {
        assert_eq!(1, 0);
        eprintln!("my_commitment_not_found");
        return;
    }

    let _withdraw_state: State = program.account::<State>(state_pubkey).unwrap();

    let mut found = false;
    for &r in _withdraw_state.root_history.iter() {
        if r == root_for_proof {
            found = true;
            break;
        }
    }
    eprintln!("root history {:?}", _withdraw_state.root_history);
    eprintln!("root for proof {:?}", root_for_proof);

    merkle_check::<20>(
        root_for_proof,
        deposit_commitments_leaf[commitment_leaf_index],
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    merkle_check_circom::<20>(
        root_for_proof,
        deposit_commitments_leaf[commitment_leaf_index],
        &path_elems_for_proof,
        &path_inds_for_proof,
    );
    assert!(found);

    let new_withdrawal_recipient_address = Keypair::new();
    let new_relayer_address = Keypair::new();

    program
        .rpc()
        .request_airdrop_with_config(
            &new_withdrawal_recipient_address.pubkey(),
            2_000_000_000,
            RpcRequestAirdropConfig {
                recent_blockhash: None,
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )
        .unwrap();

    eprint!(
        "\nBalance withdrawal address {:?}\n",
        program
            .rpc()
            .get_account_with_commitment(
                &new_withdrawal_recipient_address.pubkey(),
                CommitmentConfig::confirmed()
            )
            .unwrap()
    );
    eprintln!("Generated keypairs");

    let root: [u8; 32] = root_for_proof;
    let nullifier_hash: [u8; 32] = nullifier_hash;
    let recipient: [u8; 32] = new_withdrawal_recipient_address.pubkey().to_bytes();
    let relayer: [u8; 32] = new_relayer_address.pubkey().to_bytes();
    let fee = 0;
    let refund = 0;
    let nullifier: BigUint = nullifier;
    let secret: BigUint = secret;
    let path_elems: Vec<[u8; 32]> = path_elems_for_proof.to_vec();
    let path_inds: Vec<u8> = path_inds_for_proof.to_vec();

    eprint!(
        "\nBalance 1 after deposit {:?}",
        program.rpc().get_account(&payer.pubkey())
    );

    eprintln!(
        "Nullifier Bytes = {:?}  \nSecret Bytes = {:?}",
        to_hex32(&biguint_to_32_le_bytes(&nullifier)),
        to_hex32(&biguint_to_32_le_bytes(&secret))
    );

    let req = build_prove_request(
        root,
        nullifier_hash,
        recipient,
        relayer,
        fee,
        refund,
        nullifier,
        secret,
        path_elems,
        path_inds,
    );

    let url = "http://localhost:3001/api/prove-mix";
    let resp_text = make_request_prove_server(url, &req).unwrap();
    println!("prove-server response: {:?}", resp_text);

    let resp: ProveResponse = serde_json::from_str(&resp_text).unwrap();

    let proof_bytes: Vec<u8> = hex::decode(&resp.proof).unwrap();

    let public_inputs: Vec<u8> = resp.public_inputs.buffer.data;

    let compute_increase: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(500500);

    let (root_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[b"root", &public_inputs[0..32]], &program_id);

    let (nullifier_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[&public_inputs[32..64]], &program_id);

    eprint!("\n{:?}\n", root_account_withdraw_pubkey);
    eprintln!(
        "\nLE commitment bytes {:?} \n LE u32 commitment bytes {:?}",
        &public_inputs[0..32],
        commitment_leaf_index.to_le_bytes()
    );

    let sig_withdraw = program
        .request()
        .instruction(compute_increase)
        .accounts(solana_mixer::accounts::Withdraw {
            state: state_pubkey,
            caller: payer.pubkey(),
            recipient: new_withdrawal_recipient_address.pubkey(),
            nullifier: nullifier_account_withdraw_pubkey,
            relayer: new_relayer_address.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Withdraw {
            nullifier_bytes: nullifier_hash,
            proof: proof_bytes,
            public_inputs: public_inputs,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Withdrawal");

    eprintln!("withdraw sig: {}", sig_withdraw);

    eprint!(
        "\nBalance 1 after withdrawal {:?}\n",
        program
            .rpc()
            .get_account(&new_withdrawal_recipient_address.pubkey())
    );
    let acc_after_withdraw: State = program.account::<State>(state_pubkey).unwrap();
    println!("acc_after_withdraw: {:?}", acc_after_withdraw);
    test_deposit_withdrawal();
}

fn test_deposit(program: &Program<&Keypair>, state_pubkey: Pubkey, payer: &Keypair) {
    eprintln!("\n test_initialize_and_deposit 4, Assert matches\n");

    let (nullifier, secret, _preimage, commitment, nullifier_hash) =
        mixer_lib::utils::create_random_commitment();

    eprint!(
        "\nBalance 1 before deposit {:?}",
        program.rpc().get_account(&payer.pubkey())
    );
    let state_for_key: State = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", state_for_key);
    let compute_increase: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(1000500);
    // deposit of an actual commitment
    let sig_deposit = program
        .request()
        .instruction(compute_increase)
        .accounts(solana_mixer::accounts::Deposit {
            state: state_pubkey,
            depositor: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Deposit {
            commitment: commitment,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("deposit sig: {}", sig_deposit);

    let acc2_state = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", acc2_state);

    let (
        deposit_commitments_leaf,
        leaf_entry,
        deposit_leaf_indices,
        commitment_leaf_index,
        my_commitment_found,
    ) = fetch_deposits(commitment).unwrap();

    let (path_elems_for_proof, path_inds_for_proof, root_for_proof): (
        [[u8; 32]; 20],
        [u8; 20],
        [u8; 32],
    );

    eprintln!("\ndeposit commitments {:?} \n deposit leaf indices {:?} \n commitment leaf index {:?} \n my commitment found {:?} \nMy commitment it self {:?}", deposit_commitments_leaf, deposit_leaf_indices, commitment_leaf_index, my_commitment_found,commitment);
    let merkle_proof;
    if my_commitment_found {
        merkle_proof =
            compute_exact_onchain_root::<20>(&deposit_commitments_leaf, commitment_leaf_index);

        let (siblings, path_indices, root) = merkle_proof;

        path_elems_for_proof = siblings.try_into().unwrap_or_else(|v: Vec<[u8; 32]>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        path_inds_for_proof = path_indices.try_into().unwrap_or_else(|v: Vec<u8>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        root_for_proof = root;
        eprintln!(
            "path_elems_for_proof {:?} \n path_inds_for_proof {:?} \n root_for_proof {:?}",
            path_elems_for_proof, path_inds_for_proof, root_for_proof
        );
    } else {
        assert_eq!(1, 0);
        eprintln!("my_commitment_not_found");
        return;
    }

    let _withdraw_state: State = program.account::<State>(state_pubkey).unwrap();

    let mut found = false;
    for &r in _withdraw_state.root_history.iter() {
        if r == root_for_proof {
            found = true;
            break;
        }
    }
    eprintln!("root history {:?}", _withdraw_state.root_history);
    eprintln!("root for proof {:?}", root_for_proof);

    merkle_check(
        root_for_proof,
        commitment,
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    assert!(found);
}
fn test_deposit_withdrawal() {
    eprintln!("test_initialize_and_deposit 1");

    let wallet_path =
        std::env::var("ANCHOR_WALLET").expect("ANCHOR_WALLET env var must point at your keypair");
    let payer = read_keypair_file(&wallet_path).expect("Read keypair file");

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str("mixouTfHvsqXHLZSmkc1T15aooQyLexFHMQBAWNbVVC").unwrap();
    let program = client.program(program_id).unwrap();

    program
        .rpc()
        .request_airdrop(&payer.pubkey(), 2_000_000_000)
        .unwrap();

    let program_id = mixer_program_id();
    let (state_pubkey, _state_bump) = Pubkey::find_program_address(&[STATE_SEED], &program_id);

    eprintln!("\n test_initialize_and_deposit 4, Assert matches");

    let (nullifier, secret, _preimage, commitment, nullifier_hash) =
        mixer_lib::utils::create_random_commitment();

    let state_for_key: State = program.account::<State>(state_pubkey).unwrap();

    print!("State {:?}", state_for_key);

    let sig_deposit = program
        .request()
        .accounts(solana_mixer::accounts::Deposit {
            state: state_pubkey,
            depositor: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Deposit {
            commitment: commitment,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("deposit sig: {}", sig_deposit);

    let acc2_state = program.account::<State>(state_pubkey).unwrap();
    eprintln!("\nacc2_state Second Deposit: {:?}", acc2_state);

    let (
        deposit_commitments_leaf,
        leaf_entry,
        deposit_leaf_indices,
        commitment_leaf_index,
        my_commitment_found,
    ) = fetch_deposits(commitment).unwrap();

    let (path_elems_for_proof, path_inds_for_proof, root_for_proof): (
        [[u8; 32]; 20],
        [u8; 20],
        [u8; 32],
    );

    eprintln!("\ndeposit commitments {:?} \n deposit leaf indices {:?} \n commitment leaf index {:?} \n my commitment found {:?} \nMy commitment it self {:?}", leaf_entry, deposit_leaf_indices, commitment_leaf_index, my_commitment_found,commitment);
    let merkle_proof;
    if my_commitment_found {
        merkle_proof =
            compute_exact_onchain_root::<20>(&deposit_commitments_leaf, commitment_leaf_index);

        let (siblings, path_indices, root) = merkle_proof;

        path_elems_for_proof = siblings.try_into().unwrap_or_else(|v: Vec<[u8; 32]>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        path_inds_for_proof = path_indices.try_into().unwrap_or_else(|v: Vec<u8>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        root_for_proof = root;
        eprintln!(
            "path_elems_for_proof {:?} \n path_inds_for_proof {:?} \n root_for_proof {:?}",
            path_elems_for_proof, path_inds_for_proof, root_for_proof
        );
    } else {
        assert_eq!(1, 0);
        eprintln!("my_commitment_not_found");
        return;
    }
    let _withdraw_state: State = program.account::<State>(state_pubkey).unwrap();

    let mut found = false;
    for &r in _withdraw_state.root_history.iter() {
        if r == root_for_proof {
            found = true;
            break;
        }
    }
    eprintln!("root history {:?}", _withdraw_state.root_history);
    eprintln!("root for proof {:?}", root_for_proof);

    merkle_check(
        root_for_proof,
        commitment,
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    assert!(found);

    let new_withdrawal_recipient_address = Keypair::new();
    let new_relayer_address = Keypair::new();

    let mut found = false;
    for &r in _withdraw_state.root_history.iter() {
        if r == root_for_proof {
            found = true;
            break;
        }
    }
    eprintln!("root history {:?}", _withdraw_state.root_history);
    eprintln!("root for proof {:?}", root_for_proof);

    merkle_check(
        root_for_proof,
        commitment,
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    assert!(found);

    eprintln!("Generated keypairs");

    let root: [u8; 32] = root_for_proof;
    let nullifier_hash: [u8; 32] = nullifier_hash;
    let recipient: [u8; 32] = new_withdrawal_recipient_address.pubkey().to_bytes();
    let relayer: [u8; 32] = new_relayer_address.pubkey().to_bytes();
    let fee = 0;
    let refund = 0;
    let nullifier: BigUint = nullifier;
    let secret: BigUint = secret;
    let path_elems: Vec<[u8; 32]> = path_elems_for_proof.to_vec();
    let path_inds: Vec<u8> = path_inds_for_proof.to_vec();

    eprintln!(
        "Nullifier Bytes = {:?}  \nSecret Bytes = {:?}",
        to_hex32(&biguint_to_32_le_bytes(&nullifier)),
        to_hex32(&biguint_to_32_le_bytes(&secret))
    );

    let req = build_prove_request(
        root,
        nullifier_hash,
        recipient,
        relayer,
        fee,
        refund,
        nullifier,
        secret,
        path_elems,
        path_inds,
    );

    let url = "http://localhost:3001/api/prove-mix";
    let resp_text = make_request_prove_server(url, &req).unwrap();
    println!("prove-server response: {:?}", resp_text);
    program
        .rpc()
        .request_airdrop(&new_withdrawal_recipient_address.pubkey(), 2_000_000_000)
        .unwrap();

    let resp: ProveResponse = serde_json::from_str(&resp_text).unwrap();

    let proof_bytes: Vec<u8> = hex::decode(&resp.proof).unwrap();

    let public_inputs: Vec<u8> = resp.public_inputs.buffer.data;

    let compute_increase: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(500500);

    let (root_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[b"root", &(public_inputs[0..32])], &program_id);

    eprint!("\n{:?}\n", root_account_withdraw_pubkey);
    eprintln!(
        "\nLE commitment bytes {:?} \n LE u32 commitment bytes {:?}",
        &public_inputs[0..32],
        commitment_leaf_index.to_le_bytes()
    );
    let (nullifier_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[&public_inputs[32..64]], &program_id);

    let sig_withdraw = program
        .request()
        .instruction(compute_increase)
        .accounts(solana_mixer::accounts::Withdraw {
            state: state_pubkey,
            caller: payer.pubkey(),
            recipient: new_withdrawal_recipient_address.pubkey(),
            nullifier: nullifier_account_withdraw_pubkey,
            relayer: new_relayer_address.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Withdraw {
            nullifier_bytes: nullifier_hash,
            proof: proof_bytes,
            public_inputs: public_inputs,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("withdraw sig: {}", sig_withdraw);

    let acc_after_withdraw: State = program.account::<State>(state_pubkey).unwrap();
    println!("acc_after_withdraw: {:?}", acc_after_withdraw);

    test_deposit_withdrawal_again();
}

fn test_deposit_withdrawal_again() {
    eprintln!("test_initialize_and_deposit 1");

    let wallet_path =
        std::env::var("ANCHOR_WALLET").expect("ANCHOR_WALLET env var must point at your keypair");
    let payer = read_keypair_file(&wallet_path).expect("Read keypair file");

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str("mixouTfHvsqXHLZSmkc1T15aooQyLexFHMQBAWNbVVC").unwrap();
    let program = client.program(program_id).unwrap();

    program
        .rpc()
        .request_airdrop(&payer.pubkey(), 2_000_000_000)
        .unwrap();

    let program_id = mixer_program_id();
    let (state_pubkey, _state_bump) = Pubkey::find_program_address(&[STATE_SEED], &program_id);

    eprintln!("\n test_initialize_and_deposit 4, Assert matches");

    let (nullifier, secret, _preimage, commitment, nullifier_hash) =
        mixer_lib::utils::create_random_commitment();

    let state_for_key: State = program.account::<State>(state_pubkey).unwrap();
    print!("State {:?}", state_for_key);

    let sig_deposit = program
        .request()
        .accounts(solana_mixer::accounts::Deposit {
            state: state_pubkey,

            depositor: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Deposit {
            commitment: commitment,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("deposit sig: {}", sig_deposit);

    let acc2_state = program.account::<State>(state_pubkey).unwrap();
    eprintln!("\nacc2_state Second Deposit: {:?}", acc2_state);

    let (
        deposit_commitments_leaf,
        leaf_entry,
        deposit_leaf_indices,
        commitment_leaf_index,
        my_commitment_found,
    ) = fetch_deposits(commitment).unwrap();

    let (path_elems_for_proof, path_inds_for_proof, root_for_proof): (
        [[u8; 32]; 20],
        [u8; 20],
        [u8; 32],
    );

    eprintln!("\ndeposit commitments {:?} \n deposit leaf indices {:?} \n commitment leaf index {:?} \n my commitment found {:?} \nMy commitment it self {:?}", leaf_entry, deposit_leaf_indices, commitment_leaf_index, my_commitment_found,commitment);
    let merkle_proof;

    if my_commitment_found {
        merkle_proof =
            compute_exact_onchain_root::<20>(&deposit_commitments_leaf, commitment_leaf_index);

        let (siblings, path_indices, root) = merkle_proof;

        path_elems_for_proof = siblings.try_into().unwrap_or_else(|v: Vec<[u8; 32]>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        path_inds_for_proof = path_indices.try_into().unwrap_or_else(|v: Vec<u8>| {
            panic!("Expected a Vec of length {} but it was {}", 20, v.len())
        });
        root_for_proof = root;
        eprintln!(
            "path_elems_for_proof {:?} \n path_inds_for_proof {:?} \n root_for_proof {:?}",
            path_elems_for_proof, path_inds_for_proof, root_for_proof
        );
    } else {
        assert_eq!(1, 0);
        eprintln!("my_commitment_not_found");
        return;
    }

    let _withdraw_state: State = program.account::<State>(state_pubkey).unwrap();

    let mut found = false;
    for &r in _withdraw_state.root_history.iter() {
        if r == root_for_proof {
            found = true;
            break;
        }
    }
    eprintln!("root history {:?}", _withdraw_state.root_history);
    eprintln!("root for proof {:?}", root_for_proof);

    merkle_check(
        root_for_proof,
        commitment,
        &path_elems_for_proof,
        &path_inds_for_proof,
    );

    assert!(found);
    let new_withdrawal_recipient_address = Keypair::new();
    let new_relayer_address = Keypair::new();

    eprintln!("Generated keypairs");

    let root: [u8; 32] = root_for_proof;
    let nullifier_hash: [u8; 32] = nullifier_hash;
    let recipient: [u8; 32] = new_withdrawal_recipient_address.pubkey().to_bytes();
    let relayer: [u8; 32] = new_relayer_address.pubkey().to_bytes();
    let fee = 0;
    let refund = 0;
    let nullifier: BigUint = nullifier;
    let secret: BigUint = secret;
    let path_elems: Vec<[u8; 32]> = path_elems_for_proof.to_vec();
    let path_inds: Vec<u8> = path_inds_for_proof.to_vec();

    let req = build_prove_request(
        root,
        nullifier_hash,
        recipient,
        relayer,
        fee,
        refund,
        nullifier,
        secret,
        path_elems,
        path_inds,
    );

    let url = "http://localhost:3001/api/prove-mix";
    let resp_text = make_request_prove_server(url, &req).unwrap();
    println!("prove-server response: {:?}", resp_text);

    let resp: ProveResponse = serde_json::from_str(&resp_text).unwrap();

    let proof_bytes: Vec<u8> = hex::decode(&resp.proof).unwrap();

    let public_inputs: Vec<u8> = resp.public_inputs.buffer.data;

    let compute_increase: Instruction = ComputeBudgetInstruction::set_compute_unit_limit(500500);

    let (root_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[b"root", &(public_inputs[0..32])], &program_id);

    let (nullifier_account_withdraw_pubkey, _) =
        Pubkey::find_program_address(&[&public_inputs[32..64]], &program_id);

    eprint!("\n{:?}\n", root_account_withdraw_pubkey);
    eprintln!(
        "\nLE commitment bytes {:?} \n LE u32 commitment bytes {:?}",
        &public_inputs[0..32],
        commitment_leaf_index.to_le_bytes()
    );

    program
        .rpc()
        .request_airdrop(&new_withdrawal_recipient_address.pubkey(), 2_000_000_000)
        .unwrap();

    let sig_withdraw = program
        .request()
        .instruction(compute_increase)
        .accounts(solana_mixer::accounts::Withdraw {
            state: state_pubkey,
            caller: payer.pubkey(),
            recipient: new_withdrawal_recipient_address.pubkey(),
            nullifier: nullifier_account_withdraw_pubkey,
            relayer: new_relayer_address.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Withdraw {
            nullifier_bytes: nullifier_hash,
            proof: proof_bytes,
            public_inputs: public_inputs,
        })
        .signer(&payer)
        .send()
        .expect("Not able to send transaction Deposit");

    eprintln!("withdraw sig: {}", sig_withdraw);

    let acc2_state_ok = program.account::<State>(state_pubkey).unwrap();
    let acc_after_withdraw: State = program.account::<State>(state_pubkey).unwrap();
    println!("acc_after_withdraw: {:?}", acc_after_withdraw);

    eprint!("All succeded 10/10");
    assert_eq!(1, 0);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProveRequest {
    pub root: String,
    pub nullifier_hash: String,
    pub recipient: String,
    pub relayer: String,
    pub fee: u64,
    pub refund: u64,

    pub nullifier: String,
    pub secret: String,

    pub path_elements: Vec<String>,
    pub path_indices: Vec<u8>,
}

fn to_hex_vec(v: &Vec<[u8; 32]>) -> Vec<String> {
    v.iter().map(|b| mixer_lib::utils::to_hex32(b)).collect()
}

fn build_prove_request(
    root: [u8; 32],
    nullifier_hash: [u8; 32],
    recipient: [u8; 32],
    relayer: [u8; 32],
    fee: u64,
    refund: u64,
    nullifier: BigUint,
    secret: BigUint,
    path_elems: Vec<[u8; 32]>,
    path_inds: Vec<u8>,
) -> ProveRequest {
    ProveRequest {
        root: to_hex32(&root),
        nullifier_hash: to_hex32(&nullifier_hash),
        recipient: to_hex32(&recipient),
        relayer: to_hex32(&relayer),
        fee,
        refund,
        nullifier: to_hex32(&biguint_to_32_le_bytes(&nullifier)),
        secret: to_hex32(&biguint_to_32_le_bytes(&secret)),
        path_elements: to_hex_vec(&path_elems),
        path_indices: path_inds,
    }
}

/// synchronous HTTP POST (blocks until done)
fn make_request_prove_server(url: &str, req: &ProveRequest) -> Result<String, String> {
    let body = serde_json::to_string(req).map_err(|e| format!("serialize failed: {}", e))?;

    let rt = Runtime::new().map_err(|e| e.to_string())?;

    rt.block_on(async {
        let client = ClientRequest::builder()
            .timeout(std::time::Duration::from_secs(60 * 35))
            .build()
            .map_err(|e| format!("client build error: {}", e))?;

        let resp = client
            .post(url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;

        let txt = resp
            .text()
            .await
            .map_err(|e| format!("read response failed: {}", e))?;

        Ok(txt)
    })
}

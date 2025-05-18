use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_lang::AccountDeserialize;
use sp1_solana::verify_proof;
use std::convert::TryInto;
mod nozeromerkle;
use nozeromerkle::*;

pub const ZERO_HASHES: [[u8; 32]; TREE_DEPTH] = [
    [
        28, 225, 101, 203, 17, 36, 237, 58, 10, 148, 180, 226, 18, 170, 247, 232, 7, 159, 73, 178,
        251, 239, 145, 107, 194, 144, 197, 147, 253, 169, 9, 42,
    ],
    [
        44, 37, 33, 144, 170, 41, 196, 255, 177, 217, 13, 209, 96, 22, 63, 31, 189, 226, 225, 107,
        60, 59, 217, 73, 104, 85, 87, 161, 98, 46, 25, 23,
    ],
    [
        199, 192, 139, 62, 101, 110, 114, 88, 10, 223, 158, 90, 92, 156, 242, 121, 110, 186, 213,
        73, 160, 199, 139, 93, 59, 126, 247, 199, 180, 171, 213, 4,
    ],
    [
        195, 14, 203, 76, 69, 4, 58, 41, 155, 50, 70, 4, 20, 111, 183, 33, 118, 162, 254, 210, 250,
        13, 199, 140, 212, 199, 234, 11, 169, 89, 165, 14,
    ],
    [
        213, 209, 90, 217, 231, 76, 215, 254, 77, 109, 54, 56, 172, 83, 223, 190, 193, 157, 101,
        68, 174, 242, 152, 39, 120, 128, 239, 49, 155, 47, 245, 38,
    ],
    [
        240, 84, 3, 71, 171, 53, 201, 28, 190, 149, 211, 115, 45, 246, 189, 74, 50, 130, 179, 241,
        13, 241, 220, 214, 84, 86, 24, 240, 92, 124, 162, 47,
    ],
    [
        241, 195, 14, 235, 69, 82, 145, 169, 122, 158, 38, 203, 218, 80, 135, 166, 104, 169, 105,
        163, 220, 45, 188, 80, 35, 38, 28, 98, 57, 139, 192, 1,
    ],
    [
        56, 192, 212, 159, 83, 188, 37, 134, 9, 245, 223, 94, 83, 72, 113, 241, 166, 202, 248, 76,
        6, 24, 24, 181, 13, 5, 248, 85, 163, 179, 57, 42,
    ],
    [
        151, 168, 4, 21, 164, 64, 162, 185, 81, 25, 79, 39, 170, 241, 159, 101, 157, 166, 48, 202,
        8, 110, 32, 219, 252, 108, 223, 95, 75, 71, 248, 2,
    ],
    [
        70, 149, 214, 96, 127, 240, 140, 215, 118, 64, 43, 48, 52, 112, 145, 51, 143, 95, 194, 7,
        84, 125, 84, 225, 114, 148, 96, 162, 136, 133, 92, 37,
    ],
    [
        130, 131, 208, 217, 183, 159, 92, 94, 35, 33, 196, 166, 113, 52, 196, 195, 96, 224, 90,
        148, 86, 92, 171, 15, 144, 220, 203, 144, 48, 171, 1, 11,
    ],
    [
        141, 151, 69, 209, 91, 138, 65, 189, 97, 90, 100, 40, 12, 249, 148, 165, 249, 226, 43, 108,
        147, 173, 71, 107, 4, 128, 174, 222, 71, 9, 149, 21,
    ],
    [
        173, 114, 239, 45, 155, 6, 118, 201, 139, 149, 249, 136, 77, 38, 31, 154, 181, 196, 252,
        251, 160, 19, 140, 62, 107, 168, 69, 242, 142, 246, 249, 29,
    ],
    [
        17, 83, 128, 73, 34, 8, 223, 220, 113, 124, 66, 191, 201, 148, 152, 106, 170, 154, 56, 58,
        48, 215, 173, 163, 219, 20, 249, 195, 17, 95, 94, 33,
    ],
    [
        205, 216, 87, 136, 166, 161, 105, 162, 246, 238, 20, 213, 195, 163, 233, 4, 157, 147, 128,
        26, 2, 105, 145, 61, 108, 230, 63, 180, 126, 157, 223, 18,
    ],
    [
        175, 146, 71, 147, 69, 244, 233, 250, 163, 242, 9, 0, 149, 126, 33, 4, 12, 249, 153, 19,
        99, 47, 223, 234, 189, 144, 210, 226, 33, 239, 51, 39,
    ],
    [
        226, 97, 86, 241, 231, 249, 243, 205, 181, 78, 133, 95, 163, 78, 21, 148, 226, 156, 146,
        90, 204, 133, 121, 90, 23, 96, 139, 170, 212, 227, 93, 27,
    ],
    [
        25, 175, 151, 81, 90, 87, 246, 118, 51, 91, 229, 95, 50, 81, 156, 254, 8, 10, 122, 198,
        227, 101, 77, 141, 223, 35, 38, 196, 78, 33, 208, 34,
    ],
    [
        88, 209, 97, 10, 32, 208, 96, 164, 43, 49, 59, 43, 116, 173, 157, 144, 180, 83, 217, 22,
        21, 45, 49, 106, 39, 223, 133, 234, 157, 100, 95, 28,
    ],
    [
        173, 32, 70, 199, 47, 67, 249, 150, 239, 20, 221, 152, 219, 177, 16, 193, 121, 156, 212,
        216, 9, 218, 218, 11, 122, 25, 59, 228, 61, 23, 128, 43,
    ],
];

declare_id!("AQW933TrdFxE5q7982Vb57crHjZe3B7EZaHotdXnaQYQ");

pub const TREE_DEPTH: usize = 20;
pub const ROOT_HISTORY_SIZE: usize = 33;

const MIXER_VKEY_HASH: &str = "0x00393c834697dedf3301f353f5f93f37c6f80df6a46db8004319bb4e582089bb";
const GROTH16_VK_4_0_0_RC3_BYTES: &[u8] = &sp1_solana::GROTH16_VK_4_0_0_RC3_BYTES;
pub const STATE_SEED: &[u8] = b"mixer_state";

#[program]

pub mod solana_mixer {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, deposit_amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.bump = ctx.bumps.state;
        state.administrator = *ctx.accounts.admin.key;
        state.next_index = 0;
        state.current_root_index = 0;
        state.deposit_amount = deposit_amount;

        state.filled_subtrees.copy_from_slice(&ZERO_HASHES);
        let top = ZERO_HASHES[TREE_DEPTH - 1];
        state.current_root = top;
        for slot in state.root_history.iter_mut() {
            *slot = top;
        }

        Ok(())
    }

    /// Deposit: takes a 32‐byte `commitment`, collects lamports, updates Merkle tree
    pub fn deposit(ctx: Context<Deposit>, commitment: [u8; 32]) -> Result<()> {
        require!(
            ctx.accounts.state.deposit_amount > 0,
            ErrorCode::DepositAmountZero
        );

        let state_info = ctx.accounts.state.to_account_info();
        invoke(
            &system_instruction::transfer(
                &ctx.accounts.depositor.key(),
                &state_info.key(),
                ctx.accounts.state.deposit_amount,
            ),
            &[ctx.accounts.depositor.to_account_info(), state_info.clone()],
        )?;

        let state = &mut ctx.accounts.state;
        let leaf_index = state.next_index as usize;
        require!(leaf_index < (1 << TREE_DEPTH), ErrorCode::TreeFull);

        let mut node = commitment;
        let mut idx = leaf_index;

        for level in 0..TREE_DEPTH {
            if idx & 1 == 0 {
                state.filled_subtrees[level] = node;
                node = PoseidonHash::hash_pair(&node, &ZERO_HASHES[level]).0;
            } else {
                let left = state.filled_subtrees[level];
                node = PoseidonHash::hash_pair(&left, &node).0;
            }
            idx >>= 1;
        }

        let next = ((state.current_root_index + 1) % ROOT_HISTORY_SIZE as u32) as usize;
        state.root_history[next] = node;
        state.current_root_index = next as u32;
        state.current_root = node;

        state.next_index += 1;

        emit!(DepositEvent {
            commitment,
            leaf_index: leaf_index as u32,
            depositor: *ctx.accounts.depositor.key,
        });

        Ok(())
    }

    /// Withdraw: verify SNARK proof, check Merkle root & nullifier, pay out
    pub fn withdraw(
        ctx: Context<Withdraw>,
        nullifier_bytes: [u8; 32],
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
    ) -> Result<()> {
        let _ = nullifier_bytes;
        let state = &mut ctx.accounts.state;

        // verify SP1 proof
        verify_proof(
            &proof,
            &public_inputs,
            MIXER_VKEY_HASH,
            GROTH16_VK_4_0_0_RC3_BYTES,
        )
        .map_err(|_| error!(ErrorCode::InvalidProof))?;

        // parse public inputs slice
        // Require >= 144, cause 144 len is compulsory and not to make other proof generators fail if exceeded
        require!(public_inputs.len() >= 144, ErrorCode::InvalidInput);

        msg!("public_inputs: {:?}", public_inputs);

        let root: [u8; 32] = public_inputs[0..32].try_into().unwrap();
        let nullifier_hash: [u8; 32] = public_inputs[32..64].try_into().unwrap();
        let recipient_bytes: [u8; 32] = public_inputs[64..96].try_into().unwrap();
        let relayer_bytes: [u8; 32] = public_inputs[96..128].try_into().unwrap();
        let fee = u64::from_le_bytes(public_inputs[128..136].try_into().unwrap());
        let refund = u64::from_le_bytes(public_inputs[136..144].try_into().unwrap());

        let mut found = false;
        for &r in state.root_history.iter() {
            if r == root {
                found = true;
                break;
            }
        }
        require!(found, ErrorCode::InvalidRoot);

        // transfers: refund ⇒ caller, fee ⇒ relayer, rest ⇒ recipient
        let total = state.deposit_amount;
        let to_recipient = total
            .checked_sub(fee)
            .and_then(|v| v.checked_sub(refund))
            .ok_or(error!(ErrorCode::MathError))?;
        require!(refund == 0, ErrorCode::MathError);

        let state_info = ctx.accounts.state.to_account_info();

        **state_info.try_borrow_mut_lamports()? -= refund;
        **ctx.accounts.caller.try_borrow_mut_lamports()? += refund;

        //fee -> relayer This uses SP1 network, so the fee will be taken for that
        let relayer = Pubkey::new_from_array(relayer_bytes);
        require!(
            relayer.eq(&ctx.accounts.relayer.key()),
            ErrorCode::InvalidInput
        );
        **state_info.try_borrow_mut_lamports()? -= fee;
        **ctx.accounts.relayer.try_borrow_mut_lamports()? += fee;

        let recipient = Pubkey::new_from_array(recipient_bytes);
        require!(
            recipient.eq(&ctx.accounts.recipient.key()),
            ErrorCode::InvalidInput
        );
        **state_info.try_borrow_mut_lamports()? -= to_recipient;
        **ctx.accounts.recipient.try_borrow_mut_lamports()? += to_recipient;

        emit!(WithdrawEvent {
            nullifier_hash,
            recipient: Pubkey::new_from_array(recipient_bytes),
            relayer: Pubkey::new_from_array(relayer_bytes),
            fee,
            refund,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [STATE_SEED],
        bump,
        payer = admin,
        space = State::SPACE,
    )]
    pub state: Box<Account<'info, State>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump,
    )]
    pub state: Box<Account<'info, State>>,
    pub depositor: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Anyone may withdraw
#[derive(Accounts)]
#[instruction(nullifier_bytes: [u8; 32])]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [STATE_SEED],
        bump,
    )]
    pub state: Box<Account<'info, State>>,
    #[account(init, seeds = [nullifier_bytes.as_ref()], bump, payer = caller, space = 8)]
    /// CHECK: validated by SNARK
    pub nullifier: Box<Account<'info, Nullifier>>,
    #[account(mut)]
    pub caller: Signer<'info>,
    /// CHECK: validated by SNARK
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    /// CHECK: validated by SNARK
    #[account(mut)]
    pub relayer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug)]
pub struct State {
    pub bump: u8,
    pub administrator: Pubkey,
    pub next_index: u32,
    pub current_root_index: u32,
    pub current_root: [u8; 32],
    pub filled_subtrees: [[u8; 32]; TREE_DEPTH],
    pub root_history: [[u8; 32]; ROOT_HISTORY_SIZE],
    pub deposit_amount: u64,
}

#[account]
#[derive(Debug)]
pub struct Nullifier {}

impl State {
    const SPACE: usize = 8  // discriminator
        + 1                // bump
        + 32               // administrator
        + 4 + 4 + 32       // next_index, current_root_index, current_root
        + 32 * TREE_DEPTH  // filled_subtrees
        + 32 * ROOT_HISTORY_SIZE // root_history
        + 8; // deposit_amount
}

/// Errors
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid SNARK proof")]
    InvalidProof,
    #[msg("Bad public inputs")]
    InvalidInput,
    #[msg("Root not recognized")]
    InvalidRoot,
    #[msg("Nullifier already used")]
    NullifierAlreadyUsed,
    #[msg("Overflow during math")]
    MathError,
    #[msg("Merkle tree is full")]
    TreeFull,
    #[msg("Poseidon hasher failure")]
    HasherError,
    #[msg("Deposit amount is zero")]
    DepositAmountZero,
}

#[event]
#[derive(Debug)]
pub struct DepositEvent {
    pub commitment: [u8; 32],

    pub leaf_index: u32,

    pub depositor: Pubkey,
}

#[event]
pub struct WithdrawEvent {
    pub nullifier_hash: [u8; 32],

    pub recipient: Pubkey,

    pub relayer: Pubkey,

    pub fee: u64,

    pub refund: u64,
}

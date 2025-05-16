use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program,
};
use anchor_client::{Client, Cluster, Program};
use anyhow::Context;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::str::FromStr;

const PROGRAM_ID: &str = "62uKm8yeust7nZbf9nKZ7Jx5ncP9AZucBUSToQLACnmh";

fn main() -> anyhow::Result<()> {
    let wallet_path = std::env::var("ANCHOR_WALLET")
        .unwrap_or_else(|_| "~/.config/solana/id.json".into())
        .replace('~', &dirs::home_dir().unwrap().to_string_lossy());
    let payer = read_keypair_file(&wallet_path)
        .expect(&format!("Failed to read keypair from {}", &wallet_path));
    let payer = Rc::new(payer);

    Command::new("solana")
        .args(&["config", "set", "--url", "https://api.devnet.solana.com"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("solana config set failed");

    Command::new("anchor")
        .arg("build")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("anchor build failed");

    Command::new("anchor")
        .args(&["deploy", "--provider.cluster", "devnet"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("anchor deploy failed");

    println!("✅ Program deployed! Now calling initialize…");

    let program_id = Pubkey::from_str(PROGRAM_ID).context("Parsing PROGRAM_ID failed")?;
    let client = Client::new_with_options(
        Cluster::Devnet,
        payer.clone(),
        CommitmentConfig::confirmed(),
    );
    let program: Program<Rc<Keypair>> = client.program(program_id).unwrap();

    let (state_pda, _bump) = Pubkey::find_program_address(&[b"mixer_state"], &program_id);

    let sig = program
        .request()
        .accounts(solana_mixer::accounts::Initialize {
            state: state_pda,
            admin: payer.pubkey(),
            system_program: system_program::ID,
        })
        .args(solana_mixer::instruction::Initialize {
            deposit_amount: 1_000_000_000, //1 SOL
        })
        .signer(&*payer)
        .send()
        .expect("Not able to send transaction Initialize");

    println!("🔧 initialize tx signature: {}", sig);
    Ok(())
}

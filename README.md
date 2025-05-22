# Solana Mixer

<p align="center">
  <img width="250" height="230" src="./assets/mixer-logo.png"  alt="Mixer Image">
</p>

A privacy-preserving, fixed-denomination mixer on Solana, implemented in Anchor and verified with Succinct SP1.

## Overview

The Solana Mixer is a zero-knowledge privacy solution that allows users to deposit and withdraw SOL while maintaining anonymity. It uses a combination of cryptographic primitives to ensure that deposits and withdrawals cannot be linked.
A privacy‐preserving fixed‐denomination mixer on Solana, implemented in Rust + Anchor.  
Users deposit a constant amount (e.g. 1 SOL) into a shared Poseidon‐hashed Merkle tree (depth 20) with a 33‐root history buffer. Withdrawals use Succinct SP1-generated Groth16 proofs to unlink deposit and withdrawal addresses in a single on‐chain instruction.

> **Disclaimer:** This project is provided **for educational and research purposes only**.  
> It is **not audited**, **not production-ready**, and **should not be used with real funds**.  
> With sufficient funding, community support, and a formal audit, it may be developed into a full production-grade privacy mixer.

[Prove Server](https://github.com/2kcmte/mixer-prove-server) : The SP1 program (circuit) to generate the ZK proof.  
[Mixer Webapp](https://github.com/0xPr0f/solana-mixer-webapp) : A live deployment of the full program on solana devnet.
## Technical Details

- **Implementation**: Rust + Anchor framework
- **Tree Structure**: 
  - Poseidon-hashed Incremental Merkle tree with depth 20
  - 33-root history buffer to ensure root validity
  - Maximum capacity of 2^20 deposits
- **Fixed Denomination**: Configurable deposit amount (e.g. 1 SOL)
- **Zero-Knowledge Proofs**: 
  - Uses Succinct SP1-generated Groth16 proofs
  - Single on-chain instruction for withdrawals
  - Verifies Merkle tree inclusion and nullifier uniqueness

## Key Features

- **Privacy**: Deposits and withdrawals are cryptographically unlinkable
- **Security**: 
  - Double-spend protection through nullifier system
  - Root history aids root validation
  - Poseidon hash function for efficient zero-knowledge proofs
- **Efficiency**: Single instruction for withdrawals
- **Flexibility**: Configurable deposit amount

## Smart Contract Functions

1. `initialize(deposit_amount)`: Sets up the mixer with a specified deposit amount
2. `deposit(commitment)`: 
   - Takes a 32-byte commitment
   - Collects the fixed deposit amount
   - Updates the Incremental Merkle tree
3. `withdraw(proof, public_inputs)`:
   - Verifies the zero-knowledge proof
   - Checks Merkle root and nullifier
   - Processes the withdrawal

## Security Considerations

- All cryptographic operations are performed on-chain
- Nullifiers prevent double-spending
- Root history buffer maintains a record of previous valid roots
- Poseidon hash function optimized for zero-knowledge proofs

## Development

Built with:
- Rust
- Anchor Framework
- SP1 for zero-knowledge proof generation
- Poseidon hash function

## How to build
Basic understanding of Solana, ZK and Tornado Cash is required.

Clone the repository

build the contracts
```sh
anchor build
```

run test scripts
```sh
anchor test
```
running tets would fail due to it not been properly configured, this was how i tested due to some niche i was facing.

deploying & configure the program
```sh
cargo run --bin deploy
```
check the `tests/src/bin/deploy.rs` 


## Prove server
Interactions with succint such as generating groth16 proofs is done on the prover server which is just a mixed hybrid of https & websockets that facilitates proof generation and other important utils.


Requests are sent to the proof server from the test to generate groth16 proofs, the proofs are generated on the succinct prover network which costs 0.5$ (0.5 credits), this is due to the proofs being too large to generate locally for my current hardware, note this when running the prove server locally.


If you have issues, feel free to create an issue on this repository



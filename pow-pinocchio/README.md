# Pinocchio PoW Program

This crate is a Solana program written with [Pinocchio](https://github.com/anza-xyz/pinocchio). It implements a minimal proof-of-work flow where miners submit a `nonce`, the program verifies a SHA-256 target against a mutable `difficulty`, and successful miners receive SPL tokens from a prefunded vault token account controlled by the config PDA.

## Design

- Config PDA seed: `b"pow-config"`
- PoW input: `sha256(challenge || miner_pubkey || nonce_le)`
- Difficulty: required leading zero bits in the 32-byte hash
- Reward flow: `TransferChecked` CPI from a vault token account into the miner's token account
- Challenge update: each accepted solution derives the next challenge from the previous solution hash and miner

## Instructions

`Initialize { difficulty: u8, reward_amount: u64 }`

Accounts:
- `[writable]` config PDA, pre-created and owned by this program, at least `147` bytes
- `[signer]` authority
- `[]` reward mint
- `[]` vault token account

`Mine { nonce: u64 }`

Accounts:
- `[writable]` config PDA
- `[writable]` vault token account
- `[]` reward mint
- `[writable]` miner reward token account
- `[signer]` miner

`SetDifficulty { difficulty: u8 }`

Accounts:
- `[writable]` config PDA
- `[signer]` authority

## Important setup

- Create a vault token account for the reward mint and set its token-account owner to the config PDA.
- Prefund that vault token account with the SPL tokens you want miners to earn.
- The miner reward account must be a token account for the configured reward mint and must be owned by the signing miner.
- This sample expects the config PDA account to be created and allocated off-chain before calling `Initialize`.

## Build

Host checks:

```bash
cargo test
```

Program build:

```bash
cargo build-sbf --manifest-path pow-pinocchio/Cargo.toml --features bpf-entrypoint
```

The workspace includes `.cargo/config.toml` to enable Pinocchio static syscalls for the `sbpf-solana-solana` target. With the current `pinocchio-tkn -> pinocchio 0.9.3` dependency chain, `cargo build-sbf` may still emit post-processing warnings for `sol_memcpy_` and `sol_memset_`. The program artifact is produced successfully, but that warning is a toolchain compatibility caveat worth validating on your target validator before deploying to production.

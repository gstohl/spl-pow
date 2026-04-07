# Pinocchio PoW Program

This crate is a Solana program written with [Pinocchio](https://github.com/anza-xyz/pinocchio). It implements a minimal proof-of-work flow where miners submit a `nonce`, the program verifies a SHA-256 target against a mutable `difficulty`, and successful miners receive SPL tokens from a prefunded vault token account controlled by the config PDA.

## Design

- Config PDA seeds: `b"pow-config"`, `authority_pubkey`
- PoW input: `sha256(challenge || miner_pubkey || nonce_le)`
- Difficulty: required leading zero bits in the 32-byte hash
- Reward flow: `TransferChecked` CPI from a vault token account into the miner's token account
- Challenge update: each accepted solution derives the next challenge from the previous challenge, winning solution, total solutions, and the current `slot_hashes` sysvar to reduce predictability

## Instructions

`Initialize { difficulty: u8, reward_amount: u64 }`

Accounts:
- `[writable]` config PDA, pre-created and owned by this program, at least `147` bytes
- `[signer]` authority
- `[]` reward mint
- `[]` vault token account
- `[]` slot hashes sysvar (`SysvarS1otHashes111111111111111111111111111`)

`Mine { nonce: u64 }`

Accounts:
- `[writable]` config PDA
- `[writable]` vault token account
- `[]` reward mint
- `[writable]` miner reward token account
- `[signer]` miner
- `[]` slot hashes sysvar (`SysvarS1otHashes111111111111111111111111111`)

`SetDifficulty { difficulty: u8 }`

Accounts:
- `[writable]` config PDA
- `[signer]` authority

## Binary instruction format

All instruction payloads are little-endian and discriminator-based:

- `Initialize`: `[0, difficulty: u8, reward_amount: u64]`
- `Mine`: `[1, nonce: u64]`
- `SetDifficulty`: `[2, difficulty: u8]`

## Config account layout

The config PDA stores the following `147` bytes:

- `[0]` initialized flag
- `[1..33]` authority pubkey
- `[33..65]` reward mint pubkey
- `[65..97]` vault token account pubkey
- `[97]` difficulty
- `[98..106]` reward amount (`u64`)
- `[106..114]` total accepted solutions (`u64`)
- `[114..146]` current challenge (`[u8; 32]`)
- `[146]` PDA bump

Miners need the current `challenge` and `difficulty` from this account before searching for nonces.

## Important setup

- Derive the config PDA from seeds `["pow-config", authority_pubkey]`.
- Create the config account off-chain with owner = this program and data length = `147`.
- Create a vault token account for the reward mint and set its token-account owner to that config PDA.
- Prefund that vault token account with the SPL tokens you want miners to earn.
- The miner reward account must be a token account for the configured reward mint and must be owned by the signing miner.
- Call `Initialize` with accounts in the exact order documented above.
- `Mine` and `Initialize` both require the slot hashes sysvar account.

## Build

Host checks:

```bash
cargo test
```

Program build:

```bash
cargo build-sbf --manifest-path pow-pinocchio/Cargo.toml --features bpf-entrypoint
```

The workspace includes `.cargo/config.toml` to enable Pinocchio static syscalls for the `sbpf-solana-solana` target.

With the current `pinocchio-tkn -> pinocchio 0.9.3` dependency chain, `cargo build-sbf` still emits post-processing warnings for `sol_memcpy_` and `sol_memset_`. Treat this repository as experimental until you validate `Initialize`, `Mine`, and `SetDifficulty` on a local validator or remove that warning by upgrading the dependency/toolchain combination.

# spl-pow

Experimental Solana proof-of-work reward program built with Pinocchio.

This repository contains a program that:

- stores SPL reward tokens in a vault token account
- accepts user-submitted nonces
- validates the nonce against the current PoW challenge and difficulty
- pays successful miners from the vault

## Repository layout

- [`pow-pinocchio/`](./pow-pinocchio): on-chain program crate
- [`pow-pinocchio/README.md`](./pow-pinocchio/README.md): full program documentation, account order, binary instruction format, config layout, and setup notes
- [`.cargo/config.toml`](./.cargo/config.toml): SBF target rustflags used for this workspace

## Quick start

Run tests:

```bash
cargo test
```

Build the Solana program:

```bash
cargo build-sbf --manifest-path pow-pinocchio/Cargo.toml --features bpf-entrypoint
```

## How it works

The program uses a config PDA and a prefunded reward vault:

1. Derive the config PDA from `["pow-config", authority_pubkey]`
2. Create the config account off-chain
3. Create a vault token account owned by that config PDA
4. Fund the vault with SPL tokens
5. Call `Initialize`
6. Miners read the config account, search for winning nonces off-chain, and submit `Mine`

## Status

This project is currently experimental.

The current Pinocchio 0.9 dependency path still produces a `cargo build-sbf` post-processing warning for `sol_memcpy_` / `sol_memset_`. The artifact builds successfully, but the program should be smoke-tested on a local validator before production use.

## More details

See [`pow-pinocchio/README.md`](./pow-pinocchio/README.md) for:

- instruction account ordering
- binary instruction encoding
- config account byte layout
- vault setup requirements
- build caveats

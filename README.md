# spl-pow

Experimental Solana proof-of-work reward program built with Pinocchio.

> Warning
> This repository is unaudited and untested in any real deployment environment. Do not use it in production or with real funds.

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

## Dependency status

Dependency versions were re-verified on April 7, 2026.

- `pinocchio = 0.10.2`
- `pinocchio-token = 0.5.0`
- `solana-address = 2.5.0`

`solana-address 2.6.0` is newer on crates.io, but it requires `rustc 1.89.0`. The current Solana `cargo build-sbf` toolchain in this repo uses `rustc 1.84.1-dev`, so the repo intentionally pins `solana-address` to `2.5.0` as the newest version that still builds for SBF here.

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

The upgraded Pinocchio dependency path builds successfully with both `cargo test` and `cargo build-sbf` as of April 7, 2026, and the previous SBF post-processing warnings are gone on the current dependency set.

The program should still be smoke-tested on a local validator before production use.

## More details

See [`pow-pinocchio/README.md`](./pow-pinocchio/README.md) for:

- instruction account ordering
- binary instruction encoding
- config account byte layout
- vault setup requirements
- build notes

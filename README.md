# near-mixer

A privacy-preserving token mixer for the NEAR blockchain that allows for anonymous transactions by breaking the on-chain link between source and destination addresses.

## Overview

This smart contract implements a zero-knowledge proof system to enable private transfers on NEAR. Users can deposit tokens into the mixer contract and later withdraw them to a different address without revealing the connection between deposit and withdrawal addresses.

## Features

- Privacy-preserving token transfers
- Support for multiple token denominations
- Non-custodial design
- Zero-knowledge proofs for transaction verification

## Installation

Install [`cargo-near`](https://github.com/near/cargo-near):

```bash
cargo install cargo-near
```

Clone this repository:

```bash
git clone https://github.com/yourusername/near-mixer.git
cd near-mixer
```

## How to Build Locally?

```bash
cargo near build
```

## How to Test Locally?

```bash
cargo test
```

## How to Deploy?

Deployment is automated with GitHub Actions CI/CD pipeline.
To deploy manually, install [`cargo-near`](https://github.com/near/cargo-near) and run:

```bash
cargo near deploy build-reproducible-wasm <account-id>
```

## Usage Examples

### Depositing tokens

```bash
near call <contract-id> deposit '{"token_id": "near", "amount": "1000000000000000000000000"}' --accountId <your-account-id> --amount 1
```

### Withdrawing tokens

```bash
near call <contract-id> withdraw '{"nullifier_hash": "<hash>", "root": "<root>", "proof": "<proof>", "recipient": "<recipient-account>"}' --accountId <your-account-id>
```

## Project Structure

- `src/lib.rs` - Main contract implementation
- `src/utils.rs` - Helper functions
- `src/merkle.rs` - Merkle tree implementation for privacy proofs
- `src/crypto.rs` - Cryptographic primitives

## License

This project is licensed under [LICENSE NAME] - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Useful Links

- [cargo-near](https://github.com/near/cargo-near) - NEAR smart contract development toolkit for Rust
- [near CLI](https://near.cli.rs) - Interact with NEAR blockchain from command line
- [NEAR Rust SDK Documentation](https://docs.near.org/sdk/rust/introduction)
- [NEAR Documentation](https://docs.near.org)
- [NEAR StackOverflow](https://stackoverflow.com/questions/tagged/nearprotocol)
- [NEAR Discord](https://near.chat)
- [NEAR Telegram Developers Community Group](https://t.me/neardev)
- NEAR DevHub: [Telegram](https://t.me/neardevhub), [Twitter](https://twitter.com/neardevhub)

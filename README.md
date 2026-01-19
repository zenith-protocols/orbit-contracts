# OrbitCDP Smart Contracts

This repository contains the smart contracts of **OrbitCDP**, a decentralized stablecoin system built on the Stellar blockchain. The protocol allows users to mint fiat-pegged stablecoins via overcollateralized debt positions.

---

## Project Structure

```bash
.
├── bridge-oracle       # On-chain price feed adapter for stablecoins
├── dao-utils           # Governance and DAO utility contracts
├── treasury            # Stablecoin minting, burning, and flash loan logic
├── test-suites         # Integration and unit tests
├── wasm                # WASM output artifacts for deployment
├── Cargo.toml          # Rust workspace configuration
```

---

## Core Contracts

### 1. `treasury`
- **Purpose**: Manages stablecoin issuance and flash loans.
- **Key Roles**:
  - Mints/burns stablecoins supplied to Blend lending pools.
  - Issues flash loans for peg maintenance.
- **Security**: Only authorized contracts (like DAO contract) can manage stablecoins and supply.

### 2. `bridge-oracle`
- **Purpose**: Supplies fiat-based price feeds to the protocol.
- **Key Roles**:
  - Fetches fiat prices via third-party oracle.
  - Provides accurate pricing for stablecoin collateral valuation.
  - Maps stablecoin assets to their fiat pricing.

### 3. `dao-utils`
- **Purpose**: Provides governance helper modules.
- **Key Roles**:
  - Facilitates DAO configuration and authority enforcement.
  - Shared utility code for contract-level access control.

---

## Testing

The `test-suites` crate provides a complete testing SDK for OrbitCDP contracts. It includes a `TestFixture` that initializes the full protocol environment with mock tokens, Blend lending pools, and all OrbitCDP contracts. The SDK supports both native Rust and WASM-based testing modes, with contract client wrappers for seamless interaction with deployed contracts.

Run tests using:
```bash
make test
```

---

## Build & Deployment

Contracts are written in **Rust** using the **Stellar Soroban SDK**.

Build contracts:
```bash
make build
```

Format code:
```bash
make fmt
```

Clean build artifacts:
```bash
make clean
```

Deploy using Stellar CLI tools or Soroban CLI.

---

## License

This project is open-sourced under the MIT License. See [`LICENSE`](./LICENSE) for details.

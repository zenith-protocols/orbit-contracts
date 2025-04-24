# 🌌 Orbit Protocol Smart Contracts

This repository contains the core smart contracts powering the **Orbit Protocol** — a decentralized stablecoin system built on the Stellar blockchain. The protocol allows users to mint fiat-pegged stablecoins via overcollateralized debt positions, enabling efficient borrowing and liquidity mechanisms.

---

## 📁 Project Structure

```bash
.
├── bridge-oracle       # On-chain price feed adapter for stablecoins
├── dao-utils           # Governance and DAO utility contracts
├── pegkeeper           # Peg stability mechanism and liquidation logic
├── treasury            # Stablecoin minting, burning, and flash loan logic
├── mocks               # Testing mocks and scaffolds
├── test-suites         # Integration and unit tests
├── wasm                # WASM output artifacts for deployment
├── Cargo.toml          # Rust workspace configuration
```

---

## 🔗 Core Contracts

### 1. `treasury`
- **Purpose**: Manages stablecoin issuance and flash loans.
- **Key Roles**:
  - Mints/burns stablecoins supplied to Blend lending pools.
  - Issues flash loans to PegKeeper for peg maintenance.
- **Security**: Only authorized contracts (like DAO contract) can managa stablecoins and supply

### 2. `bridge-oracle`
- **Purpose**: Supplies fiat-based price feeds to the protocol.
- **Key Roles**:
  - Fetches fiat prices via third-party oracle.
  - Provides accurate pricing for stablecoin collateral valuation.
  - Maps stablecoin assets to their fiat pricing

### 3. `pegkeeper`
- **Purpose**: Maintains stablecoin peg across markets.
- **Key Roles**:
  - Executes liquidations and settles debts.
  - Performs AMM trades using Treasury flash loans.

### 4. `dao-utils`
- **Purpose**: Provides governance helper modules.
- **Key Roles**:
  - Facilitates DAO configuration and authority enforcement.
  - Shared utility code for contract-level access control.

---

## 🧪 Testing

Test files are organized under:

- `test-suites/`: Full integration scenarios and edge cases.
- `mocks/`: Mock contracts and test scaffolding for isolated environments.

Run tests using:
```bash
cargo test --workspace
```

---

## 📦 Build & Deployment

Contracts are written in **Rust** using the **Stellar Soroban SDK**.

Build contracts:
```bash
stellar contract build
```

Deploy using Stellar CLI tools or Soroban CLI.

---

## ⚖️ License

This project is open-sourced under the MIT License. See [`LICENSE`](./LICENSE) for details.
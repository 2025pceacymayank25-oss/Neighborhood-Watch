# 🛡️ SafetyNet — Decentralized Community Safety on Stellar

> *Residents stake tokens, report local hazards, and collectively verify incidents — building a tamper-proof, incentive-aligned safety network powered by Soroban smart contracts.*

---

## 📖 Project Description

SafetyNet is a **Soroban smart contract** deployed on the Stellar blockchain that transforms how communities report and validate local safety incidents. Instead of relying on centralized authorities or unverified social media posts, SafetyNet uses **economic staking** and **collective verification** to create a trustworthy, real-time safety layer for any neighborhood.

Residents lock up XLM as a stake to gain the right to report hazards — from broken streetlights and gas leaks to flooding and suspicious activity. Other staked residents then independently verify or dispute each report. Truthful reporters and accurate verifiers earn reputation and keep their stake; bad actors who file false reports get slashed.

The result is a **self-governing safety network** where every participant has skin in the game.

---

## ⚙️ What It Does

### 1. Stake to Participate
Residents deposit a minimum XLM stake to join the network. This stake acts as a bond — it signals good faith and is at risk if you behave dishonestly. Without a stake, you cannot report or verify incidents.

### 2. Report Safety Incidents
A staked resident files an incident by providing:
- **Description** — what happened
- **Location** — where it happened
- **Severity** — rated 1 (minor) to 5 (critical emergency)
- **Locked stake** — a portion of their stake is locked with the report

The report enters a `Pending` state and is broadcast on-chain via events.

### 3. Community Verification
Other staked residents (who are NOT the reporter) can independently **verify** or **dispute** the report:

| Action | Threshold | Outcome |
|---|---|---|
| ✅ **Verify** | 3+ independent verifications | Incident → `Verified`, reporter gets stake back + +10 reputation |
| ❌ **Dispute** | Disputes outnumber verifications (≥2) | Incident → `Disputed`, reporter stake slashed 50%, −20 reputation |

Verifiers earn +5 reputation for each verification regardless of outcome.

### 4. Resolution
Once a verified incident is addressed in the real world, the **admin** (e.g., a municipal authority or DAO multisig) marks it `Resolved`, closing the lifecycle.

### 5. Reputation System
Every address accumulates a **reputation score** based on their history:
- Start at **100**
- Gain reputation for successful reports (+10) and verifications (+5)
- Lose reputation for false/disputed reports (−20)

Reputation can be used by front-end applications to weight or highlight trusted reporters.

---

## ✨ Features

### 🔒 Stake-Gated Participation
Only residents with an active stake can report or verify. This economic barrier filters out spam and Sybil attacks without requiring identity verification.

### 📋 On-Chain Incident Lifecycle
Every incident moves through a well-defined state machine — `Pending → Verified → Resolved` or `Pending → Disputed` — with all transitions recorded immutably on the Stellar ledger.

### ⚖️ Incentive-Aligned Verification
The 3-of-N verification model, combined with stake slashing for bad actors, makes honest participation the rational strategy for all parties.

### 🏷️ Severity Scoring
Incidents are tagged with severity levels (1–5), enabling front-end dashboards and alerting systems to prioritize critical events automatically.

### 📢 On-Chain Event Emission
Key lifecycle events (`incident_reported`, `incident_verified`, `incident_disputed`) are emitted as Soroban events, making it easy to build real-time notification systems and analytics dashboards on top.

### 🔑 Role-Based Access
- **Residents** — stake, report, verify, dispute
- **Admin** — resolve incidents, update minimum stake threshold
- Contract is admin-agnostic and can be transferred to a DAO multisig

### 📊 Query Interface
Rich read-only functions for front-end integration:
- `get_incident(id)` — full incident details
- `get_stake_info(address)` — stake balance + reputation + history
- `get_pending_incidents()` — list of all open reports
- `get_incident_count()` — total reports ever filed
- `get_min_stake()` — current minimum stake requirement

### 🧪 Fully Tested
Comprehensive test suite covering:
- Initialization & configuration
- Stake deposit / withdrawal edge cases
- Incident reporting & state transitions
- 3-verifier auto-approval flow
- Dispute threshold & stake slashing
- Admin resolution
- Pending incident enumeration

---

## 🗂️ Project Structure

```
SafetyNet/
├── Cargo.toml                          # Workspace manifest
├── README.md
└── contracts/
    └── safety_net/
        ├── Cargo.toml                  # Contract dependencies
        └── src/
            ├── lib.rs                  # Contract implementation
            └── test.rs                 # Unit & integration tests
```

---

## 🚀 Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cd SafetyNet
stellar contract build
```

The compiled `.wasm` file will be at:
```
target/wasm32-unknown-unknown/release/safety_net.wasm
```

### Run Tests

```bash
cargo test
```

### Deploy to Testnet

```bash
# Configure testnet identity
stellar keys generate --global mykey --network testnet

# Fund with Friendbot
stellar keys fund mykey --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/safety_net.wasm \
  --source mykey \
  --network testnet
```

### Initialize the Contract

```bash
stellar contract invoke \
  
  --source mykey \
  --network testnet \
  -- initialize \
  --min_stake 10000000
```

---

## 🔮 Potential Extensions

- **Token Integration** — Use a custom SAC (Stellar Asset Contract) token instead of raw XLM for stakes and rewards
- **DAO Governance** — Replace admin with a multisig or governance contract for fully decentralized resolution
- **Reputation-Weighted Voting** — Weight verifications by the verifier's reputation score
- **Time-Locked Disputes** — Add a dispute window after which incidents auto-verify
- **Geo-Hashing** — Store incident locations as geohashes for privacy-preserving proximity queries
- **Tiered Severity Staking** — Require higher stakes for higher-severity incident reports

---

## 📄 License

MIT © 2024 SafetyNet Contributors

wallet address: GDQY2J5NSTSGHKC7EGHNKQPGGS4QIQ6R56IHHXB5EZHT3PADWMLOI5MV

contract address: CDAGY2ZAPZISMGEB5QHPZDWESZTI62ZTUG5GOPNQIAHLXL3TNOVQXUPY

https://stellar.expert/explorer/testnet/contract/CDAGY2ZAPZISMGEB5QHPZDWESZTI62ZTUG5GOPNQIAHLXL3TNOVQXUPY

<img width="1907" height="900" alt="image" src="https://github.com/user-attachments/assets/7ce1de36-b96c-42d1-be55-130cd7823cbc" />

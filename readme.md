# Satellion - open-source community driven bitcoin wallet

**Annotation**

We will develop a lightweight, secure, and transparent Bitcoin wallet featuring a friendly, step-by-step ask-and-confirm interaction model. The project is non-profit and aimed primarily at experienced users who want straightforward, trustworthy self-custody without trading functions or unnecessary complexity.

The initial release will support Bitcoin and Ethereum. Consideration of other assets will be deferred until much later, and only after careful evaluation.

## Principles

### Neutrino Client Protocol
No dependency on centralized RPC servers. By embedding the open-source Neutrino client (BIP157/158), our wallet independently verifies compact filters and blocks. This raises the trust model above Electrum-style servers and aligns us with Bitcoin’s decentralized ethos.

### Simplicity by Design
Minimize attack surface and cognitive load. Every interaction is step-by-step with safe defaults, clear prompts, and no hidden “expert” traps.

### Privacy & Security by Default
No telemetry. Deterministic address rotation. Strong key encryption using OS-level secure enclaves. Always assume the user’s device can be lost or compromised.

### Open & Verifiable
Roadmaps, discussions, and funding are public. Builds are reproducible and releases are signed. No “black box” binaries or hidden backdoors.

### Bitcoin-Native, Nothing Else
Hierarchical Deterministic wallets (BIP32/39) with modern address standards only - Taproot. No legacy formats, no altcoins, no distractions.

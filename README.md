Combine various components of zkMove together.

**zkMove Client**

- Users execute off-chain contracts to encrypt data and generate proofs.
- Users execute regular contracts to verify proofs on the Move blockchains (Aptos/Sui) or submit proofs to the Verification Network for validation, achieving low costs.

**Onchain Verifier**

- A Halo2 verifier written in the Move language and deployed on the Move chains.

**Verification Network**

- A third-party L1 network to verify proofs of zkMove circuits.
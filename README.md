# Export Aptos Verifier

[![dependency status](https://deps.rs/repo/github/mysteryon88/export-aptos-verifier/status.svg)](https://deps.rs/repo/github/mysteryon88/export-aptos-verifier)

**Export Aptos Verifier** is a CLI tool and Rust library for generating **Groth16** Aptos Move verifier packages from `verification_key.json`, native Gnark artifacts, SP1 Groth16 wrapper artifacts, Arkworks JSON/hex inputs, or compact Arkworks bundle files.

It supports **BN254** and **BLS12-381**. Circuits built with **Circom** and **Noname** are supported through `snarkjs`-compatible JSON; **Gnark** is supported through native JSON, native `vk.WriteTo` / `proof.WriteTo` binaries, and `snarkjs`-compatible JSON; **SP1** is supported through the Groth16 wrapper verification key and serialized `SP1ProofWithPublicValues`; **Arkworks** is supported through direct JSON/hex inputs or compact bundles. The curve and input format are inferred from the supplied artifacts.

When proof data is supplied, the tool validates the artifacts, runs local Arkworks Groth16 verification, and emits Move tests with the generated package. VK-only generation is also supported.

Generated packages use Aptos `crypto_algebra` byte layouts and contain `Move.toml`, `sources/verifier.move`, and optional `tests/verifier_tests.move`. Generation uses root-level CLI flags; `proof-data` is the only subcommand.

## Installation

```bash
cargo install export-aptos-verifier

# Help
export-aptos-verifier --help
```

## Import as a library

```bash
cargo add export-aptos-verifier-core
```

```rust
use export_aptos_verifier_core::{
    curves::create_adapter,
    formats::{
        load_compact_bundle, load_gnark_binary_inputs, load_gnark_json_inputs,
        load_snarkjs_json_inputs_with_optional_proof, load_sp1_groth16_inputs,
    },
    movegen::{generate_move_package, GenerateMovePackageOptions, MovegenMode},
};
```

Most users only need the CLI. Use the core crate when embedding verifier generation into another Rust tool.

## Usage CLI

```sh
# From snarkjs-compatible verification_key.json:
export-aptos-verifier --vk ./verification_key.json --out ./generated/my_verifier --force

# Include proof vectors for local verification and generated Move tests:
export-aptos-verifier --vk ./verification_key.json --proof ./proof.json --public ./public.json --out ./generated/my_verifier --force

# If proof.json contains publicSignals, --public can be omitted:
export-aptos-verifier --vk ./verification_key.json --proof ./proof.json --out ./generated/my_verifier --force

# From Arkworks JSON/hex inputs:
export-aptos-verifier --vk ./arkworks_verification_key.json --proof ./arkworks_proof.json --public ./public_inputs.json --out ./generated/arkworks_verifier --force

# From native Gnark JSON artifacts:
export-aptos-verifier --vk ./verification_key_gnark.json --proof ./proof_gnark.json --public ./public.json --out ./generated/gnark_json_verifier --force

# From native Gnark vk.WriteTo/proof.WriteTo binary artifacts:
export-aptos-verifier --vk ./verification_key.bin --proof ./proof.bin --public ./public.json --out ./generated/gnark_binary_verifier --force

# From SP1 Groth16 wrapper artifacts:
export-aptos-verifier --vk ./sp1_groth16_vk.bin --proof ./proof_with_public_values.bin --out ./generated/sp1_verifier --force

# From a compact Arkworks bundle:
export-aptos-verifier --bundle ./groth16_artifacts.json --out ./generated/ark_mimc_bn254 --force

# Customize the generated Move package:
export-aptos-verifier --vk ./verification_key.json --out ./generated/my_verifier --package-name my_verifier --module-name verifier --account-address 0x0 --mode entry --force

# Generate proof helper functions for tests:
export-aptos-verifier proof-data --vk ./verification_key.json --proof ./proof.json

# Generate and run aptos move test:
export-aptos-verifier --vk ./verification_key.json --proof ./proof.json --out ./generated/my_verifier --run-aptos-test --force
```

`--package-name` is derived from `--out` by default, `--module-name` defaults to `verifier`, `--account-address` defaults to `0x0`, and `--mode` defaults to `entry`. `--mode` accepts `library`, `entry`, or `test`. Use `--skip-local-verify` only when you want to bypass local Arkworks proof verification. SP1 public inputs are read from `SP1ProofWithPublicValues`, so `--public` is not needed for SP1. `--prepared` is intentionally not implemented yet.

## References

- [Aptos Move documentation](https://aptos.dev/network/blockchain/move)
- [Aptos `crypto_algebra` Move module](https://aptos.dev/move-reference/mainnet/aptos-stdlib/crypto_algebra)
- Examples
  - [examples](./examples/)
- Export of proof and verification key in JSON format compatible with snarkjs
  - [gnark-to-snarkjs](https://github.com/mysteryon88/gnark-to-snarkjs)
  - [ark-snarkjs](https://github.com/mysteryon88/ark-snarkjs)
- Frameworks verified for compatibility
  - [Circom](https://docs.circom.io/)
  - [Noname](https://github.com/zksecurity/noname)
  - [Gnark](https://github.com/Consensys/gnark)
  - [SP1](https://github.com/succinctlabs/sp1)
  - [Arkworks](https://github.com/arkworks-rs)

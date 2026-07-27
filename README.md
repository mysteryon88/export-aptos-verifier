# Export Aptos Verifier

[![dependency status](https://deps.rs/repo/github/mysteryon88/export-aptos-verifier/status.svg)](https://deps.rs/repo/github/mysteryon88/export-aptos-verifier)

**Export Aptos Verifier** is a CLI tool and Rust library for generating **Groth16** Aptos Move verifier packages.

It supports **BN254** and **BLS12-381** verification artifacts from **snarkjs**, **Gnark**, **SP1**, and **Arkworks**. Supported inputs include JSON, native Gnark binary files, SP1 Groth16 wrapper proofs, Arkworks JSON/hex files, and compact Arkworks bundles. The curve and input format are auto-detected.

When proof data is supplied, the tool verifies it locally and generates Move tests with the package. VK-only generation is also supported.

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
export-aptos-verifier --vk ./verification_key.json --out ./generated/verifier --force

# Include proof data for local verification and generated Move tests:
export-aptos-verifier --vk ./verification_key.json --proof ./proof.json --public ./public.json --out ./generated/verifier --force

# From native Gnark JSON or binary artifacts:
export-aptos-verifier --vk ./verification_key_gnark.json --proof ./proof_gnark.json --public ./public.json --out ./generated/gnark_verifier --force
export-aptos-verifier --vk ./verification_key.bin --proof ./proof.bin --public ./public.json --out ./generated/gnark_verifier --force

# From an SP1 Groth16 wrapper proof:
export-aptos-verifier --vk ./groth16_vk.bin --proof ./sp1_proof.bin --out ./generated/sp1_verifier --force

# From a compact Arkworks bundle:
export-aptos-verifier --bundle ./groth16_artifacts.json --out ./generated/arkworks_verifier --force

# Customize the generated Move package:
export-aptos-verifier --vk ./verification_key.json --out ./generated/verifier --package-name verifier --module-name verifier --account-address 0x0 --mode entry --force

# Generate proof helper functions or run Aptos Move tests:
export-aptos-verifier proof-data --vk ./verification_key.json --proof ./proof.json
export-aptos-verifier --vk ./verification_key.json --proof ./proof.json --out ./generated/verifier --run-aptos-test --force
```

`--package-name` is derived from `--out` by default. `--module-name` defaults to `verifier`, `--account-address` defaults to `0x0`, and `--mode` defaults to `entry`. Available modes are `library`, `entry`, and `test`.

## License

MIT.

## References

- [Aptos Move documentation](https://aptos.dev/network/blockchain/move)
- [Examples](./examples/)
- [gnark-to-snarkjs](https://github.com/mysteryon88/gnark-to-snarkjs)
- [ark-snarkjs](https://github.com/mysteryon88/ark-snarkjs)
- [Circom](https://docs.circom.io/)
- [Noname](https://github.com/zksecurity/noname)
- [Gnark](https://github.com/Consensys/gnark)
- [SP1](https://github.com/succinctlabs/sp1)
- [Arkworks](https://github.com/arkworks-rs)

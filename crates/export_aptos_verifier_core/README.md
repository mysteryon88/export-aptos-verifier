# export-aptos-verifier-core

Library crate for loading Groth16 artifacts and rendering Aptos Move verifier packages.

## Capabilities

- loads snarkjs-compatible JSON inputs
- loads native Gnark JSON verification keys, proofs, and public inputs
- loads native Gnark `vk.WriteTo` / `proof.WriteTo` binary artifacts
- loads SP1 Groth16 wrapper verification keys and serialized `SP1ProofWithPublicValues`
- loads Arkworks VK/proof JSON or raw hex inputs
- loads compact Arkworks bundle JSON
- infers the curve and input format from artifact metadata or binary decoding
- supports BN254 and BLS12-381
- validates protocol, curve, subgroup membership, input counts, and field bounds
- serializes verification keys, proofs, and public inputs for Aptos `crypto_algebra`
- performs local Arkworks Groth16 verification when proof vectors are supplied
- renders Aptos Move packages with `Move.toml`, `sources/verifier.move`, optional proof/public-input tests, and generated package README

## Generated Move API

Generated modules expose:

- `verify(public_inputs, proof_a, proof_b, proof_c): bool`
- `verify_entry(_signer, public_inputs, proof_a, proof_b, proof_c)` when generated in `entry` or `test` mode

`public_inputs` is `vector<vector<u8>>`. Proof points are serialized byte vectors in the Aptos `crypto_algebra` layout for the selected curve.

## Main Modules

- `formats`: high-level loaders for snarkjs JSON, Gnark JSON/BIN, SP1 Groth16, and Arkworks inputs
- `parser::arkworks`: direct Arkworks VK/proof/public input parser
- `snarkjs`: strict snarkjs-compatible JSON parsing
- `model`: normalized Groth16 IR
- `curves`: curve-specific adapters for BN254 and BLS12-381
- `movegen`: Aptos Move package rendering and proof-data snippets
- `verifier`: local Arkworks verification helpers

## Rust Usage

Use the crate directly when embedding generation in another Rust tool. Most users should use the `export-aptos-verifier` CLI.

```rust
use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::formats::load_gnark_json_inputs;
use export_aptos_verifier_core::movegen::{
    generate_move_package, GenerateMovePackageOptions, MovegenMode,
};

# fn main() -> export_aptos_verifier_core::Result<()> {
let inputs = load_gnark_json_inputs(
    "verification_key_gnark.json".as_ref(),
    Some("proof_gnark.json".as_ref()),
    Some("public.json".as_ref()),
    None,
)?;
let adapter = create_adapter(inputs.curve.canonical_name())?;

generate_move_package(
    "generated".as_ref(),
    adapter.as_ref(),
    &inputs,
    &GenerateMovePackageOptions {
        package_name: "generated",
        module_name: "verifier",
        account_address: "0x0",
        mode: MovegenMode::Entry,
        force: true,
    },
)?;
# Ok(())
# }
```

## Crate Docs

- docs.rs: `https://docs.rs/export-aptos-verifier-core`
- Rust import path: `export_aptos_verifier_core`

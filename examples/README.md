# export-aptos-verifier examples

Fixtures and generated Aptos Move packages for `export-aptos-verifier`.

Run commands in this file from the `export-aptos-verifier` directory.

## What Is Here

- `ark-mimc`: Rust arkworks example that exports BN254 and BLS12-381 `verification_key.json`, `proof.json`, and compact `groth16_artifacts.json` bundles.
- `MulCircuit`: Rust BLS12-381 multiplication circuit that exports snarkjs-style `verification_key.json`, `proof.json`, and `public.json`.
- `generated`: Aptos Move packages generated from the checked artifacts.

Proof-based generated packages include `tests/verifier_tests.move`. VK-only packages are generated without tests and are checked by compilation.

## 1. Regenerate Source Artifacts

This is optional if the checked-in artifacts are already current. Run it when you changed the example circuits.

```sh
cd ./examples/ark-mimc
cargo run -- export bn254 artifacts
cargo run -- export bls12_381 artifacts
cd ../..

cd ./examples/MulCircuit
cargo run
cd ../..
```

## 2. Generate Proof Packages

These commands regenerate the proof-based Aptos Move packages under `examples/generated`. They also run local Rust Groth16 verification before writing the Move package.

```sh
cargo run -- --bundle examples/ark-mimc/artifacts/bn254/groth16_artifacts.json --out examples/generated/ark_mimc_bn254_arkworks --account-address 0xCAFE --force

cargo run -- --bundle examples/ark-mimc/artifacts/bls12_381/groth16_artifacts.json --out examples/generated/ark_mimc_bls12381_arkworks --account-address 0xCAFE --force

cargo run -- --vk examples/ark-mimc/artifacts/bn254/verification_key.json --proof examples/ark-mimc/artifacts/bn254/proof.json --out examples/generated/ark_mimc_bn254_snarkjs --account-address 0xCAFE --force

cargo run -- --vk examples/ark-mimc/artifacts/bls12_381/verification_key.json --proof examples/ark-mimc/artifacts/bls12_381/proof.json --out examples/generated/ark_mimc_bls12381_snarkjs --account-address 0xCAFE --force

cargo run -- --vk examples/MulCircuit/artifacts/bls12_381/verification_key.json --proof examples/MulCircuit/artifacts/bls12_381/proof.json --public examples/MulCircuit/artifacts/bls12_381/public.json --out examples/generated/mul_circuit_bls12381_snarkjs --account-address 0xCAFE --force
```

Add `--run-aptos-test` to any command above to run `aptos move test --package-dir <out>` immediately after generation.

## 3. Run Aptos Move Tests

Run these after generation to verify the generated Move packages on Aptos.

```sh
aptos move test --package-dir examples/generated/ark_mimc_bn254_arkworks
aptos move test --package-dir examples/generated/ark_mimc_bls12381_arkworks
aptos move test --package-dir examples/generated/ark_mimc_bn254_snarkjs
aptos move test --package-dir examples/generated/ark_mimc_bls12381_snarkjs
aptos move test --package-dir examples/generated/mul_circuit_bls12381_snarkjs
```

## 4. Generate VK-Only Packages

VK-only packages prove that the verifier can be generated from a verification key alone. They do not contain `tests/`.

For snarkjs JSON, omit `--proof`:

```sh
cargo run -- --vk examples/ark-mimc/artifacts/bn254/verification_key.json --out examples/generated/ark_mimc_bn254_snarkjs_vk_only --account-address 0xCAFE --force

cargo run -- --vk examples/ark-mimc/artifacts/bls12_381/verification_key.json --out examples/generated/ark_mimc_bls12381_snarkjs_vk_only --account-address 0xCAFE --force

cargo run -- --vk examples/MulCircuit/artifacts/bls12_381/verification_key.json --out examples/generated/mul_circuit_bls12381_snarkjs_vk_only --account-address 0xCAFE --force
```

For Arkworks VK-only packages, create temporary VK-only JSON files from the full bundles, then pass them through `--vk`:

```sh
mkdir -p target/tmp-vk-only

jq -c '{ curve, verification_key: .vk }' \
  examples/ark-mimc/artifacts/bn254/groth16_artifacts.json \
  > target/tmp-vk-only/ark_mimc_bn254_vk_only.json

jq -c '{ curve, verification_key: .vk }' \
  examples/ark-mimc/artifacts/bls12_381/groth16_artifacts.json \
  > target/tmp-vk-only/ark_mimc_bls12381_vk_only.json

cargo run -- --vk target/tmp-vk-only/ark_mimc_bn254_vk_only.json --out examples/generated/ark_mimc_bn254_arkworks_vk_only --account-address 0xCAFE --force

cargo run -- --vk target/tmp-vk-only/ark_mimc_bls12381_vk_only.json --out examples/generated/ark_mimc_bls12381_arkworks_vk_only --account-address 0xCAFE --force
```

## 5. Check VK-Only Packages

For snarkjs VK-only packages, run compile checks:

```sh
aptos move compile --package-dir examples/generated/ark_mimc_bn254_snarkjs_vk_only
aptos move compile --package-dir examples/generated/ark_mimc_bls12381_snarkjs_vk_only
aptos move compile --package-dir examples/generated/mul_circuit_bls12381_snarkjs_vk_only
```

For Arkworks VK-only packages, run compile checks:

```sh
aptos move compile --package-dir examples/generated/ark_mimc_bn254_arkworks_vk_only
aptos move compile --package-dir examples/generated/ark_mimc_bls12381_arkworks_vk_only
```

## Proof Data Helpers

Use `proof-data` when you have a VK-only package and want to print Move helper functions for a later test file.

```sh
cargo run -- proof-data --vk examples/ark-mimc/artifacts/bn254/verification_key.json --proof examples/ark-mimc/artifacts/bn254/proof.json
```

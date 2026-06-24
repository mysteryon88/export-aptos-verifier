# Gnark Native Cubic Artifacts

This example contains native Gnark Groth16 artifacts for a small cubic circuit.

Checked artifacts:

- `artifacts/bn254/verification_key_gnark.json`
- `artifacts/bn254/proof_gnark.json`
- `artifacts/bn254/verification_key.bin`
- `artifacts/bn254/proof.bin`
- `artifacts/bn254/public.json`
- the same file set under `artifacts/bls12381/`

Generate Aptos Move packages:

```sh
cargo run -- --vk examples/gnark-native/cubic/artifacts/bn254/verification_key_gnark.json --proof examples/gnark-native/cubic/artifacts/bn254/proof_gnark.json --public examples/gnark-native/cubic/artifacts/bn254/public.json --out examples/generated/gnark_cubic_bn254_json --account-address 0xCAFE --force

cargo run -- --vk examples/gnark-native/cubic/artifacts/bls12381/verification_key.bin --proof examples/gnark-native/cubic/artifacts/bls12381/proof.bin --public examples/gnark-native/cubic/artifacts/bls12381/public.json --out examples/generated/gnark_cubic_bls12381_bin --account-address 0xCAFE --force
```

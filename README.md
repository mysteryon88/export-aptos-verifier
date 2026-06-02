# Export Aptos Verifier

`export-aptos-verifier` converts Groth16 artifacts into working Aptos Move verifier packages. It supports `BN254` and `BLS12-381`, accepts both classic `snarkjs` JSON files and compact Arkworks bundles, and performs strict validation plus local Arkworks verification before emitting Move code.

## Install

```bash
cargo install export-aptos-verifier-cli
```

## Supported inputs

- `snarkjs` JSON: `verification_key.json` + `proof.json` + optional `public.json`
- compact bundle JSON: one file with `curve`, `vk`, `proof`, `public_input`
- `proof.json` fallback: if `public.json` is omitted and the proof contains `publicSignals`, they are used automatically

## What it generates

- Aptos Move package with `Move.toml`
- verifier module in `sources/verifier.move`
- Move tests in `tests/verifier_tests.move`

## Main CLI flags

- `--input-format auto|snarkjs-json|arkworks-compact`
- `--curve auto|bn254|bls12381`
- `--mode library|entry|test`
- `--run-aptos-test` to run `aptos move test` after generation
- `--skip-local-verify` to skip Arkworks proof verification before generation
- `--force` to overwrite output directory

## CLI examples

Classic JSON mode:

```bash
export-aptos-verifier generate \
  --vk ./verification_key.json \
  --proof ./proof.json \
  --out ./generated \
  --package-name groth16_verifier \
  --module-name multiplier_verifier \
  --account-address 0xCAFE \
  --curve auto
```

Compact bundle mode:

```bash
export-aptos-verifier generate \
  --bundle ./groth16_artifacts.json \
  --out ./generated \
  --package-name groth16_verifier \
  --module-name multiplier_verifier \
  --account-address 0xCAFE \
  --curve auto
```

## Validation and limits

- validates protocol, curve, subgroup membership, field bounds, and public input counts
- runs local Arkworks Groth16 verification by default before writing Move code
- supports `Groth16` only in this version
- `--prepared` is intentionally not implemented yet

## Included examples

- `examples/MulCircuit`: generates BLS12-381 `snarkjs`-style artifacts
- `examples/ark-mimc`: generates BN254 and BLS12-381 artifacts, plus compact `groth16_artifacts.json` bundles

## Safety checks

Generated verifier packages are suitable as a starting point, but the emitted Move code should still be reviewed before production deployment.

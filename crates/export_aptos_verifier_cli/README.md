# export-aptos-verifier

CLI tool that generates Aptos Move Groth16 verifier packages from Groth16 artifacts.

## Install

```bash
cargo install export-aptos-verifier-cli
```

## Usage

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

`--public` is optional when `proof.json` already contains `publicSignals`.

Bundle mode:

```bash
export-aptos-verifier generate \
  --bundle ./groth16_artifacts.json \
  --out ./generated \
  --package-name groth16_verifier \
  --module-name multiplier_verifier \
  --account-address 0xCAFE \
  --curve auto
```

## Notes

- Supports `snarkjs-json` and `arkworks-compact` input modes.
- Supports `BN254` and `BLS12-381`.
- `--prepared` intentionally returns `ERR_PREPARED_NOT_IMPLEMENTED` in this version.
- For BN254 use `--bn254-format` if you need explicit format handling.
- For BLS12-381 use `--bls-format` if needed.

## License

MIT.

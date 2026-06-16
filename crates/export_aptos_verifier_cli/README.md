# export-aptos-verifier

CLI for generating Aptos Move Groth16 verifier packages.

Generation uses root-level flags:

```sh
export-aptos-verifier \
  --vk ./verification_key.json \
  --proof ./proof.json \
  --public ./public.json \
  --out ./generated/my_verifier \
  --force
```

Compact Arkworks bundle mode:

```sh
export-aptos-verifier \
  --bundle ./groth16_artifacts.json \
  --out ./generated/arkworks_verifier \
  --force
```

Common options:

- `--package-name <name>`: defaults to the sanitized `--out` directory name
- `--module-name <name>`: defaults to `verifier`
- `--account-address <address>`: defaults to `0x0`
- `--mode library|entry|test`: defaults to `entry`
- `--run-aptos-test`: runs `aptos move test --package-dir <out>`
- `--skip-local-verify`: skips local Arkworks proof verification
- `--prepared`: intentionally returns `ERR_PREPARED_NOT_IMPLEMENTED`
- `--force`: overwrites the output directory

`--proof` is optional. Supplying proof data enables local verification and generated Move tests. `--public` is optional when `proof.json` already contains `publicSignals`.

`proof-data` is the only subcommand:

```sh
export-aptos-verifier proof-data \
  --vk ./verification_key.json \
  --proof ./proof.json
```

It prints Move helper functions for `proof_a_bytes()`, `proof_b_bytes()`, `proof_c_bytes()`, and `public_inputs_bytes()` using the same serialization as generated tests.

Supported inputs:

- snarkjs-compatible JSON
- Arkworks VK/proof JSON or raw hex inputs
- compact Arkworks bundles

Supported curves:

- BN254
- BLS12-381

MIT.

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

Native Gnark JSON mode:

```sh
export-aptos-verifier \
  --vk ./verification_key_gnark.json \
  --proof ./proof_gnark.json \
  --public ./public.json \
  --out ./generated/gnark_json_verifier \
  --force
```

Native Gnark binary mode (`vk.WriteTo` / `proof.WriteTo`):

```sh
export-aptos-verifier \
  --vk ./verification_key.bin \
  --proof ./proof.bin \
  --public ./public.json \
  --out ./generated/gnark_binary_verifier \
  --force
```

SP1 Groth16 wrapper mode:

```sh
export-aptos-verifier \
  --vk ./sp1_groth16_vk.bin \
  --proof ./proof_with_public_values.bin \
  --out ./generated/sp1_verifier \
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

`--proof` is optional. Supplying proof data enables local verification and generated Move tests. `--public` is optional when `proof.json` already contains `publicSignals`. SP1 proofs carry public values inside the serialized proof wrapper, so SP1 commands do not use `--public`.

`proof-data` is the only subcommand:

```sh
export-aptos-verifier proof-data \
  --vk ./verification_key.json \
  --proof ./proof.json
```

It prints Move helper functions for `proof_a_bytes()`, `proof_b_bytes()`, `proof_c_bytes()`, and `public_inputs_bytes()` using the same serialization as generated tests.

Supported inputs:

- snarkjs-compatible JSON
- native Gnark JSON
- native Gnark binary `vk.WriteTo` / `proof.WriteTo`
- SP1 Groth16 wrapper verification key plus serialized `SP1ProofWithPublicValues`
- Arkworks VK/proof JSON or raw hex inputs
- compact Arkworks bundles

Supported curves:

- BN254
- BLS12-381

MIT.

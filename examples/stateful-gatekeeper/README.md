# stateful_gatekeeper

Generated Aptos Move Groth16 verifier package.

## Generated API

The verifier module is `stateful_gatekeeper::verifier` at named address `0xCAFE`.

- `verify(public_inputs, proof_a, proof_b, proof_c): bool`
- `verify_entry(_signer, public_inputs, proof_a, proof_b, proof_c)` when generated in `entry` or `test` mode

`public_inputs` is `vector<vector<u8>>`. Proof points are serialized byte vectors in the Aptos `crypto_algebra` layout for the selected curve.

## Regenerate

Run `export-aptos-verifier` with root-level generation flags:

```sh
export-aptos-verifier --vk ./verification_key.json --out ./generated --account-address 0xCAFE --force
export-aptos-verifier --bundle ./groth16_artifacts.json --out ./generated --account-address 0xCAFE --force
```

Add `--proof ./proof.json` and optional `--public ./public.json` to include local proof verification and generated Move tests.

Useful flags:

- `--package-name stateful_gatekeeper`
- `--module-name verifier`
- `--mode library|entry|test`
- `--run-aptos-test`
- `--skip-local-verify`

VK-only packages are generated without `tests/`. To print proof helpers for a later test file, run:

```sh
export-aptos-verifier proof-data --vk ./verification_key.json --proof ./proof.json
```

## Stateful authorization flow

`gatekeeper.move` keeps the generated verifier stateless and adds a resource-based `ReplayGuard` under the authorizing signer. The signer address is part of the proved context and selects the resource that stores used nullifiers. `authorize` binds domain, nullifier, canonical VK fingerprint, signer/account, the module identifier compiled into the package, the runtime Aptos chain ID, and operation into the sole public input, verifies the real BN254 proof, and only then writes the nullifier.

The fixed `CONTEXT_MASK` is a bijective 248-bit output encoding chosen so this example can reuse the repository's existing proof fixture. The final byte is fixed to a canonical scalar value; the remaining 31 bytes are `SHA-256(context) XOR mask`. A production circuit may prove an unmasked hash-to-field output directly.

Every context field is length-prefixed. Domain and nullifier are therefore part of the proved statement, not side arguments to an unrelated proof.

Move tests cover a first valid authorization, replay, another signer/account, another domain, an invalid proof, a wrong VK fingerprint, and runtime chain-ID separation. Run them with:

```sh
aptos move test --package-dir examples/stateful-gatekeeper
```

## Known limitations

- Supported curves: BN254 and BLS12-381.
- The curve and input format are inferred from artifact metadata.
- `--prepared` is intentionally not implemented yet.
- Generated verifier code is not audited. Review it before production use.

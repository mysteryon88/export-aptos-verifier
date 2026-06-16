pub fn readme_content(package_name: &str, module_name: &str, account_address: &str) -> String {
    format!(
        r#"# {package_name}

Generated Aptos Move Groth16 verifier package.

## Generated API

The verifier module is `{package_name}::{module_name}` at named address `{account_address}`.

- `verify(public_inputs, proof_a, proof_b, proof_c): bool`
- `verify_entry(_signer, public_inputs, proof_a, proof_b, proof_c)` when generated in `entry` or `test` mode

`public_inputs` is `vector<vector<u8>>`. Proof points are serialized byte vectors in the Aptos `crypto_algebra` layout for the selected curve.

## Regenerate

Run `export-aptos-verifier` with root-level generation flags:

```sh
export-aptos-verifier --vk ./verification_key.json --out ./generated --account-address {account_address} --force
export-aptos-verifier --bundle ./groth16_artifacts.json --out ./generated --account-address {account_address} --force
```

Add `--proof ./proof.json` and optional `--public ./public.json` to include local proof verification and generated Move tests.

Useful flags:

- `--package-name {package_name}`
- `--module-name {module_name}`
- `--mode library|entry|test`
- `--run-aptos-test`
- `--skip-local-verify`

VK-only packages are generated without `tests/`. To print proof helpers for a later test file, run:

```sh
export-aptos-verifier proof-data --vk ./verification_key.json --proof ./proof.json
```

## Known limitations

- Supported curves: BN254 and BLS12-381.
- The curve and input format are inferred from artifact metadata.
- `--prepared` is intentionally not implemented yet.
- Generated verifier code is not audited. Review it before production use.
"#,
        package_name = package_name,
        module_name = module_name,
        account_address = account_address
    )
}

pub fn vector_of_hex(values: &[String]) -> String {
    if values.is_empty() {
        return "vector[]".to_string();
    }
    let body = values
        .iter()
        .map(|value| format!("        {value},"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("vector[\n{body}\n    ]")
}

# SP1 Groth16 Fibonacci Artifacts

This example contains SP1 Groth16 wrapper artifacts for a Fibonacci program.
The checked fixtures are copied from
[`mysteryon88/export-sui-verifier`](https://github.com/mysteryon88/export-sui-verifier),
and the Aptos tests exercise SP1 support against those Sui verifier reference artifacts.

Checked artifacts:

- `artifacts/groth16_vk_v5.bin`
- `artifacts/fibonacci_proof.bin`
- `artifacts/sp1_groth16_vk.bin`
- `artifacts/fibonacci_sp1_6_proof.bin`

Generate Aptos Move packages:

```sh
cargo run -- --vk examples/sp1-groth16/fibonacci/artifacts/groth16_vk_v5.bin --proof examples/sp1-groth16/fibonacci/artifacts/fibonacci_proof.bin --out examples/generated/sp1_fibonacci_groth16 --account-address 0xCAFE --force

cargo run -- --vk examples/sp1-groth16/fibonacci/artifacts/sp1_groth16_vk.bin --proof examples/sp1-groth16/fibonacci/artifacts/fibonacci_sp1_6_proof.bin --out examples/generated/sp1_fibonacci_groth16_v6 --account-address 0xCAFE --force
```

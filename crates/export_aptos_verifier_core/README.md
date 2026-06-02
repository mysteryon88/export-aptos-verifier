# export-aptos-verifier-core

Library crate for loading Groth16 artifacts and generating Aptos Move verifier packages.

## Capabilities

- loads `snarkjs` JSON inputs and compact Arkworks bundle JSON
- supports `BN254` and `BLS12-381`
- validates protocol, curve, subgroup membership, input counts, and field bounds
- serializes values into the byte layouts expected by Aptos `crypto_algebra`
- performs local Arkworks Groth16 verification before Move generation

## Main modules

- `formats`: loaders for `snarkjs` JSON and compact bundles
- `model`: normalized Groth16 IR used by the whole pipeline
- `curves`: curve-specific adapters for BN254 and BLS12-381
- `movegen`: Aptos Move package rendering

## Crate docs

- docs.rs: `https://docs.rs/export-aptos-verifier-core`
- Rust import path: `export_aptos_verifier_core`

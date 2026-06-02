use std::path::PathBuf;

use export_aptos_verifier_core::formats::{load_compact_bundle, load_snarkjs_json_inputs};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn ark_mimc_json_mode_uses_public_signals_when_public_file_is_missing() {
    let root = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");

    let inputs = load_snarkjs_json_inputs(
        &root.join("verification_key.json"),
        &root.join("proof.json"),
        None,
    )
    .unwrap();

    assert_eq!(inputs.public_inputs.len(), 1);
}

#[test]
fn ark_mimc_compact_bundle_loads_for_bls12_381() {
    let path = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381")
        .join("groth16_artifacts.json");

    let inputs = load_compact_bundle(&path, None).unwrap();

    assert_eq!(inputs.public_inputs.len(), 1);
}

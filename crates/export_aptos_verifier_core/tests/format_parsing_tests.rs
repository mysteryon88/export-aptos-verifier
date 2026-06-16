use std::fs;
use std::path::PathBuf;

use export_aptos_verifier_core::formats::{
    load_arkworks_inputs, load_compact_bundle, load_snarkjs_json_inputs,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn temp_output_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "export_aptos_verifier_format_{name}_{}",
        std::process::id()
    ));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
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

#[test]
fn arkworks_json_hex_inputs_load_without_compact_bundle() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let bundle_json = fs::read_to_string(&bundle).unwrap();
    let bundle_value: serde_json::Value = serde_json::from_str(&bundle_json).unwrap();

    let input_dir = temp_output_dir("arkworks_json_hex");
    fs::create_dir_all(&input_dir).unwrap();
    let vk_path = input_dir.join("verification_key.json");
    let proof_path = input_dir.join("proof.json");
    let public_path = input_dir.join("public.json");
    fs::write(
        &vk_path,
        serde_json::json!({
            "curve": "bn254",
            "verification_key": bundle_value.get("vk").unwrap(),
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        &proof_path,
        serde_json::json!({
            "proof": bundle_value.get("proof").unwrap(),
        })
        .to_string(),
    )
    .unwrap();
    fs::write(
        &public_path,
        serde_json::json!({
            "public_inputs": [bundle_value.get("public_input").unwrap()],
        })
        .to_string(),
    )
    .unwrap();

    let inputs =
        load_arkworks_inputs(&vk_path, Some(&proof_path), Some(&public_path), None).unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bn254");
    assert!(inputs.has_test_vectors());
    assert_eq!(inputs.public_inputs.len(), 1);
}

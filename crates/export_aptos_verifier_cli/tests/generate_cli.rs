use std::path::PathBuf;

use assert_cmd::Command;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn temp_output_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "export_aptos_verifier_cli_{name}_{}",
        std::process::id()
    ));
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
    dir
}

fn strip_curve_metadata(json: &str) -> String {
    json.lines()
        .filter(|line| !line.trim_start().starts_with("\"curve\""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn ark_mimc_bundle_mode_generates_output() {
    let repo = repo_root();
    let bundle = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let out_dir = temp_output_dir("bundle");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "generate",
            "--bundle",
            bundle.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254",
            "--module-name",
            "ark_mimc_bn254",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();
}

#[test]
fn ark_mimc_json_mode_generates_without_public_file() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381");
    let out_dir = temp_output_dir("json");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "generate",
            "--vk",
            artifact_dir.join("verification_key.json").to_str().unwrap(),
            "--proof",
            artifact_dir.join("proof.json").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bls_json",
            "--module-name",
            "ark_mimc_bls_json",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();
}

#[test]
fn snarkjs_json_mode_uses_curve_flag_when_metadata_is_missing() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let input_dir = temp_output_dir("missing_curve_inputs");
    std::fs::create_dir_all(&input_dir).unwrap();
    std::fs::write(
        input_dir.join("verification_key.json"),
        strip_curve_metadata(
            &std::fs::read_to_string(artifact_dir.join("verification_key.json")).unwrap(),
        ),
    )
    .unwrap();
    std::fs::write(
        input_dir.join("proof.json"),
        strip_curve_metadata(&std::fs::read_to_string(artifact_dir.join("proof.json")).unwrap()),
    )
    .unwrap();
    let out_dir = temp_output_dir("missing_curve_json");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "generate",
            "--vk",
            input_dir.join("verification_key.json").to_str().unwrap(),
            "--proof",
            input_dir.join("proof.json").to_str().unwrap(),
            "--curve",
            "bn254",
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254_missing_curve",
            "--module-name",
            "ark_mimc_bn254_missing_curve",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();
}

#[test]
fn invalid_account_address_is_rejected() {
    let repo = repo_root();
    let bundle = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let out_dir = temp_output_dir("invalid_address");

    let assert = Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "generate",
            "--bundle",
            bundle.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254",
            "--module-name",
            "ark_mimc_bn254",
            "--account-address",
            "CAFE",
            "--force",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("ERR_INVALID_ACCOUNT_ADDRESS"));
}

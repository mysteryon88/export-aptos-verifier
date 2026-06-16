use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::json;

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
fn bundle_and_vk_cannot_be_used_together() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let bundle = artifact_dir.join("groth16_artifacts.json");
    let vk = artifact_dir.join("verification_key.json");
    let out_dir = temp_output_dir("bundle_and_vk");

    let assert = Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--bundle",
            bundle.to_str().unwrap(),
            "--vk",
            vk.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("use either --bundle or --vk, not both"));
}

#[test]
fn arkworks_compact_vk_only_bundle_generates_without_tests() {
    let repo = repo_root();
    let bundle = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let bundle_json = std::fs::read_to_string(&bundle).unwrap();
    let bundle_value: serde_json::Value = serde_json::from_str(&bundle_json).unwrap();
    let input_dir = temp_output_dir("bundle_vk_only_input");
    std::fs::create_dir_all(&input_dir).unwrap();
    let vk_only_bundle = input_dir.join("groth16_vk_only.json");
    std::fs::write(
        &vk_only_bundle,
        json!({
            "curve": "bn254",
            "vk": bundle_value.get("vk").unwrap(),
        })
        .to_string(),
    )
    .unwrap();
    let out_dir = temp_output_dir("bundle_vk_only");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--bundle",
            vk_only_bundle.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254_vk_only_bundle",
            "--module-name",
            "ark_mimc_bn254_vk_only_bundle",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(!out_dir.join("tests").exists());
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
fn snarkjs_json_vk_only_mode_generates_without_tests() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let out_dir = temp_output_dir("json_vk_only");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--vk",
            artifact_dir.join("verification_key.json").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254_vk_only",
            "--module-name",
            "ark_mimc_bn254_vk_only",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(!out_dir.join("tests").exists());
}

#[test]
fn arkworks_vk_only_mode_generates_without_bundle_flag() {
    let repo = repo_root();
    let bundle = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let bundle_json = std::fs::read_to_string(&bundle).unwrap();
    let bundle_value: serde_json::Value = serde_json::from_str(&bundle_json).unwrap();
    let input_dir = temp_output_dir("arkworks_vk_only_input");
    std::fs::create_dir_all(&input_dir).unwrap();
    let vk_path = input_dir.join("arkworks_verification_key.json");
    std::fs::write(
        &vk_path,
        json!({
            "curve": "bn254",
            "verification_key": bundle_value.get("vk").unwrap(),
        })
        .to_string(),
    )
    .unwrap();
    let out_dir = temp_output_dir("arkworks_vk_only");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--vk",
            vk_path.to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254_arkworks_vk_only",
            "--module-name",
            "ark_mimc_bn254_arkworks_vk_only",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(!out_dir.join("tests").exists());
}

#[test]
fn generation_uses_simple_defaults_when_names_are_omitted() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let out_dir = temp_output_dir("default_names").join("my-verifier");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--vk",
            artifact_dir.join("verification_key.json").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--force",
        ])
        .assert()
        .success();

    let move_toml = std::fs::read_to_string(out_dir.join("Move.toml")).unwrap();
    let verifier = std::fs::read_to_string(out_dir.join("sources").join("verifier.move")).unwrap();
    assert!(move_toml.contains("name = \"my_verifier\""));
    assert!(move_toml.contains("my_verifier = \"0x0\""));
    assert!(verifier.contains("module my_verifier::verifier"));
}

#[test]
fn generate_subcommand_is_no_longer_supported() {
    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args(["generate", "--help"])
        .assert()
        .failure();
}

#[test]
fn help_does_not_expose_format_or_curve_flags() {
    let assert = Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args(["--help"])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(!stdout.contains("--curve"));
    assert!(!stdout.contains("--input-format"));
    assert!(!stdout.contains("--bn254-format"));
    assert!(!stdout.contains("--bls-format"));
    assert!(!stdout.contains("--format"));
    assert!(!stdout.contains("--output"));
}

#[test]
fn format_and_curve_flags_are_rejected() {
    for flag in [
        "--curve",
        "--input-format",
        "--bn254-format",
        "--bls-format",
        "--format",
        "--output",
    ] {
        Command::cargo_bin("export-aptos-verifier")
            .unwrap()
            .args([flag, "auto"])
            .assert()
            .failure();
    }
}

#[test]
fn snarkjs_json_without_curve_metadata_is_rejected() {
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
        std::fs::read_to_string(artifact_dir.join("verification_key.json"))
            .unwrap()
            .lines()
            .filter(|line| !line.trim_start().starts_with("\"curve\""))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    std::fs::write(
        input_dir.join("proof.json"),
        std::fs::read_to_string(artifact_dir.join("proof.json"))
            .unwrap()
            .lines()
            .filter(|line| !line.trim_start().starts_with("\"curve\""))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .unwrap();
    let out_dir = temp_output_dir("missing_curve_json");

    let assert = Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--vk",
            input_dir.join("verification_key.json").to_str().unwrap(),
            "--proof",
            input_dir.join("proof.json").to_str().unwrap(),
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
        .failure();

    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("curve"));
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

#[test]
fn proof_data_prints_snippets_matching_generated_aptos_tests() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let out_dir = temp_output_dir("proof_data_matches_tests");

    Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "--vk",
            artifact_dir.join("verification_key.json").to_str().unwrap(),
            "--proof",
            artifact_dir.join("proof.json").to_str().unwrap(),
            "--out",
            out_dir.to_str().unwrap(),
            "--package-name",
            "ark_mimc_bn254_proof_data",
            "--module-name",
            "ark_mimc_bn254_proof_data",
            "--account-address",
            "0xCAFE",
            "--force",
        ])
        .assert()
        .success();

    let generated_tests =
        std::fs::read_to_string(out_dir.join("tests").join("verifier_tests.move")).unwrap();

    let assert = Command::cargo_bin("export-aptos-verifier")
        .unwrap()
        .args([
            "proof-data",
            "--vk",
            artifact_dir.join("verification_key.json").to_str().unwrap(),
            "--proof",
            artifact_dir.join("proof.json").to_str().unwrap(),
        ])
        .assert()
        .success();

    let stdout = String::from_utf8_lossy(&assert.get_output().stdout);
    assert!(stdout.contains("fun proof_a_bytes(): vector<u8>"));
    assert!(stdout.contains("fun proof_b_bytes(): vector<u8>"));
    assert!(stdout.contains("fun proof_c_bytes(): vector<u8>"));
    assert!(stdout.contains("fun public_inputs_bytes(): vector<vector<u8>>"));
    assert!(generated_tests.contains(stdout.trim()));
}

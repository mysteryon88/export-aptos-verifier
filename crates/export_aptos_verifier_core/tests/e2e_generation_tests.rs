use std::fs;
use std::path::PathBuf;
use std::process::Command;

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::formats::{load_compact_bundle, load_snarkjs_json_inputs};
use export_aptos_verifier_core::movegen::{
    generate_move_package, GenerateMovePackageOptions, MovegenMode,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn temp_output_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "export_aptos_verifier_{name}_{}",
        std::process::id()
    ));
    if dir.exists() {
        let _ = fs::remove_dir_all(&dir);
    }
    dir
}

fn aptos_move_test(package_dir: &PathBuf) {
    let output = Command::new("aptos")
        .args(["move", "test", "--package-dir"])
        .arg(package_dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "aptos move test failed for {}\nstdout:\n{}\nstderr:\n{}",
        package_dir.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn mul_circuit_json_inputs_generate_a_package() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("MulCircuit")
        .join("artifacts")
        .join("bls12_381");
    let inputs = load_snarkjs_json_inputs(
        &artifact_dir.join("verification_key.json"),
        &artifact_dir.join("proof.json"),
        Some(&artifact_dir.join("public.json")),
    )
    .unwrap();

    let out_dir = temp_output_dir("mul_circuit");
    generate_move_package(
        &out_dir,
        create_adapter("bls12_381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "mul_circuit_verifier",
            module_name: "mul_circuit_verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    aptos_move_test(&out_dir);
}

#[test]
fn ark_mimc_bn254_json_inputs_generate_a_package() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let inputs = load_snarkjs_json_inputs(
        &artifact_dir.join("verification_key.json"),
        &artifact_dir.join("proof.json"),
        None,
    )
    .unwrap();

    let out_dir = temp_output_dir("ark_mimc_bn254_json");
    generate_move_package(
        &out_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bn254_json",
            module_name: "ark_mimc_bn254_json",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    aptos_move_test(&out_dir);
}

#[test]
fn ark_mimc_bls_json_inputs_generate_a_package() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381");
    let inputs = load_snarkjs_json_inputs(
        &artifact_dir.join("verification_key.json"),
        &artifact_dir.join("proof.json"),
        None,
    )
    .unwrap();

    let out_dir = temp_output_dir("ark_mimc_bls_json");
    generate_move_package(
        &out_dir,
        create_adapter("bls12_381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bls_json",
            module_name: "ark_mimc_bls_json",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    aptos_move_test(&out_dir);
}

#[test]
fn ark_mimc_bn254_bundle_inputs_generate_a_package() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let inputs = load_compact_bundle(&bundle, None).unwrap();

    let out_dir = temp_output_dir("ark_mimc_bn254_bundle");
    generate_move_package(
        &out_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bn254_bundle",
            module_name: "ark_mimc_bn254_bundle",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    aptos_move_test(&out_dir);
}

#[test]
fn ark_mimc_bls_bundle_inputs_generate_a_package() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381")
        .join("groth16_artifacts.json");
    let inputs = load_compact_bundle(&bundle, None).unwrap();

    let out_dir = temp_output_dir("ark_mimc_bls_bundle");
    generate_move_package(
        &out_dir,
        create_adapter("bls12_381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bls_bundle",
            module_name: "ark_mimc_bls_bundle",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    aptos_move_test(&out_dir);
}

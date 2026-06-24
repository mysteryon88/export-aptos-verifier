use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::formats::{
    load_compact_bundle, load_gnark_binary_inputs, load_gnark_json_inputs,
    load_snarkjs_json_inputs, load_snarkjs_json_inputs_with_optional_proof,
    load_sp1_groth16_inputs,
};
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

fn aptos_cli_guard() -> MutexGuard<'static, ()> {
    static APTOS_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    APTOS_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn aptos_move_test(package_dir: &PathBuf) {
    aptos_move(package_dir, "test");
}

fn aptos_move_compile(package_dir: &PathBuf) {
    aptos_move(package_dir, "compile");
}

fn aptos_move(package_dir: &PathBuf, command: &str) {
    let _guard = aptos_cli_guard();
    let output = Command::new("aptos")
        .args(["move", command, "--package-dir"])
        .arg(package_dir)
        .output();

    match output {
        Ok(output) => assert!(
            output.status.success(),
            "aptos move {command} failed for {}\nstdout:\n{}\nstderr:\n{}",
            package_dir.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        Err(err) if err.kind() == ErrorKind::NotFound => {
            eprintln!(
                "skipping aptos move {command} for {}: Aptos CLI not found",
                package_dir.display()
            );
        }
        Err(err) => panic!(
            "failed to run aptos move {command} for {}: {err}",
            package_dir.display()
        ),
    }
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

    let generated_tests =
        fs::read_to_string(out_dir.join("tests").join("verifier_tests.move")).unwrap();
    assert!(generated_tests.contains("fun test_invalid_proof_fails()"));
    assert!(!generated_tests.contains("vector::empty"));
    let generated_verifier =
        fs::read_to_string(out_dir.join("sources").join("verifier.move")).unwrap();
    assert!(!generated_verifier.contains("vector::empty"));

    aptos_move_test(&out_dir);
}

#[test]
fn ark_mimc_bn254_json_vk_only_generates_buildable_package_without_tests() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let inputs = load_snarkjs_json_inputs_with_optional_proof(
        &artifact_dir.join("verification_key.json"),
        None,
        None,
        Some("bn254"),
    )
    .unwrap();
    assert!(!inputs.has_test_vectors());

    let out_dir = temp_output_dir("ark_mimc_bn254_json_vk_only");
    generate_move_package(
        &out_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bn254_json_vk_only",
            module_name: "ark_mimc_bn254_json_vk_only",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(!out_dir.join("tests").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    aptos_move_compile(&out_dir);
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
fn ark_mimc_bls_json_vk_only_generates_buildable_package_without_tests() {
    let repo = repo_root();
    let artifact_dir = repo
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381");
    let inputs = load_snarkjs_json_inputs_with_optional_proof(
        &artifact_dir.join("verification_key.json"),
        None,
        None,
        Some("bls12381"),
    )
    .unwrap();
    assert!(!inputs.has_test_vectors());

    let out_dir = temp_output_dir("ark_mimc_bls_json_vk_only");
    generate_move_package(
        &out_dir,
        create_adapter("bls12_381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bls_json_vk_only",
            module_name: "ark_mimc_bls_json_vk_only",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(!out_dir.join("tests").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    aptos_move_compile(&out_dir);
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
fn ark_mimc_bls_compact_vk_only_generates_buildable_package_without_tests() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bls12_381")
        .join("groth16_artifacts.json");
    let bundle_json = fs::read_to_string(&bundle).unwrap();
    let bundle_value: serde_json::Value = serde_json::from_str(&bundle_json).unwrap();
    let input_dir = temp_output_dir("ark_mimc_bls_compact_vk_only_input");
    fs::create_dir_all(&input_dir).unwrap();
    let vk_only_bundle = input_dir.join("groth16_vk_only.json");
    fs::write(
        &vk_only_bundle,
        serde_json::json!({
            "curve": "bls12381",
            "vk": bundle_value.get("vk").unwrap(),
        })
        .to_string(),
    )
    .unwrap();

    let inputs = load_compact_bundle(&vk_only_bundle, None).unwrap();
    assert!(!inputs.has_test_vectors());

    let out_dir = temp_output_dir("ark_mimc_bls_compact_vk_only");
    generate_move_package(
        &out_dir,
        create_adapter("bls12_381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "ark_mimc_bls_compact_vk_only",
            module_name: "ark_mimc_bls_compact_vk_only",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(!out_dir.join("tests").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    aptos_move_compile(&out_dir);
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

#[test]
fn gnark_native_json_inputs_generate_aptos_package_files() {
    let artifact_dir = repo_root()
        .join("examples")
        .join("gnark-native")
        .join("cubic")
        .join("artifacts")
        .join("bn254");
    let inputs = load_gnark_json_inputs(
        &artifact_dir.join("verification_key_gnark.json"),
        Some(&artifact_dir.join("proof_gnark.json")),
        Some(&artifact_dir.join("public.json")),
        None,
    )
    .unwrap();

    let out_dir = temp_output_dir("gnark_json_bn254");
    generate_move_package(
        &out_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "gnark_json_bn254",
            module_name: "gnark_json_bn254",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(out_dir.join("tests").join("verifier_tests.move").exists());
}

#[test]
fn gnark_native_binary_inputs_generate_aptos_package_files() {
    let artifact_dir = repo_root()
        .join("examples")
        .join("gnark-native")
        .join("cubic")
        .join("artifacts")
        .join("bls12381");
    let inputs = load_gnark_binary_inputs(
        &artifact_dir.join("verification_key.bin"),
        Some(&artifact_dir.join("proof.bin")),
        Some(&artifact_dir.join("public.json")),
        "bls12381",
    )
    .unwrap();

    let out_dir = temp_output_dir("gnark_bin_bls12381");
    generate_move_package(
        &out_dir,
        create_adapter("bls12381").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "gnark_bin_bls12381",
            module_name: "gnark_bin_bls12381",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(out_dir.join("tests").join("verifier_tests.move").exists());
}

#[test]
fn sp1_groth16_inputs_generate_aptos_package_files() {
    let artifact_dir = repo_root()
        .join("examples")
        .join("sp1-groth16")
        .join("fibonacci")
        .join("artifacts");
    let inputs = load_sp1_groth16_inputs(
        &artifact_dir.join("groth16_vk_v5.bin"),
        &artifact_dir.join("fibonacci_proof.bin"),
    )
    .unwrap();

    let out_dir = temp_output_dir("sp1_groth16");
    generate_move_package(
        &out_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "sp1_groth16",
            module_name: "sp1_groth16",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    assert!(out_dir.join("Move.toml").exists());
    assert!(out_dir.join("sources").join("verifier.move").exists());
    assert!(out_dir.join("tests").join("verifier_tests.move").exists());
}

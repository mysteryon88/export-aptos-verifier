use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::error::Error;
use export_aptos_verifier_core::formats::{
    load_compact_bundle, load_snarkjs_json_inputs_with_curve_hint,
};
use export_aptos_verifier_core::model::{
    CurveKind, Groth16G1Point, Groth16G2Point, Groth16Proof, Groth16VerificationKey,
    Groth16VerifierInputs, SourceFormat,
};
use export_aptos_verifier_core::movegen::{
    generate_move_package, GenerateMovePackageOptions, MovegenMode,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn temp_path(name: &str) -> PathBuf {
    env::temp_dir().join(format!(
        "export_aptos_verifier_regression_{name}_{}",
        std::process::id()
    ))
}

fn fresh_dir(name: &str) -> PathBuf {
    let dir = temp_path(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_missing_curve_json_inputs(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    fs::create_dir_all(dir).unwrap();
    let vk = dir.join("verification_key.json");
    let proof = dir.join("proof.json");
    let public = dir.join("public.json");

    fs::write(
        &vk,
        r#"{
            "protocol":"groth16",
            "nPublic":1,
            "vk_alpha_1":["1","2","1"],
            "vk_beta_2":[["1","0"],["1","0"],["1","0"]],
            "vk_gamma_2":[["1","0"],["1","0"],["1","0"]],
            "vk_delta_2":[["1","0"],["1","0"],["1","0"]],
            "IC":[["1","2","1"],["1","2","1"]]
        }"#,
    )
    .unwrap();
    fs::write(
        &proof,
        r#"{
            "protocol":"groth16",
            "pi_a":["1","2","1"],
            "pi_b":[["1","0"],["1","0"],["1","0"]],
            "pi_c":["1","2","1"]
        }"#,
    )
    .unwrap();
    fs::write(&public, r#"["3"]"#).unwrap();

    (vk, proof, public)
}

fn dummy_g1() -> Groth16G1Point {
    Groth16G1Point {
        x: "1".to_string(),
        y: "2".to_string(),
        z: "1".to_string(),
    }
}

fn dummy_g2() -> Groth16G2Point {
    Groth16G2Point {
        x0: "1".to_string(),
        x1: "0".to_string(),
        y0: "1".to_string(),
        y1: "0".to_string(),
        z0: "1".to_string(),
        z1: "0".to_string(),
    }
}

fn dummy_inputs() -> Groth16VerifierInputs {
    Groth16VerifierInputs {
        curve: CurveKind::Bn254,
        protocol: "groth16".to_string(),
        verifying_key: Groth16VerificationKey {
            n_public: 0,
            vk_alpha_1: dummy_g1(),
            vk_beta_2: dummy_g2(),
            vk_gamma_2: dummy_g2(),
            vk_delta_2: dummy_g2(),
            ic: vec![dummy_g1()],
        },
        proof: Some(Groth16Proof {
            pi_a: dummy_g1(),
            pi_b: dummy_g2(),
            pi_c: dummy_g1(),
        }),
        public_inputs: vec![],
        source_format: SourceFormat::SnarkjsJson,
    }
}

#[test]
fn snarkjs_json_curve_hint_is_used_when_metadata_is_missing() {
    let dir = fresh_dir("missing_curve_json");
    let (vk, proof, public) = write_missing_curve_json_inputs(&dir);

    let inputs =
        load_snarkjs_json_inputs_with_curve_hint(&vk, &proof, Some(&public), Some("bn254"))
            .unwrap();

    assert_eq!(inputs.curve, CurveKind::Bn254);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn snarkjs_json_curve_hint_rejects_conflicting_metadata() {
    let dir = fresh_dir("conflicting_curve_json");
    let (vk, proof, public) = write_missing_curve_json_inputs(&dir);
    let vk_json = fs::read_to_string(&vk)
        .unwrap()
        .replace(r#""nPublic":1"#, r#""curve":"bls12_381","nPublic":1"#);
    fs::write(&vk, vk_json).unwrap();

    let err = load_snarkjs_json_inputs_with_curve_hint(&vk, &proof, Some(&public), Some("bn254"))
        .unwrap_err();

    assert!(matches!(err, Error::CurveMismatch(_)));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_public_input_keeps_64_digit_decimal_as_decimal() {
    let scalar = "1234567890123456789012345678901234567890123456789012345678901234";
    let source = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    json["public_input"] = serde_json::Value::String(scalar.to_string());

    let dir = fresh_dir("compact_decimal_scalar");
    let bundle = dir.join("groth16_artifacts.json");
    fs::write(&bundle, serde_json::to_string(&json).unwrap()).unwrap();

    let inputs = load_compact_bundle(&bundle, None).unwrap();

    assert_eq!(inputs.public_inputs, vec![scalar.to_string()]);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn generate_move_package_rejects_parent_traversal_force_output() {
    let base = fresh_dir("unsafe_force_parent");
    let child = base.join("child");
    fs::create_dir_all(&child).unwrap();
    let out = child.join("..");
    let inputs = dummy_inputs();
    let adapter = create_adapter("bn254").unwrap();

    let err = generate_move_package(
        &out,
        adapter.as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "unsafe_force_parent",
            module_name: "unsafe_force_parent",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap_err();

    assert!(matches!(err, Error::UnsafeOutputDirectory(_)));
    assert!(base.exists());
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn generate_move_package_rejects_invalid_account_address_before_writing() {
    let out = temp_path("invalid_account_address");
    if out.exists() {
        fs::remove_dir_all(&out).unwrap();
    }
    let inputs = dummy_inputs();
    let adapter = create_adapter("bn254").unwrap();

    let err = generate_move_package(
        &out,
        adapter.as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "invalid_account_address",
            module_name: "invalid_account_address",
            account_address: "CAFE",
            mode: MovegenMode::Entry,
            force: false,
        },
    )
    .unwrap_err();

    assert!(matches!(err, Error::InvalidAccountAddress(_)));
    assert!(!out.exists());
}

#[test]
fn generated_readme_documents_root_generate_flags() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let inputs = load_compact_bundle(&bundle, None).unwrap();
    let out = temp_path("generated_readme");
    if out.exists() {
        fs::remove_dir_all(&out).unwrap();
    }

    generate_move_package(
        &out,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "generated_readme",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Entry,
            force: true,
        },
    )
    .unwrap();

    let readme = fs::read_to_string(out.join("README.md")).unwrap();
    assert!(readme.contains("export-aptos-verifier --vk"));
    assert!(readme.contains("export-aptos-verifier --bundle"));
    assert!(!readme.contains("generate subcommand"));

    fs::remove_dir_all(out).unwrap();
}

fn compact_bundle_with_appended_hex_field(
    name: &str,
    field: &str,
    suffix: &str,
) -> (PathBuf, PathBuf) {
    let source = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    let original = json
        .get(field)
        .and_then(serde_json::Value::as_str)
        .unwrap()
        .to_string();
    json[field] = serde_json::Value::String(format!("{original}{suffix}"));

    let dir = fresh_dir(name);
    let bundle = dir.join("groth16_artifacts.json");
    fs::write(&bundle, serde_json::to_string(&json).unwrap()).unwrap();
    (dir, bundle)
}

#[test]
fn compact_bundle_rejects_trailing_vk_bytes() {
    let (dir, bundle) = compact_bundle_with_appended_hex_field("trailing_vk_bytes", "vk", "00");

    let err = load_compact_bundle(&bundle, None).unwrap_err();

    assert!(
        err.to_string().contains("trailing bytes"),
        "unexpected error: {err}"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_bundle_rejects_trailing_proof_bytes() {
    let (dir, bundle) =
        compact_bundle_with_appended_hex_field("trailing_proof_bytes", "proof", "00");

    let err = load_compact_bundle(&bundle, None).unwrap_err();

    assert!(
        err.to_string().contains("trailing bytes"),
        "unexpected error: {err}"
    );
    fs::remove_dir_all(dir).unwrap();
}

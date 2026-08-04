use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use ark_bn254::{Fq, G1Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::Field;
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
    generate_move_package, generate_move_package_with_framework_rev, proof_data_snippet,
    GenerateMovePackageOptions, MovegenMode, DEFAULT_APTOS_FRAMEWORK_REV,
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
fn public_generation_api_rejects_invalid_move_names_before_writing() {
    let inputs = dummy_inputs();
    let adapter = create_adapter("bn254").unwrap();

    for (idx, package_name, module_name, expected_module_error) in [
        (0, "bad\n[dependencies]", "verifier", false),
        (1, "verifier", "bad } public fun injected() {}", true),
    ] {
        let out = temp_path(&format!("invalid_move_name_{idx}"));
        if out.exists() {
            fs::remove_dir_all(&out).unwrap();
        }

        let err = generate_move_package(
            &out,
            adapter.as_ref(),
            &inputs,
            &GenerateMovePackageOptions {
                package_name,
                module_name,
                account_address: "0xCAFE",
                mode: MovegenMode::Library,
                force: false,
            },
        )
        .unwrap_err();

        if expected_module_error {
            assert!(matches!(err, Error::InvalidModuleName(_)));
        } else {
            assert!(matches!(err, Error::InvalidPackageName(_)));
        }
        assert!(!out.exists());
    }
}

#[test]
fn framework_revision_must_be_full_sha_and_is_recorded() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let inputs = load_compact_bundle(&bundle, None).unwrap();
    let adapter = create_adapter("bn254").unwrap();

    for (idx, revision) in [
        "",
        "mainnet",
        "deadbeef",
        "0123456789abcdef0123456789abcdef0123456z",
    ]
    .into_iter()
    .enumerate()
    {
        let out = temp_path(&format!("invalid_framework_rev_{idx}"));
        let result = generate_move_package_with_framework_rev(
            &out,
            adapter.as_ref(),
            &inputs,
            &GenerateMovePackageOptions {
                package_name: "invalid_framework_rev",
                module_name: "verifier",
                account_address: "0xCAFE",
                mode: MovegenMode::Library,
                force: true,
            },
            revision,
        );
        assert!(result.is_err());
        assert!(!out.exists());
    }

    assert_eq!(DEFAULT_APTOS_FRAMEWORK_REV.len(), 40);
    assert!(DEFAULT_APTOS_FRAMEWORK_REV
        .chars()
        .all(|character| character.is_ascii_hexdigit()));
    let out = temp_path("valid_framework_rev");
    generate_move_package_with_framework_rev(
        &out,
        adapter.as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "valid_framework_rev",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        },
        DEFAULT_APTOS_FRAMEWORK_REV,
    )
    .unwrap();
    let move_toml = fs::read_to_string(out.join("Move.toml")).unwrap();
    assert!(move_toml.contains(&format!("rev = \"{DEFAULT_APTOS_FRAMEWORK_REV}\"")));
    assert!(move_toml.contains("upgrade_policy = \"immutable\""));
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(out.join("verifier-manifest.json")).unwrap())
            .unwrap();
    assert_eq!(
        manifest["dependencies"]["framework_revision"],
        DEFAULT_APTOS_FRAMEWORK_REV
    );
    assert_eq!(manifest["upgrade_policy"], "immutable");
}

#[test]
fn public_generation_apis_reject_curve_confused_adapters() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let inputs = load_compact_bundle(&bundle, None).unwrap();
    let wrong_adapter = create_adapter("bls12381").unwrap();
    let out = temp_path("curve_confused_adapter");
    if out.exists() {
        fs::remove_dir_all(&out).unwrap();
    }

    let err = generate_move_package(
        &out,
        wrong_adapter.as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "curve_confused_adapter",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        },
    )
    .unwrap_err();
    assert!(matches!(err, Error::CurveMismatch(_)));
    assert!(!out.exists());

    let err = proof_data_snippet(wrong_adapter.as_ref(), &inputs).unwrap_err();
    assert!(matches!(err, Error::CurveMismatch(_)));
}

#[test]
fn force_generation_validates_vk_before_replacing_existing_output() {
    let bundle = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let mut inputs = load_compact_bundle(&bundle, None).unwrap();
    inputs.verifying_key.vk_alpha_1 = Groth16G1Point {
        x: "1".to_string(),
        y: "1".to_string(),
        z: "1".to_string(),
    };
    let out = fresh_dir("validate_before_replace");
    let sentinel = out.join("keep.txt");
    fs::write(&sentinel, "existing output").unwrap();

    let result = generate_move_package(
        &out,
        create_adapter("bn254").unwrap().as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: "validate_before_replace",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        },
    );

    assert!(result.is_err());
    assert_eq!(fs::read_to_string(sentinel).unwrap(), "existing output");
}

#[test]
fn canonical_vk_fingerprint_is_format_and_projective_invariant() {
    let artifact_dir = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254");
    let inputs = load_snarkjs_json_inputs_with_curve_hint(
        &artifact_dir.join("verification_key.json"),
        &artifact_dir.join("proof.json"),
        None,
        Some("bn254"),
    )
    .unwrap();
    let compact =
        load_compact_bundle(&artifact_dir.join("groth16_artifacts.json"), Some("bn254")).unwrap();
    let mut projective = inputs.clone();
    let alpha = &mut projective.verifying_key.vk_alpha_1;
    let z = Fq::from(2u64);
    alpha.x = (Fq::from_str(&alpha.x).unwrap() * z.square()).to_string();
    alpha.y = (Fq::from_str(&alpha.y).unwrap() * z.square() * z).to_string();
    alpha.z = z.to_string();

    let original_dir = fresh_dir("fingerprint_original");
    let compact_dir = fresh_dir("fingerprint_compact");
    let projective_dir = fresh_dir("fingerprint_projective");
    for (out_dir, candidate) in [
        (&original_dir, &inputs),
        (&compact_dir, &compact),
        (&projective_dir, &projective),
    ] {
        generate_move_package(
            out_dir,
            create_adapter("bn254").unwrap().as_ref(),
            candidate,
            &GenerateMovePackageOptions {
                package_name: "fingerprint_verifier",
                module_name: "verifier",
                account_address: "0xCAFE",
                mode: MovegenMode::Library,
                force: true,
            },
        )
        .unwrap();
    }

    let read_fingerprint = |dir: &Path| {
        let manifest: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join("verifier-manifest.json")).unwrap())
                .unwrap();
        manifest["vk_sha256"].as_str().unwrap().to_string()
    };
    let fingerprint = read_fingerprint(&original_dir);
    assert_eq!(fingerprint, read_fingerprint(&compact_dir));
    assert_eq!(fingerprint, read_fingerprint(&projective_dir));
    let source = fs::read_to_string(original_dir.join("sources/verifier.move")).unwrap();
    assert!(source.contains(&format!("x\"{fingerprint}\"")));

    let mut different = inputs;
    let replacement = G1Affine::generator().mul_bigint([2u64]).into_affine();
    different.verifying_key.vk_alpha_1 = Groth16G1Point {
        x: replacement.x.to_string(),
        y: replacement.y.to_string(),
        z: "1".to_string(),
    };
    let different_dir = fresh_dir("fingerprint_different");
    generate_move_package(
        &different_dir,
        create_adapter("bn254").unwrap().as_ref(),
        &different,
        &GenerateMovePackageOptions {
            package_name: "fingerprint_verifier",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        },
    )
    .unwrap();
    assert_ne!(fingerprint, read_fingerprint(&different_dir));
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

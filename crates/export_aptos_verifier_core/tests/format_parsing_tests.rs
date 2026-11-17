use std::fs;
use std::path::PathBuf;

use export_aptos_verifier_core::formats::{
    load_arkworks_inputs, load_compact_bundle, load_gnark_binary_inputs,
    load_gnark_binary_inputs_auto, load_gnark_json_inputs, load_snarkjs_json_inputs,
    load_sp1_groth16_inputs,
};
use export_aptos_verifier_core::{Error, SourceFormat};
use tempfile::NamedTempFile;

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

#[test]
fn gnark_native_json_bn254_inputs_load() {
    let root = repo_root()
        .join("examples")
        .join("gnark-native")
        .join("cubic")
        .join("artifacts")
        .join("bn254");

    let inputs = load_gnark_json_inputs(
        &root.join("verification_key_gnark.json"),
        Some(&root.join("proof_gnark.json")),
        Some(&root.join("public.json")),
        None,
    )
    .unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bn254");
    assert_eq!(inputs.source_format, SourceFormat::GnarkJson);
    assert!(inputs.has_test_vectors());
    assert_eq!(inputs.public_inputs.len(), 1);
}

#[test]
fn gnark_native_binary_bls12381_inputs_load() {
    let root = repo_root()
        .join("examples")
        .join("gnark-native")
        .join("cubic")
        .join("artifacts")
        .join("bls12381");

    let inputs = load_gnark_binary_inputs(
        &root.join("verification_key.bin"),
        Some(&root.join("proof.bin")),
        Some(&root.join("public.json")),
        "bls12381",
    )
    .unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bls12381");
    assert_eq!(inputs.source_format, SourceFormat::GnarkBinary);
    assert!(inputs.has_test_vectors());
    assert_eq!(inputs.public_inputs.len(), 1);
}

#[test]
fn gnark_native_binary_curve_can_be_auto_detected() {
    let root = repo_root()
        .join("examples")
        .join("gnark-native")
        .join("cubic")
        .join("artifacts")
        .join("bn254");

    let inputs = load_gnark_binary_inputs_auto(
        &root.join("verification_key.bin"),
        Some(&root.join("proof.bin")),
        Some(&root.join("public.json")),
    )
    .unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bn254");
    assert_eq!(inputs.source_format, SourceFormat::GnarkBinary);
}

#[test]
fn sp1_groth16_wrapper_inputs_load() {
    // SP1 fixtures are copied from mysteryon88/export-sui-verifier and
    // reused here to verify Aptos generation against the Sui reference artifacts.
    let root = repo_root()
        .join("examples")
        .join("sp1-groth16")
        .join("fibonacci")
        .join("artifacts");

    let inputs = load_sp1_groth16_inputs(
        &root.join("groth16_vk_v5.bin"),
        &root.join("fibonacci_proof.bin"),
    )
    .unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bn254");
    assert_eq!(inputs.source_format, SourceFormat::Sp1Groth16);
    assert!(inputs.has_test_vectors());
    assert_eq!(inputs.public_inputs.len(), 2);
}

#[test]
fn sp1_groth16_v6_wrapper_inputs_load() {
    // SP1 fixtures are copied from mysteryon88/export-sui-verifier and
    // reused here to verify Aptos generation against the Sui reference artifacts.
    let root = repo_root()
        .join("examples")
        .join("sp1-groth16")
        .join("fibonacci")
        .join("artifacts");

    let inputs = load_sp1_groth16_inputs(
        &root.join("sp1_groth16_vk.bin"),
        &root.join("fibonacci_sp1_6_proof.bin"),
    )
    .unwrap();

    assert_eq!(inputs.curve.canonical_name(), "bn254");
    assert_eq!(inputs.source_format, SourceFormat::Sp1Groth16);
    assert!(inputs.has_test_vectors());
    assert_eq!(inputs.public_inputs.len(), 5);
}

#[test]
fn compact_proof_hex_is_bounded_before_decoding() {
    let source = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let mut bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    bundle["proof"] = serde_json::Value::String("00".repeat(1025));

    let input_dir = temp_output_dir("oversized_compact_proof");
    fs::create_dir_all(&input_dir).unwrap();
    let path = input_dir.join("groth16_artifacts.json");
    fs::write(&path, bundle.to_string()).unwrap();

    let err = load_compact_bundle(&path, None).unwrap_err();

    assert!(matches!(err, Error::HexParse(_)));
}

#[test]
fn public_json_loaders_reject_oversized_artifacts_before_parsing() {
    let file = NamedTempFile::new().unwrap();
    file.as_file().set_len(16 * 1024 * 1024 + 1).unwrap();

    let gnark = load_gnark_json_inputs(file.path(), None, None, None);
    let arkworks = load_arkworks_inputs(file.path(), None, None, Some("bn254"));

    assert!(matches!(gnark, Err(Error::InputTooLarge { .. })));
    assert!(matches!(arkworks, Err(Error::InputTooLarge { .. })));
}

#[test]
fn compact_vk_declared_ic_count_is_bounded_before_arkworks_allocation() {
    let source = repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join("bn254")
        .join("groth16_artifacts.json");
    let mut bundle: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    let mut vk = hex::decode(bundle["vk"].as_str().unwrap()).unwrap();
    vk[224..232].copy_from_slice(&65_538u64.to_le_bytes());
    bundle["vk"] = serde_json::Value::String(hex::encode(vk));

    let input_dir = temp_output_dir("oversized_compact_ic");
    fs::create_dir_all(&input_dir).unwrap();
    let path = input_dir.join("groth16_artifacts.json");
    fs::write(&path, bundle.to_string()).unwrap();

    let err = load_compact_bundle(&path, None).unwrap_err();

    assert!(matches!(err, Error::IcLengthMismatch(_)));
}

#[test]
fn hinted_gnark_rejects_invalid_points_and_scalars() {
    use ark_ff::PrimeField;
    let dir = tempfile::tempdir().unwrap();
    let vk_path = dir.path().join("vk.json");
    let public_path = dir.path().join("public.json");
    let root = repo_root().join("examples/gnark-native/cubic/artifacts");
    for (folder, hint) in [("bn254", "bn254"), ("bls12381", "bls12381")] {
        let valid: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(root.join(folder).join("verification_key_gnark.json")).unwrap(),
        )
        .unwrap();
        for (x, y) in [
            ("1", "1"),
            ("0", "0"),
            ("nonnumeric", "2"),
            (&"9".repeat(200), "2"),
        ] {
            let mut bad = valid.clone();
            bad["G1"]["Alpha"]["X"] = x.into();
            bad["G1"]["Alpha"]["Y"] = y.into();
            fs::write(&vk_path, bad.to_string()).unwrap();
            for h in [Some(hint), None] {
                assert!(
                    load_gnark_json_inputs(&vk_path, None, None, h).is_err(),
                    "accepted {folder} ({x}, {y}) hint={h:?}"
                );
            }
        }
        if folder == "bls12381" {
            // (0, 2) is on BLS12-381 G1 but has order 3, outside the prime subgroup.
            let mut bad = valid.clone();
            bad["G1"]["Alpha"]["X"] = "0".into();
            bad["G1"]["Alpha"]["Y"] = "2".into();
            fs::write(&vk_path, bad.to_string()).unwrap();
            for h in [Some(hint), None] {
                assert!(load_gnark_json_inputs(&vk_path, None, None, h).is_err());
            }
        }
        fs::write(&vk_path, valid.to_string()).unwrap();
        assert!(load_gnark_json_inputs(&vk_path, None, None, Some(hint)).is_ok());
        let modulus = if folder == "bn254" {
            ark_bn254::Fr::MODULUS.to_string()
        } else {
            ark_bls12_381::Fr::MODULUS.to_string()
        };
        fs::write(&public_path, serde_json::json!([modulus]).to_string()).unwrap();
        assert!(load_gnark_json_inputs(&vk_path, None, Some(&public_path), Some(hint)).is_err());
        fs::write(&public_path, "[\"1\"]").unwrap();
        assert!(load_gnark_json_inputs(&vk_path, None, Some(&public_path), Some(hint)).is_ok());
    }
}

#[test]
fn sp1_wrapper_reconciles_redundant_data_and_trailer() {
    let root = repo_root().join("examples/sp1-groth16/fibonacci/artifacts");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("proof.bin");
    for (name, key, fields) in [
        ("fibonacci_proof.bin", "groth16_vk_v5.bin", 4),
        ("fibonacci_sp1_6_proof.bin", "sp1_groth16_vk.bin", 7),
    ] {
        let valid = fs::read(root.join(name)).unwrap();
        assert!(load_sp1_groth16_inputs(&root.join(key), &root.join(name)).is_ok());
        let mut offset = 4;
        let mut spans = Vec::new();
        for _ in 0..fields {
            let len = u64::from_le_bytes(valid[offset..offset + 8].try_into().unwrap()) as usize;
            offset += 8;
            spans.push(offset..offset + len);
            offset += len;
        }
        let hash_offset = offset;
        let pv_len =
            u64::from_le_bytes(valid[offset + 32..offset + 40].try_into().unwrap()) as usize;
        let pv_offset = offset + 40;
        let version_offset = pv_offset + pv_len + 8;
        for mutation in [
            "raw",
            "hash",
            "version",
            "public_values",
            "trailing",
            "huge",
            "metadata",
            "truncated",
            "tee",
        ] {
            let mut bad = valid.clone();
            match mutation {
                "raw" => {
                    let other = if fields == 4 {
                        "fibonacci_sp1_6_proof.bin"
                    } else {
                        "fibonacci_proof.bin"
                    };
                    let other = fs::read(root.join(other)).unwrap();
                    let mut other_offset = 4;
                    let mut raw = &other[0..0];
                    for _ in 0..if fields == 4 { 7 } else { 4 } {
                        let len = u64::from_le_bytes(
                            other[other_offset..other_offset + 8].try_into().unwrap(),
                        ) as usize;
                        other_offset += 8;
                        raw = &other[other_offset..other_offset + len];
                        other_offset += len;
                    }
                    bad[spans.last().unwrap().clone()].copy_from_slice(raw);
                }
                "hash" => bad[hash_offset] ^= 1,
                "version" => bad[version_offset + 1] = b'9',
                "public_values" => bad[pv_offset] ^= 1,
                "trailing" => bad.push(0),
                "huge" => bad[4..12].copy_from_slice(&u64::MAX.to_le_bytes()),
                "metadata" => {
                    if fields == 4 {
                        continue;
                    }
                    let i = spans[5].start + 63;
                    bad[i] = if bad[i] == b'0' { b'1' } else { b'0' };
                }
                "truncated" => bad.truncate(hash_offset),
                "tee" => *bad.last_mut().unwrap() = 2,
                _ => unreachable!(),
            }
            fs::write(&path, bad).unwrap();
            assert!(
                load_sp1_groth16_inputs(&root.join(key), &path).is_err(),
                "accepted {name} mutation {mutation}"
            );
        }
        fs::write(&path, &valid[..valid.len() - 1]).unwrap();
        assert!(load_sp1_groth16_inputs(&root.join(key), &path).is_ok());
    }
}

#[test]
fn sp1_wrapper_accepts_producer_blake3_public_values_hash() {
    let root = repo_root().join("examples/sp1-groth16/fibonacci/artifacts");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("blake3.bin");
    // Official SP1PublicValues Blake3 vector (v5.0.0 and v6.1.0 primitives/src/io.rs).
    let values = hex::decode("1234567890abcdef".repeat(8)).unwrap();
    let digest = num_bigint::BigUint::parse_bytes(
        b"1639647ab9e42519f0cbdcf343033482b018c0c1ed48f8367f32381c60913447",
        16,
    )
    .unwrap()
    .to_string();
    for (name, key, fields) in [
        ("fibonacci_proof.bin", "groth16_vk_v5.bin", 4),
        ("fibonacci_sp1_6_proof.bin", "sp1_groth16_vk.bin", 7),
    ] {
        let valid = fs::read(root.join(name)).unwrap();
        let mut output = valid[..4].to_vec();
        let mut offset = 4;
        for index in 0..fields {
            let len = u64::from_le_bytes(valid[offset..offset + 8].try_into().unwrap()) as usize;
            offset += 8;
            let value = if index == 1 {
                digest.as_bytes()
            } else {
                &valid[offset..offset + len]
            };
            output.extend_from_slice(&(value.len() as u64).to_le_bytes());
            output.extend_from_slice(value);
            offset += len;
        }
        output.extend_from_slice(&valid[offset..offset + 32]);
        offset += 32;
        let len = u64::from_le_bytes(valid[offset..offset + 8].try_into().unwrap()) as usize;
        offset += 8 + len;
        let values_offset = output.len() + 8;
        output.extend_from_slice(&(values.len() as u64).to_le_bytes());
        output.extend_from_slice(&values);
        output.extend_from_slice(&valid[offset..]);
        fs::write(&path, &output).unwrap();
        let inputs = load_sp1_groth16_inputs(&root.join(key), &path).unwrap();
        assert_eq!(inputs.public_inputs[1], digest);
        // Hash/geometry checks do not silently enforce the optional pairing policy.
        assert!(!export_aptos_verifier_core::create_adapter("bn254")
            .unwrap()
            .local_verify(&inputs)
            .unwrap());
        output[values_offset] ^= 1;
        fs::write(&path, &output).unwrap();
        assert!(load_sp1_groth16_inputs(&root.join(key), &path).is_err());
    }
}

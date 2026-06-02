use std::path::PathBuf;

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::snarkjs::{
    parse_proof, parse_public_inputs, parse_verification_key, validate_curve_match,
    validate_protocol, validate_public_counts,
};
use export_aptos_verifier_core::{Groth16G1Point, Groth16G2Point};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[test]
fn parse_bn254_fixtures() {
    let root = fixture_root().join("bn254");
    let vk = parse_verification_key(&root.join("verification_key.json")).unwrap();
    let proof = parse_proof(&root.join("proof.json")).unwrap();
    let public = parse_public_inputs(&root.join("public.json")).unwrap();

    validate_protocol(vk.protocol.as_ref(), proof.protocol.as_ref()).unwrap();
    assert!(validate_curve_match(vk.curve.as_ref(), proof.curve.as_ref()).is_ok());
    assert!(validate_public_counts(&vk, &public).is_ok());
    assert_eq!(public.len(), vk.n_public);
    assert_eq!(vk.ic.len(), vk.n_public + 1);

    let adapter = create_adapter("bn254").unwrap();
    let alpha: Groth16G1Point = vk.vk_alpha_1.clone().into();
    let beta: Groth16G2Point = vk.vk_beta_2.clone().into();
    assert_eq!(adapter.serialize_g1_vk(&alpha).unwrap().len(), 64);
    assert_eq!(adapter.serialize_g2_vk(&beta).unwrap().len(), 128);
    assert_eq!(
        adapter.serialize_fr_public_input(&public[0]).unwrap().len(),
        32
    );
}

#[test]
fn parse_bls12381_fixtures() {
    let root = fixture_root().join("bls12381");
    let vk = parse_verification_key(&root.join("verification_key.json")).unwrap();
    let proof = parse_proof(&root.join("proof.json")).unwrap();
    let public = parse_public_inputs(&root.join("public.json")).unwrap();
    assert!(validate_protocol(vk.protocol.as_ref(), proof.protocol.as_ref()).is_ok());
    assert!(validate_curve_match(vk.curve.as_ref(), proof.curve.as_ref()).is_ok());
    assert_eq!(public.len(), vk.n_public);
    assert_eq!(vk.ic.len(), vk.n_public + 1);

    let adapter = create_adapter("bls12_381").unwrap();
    let alpha: Groth16G1Point = vk.vk_alpha_1.clone().into();
    let beta: Groth16G2Point = vk.vk_beta_2.clone().into();
    assert_eq!(adapter.serialize_g1_vk(&alpha).unwrap().len(), 48);
    assert_eq!(adapter.serialize_g2_vk(&beta).unwrap().len(), 96);
    assert_eq!(
        adapter.serialize_fr_public_input(&public[0]).unwrap().len(),
        32
    );
}

#[test]
fn reject_wrong_ic_length() {
    let root = fixture_root().join("bn254");
    let mut vk = parse_verification_key(&root.join("verification_key.json")).unwrap();
    let public = parse_public_inputs(&root.join("public.json")).unwrap();

    vk.ic.pop();
    let err = validate_public_counts(&vk, &public).unwrap_err();
    assert!(format!("{err}").contains("ERR_IC_LENGTH"));
}

#[test]
fn curve_names_are_normalized() {
    let bn254 =
        validate_curve_match(Some(&"bn128".to_string()), Some(&"alt_bn128".to_string())).unwrap();
    assert_eq!(bn254, "bn254");

    let bls = validate_curve_match(
        Some(&"bls12-381".to_string()),
        Some(&"bls12_381".to_string()),
    )
    .unwrap();
    assert_eq!(bls, "bls12381");
}

#[test]
fn curve_mismatch_is_rejected() {
    let err = validate_curve_match(Some(&"bn254".to_string()), Some(&"bls12_381".to_string()))
        .unwrap_err();
    assert!(format!("{err}").contains("ERR_CURVE_MISMATCH"));
}

#[test]
fn reject_public_input_count_mismatch() {
    let root = fixture_root().join("bn254");
    let vk = parse_verification_key(&root.join("verification_key.json")).unwrap();
    let mut public = parse_public_inputs(&root.join("public.json")).unwrap();
    public.push("1".to_string());
    let err = validate_public_counts(&vk, &public).unwrap_err();
    assert!(format!("{err}").contains("ERR_PUBLIC_INPUT_COUNT_MISMATCH"));
}

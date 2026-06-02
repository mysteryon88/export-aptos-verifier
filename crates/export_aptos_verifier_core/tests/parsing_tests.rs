use ark_bls12_381::Fr as BlsFr;
use ark_bn254::Fr;
use ark_ff::PrimeField;
use std::env;
use std::fs::write;
use std::path::PathBuf;

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::error::Error;
use export_aptos_verifier_core::snarkjs::{
    parse_proof, parse_public_inputs, parse_verification_key,
};

fn tmp(path: &str) -> PathBuf {
    let mut p = env::temp_dir();
    p.push(format!("snarkjs_aptos_tests_{path}"));
    p
}

#[test]
fn malformed_g1_is_rejected() {
    let path = tmp("malformed_vk.json");
    let _ = write(
        &path,
        r#"{
            "protocol":"groth16",
            "curve":"bn254",
            "nPublic":1,
            "vk_alpha_1":["1","2"],
            "vk_beta_2":[["1","0"],["1","0"],["1","0"]],
            "vk_gamma_2":[["1","0"],["1","0"],["1","0"]],
            "vk_delta_2":[["1","0"],["1","0"],["1","0"]],
            "IC":[["1","2","1"],["1","2","1"]]
        }"#,
    );
    let err = parse_verification_key(&path).unwrap_err();
    assert!(matches!(err, Error::MalformedG1(_)));
}

#[test]
fn malformed_g2_is_rejected() {
    let path = tmp("malformed_proof.json");
    let _ = write(
        &path,
        r#"{
            "protocol":"groth16",
            "curve":"bn254",
            "pi_a":["1","2","1"],
            "pi_b":[["1","0"],["1","0"]],
            "pi_c":["1","2","1"]
        }"#,
    );
    let err = parse_proof(&path).unwrap_err();
    assert!(matches!(err, Error::MalformedG2(_)));
}

#[test]
fn public_numbers_and_strings_supported() {
    let path = tmp("public_inputs.json");
    let _ = write(&path, b"[1, \"2\", 3, \"4\"]");
    let values = parse_public_inputs(&path).unwrap();
    assert_eq!(
        values,
        vec![
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string()
        ]
    );
}

#[test]
fn bn254_public_input_field_overflow_rejected() {
    let adapter = create_adapter("bn254").unwrap();
    let overflow = Fr::MODULUS.to_string();
    let err = adapter.serialize_fr_public_input(&overflow).unwrap_err();
    assert!(matches!(err, Error::FieldOverflow(_)));
}

#[test]
fn bls12381_public_input_field_overflow_rejected() {
    let adapter = create_adapter("bls12381").unwrap();
    let overflow = BlsFr::MODULUS.to_string();
    let err = adapter.serialize_fr_public_input(&overflow).unwrap_err();
    assert!(matches!(err, Error::FieldOverflow(_)));
}

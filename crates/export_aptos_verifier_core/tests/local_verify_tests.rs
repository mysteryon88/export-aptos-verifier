use std::path::PathBuf;

use export_aptos_verifier_core::curves::create_adapter;
use export_aptos_verifier_core::formats::{load_compact_bundle, load_snarkjs_json_inputs};
use export_aptos_verifier_core::local_verify;
use export_aptos_verifier_core::model::Groth16VerifierInputs;
use export_aptos_verifier_core::Error;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn ark_mimc_artifact_dir(curve: &str) -> PathBuf {
    repo_root()
        .join("examples")
        .join("ark-mimc")
        .join("artifacts")
        .join(curve)
}

fn load_ark_mimc_snarkjs(curve: &str) -> Groth16VerifierInputs {
    let dir = ark_mimc_artifact_dir(curve);
    load_snarkjs_json_inputs(
        &dir.join("verification_key.json"),
        &dir.join("proof.json"),
        None,
    )
    .unwrap()
}

fn assert_local_verify(curve: &str, inputs: &Groth16VerifierInputs, expected: bool) {
    let adapter = create_adapter(curve).unwrap();
    assert_eq!(local_verify(adapter.as_ref(), inputs).unwrap(), expected);
}

#[test]
fn local_verify_accepts_valid_snarkjs_proofs_for_both_curves() {
    let bn254 = load_ark_mimc_snarkjs("bn254");
    assert_local_verify("bn254", &bn254, true);

    let bls12381 = load_ark_mimc_snarkjs("bls12_381");
    assert_local_verify("bls12_381", &bls12381, true);
}

#[test]
fn local_verify_accepts_valid_arkworks_compact_bundles_for_both_curves() {
    let bn254 = load_compact_bundle(
        &ark_mimc_artifact_dir("bn254").join("groth16_artifacts.json"),
        None,
    )
    .unwrap();
    assert_local_verify("bn254", &bn254, true);

    let bls12381 = load_compact_bundle(
        &ark_mimc_artifact_dir("bls12_381").join("groth16_artifacts.json"),
        None,
    )
    .unwrap();
    assert_local_verify("bls12_381", &bls12381, true);
}

#[test]
fn local_verify_rejects_wrong_public_input() {
    let mut inputs = load_ark_mimc_snarkjs("bn254");
    inputs.public_inputs[0] = if inputs.public_inputs[0] == "0" {
        "1".to_string()
    } else {
        "0".to_string()
    };

    assert_local_verify("bn254", &inputs, false);
}

#[test]
fn local_verify_rejects_swapped_proof_points() {
    let mut inputs = load_ark_mimc_snarkjs("bls12_381");
    let proof = inputs.proof.as_mut().unwrap();
    std::mem::swap(&mut proof.pi_a, &mut proof.pi_c);

    assert_local_verify("bls12_381", &inputs, false);
}

#[test]
fn local_verify_rejects_trailing_public_inputs_at_both_boundaries() {
    for curve in ["bn254", "bls12_381"] {
        let mut inputs = load_ark_mimc_snarkjs(curve);
        inputs.public_inputs.push("0".to_string());
        let adapter = create_adapter(curve).unwrap();

        let wrapped = local_verify(adapter.as_ref(), &inputs);
        let direct = adapter.local_verify(&inputs);

        assert!(matches!(wrapped, Err(Error::PublicInputCountMismatch(_))));
        assert!(matches!(direct, Err(Error::PublicInputCountMismatch(_))));
    }
}

#[test]
fn local_verify_rejects_empty_ic_without_panicking_at_both_boundaries() {
    for curve in ["bn254", "bls12_381"] {
        let mut inputs = load_ark_mimc_snarkjs(curve);
        inputs.verifying_key.ic.clear();
        let adapter = create_adapter(curve).unwrap();

        let wrapped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            local_verify(adapter.as_ref(), &inputs)
        }));
        let direct = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            adapter.local_verify(&inputs)
        }));

        assert!(matches!(wrapped, Ok(Err(Error::IcLengthMismatch(_)))));
        assert!(matches!(direct, Ok(Err(Error::IcLengthMismatch(_)))));
    }
}

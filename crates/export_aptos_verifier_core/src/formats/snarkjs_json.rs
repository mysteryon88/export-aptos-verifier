use crate::error::{Error, Result};
use crate::model::{CurveKind, Groth16VerifierInputs, SourceFormat};
use crate::snarkjs::{parse_decimal, parse_proof, parse_public_inputs, parse_verification_key};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ProofPublicSignals {
    #[serde(rename = "publicSignals", default)]
    public_signals: Vec<Value>,
}

pub fn load_snarkjs_json_inputs(
    vk_path: &Path,
    proof_path: &Path,
    public_path: Option<&Path>,
) -> Result<Groth16VerifierInputs> {
    load_snarkjs_json_inputs_with_curve_hint(vk_path, proof_path, public_path, None)
}

pub fn load_snarkjs_json_inputs_with_curve_hint(
    vk_path: &Path,
    proof_path: &Path,
    public_path: Option<&Path>,
    curve_hint: Option<&str>,
) -> Result<Groth16VerifierInputs> {
    let mut vk = parse_verification_key(vk_path)?;
    let mut proof = parse_proof(proof_path)?;
    apply_curve_hint(&mut vk.curve, &mut proof.curve, curve_hint)?;

    let public_inputs = match public_path {
        Some(path) => parse_public_inputs(path)?,
        None => load_public_signals_from_proof(proof_path)?,
    };

    Groth16VerifierInputs::from_legacy(vk, proof, public_inputs, SourceFormat::SnarkjsJson)
}

fn apply_curve_hint(
    vk_curve: &mut Option<String>,
    proof_curve: &mut Option<String>,
    curve_hint: Option<&str>,
) -> Result<()> {
    let Some(raw_hint) = curve_hint else {
        return Ok(());
    };

    let hinted_curve = CurveKind::from_name(raw_hint)?;
    let canonical_hint = hinted_curve.canonical_name().to_string();

    for (field, existing) in [
        ("verification key", vk_curve.as_deref()),
        ("proof", proof_curve.as_deref()),
    ] {
        if let Some(existing) = existing {
            let existing_curve = CurveKind::from_name(existing)?;
            if existing_curve != hinted_curve {
                return Err(Error::CurveMismatch(format!(
                    "requested curve {canonical_hint} does not match {field} curve {existing}"
                )));
            }
        }
    }

    if vk_curve.is_none() {
        *vk_curve = Some(canonical_hint.clone());
    }
    if proof_curve.is_none() {
        *proof_curve = Some(canonical_hint);
    }

    Ok(())
}

fn load_public_signals_from_proof(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path).map_err(|e| Error::Io {
        source: e,
        context: format!("failed to read file {}", path.display()),
    })?;
    let proof: ProofPublicSignals =
        serde_json::from_str(&content).map_err(|e| Error::JsonParse {
            source: e,
            context: format!(
                "failed to deserialize proof publicSignals from {}",
                path.display()
            ),
        })?;

    if proof.public_signals.is_empty() {
        return Err(Error::MissingInput(
            "proof.json does not contain publicSignals and --public was not supplied".to_string(),
        ));
    }

    proof
        .public_signals
        .iter()
        .enumerate()
        .map(|(idx, value)| match value {
            Value::String(raw) => {
                parse_decimal(raw, &format!("publicSignals[{idx}]"))?;
                Ok(raw.clone())
            }
            Value::Number(num) => {
                let decimal = num.to_string();
                parse_decimal(&decimal, &format!("publicSignals[{idx}]"))?;
                Ok(decimal)
            }
            _ => Err(Error::DecimalParse(format!(
                "publicSignals[{idx}] must be string or number"
            ))),
        })
        .collect()
}

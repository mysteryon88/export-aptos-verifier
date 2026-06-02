use crate::error::{Error, Result};
use crate::model::{Groth16VerifierInputs, SourceFormat};
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
    let vk = parse_verification_key(vk_path)?;
    let proof = parse_proof(proof_path)?;
    let public_inputs = match public_path {
        Some(path) => parse_public_inputs(path)?,
        None => load_public_signals_from_proof(proof_path)?,
    };

    Groth16VerifierInputs::from_legacy(vk, proof, public_inputs, SourceFormat::SnarkjsJson)
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

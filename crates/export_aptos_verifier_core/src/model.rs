use crate::error::{Error, Result};
use crate::snarkjs::{
    validate_curve_match, validate_protocol, validate_public_counts,
    validate_verification_key_geometry, Proof as LegacyProof, SnarkJsG1, SnarkJsG2,
    VerificationKey as LegacyVerificationKey,
};

pub type DecimalValue = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind {
    Bn254,
    Bls12_381,
}

impl CurveKind {
    pub fn from_name(value: &str) -> Result<Self> {
        match normalize_curve_name(value).as_str() {
            "bn128" | "bn254" | "altbn128" => Ok(Self::Bn254),
            "bls12381" => Ok(Self::Bls12_381),
            _ => Err(Error::UnsupportedCurve(format!(
                "unsupported curve: {value}"
            ))),
        }
    }

    pub fn canonical_name(self) -> &'static str {
        match self {
            Self::Bn254 => "bn254",
            Self::Bls12_381 => "bls12381",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    SnarkjsJson,
    ArkworksCompact,
}

#[derive(Debug, Clone)]
pub struct Groth16G1Point {
    pub x: DecimalValue,
    pub y: DecimalValue,
    pub z: DecimalValue,
}

#[derive(Debug, Clone)]
pub struct Groth16G2Point {
    pub x0: DecimalValue,
    pub x1: DecimalValue,
    pub y0: DecimalValue,
    pub y1: DecimalValue,
    pub z0: DecimalValue,
    pub z1: DecimalValue,
}

#[derive(Debug, Clone)]
pub struct Groth16VerificationKey {
    pub n_public: usize,
    pub vk_alpha_1: Groth16G1Point,
    pub vk_beta_2: Groth16G2Point,
    pub vk_gamma_2: Groth16G2Point,
    pub vk_delta_2: Groth16G2Point,
    pub ic: Vec<Groth16G1Point>,
}

#[derive(Debug, Clone)]
pub struct Groth16Proof {
    pub pi_a: Groth16G1Point,
    pub pi_b: Groth16G2Point,
    pub pi_c: Groth16G1Point,
}

#[derive(Debug, Clone)]
pub struct Groth16VerifierInputs {
    pub curve: CurveKind,
    pub protocol: String,
    pub verifying_key: Groth16VerificationKey,
    pub proof: Groth16Proof,
    pub public_inputs: Vec<DecimalValue>,
    pub source_format: SourceFormat,
}

impl Groth16VerifierInputs {
    pub fn from_legacy(
        vk: LegacyVerificationKey,
        proof: LegacyProof,
        public_inputs: Vec<DecimalValue>,
        source_format: SourceFormat,
    ) -> Result<Self> {
        validate_protocol(vk.protocol.as_ref(), proof.protocol.as_ref())?;
        validate_verification_key_geometry(&vk)?;
        validate_public_counts(&vk, &public_inputs)?;

        let curve_name = validate_curve_match(vk.curve.as_ref(), proof.curve.as_ref())?;
        let curve = CurveKind::from_name(&curve_name)?;
        let protocol = vk
            .protocol
            .clone()
            .or_else(|| proof.protocol.clone())
            .unwrap_or_else(|| "groth16".to_string());

        Ok(Self {
            curve,
            protocol,
            verifying_key: Groth16VerificationKey {
                n_public: vk.n_public,
                vk_alpha_1: vk.vk_alpha_1.into(),
                vk_beta_2: vk.vk_beta_2.into(),
                vk_gamma_2: vk.vk_gamma_2.into(),
                vk_delta_2: vk.vk_delta_2.into(),
                ic: vk.ic.into_iter().map(Into::into).collect(),
            },
            proof: Groth16Proof {
                pi_a: proof.pi_a.into(),
                pi_b: proof.pi_b.into(),
                pi_c: proof.pi_c.into(),
            },
            public_inputs,
            source_format,
        })
    }
}

impl From<SnarkJsG1> for Groth16G1Point {
    fn from(value: SnarkJsG1) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<SnarkJsG2> for Groth16G2Point {
    fn from(value: SnarkJsG2) -> Self {
        Self {
            x0: value.x0,
            x1: value.x1,
            y0: value.y0,
            y1: value.y1,
            z0: value.z0,
            z1: value.z1,
        }
    }
}

fn normalize_curve_name(value: &str) -> String {
    value.to_lowercase().replace(['-', '_'], "")
}

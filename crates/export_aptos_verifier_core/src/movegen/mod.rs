mod context;
mod render;

use crate::bytes::move_hex_literal;
use crate::curves::{CurveAdapter, CurveId};
use crate::error::{Error, Result};
use crate::model::{CurveKind, Groth16VerifierInputs, SourceFormat};
pub use context::{MovegenMode, MovegenTemplateInput};
use handlebars::Handlebars;
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, create_dir_all, write};
use std::path::{Component, Path};
use tempfile::{Builder as TempDirBuilder, TempDir};

#[derive(Debug, Clone)]
pub struct GenerateMovePackageOptions<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub account_address: &'a str,
    pub mode: MovegenMode,
    pub force: bool,
}

/// Aptos Framework revision shipped with Aptos CLI 9.4.0.
pub const DEFAULT_APTOS_FRAMEWORK_REV: &str = "7f900aa660b13bb674924f279f1fd7d55e0cf79e";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofDataSnippet {
    pub proof_a: String,
    pub proof_b: String,
    pub proof_c: String,
    pub public_inputs_rendered: String,
}

impl ProofDataSnippet {
    pub fn render_aptos_test_functions(&self) -> String {
        format!(
            r#"    fun proof_a_bytes(): vector<u8> {{ {} }}
    fun proof_b_bytes(): vector<u8> {{ {} }}
    fun proof_c_bytes(): vector<u8> {{ {} }}
    fun public_inputs_bytes(): vector<vector<u8>> {{ {} }}"#,
            self.proof_a, self.proof_b, self.proof_c, self.public_inputs_rendered
        )
    }
}

pub fn proof_data_snippet(
    adapter: &dyn CurveAdapter,
    inputs: &Groth16VerifierInputs,
) -> Result<ProofDataSnippet> {
    ensure_adapter_matches_inputs(adapter, inputs)?;
    let proof = inputs.proof.as_ref().ok_or_else(|| {
        Error::MissingInput("proof-data requires proof input; VK-only inputs have no proof".into())
    })?;

    let public_inputs_bytes: Vec<String> = inputs
        .public_inputs
        .iter()
        .map(|value| {
            adapter
                .serialize_fr_public_input(value)
                .map(|bytes| move_hex_literal(&bytes))
        })
        .collect::<Result<_>>()?;

    Ok(ProofDataSnippet {
        proof_a: move_hex_literal(&adapter.serialize_g1_proof(&proof.pi_a)?),
        proof_b: move_hex_literal(&adapter.serialize_g2_proof(&proof.pi_b)?),
        proof_c: move_hex_literal(&adapter.serialize_g1_proof(&proof.pi_c)?),
        public_inputs_rendered: render::vector_of_hex(&public_inputs_bytes),
    })
}

pub fn generate_move_package(
    out_dir: &Path,
    adapter: &dyn CurveAdapter,
    inputs: &Groth16VerifierInputs,
    options: &GenerateMovePackageOptions<'_>,
) -> Result<()> {
    generate_move_package_with_framework_rev(
        out_dir,
        adapter,
        inputs,
        options,
        DEFAULT_APTOS_FRAMEWORK_REV,
    )
}

pub fn generate_move_package_with_framework_rev(
    out_dir: &Path,
    adapter: &dyn CurveAdapter,
    inputs: &Groth16VerifierInputs,
    options: &GenerateMovePackageOptions<'_>,
    aptos_framework_rev: &str,
) -> Result<()> {
    ensure_adapter_matches_inputs(adapter, inputs)?;
    validate_move_names(options.package_name, options.module_name)?;
    validate_account_address(options.account_address)?;
    validate_aptos_framework_rev(aptos_framework_rev)?;
    if options.force {
        validate_safe_force_output_dir(out_dir)?;
    }

    if out_dir.exists() && !options.force {
        return Err(Error::OutputExists(out_dir.to_path_buf()));
    }

    inputs.validate()?;
    let mut reg = Handlebars::new();
    register_templates(&mut reg)?;

    let vk = &inputs.verifying_key;
    let public_inputs = &inputs.public_inputs;

    let raw_vk_alpha_g1 = adapter.serialize_g1_vk(&vk.vk_alpha_1)?;
    let raw_vk_beta_g2 = adapter.serialize_g2_vk(&vk.vk_beta_2)?;
    let raw_vk_gamma_g2 = adapter.serialize_g2_vk(&vk.vk_gamma_2)?;
    let raw_vk_delta_g2 = adapter.serialize_g2_vk(&vk.vk_delta_2)?;
    let raw_vk_gamma_abc_g1: Vec<Vec<u8>> = vk
        .ic
        .iter()
        .map(|point| adapter.serialize_g1_vk(point))
        .collect::<Result<_>>()?;
    let vk_gamma_abc_g1: Vec<String> = raw_vk_gamma_abc_g1
        .iter()
        .map(|bytes| move_hex_literal(bytes))
        .collect();
    let vk_gamma_abc_g1_rendered = render::vector_of_hex(&vk_gamma_abc_g1);
    let (raw_proof_a, raw_proof_b, raw_proof_c) = match inputs.proof.as_ref() {
        Some(proof) => (
            adapter.serialize_g1_proof(&proof.pi_a)?,
            adapter.serialize_g2_proof(&proof.pi_b)?,
            adapter.serialize_g1_proof(&proof.pi_c)?,
        ),
        None => (Vec::new(), Vec::new(), Vec::new()),
    };
    let raw_public_inputs: Vec<Vec<u8>> = public_inputs
        .iter()
        .map(|value| adapter.serialize_fr_public_input(value))
        .collect::<Result<_>>()?;
    let public_inputs_bytes = render_bytes(&raw_public_inputs);
    let public_inputs_rendered = render::vector_of_hex(&public_inputs_bytes);
    let invalid_public_inputs = invalid_public_inputs(public_inputs);
    let invalid_public_inputs_bytes: Vec<Vec<u8>> = invalid_public_inputs
        .iter()
        .map(|value| adapter.serialize_fr_public_input(value))
        .collect::<Result<_>>()?;
    let invalid_public_inputs_rendered = render_bytes_vector(&invalid_public_inputs_bytes);
    let modulus = adapter.scalar_modulus_le();
    let mut modulus_plus_one = modulus.clone();
    increment_le(&mut modulus_plus_one);
    let noncanonical_public_inputs = replace_first(&raw_public_inputs, modulus);
    let modulus_plus_one_public_inputs = replace_first(&raw_public_inputs, modulus_plus_one);
    let short_public_inputs = replace_first(&raw_public_inputs, vec![0; 31]);
    let long_public_inputs = replace_first(&raw_public_inputs, vec![0; 33]);
    let fingerprint = vk_fingerprint(
        inputs,
        &raw_vk_alpha_g1,
        &raw_vk_beta_g2,
        &raw_vk_gamma_g2,
        &raw_vk_delta_g2,
        &raw_vk_gamma_abc_g1,
    );

    let input = MovegenTemplateInput {
        package_name: options.package_name.to_string(),
        module_name: options.module_name.to_string(),
        account_address: options.account_address.to_string(),
        named_address: options.package_name.to_string(),
        aptos_framework_rev: aptos_framework_rev.to_string(),
        vk_alpha_g1: move_hex_literal(&raw_vk_alpha_g1),
        vk_beta_g2: move_hex_literal(&raw_vk_beta_g2),
        vk_gamma_g2: move_hex_literal(&raw_vk_gamma_g2),
        vk_delta_g2: move_hex_literal(&raw_vk_delta_g2),
        vk_gamma_abc_g1,
        vk_gamma_abc_g1_rendered,
        proof_a: move_hex_literal(&raw_proof_a),
        proof_b: move_hex_literal(&raw_proof_b),
        proof_c: move_hex_literal(&raw_proof_c),
        public_inputs_bytes,
        public_inputs_rendered,
        invalid_public_inputs_rendered,
        noncanonical_public_inputs_rendered: render_bytes_vector(&noncanonical_public_inputs),
        modulus_plus_one_public_inputs_rendered: render_bytes_vector(
            &modulus_plus_one_public_inputs,
        ),
        short_public_inputs_rendered: render_bytes_vector(&short_public_inputs),
        long_public_inputs_rendered: render_bytes_vector(&long_public_inputs),
        has_public_inputs: !raw_public_inputs.is_empty(),
        vk_fingerprint_bytes: move_hex_literal(&fingerprint),
        include_entry: options.mode.include_entry(),
    };

    let move_toml = reg
        .render("move_toml", &input)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    let verifier_template = adapter.move_template_name();
    let verifier_source = reg
        .render(verifier_template, &input)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    let move_tests = inputs
        .has_test_vectors()
        .then(|| {
            reg.render("move_tests", &input)
                .map_err(|e| Error::TemplateRender(e.to_string()))
        })
        .transpose()?;
    let generated_readme = render::readme_content(
        options.package_name,
        options.module_name,
        &input.account_address,
    );
    let manifest = render_manifest(inputs, &input, &fingerprint)?;

    let staging = create_staging_directory(out_dir)?;
    let staged_out = staging.path().join("output");
    create_dir_all(staged_out.join("sources")).map_err(|e| Error::Io {
        source: e,
        context: format!(
            "create staged sources dir {}",
            staged_out.join("sources").display()
        ),
    })?;
    write_generated(staged_out.join("Move.toml"), move_toml, "Move.toml")?;
    write_generated(
        staged_out.join("sources").join("verifier.move"),
        verifier_source,
        "verifier.move",
    )?;
    if let Some(tests) = move_tests {
        create_dir_all(staged_out.join("tests")).map_err(|e| Error::Io {
            source: e,
            context: format!(
                "create staged tests dir {}",
                staged_out.join("tests").display()
            ),
        })?;
        write_generated(
            staged_out.join("tests").join("verifier_tests.move"),
            tests,
            "verifier_tests.move",
        )?;
    }
    write_generated(staged_out.join("README.md"), generated_readme, "README.md")?;
    write_generated(
        staged_out.join("verifier-manifest.json"),
        format!("{manifest}\n"),
        "verifier-manifest.json",
    )?;
    publish_staged_directory(staging, &staged_out, out_dir)?;

    Ok(())
}

fn create_staging_directory(out_dir: &Path) -> Result<TempDir> {
    let parent = out_dir
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    create_dir_all(parent).map_err(|e| Error::Io {
        source: e,
        context: format!("create output parent {}", parent.display()),
    })?;
    TempDirBuilder::new()
        .prefix(".export-aptos-verifier-")
        .tempdir_in(parent)
        .map_err(|e| Error::Io {
            source: e,
            context: format!("create staging directory in {}", parent.display()),
        })
}

fn publish_staged_directory(staging: TempDir, staged_out: &Path, out_dir: &Path) -> Result<()> {
    let backup = staging.path().join("previous-output");
    let had_existing = out_dir.exists();
    if had_existing {
        fs::rename(out_dir, &backup).map_err(|e| Error::Io {
            source: e,
            context: format!("stage existing output {}", out_dir.display()),
        })?;
    }

    if let Err(publish_error) = fs::rename(staged_out, out_dir) {
        if had_existing {
            return restore_previous_output_or_preserve(staging, &backup, out_dir, publish_error);
        }
        return Err(Error::Io {
            source: publish_error,
            context: format!("publish generated output {}", out_dir.display()),
        });
    }
    Ok(())
}

fn restore_previous_output_or_preserve(
    staging: TempDir,
    backup: &Path,
    out_dir: &Path,
    publish_error: std::io::Error,
) -> Result<()> {
    match fs::rename(backup, out_dir) {
        Ok(()) => Err(Error::Io {
            source: publish_error,
            context: format!("publish generated output {}", out_dir.display()),
        }),
        Err(rollback_error) => {
            let preserved = staging.keep().join("previous-output");
            Err(Error::Io {
                source: rollback_error,
                context: format!(
                    "restore {} after publish failed ({publish_error}); previous output preserved at {}",
                    out_dir.display(),
                    preserved.display()
                ),
            })
        }
    }
}

fn ensure_adapter_matches_inputs(
    adapter: &dyn CurveAdapter,
    inputs: &Groth16VerifierInputs,
) -> Result<()> {
    let matches = matches!(
        (adapter.id(), inputs.curve),
        (CurveId::Bn254, CurveKind::Bn254) | (CurveId::Bls12381, CurveKind::Bls12_381)
    );
    if !matches {
        return Err(Error::CurveMismatch(format!(
            "adapter {:?} does not match input curve {}",
            adapter.id(),
            inputs.curve.canonical_name()
        )));
    }
    Ok(())
}

fn write_generated(path: impl AsRef<Path>, contents: String, label: &str) -> Result<()> {
    write(path, contents).map_err(|e| Error::Io {
        source: e,
        context: format!("write {label}"),
    })
}

fn render_bytes(values: &[Vec<u8>]) -> Vec<String> {
    values.iter().map(|bytes| move_hex_literal(bytes)).collect()
}

fn render_bytes_vector(values: &[Vec<u8>]) -> String {
    render::vector_of_hex(&render_bytes(values))
}

fn replace_first(values: &[Vec<u8>], replacement: Vec<u8>) -> Vec<Vec<u8>> {
    let mut replaced = values.to_vec();
    if let Some(first) = replaced.first_mut() {
        *first = replacement;
    }
    replaced
}

fn increment_le(bytes: &mut [u8]) {
    for byte in bytes {
        let (next, carry) = byte.overflowing_add(1);
        *byte = next;
        if !carry {
            break;
        }
    }
}

fn vk_fingerprint(
    inputs: &Groth16VerifierInputs,
    alpha: &[u8],
    beta: &[u8],
    gamma: &[u8],
    delta: &[u8],
    ic: &[Vec<u8>],
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"export-aptos-verifier:groth16-vk:v1\0");
    hash_component(&mut hasher, inputs.curve.canonical_name().as_bytes());
    hash_component(
        &mut hasher,
        &(inputs.verifying_key.n_public as u64).to_be_bytes(),
    );
    for component in [alpha, beta, gamma, delta] {
        hash_component(&mut hasher, component);
    }
    for point in ic {
        hash_component(&mut hasher, point);
    }
    hasher.finalize().into()
}

fn hash_component(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
}

fn source_format_name(source_format: SourceFormat) -> &'static str {
    match source_format {
        SourceFormat::SnarkjsJson => "snarkjs-json",
        SourceFormat::Arkworks => "arkworks",
        SourceFormat::ArkworksCompact => "arkworks-compact",
        SourceFormat::GnarkJson => "gnark-json",
        SourceFormat::GnarkBinary => "gnark-binary",
        SourceFormat::Sp1Groth16 => "sp1-groth16",
    }
}

fn render_manifest(
    inputs: &Groth16VerifierInputs,
    template: &MovegenTemplateInput,
    fingerprint: &[u8; 32],
) -> Result<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": "groth16-verifier-manifest-v1",
        "generator": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
        },
        "protocol": "groth16",
        "curve": inputs.curve.canonical_name(),
        "public_inputs": inputs.verifying_key.n_public,
        "vk_sha256": hex::encode(fingerprint),
        "circuit_sha256": serde_json::Value::Null,
        "source_format": source_format_name(inputs.source_format),
        "serialization_format": serialization_format(inputs.curve),
        "upgrade_policy": "immutable",
        "package": template.package_name,
        "module": template.module_name,
        "account_address": template.account_address,
        "dependencies": {
            "framework_revision": template.aptos_framework_rev,
            "arkworks": "0.6",
        },
    }))
    .map_err(|e| Error::TemplateRender(format!("failed to render verifier manifest: {e}")))
}

fn serialization_format(curve: CurveKind) -> &'static str {
    match curve {
        CurveKind::Bn254 => "aptos-crypto-algebra-canonical-uncompressed-little-endian-v1",
        CurveKind::Bls12_381 => "aptos-crypto-algebra-canonical-compressed-little-endian-v1",
    }
}

fn invalid_public_inputs(public_inputs: &[String]) -> Vec<String> {
    let mut invalid = public_inputs.to_vec();
    if let Some(last) = invalid.last_mut() {
        *last = if last == "0" {
            "1".to_string()
        } else {
            "0".to_string()
        };
    }
    invalid
}

fn validate_aptos_framework_rev(value: &str) -> Result<()> {
    if value.len() != 40 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(Error::InvalidAptosFrameworkRevision(
            "revision must be exactly 40 hexadecimal commit-SHA characters".to_string(),
        ));
    }
    Ok(())
}

fn validate_move_names(package_name: &str, module_name: &str) -> Result<()> {
    if !is_move_identifier(package_name) {
        return Err(Error::InvalidPackageName(
            "package_name must match [A-Za-z_][A-Za-z0-9_]*".to_string(),
        ));
    }
    if !is_move_identifier(module_name) {
        return Err(Error::InvalidModuleName(
            "module_name must match [A-Za-z_][A-Za-z0-9_]*".to_string(),
        ));
    }
    Ok(())
}

fn is_move_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn validate_account_address(value: &str) -> Result<()> {
    let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    else {
        return Err(Error::InvalidAccountAddress(
            "account_address must start with 0x".to_string(),
        ));
    };
    if hex.is_empty() || hex.len() > 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::InvalidAccountAddress(
            "account_address must match 0x[0-9a-fA-F]{1,64}".to_string(),
        ));
    }
    Ok(())
}

fn validate_safe_force_output_dir(out_dir: &Path) -> Result<()> {
    if out_dir
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(Error::UnsafeOutputDirectory(out_dir.to_path_buf()));
    }

    if !out_dir.exists() {
        return Ok(());
    }

    let target = out_dir.canonicalize().map_err(|e| Error::Io {
        source: e,
        context: format!("canonicalize output dir {}", out_dir.display()),
    })?;
    if target.parent().is_none() {
        return Err(Error::UnsafeOutputDirectory(target));
    }

    let cwd = env::current_dir().map_err(|e| Error::Io {
        source: e,
        context: "get current working directory".to_string(),
    })?;
    let cwd = cwd.canonicalize().map_err(|e| Error::Io {
        source: e,
        context: format!("canonicalize current working directory {}", cwd.display()),
    })?;
    if target == cwd || cwd.starts_with(&target) {
        return Err(Error::UnsafeOutputDirectory(target));
    }

    Ok(())
}

fn register_templates(handlebars: &mut Handlebars) -> Result<()> {
    let move_toml = include_str!("../../templates/Move.toml.hbs");
    let verifier_bn254 = include_str!("../../templates/verifier_bn254.move.hbs");
    let verifier_bls = include_str!("../../templates/verifier_bls12381.move.hbs");
    let tests = include_str!("../../templates/tests.move.hbs");

    handlebars
        .register_template_string("move_toml", move_toml)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    handlebars
        .register_template_string("verifier_bn254.move.hbs", verifier_bn254)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    handlebars
        .register_template_string("verifier_bls12381.move.hbs", verifier_bls)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    handlebars
        .register_template_string("move_tests", tests)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{create_staging_directory, publish_staged_directory};
    use std::fs;

    #[test]
    fn failed_publish_restores_existing_output() {
        let parent = tempfile::tempdir().unwrap();
        let out = parent.path().join("generated");
        fs::create_dir(&out).unwrap();
        fs::write(out.join("keep.txt"), "existing output").unwrap();
        let staging = create_staging_directory(&out).unwrap();
        let missing_staged_output = staging.path().join("missing");

        assert!(publish_staged_directory(staging, &missing_staged_output, &out).is_err());
        assert_eq!(
            fs::read_to_string(out.join("keep.txt")).unwrap(),
            "existing output"
        );
    }

    #[test]
    fn failed_rollback_preserves_backup_on_disk() {
        let parent = tempfile::tempdir().unwrap();
        let out = parent.path().join("generated");
        fs::create_dir(&out).unwrap();
        fs::write(out.join("occupied.txt"), "concurrent output").unwrap();
        let staging = create_staging_directory(&out).unwrap();
        let staging_path = staging.path().to_path_buf();
        let backup = staging.path().join("previous-output");
        fs::create_dir(&backup).unwrap();
        fs::write(backup.join("keep.txt"), "existing output").unwrap();

        let err = super::restore_previous_output_or_preserve(
            staging,
            &backup,
            &out,
            std::io::Error::other("publish failed"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("previous output preserved"));
        assert_eq!(
            fs::read_to_string(staging_path.join("previous-output/keep.txt")).unwrap(),
            "existing output"
        );
        fs::remove_dir_all(staging_path).unwrap();
    }
}

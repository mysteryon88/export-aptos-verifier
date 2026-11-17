mod context;
mod render;

use crate::bytes::move_hex_literal;
use crate::curves::{CurveAdapter, CurveId};
use crate::error::{Error, Result};
use crate::model::{CurveKind, Groth16VerifierInputs, SourceFormat};
pub use context::{MovegenMode, MovegenTemplateInput};
use handlebars::Handlebars;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};

use crate::output::{Dir, Output};
use std::path::Path;

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
const MANIFEST_FILE_NAME: &str = "verifier-manifest.json";
const MANIFEST_SCHEMA: &str = "groth16-verifier-manifest-v1";
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;

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
    crate::output::validate_path(out_dir).map_err(|source| {
        if source.kind() == std::io::ErrorKind::Unsupported {
            output_error(source)
        } else {
            Error::UnsafeOutputDirectory(out_dir.to_owned())
        }
    })?;
    Output::check_existing(out_dir, options.force, |dir| {
        validate_generated_output(dir, options)
    })
    .map_err(|source| {
        if options.force && source.kind() != std::io::ErrorKind::Unsupported {
            Error::UnsafeOutputDirectory(out_dir.to_owned())
        } else if source.kind() == std::io::ErrorKind::AlreadyExists {
            Error::OutputExists(out_dir.to_owned())
        } else {
            output_error(source)
        }
    })?;
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
    let mut files = std::collections::BTreeMap::new();
    for (name, content) in [
        ("Move.toml", &move_toml),
        ("sources/verifier.move", &verifier_source),
        ("README.md", &generated_readme),
    ] {
        files.insert(name, hex::encode(Sha256::digest(content.as_bytes())));
    }
    if let Some(tests) = &move_tests {
        files.insert(
            "tests/verifier_tests.move",
            hex::encode(Sha256::digest(tests.as_bytes())),
        );
    }
    let manifest = render_manifest(inputs, &input, &fingerprint, &files)?;

    let output = Output::new(out_dir).map_err(output_error)?;
    output
        .write("Move.toml", move_toml.as_bytes())
        .map_err(output_error)?;
    output
        .write("sources/verifier.move", verifier_source.as_bytes())
        .map_err(output_error)?;
    if let Some(tests) = &move_tests {
        output
            .write("tests/verifier_tests.move", tests.as_bytes())
            .map_err(output_error)?;
    }
    output
        .write("README.md", generated_readme.as_bytes())
        .map_err(output_error)?;
    output
        .write("verifier-manifest.json", format!("{manifest}\n").as_bytes())
        .map_err(output_error)?;
    output
        .publish(options.force, |dir| validate_generated_output(dir, options))
        .map_err(|source| {
            if !options.force && source.kind() == std::io::ErrorKind::AlreadyExists {
                Error::OutputExists(out_dir.to_owned())
            } else {
                output_error(source)
            }
        })?;

    Ok(())
}

fn output_error(source: std::io::Error) -> Error {
    Error::Io {
        source,
        context: "publish output safely".into(),
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
    files: &std::collections::BTreeMap<&str, String>,
) -> Result<String> {
    serde_json::to_string_pretty(&serde_json::json!({
        "schema": MANIFEST_SCHEMA,
        "files": files,
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

// Public metadata protects against accidental deletion; it is not ownership authentication.
fn validate_generated_output(
    dir: &Dir,
    options: &GenerateMovePackageOptions<'_>,
) -> std::io::Result<()> {
    use std::{ffi::OsStr, io};
    let invalid = || {
        io::Error::other("unsafe output: expected complete generated manifest, requested identity and unmodified file inventory; regenerate legacy output into a new directory and migrate explicitly")
    };
    let entries = dir.entries()?;
    if entries.is_empty() {
        return Ok(());
    }
    let bytes = read_bounded(
        dir.open_file(OsStr::new(MANIFEST_FILE_NAME))?,
        MAX_MANIFEST_BYTES,
    )?;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    let object = manifest.as_object().ok_or_else(invalid)?;
    let required = [
        "schema",
        "generator",
        "protocol",
        "curve",
        "public_inputs",
        "vk_sha256",
        "circuit_sha256",
        "source_format",
        "serialization_format",
        "upgrade_policy",
        "package",
        "module",
        "account_address",
        "dependencies",
        "files",
    ];
    if object.len() != required.len() || required.iter().any(|key| !object.contains_key(*key)) {
        return Err(invalid());
    }
    let curve = match manifest["curve"].as_str() {
        Some("bn254") => CurveKind::Bn254,
        Some("bls12381") => CurveKind::Bls12_381,
        _ => return Err(invalid()),
    };
    let hash_is_valid = |v: &serde_json::Value| {
        v.as_str()
            .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
    };
    if manifest["schema"] != MANIFEST_SCHEMA
        || manifest["generator"]
            != serde_json::json!({"name": env!("CARGO_PKG_NAME"), "version": env!("CARGO_PKG_VERSION")})
        || manifest["protocol"] != "groth16"
        || manifest["package"] != options.package_name
        || manifest["module"] != options.module_name
        || manifest["account_address"] != options.account_address
        || manifest["upgrade_policy"] != "immutable"
        || !manifest["circuit_sha256"].is_null()
        || !hash_is_valid(&manifest["vk_sha256"])
        || !manifest["public_inputs"]
            .as_u64()
            .is_some_and(|n| n <= crate::model::MAX_PUBLIC_INPUTS as u64)
        || manifest["serialization_format"] != serialization_format(curve)
        || !matches!(
            manifest["source_format"].as_str(),
            Some(
                "snarkjs-json"
                    | "arkworks"
                    | "arkworks-compact"
                    | "gnark-json"
                    | "gnark-binary"
                    | "sp1-groth16"
            )
        )
        || manifest["dependencies"]
            .as_object()
            .is_none_or(|m| m.len() != 2)
        || manifest["dependencies"]["arkworks"] != "0.6"
        || manifest["dependencies"]["framework_revision"]
            .as_str()
            .is_none_or(|rev| validate_aptos_framework_rev(rev).is_err())
    {
        return Err(invalid());
    }
    let files = manifest["files"].as_object().ok_or_else(invalid)?;
    let has_tests = files.contains_key("tests/verifier_tests.move");
    let mut expected_root = vec!["Move.toml", "README.md", MANIFEST_FILE_NAME, "sources"];
    if has_tests {
        expected_root.push("tests");
    }
    expected_root.sort();
    let mut actual_root: Vec<_> = entries
        .iter()
        .map(|s| s.to_str().ok_or_else(invalid))
        .collect::<io::Result<_>>()?;
    actual_root.sort();
    if actual_root != expected_root || files.len() != 3 + usize::from(has_tests) {
        return Err(invalid());
    }
    let mut paths = vec!["Move.toml", "README.md", "sources/verifier.move"];
    if has_tests {
        paths.push("tests/verifier_tests.move");
    }
    for path in paths {
        let (parent, filename) = match path.split_once('/') {
            Some((directory, filename)) => {
                let child = dir.open_dir(OsStr::new(directory))?;
                if child.entries()? != [OsStr::new(filename)] {
                    return Err(invalid());
                }
                (child, filename)
            }
            None => (dir.try_clone()?, path),
        };
        let content = read_bounded(parent.open_file(OsStr::new(filename))?, 32 * 1024 * 1024)?;
        if files.get(path).is_none_or(|expected| {
            !hash_is_valid(expected)
                || expected.as_str() != Some(hex::encode(Sha256::digest(&content)).as_str())
        }) {
            return Err(invalid());
        }
    }
    dir.validate_tree()?;
    Ok(())
}

fn read_bounded(mut file: File, max: u64) -> io::Result<Vec<u8>> {
    if !file.metadata()?.is_file() || file.metadata()?.len() > max {
        return Err(io::Error::other("unsafe generated file size/type"));
    }
    let mut bytes = Vec::new();
    (&mut file).take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(io::Error::other("generated file grew beyond limit"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn force_guard_rejects_markers_modifications_and_unexpected_files() {
        let inputs = crate::formats::load_compact_bundle(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ark-mimc/artifacts/bn254/groth16_artifacts.json"),
            None,
        )
        .unwrap();
        let adapter = crate::curves::create_adapter("bn254").unwrap();
        let options = GenerateMovePackageOptions {
            package_name: "guard",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        };
        let parent = tempfile::tempdir().unwrap();
        let marker = parent.path().join("marker");
        fs::create_dir(&marker).unwrap();
        fs::write(marker.join(MANIFEST_FILE_NAME), serde_json::to_vec(&serde_json::json!({
            "schema": MANIFEST_SCHEMA,
            "generator": { "name": env!("CARGO_PKG_NAME"), "version": env!("CARGO_PKG_VERSION") }
        })).unwrap()).unwrap();
        fs::write(marker.join("keep"), "user data").unwrap();
        assert!(generate_move_package(&marker, adapter.as_ref(), &inputs, &options).is_err());
        assert_eq!(fs::read(marker.join("keep")).unwrap(), b"user data");
        for mutation in ["extra", "changed", "identity", "inventory", "legacy"] {
            let out = parent.path().join(mutation);
            generate_move_package(&out, adapter.as_ref(), &inputs, &options).unwrap();
            match mutation {
                "extra" => fs::write(out.join("user.txt"), "keep").unwrap(),
                "changed" => {
                    fs::write(out.join("sources/verifier.move"), "user modification").unwrap()
                }
                other => {
                    let path = out.join(MANIFEST_FILE_NAME);
                    let mut manifest: serde_json::Value =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    if other == "identity" {
                        manifest["package"] = serde_json::json!("other");
                    } else if other == "inventory" {
                        manifest["files"]["unexpected.txt"] = serde_json::json!("00".repeat(32));
                    } else {
                        manifest.as_object_mut().unwrap().remove("files");
                    }
                    fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
                }
            }
            let before = fs::read(out.join(MANIFEST_FILE_NAME)).unwrap();
            assert!(
                generate_move_package(&out, adapter.as_ref(), &inputs, &options).is_err(),
                "{mutation}"
            );
            assert_eq!(fs::read(out.join(MANIFEST_FILE_NAME)).unwrap(), before);
            if mutation == "extra" {
                assert_eq!(fs::read(out.join("user.txt")).unwrap(), b"keep");
            }
            if mutation == "changed" {
                assert_eq!(
                    fs::read(out.join("sources/verifier.move")).unwrap(),
                    b"user modification"
                );
            }
        }
    }

    #[test]
    fn force_guard_rejects_nested_symlinks_without_external_mutation() {
        let inputs = crate::formats::load_compact_bundle(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples/ark-mimc/artifacts/bn254/groth16_artifacts.json"),
            None,
        )
        .unwrap();
        let adapter = crate::curves::create_adapter("bn254").unwrap();
        let options = GenerateMovePackageOptions {
            package_name: "guard",
            module_name: "verifier",
            account_address: "0xCAFE",
            mode: MovegenMode::Library,
            force: true,
        };
        let p = tempfile::tempdir().unwrap();
        let out = p.path().join("out");
        generate_move_package(&out, adapter.as_ref(), &inputs, &options).unwrap();
        let external = p.path().join("external");
        fs::rename(out.join("sources"), &external).unwrap();
        let before = fs::read(external.join("verifier.move")).unwrap();
        crate::output::tests::link(&external, &out.join("sources"));
        assert!(generate_move_package(&out, adapter.as_ref(), &inputs, &options).is_err());
        assert_eq!(fs::read(external.join("verifier.move")).unwrap(), before);
        assert!(fs::symlink_metadata(out.join("sources"))
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn generated_force_guard_checks_requested_identity_on_both_curves() {
        for curve in ["bn254", "bls12_381"] {
            let bundle = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                "../../examples/ark-mimc/artifacts/{curve}/groth16_artifacts.json"
            ));
            let inputs = crate::formats::load_compact_bundle(&bundle, None).unwrap();
            let adapter = crate::curves::create_adapter(inputs.curve.canonical_name()).unwrap();
            let mut options = GenerateMovePackageOptions {
                package_name: "guard",
                module_name: "verifier",
                account_address: "0xCAFE",
                mode: MovegenMode::Library,
                force: true,
            };
            let parent = tempfile::tempdir().unwrap();
            let out = parent.path().join("generated");
            generate_move_package(&out, adapter.as_ref(), &inputs, &options).unwrap();
            generate_move_package(&out, adapter.as_ref(), &inputs, &options).unwrap();
            options.module_name = "different";
            assert!(generate_move_package(&out, adapter.as_ref(), &inputs, &options).is_err());
        }
    }
}

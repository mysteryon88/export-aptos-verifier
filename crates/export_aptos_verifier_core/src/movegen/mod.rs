mod context;
mod render;

use crate::bytes::move_hex_literal;
use crate::curves::CurveAdapter;
use crate::error::{Error, Result};
use crate::model::Groth16VerifierInputs;
pub use context::{MovegenMode, MovegenTemplateInput};
use handlebars::Handlebars;
use std::fs::{self, create_dir_all, write};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct GenerateMovePackageOptions<'a> {
    pub package_name: &'a str,
    pub module_name: &'a str,
    pub account_address: &'a str,
    pub mode: MovegenMode,
    pub force: bool,
}

pub fn generate_move_package(
    out_dir: &Path,
    adapter: &dyn CurveAdapter,
    inputs: &Groth16VerifierInputs,
    options: &GenerateMovePackageOptions<'_>,
) -> Result<()> {
    if out_dir.exists() && !options.force {
        return Err(Error::OutputExists(out_dir.to_path_buf()));
    }

    if out_dir.exists() {
        fs::remove_dir_all(out_dir).map_err(|e| Error::Io {
            source: e,
            context: format!("failed to clear existing output dir {}", out_dir.display()),
        })?;
    }

    create_dir_all(out_dir).map_err(|e| Error::Io {
        source: e,
        context: format!("create output dir {}", out_dir.display()),
    })?;

    create_dir_all(out_dir.join("sources")).map_err(|e| Error::Io {
        source: e,
        context: format!("create sources dir {}", out_dir.join("sources").display()),
    })?;
    create_dir_all(out_dir.join("tests")).map_err(|e| Error::Io {
        source: e,
        context: format!("create tests dir {}", out_dir.join("tests").display()),
    })?;

    let mut reg = Handlebars::new();
    register_templates(&mut reg)?;

    let vk = &inputs.verifying_key;
    let proof = &inputs.proof;
    let public_inputs = &inputs.public_inputs;

    let vk_alpha_g1 = move_hex_literal(&adapter.serialize_g1_vk(&vk.vk_alpha_1)?);
    let vk_beta_g2 = move_hex_literal(&adapter.serialize_g2_vk(&vk.vk_beta_2)?);
    let vk_gamma_g2 = move_hex_literal(&adapter.serialize_g2_vk(&vk.vk_gamma_2)?);
    let vk_delta_g2 = move_hex_literal(&adapter.serialize_g2_vk(&vk.vk_delta_2)?);
    let vk_gamma_abc_g1: Vec<String> = vk
        .ic
        .iter()
        .map(|point| {
            let bytes = adapter.serialize_g1_vk(point)?;
            Ok(move_hex_literal(&bytes))
        })
        .collect::<Result<_>>()?;
    let vk_gamma_abc_g1_rendered = render::vector_of_hex(&vk_gamma_abc_g1);
    let proof_a = move_hex_literal(&adapter.serialize_g1_proof(&proof.pi_a)?);
    let proof_b = move_hex_literal(&adapter.serialize_g2_proof(&proof.pi_b)?);
    let proof_c = move_hex_literal(&adapter.serialize_g1_proof(&proof.pi_c)?);
    let public_inputs_bytes: Vec<String> = public_inputs
        .iter()
        .map(|value| {
            adapter
                .serialize_fr_public_input(value)
                .map(|bytes| move_hex_literal(&bytes))
        })
        .collect::<Result<_>>()?;
    let public_inputs_rendered = render::vector_of_hex(&public_inputs_bytes);

    let input = MovegenTemplateInput {
        package_name: options.package_name.to_string(),
        module_name: options.module_name.to_string(),
        account_address: options.account_address.to_string(),
        named_address: options.package_name.to_string(),
        vk_alpha_g1,
        vk_beta_g2,
        vk_gamma_g2,
        vk_delta_g2,
        vk_gamma_abc_g1,
        vk_gamma_abc_g1_rendered,
        proof_a,
        proof_b,
        proof_c,
        public_inputs_bytes,
        public_inputs_rendered,
        include_entry: options.mode.include_entry(),
    };

    let move_toml = reg
        .render("move_toml", &input)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    fs::write(out_dir.join("Move.toml"), move_toml).map_err(|e| Error::Io {
        source: e,
        context: "write Move.toml".to_string(),
    })?;

    let verifier_template = adapter.move_template_name();
    let verifier_source = reg
        .render(verifier_template, &input)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    fs::write(
        out_dir.join("sources").join("verifier.move"),
        verifier_source,
    )
    .map_err(|e| Error::Io {
        source: e,
        context: "write verifier.move".to_string(),
    })?;

    let tests = reg
        .render("move_tests", &input)
        .map_err(|e| Error::TemplateRender(e.to_string()))?;
    fs::write(out_dir.join("tests").join("verifier_tests.move"), tests).map_err(|e| Error::Io {
        source: e,
        context: "write verifier_tests.move".to_string(),
    })?;

    write(
        out_dir.join("README.md"),
        render::readme_content(options.package_name, &input.account_address),
    )
    .map_err(|e| Error::Io {
        source: e,
        context: "write README.md".to_string(),
    })?;

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

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MovegenTemplateInput {
    pub package_name: String,
    pub module_name: String,
    pub account_address: String,
    pub named_address: String,
    pub vk_alpha_g1: String,
    pub vk_beta_g2: String,
    pub vk_gamma_g2: String,
    pub vk_delta_g2: String,
    pub vk_gamma_abc_g1: Vec<String>,
    pub vk_gamma_abc_g1_rendered: String,
    pub proof_a: String,
    pub proof_b: String,
    pub proof_c: String,
    pub public_inputs_bytes: Vec<String>,
    pub public_inputs_rendered: String,
    pub invalid_public_inputs_rendered: String,
    pub include_entry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovegenMode {
    Library,
    Entry,
    Test,
}

impl MovegenMode {
    pub fn include_entry(self) -> bool {
        matches!(self, Self::Entry | Self::Test)
    }
}

use clap::{Parser, Subcommand, ValueEnum};
use regex::Regex;
use std::fmt;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use export_aptos_verifier_core::curves::{create_adapter, PointFormat};
use export_aptos_verifier_core::error::{Error, Result};
use export_aptos_verifier_core::formats::{
    load_compact_bundle, load_snarkjs_json_inputs_with_curve_hint,
};
use export_aptos_verifier_core::local_verify;
use export_aptos_verifier_core::movegen::{
    generate_move_package, GenerateMovePackageOptions, MovegenMode,
};
use export_aptos_verifier_core::CurveKind;

#[derive(Parser)]
#[command(
    name = "export-aptos-verifier",
    version,
    about = "Export Groth16 artifacts to an Aptos Move verifier package"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    Generate(GenerateArgs),
}

#[derive(clap::Args)]
struct GenerateArgs {
    #[arg(long)]
    vk: Option<PathBuf>,
    #[arg(long)]
    proof: Option<PathBuf>,
    #[arg(long)]
    public: Option<PathBuf>,
    #[arg(long)]
    out: PathBuf,
    #[arg(long)]
    package_name: String,
    #[arg(long)]
    module_name: String,
    #[arg(long)]
    account_address: String,

    #[arg(long, default_value_t = CurveArg::Auto)]
    curve: CurveArg,
    #[arg(long, default_value_t = InputFormatArg::Auto)]
    input_format: InputFormatArg,
    #[arg(long, default_value_t = PointFormatArg::Uncompressed)]
    bn254_format: PointFormatArg,
    #[arg(long, default_value_t = PointFormatArg::Compressed)]
    bls_format: PointFormatArg,
    #[arg(long, default_value_t = ModeArg::Entry)]
    mode: ModeArg,
    #[arg(long, default_value_t = false)]
    run_aptos_test: bool,
    #[arg(long, default_value_t = false)]
    force: bool,
    #[arg(long, default_value_t = false)]
    skip_local_verify: bool,
    #[arg(long, default_value_t = false)]
    prepared: bool,
    #[arg(long)]
    bundle: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum CurveArg {
    Auto,
    Bn254,
    Bls12381,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum InputFormatArg {
    Auto,
    SnarkjsJson,
    ArkworksCompact,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum PointFormatArg {
    Compressed,
    Uncompressed,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ModeArg {
    Library,
    Entry,
    Test,
}

impl ModeArg {
    fn into_move_mode(self) -> MovegenMode {
        match self {
            Self::Library => MovegenMode::Library,
            Self::Entry => MovegenMode::Entry,
            Self::Test => MovegenMode::Test,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        CliCommand::Generate(args) => run_generate(args),
    };
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run_generate(args: GenerateArgs) -> Result<()> {
    validate_names(&args.package_name, "package_name")?;
    validate_names(&args.module_name, "module_name")?;
    validate_account_address(&args.account_address)?;

    let curve_hint = if matches!(args.curve, CurveArg::Auto) {
        None
    } else {
        Some(args.curve.to_string())
    };

    let inputs = match (args.bundle.as_ref(), args.input_format) {
        (Some(bundle), InputFormatArg::Auto | InputFormatArg::ArkworksCompact) => {
            load_compact_bundle(bundle, curve_hint.as_deref())?
        }
        (None, InputFormatArg::Auto | InputFormatArg::SnarkjsJson) => {
            let vk = args.vk.as_ref().ok_or_else(|| {
                Error::MissingInput("--vk is required unless --bundle is used".to_string())
            })?;
            let proof = args.proof.as_ref().ok_or_else(|| {
                Error::MissingInput("--proof is required unless --bundle is used".to_string())
            })?;
            load_snarkjs_json_inputs_with_curve_hint(
                vk,
                proof,
                args.public.as_deref(),
                curve_hint.as_deref(),
            )?
        }
        (Some(_), InputFormatArg::SnarkjsJson) => {
            return Err(Error::MissingInput(
                "snarkjs-json mode requires --vk and --proof inputs".to_string(),
            ));
        }
        (None, InputFormatArg::ArkworksCompact) => {
            return Err(Error::MissingInput(
                "arkworks-compact mode requires --bundle".to_string(),
            ));
        }
    };

    let requested_curve = match args.curve {
        CurveArg::Auto => inputs.curve.canonical_name().to_string(),
        CurveArg::Bn254 => {
            if inputs.curve != CurveKind::Bn254 {
                return Err(Error::CurveMismatch(
                    "requested curve bn254 does not match input curve metadata".to_string(),
                ));
            }
            "bn254".to_string()
        }
        CurveArg::Bls12381 => {
            if inputs.curve != CurveKind::Bls12_381 {
                return Err(Error::CurveMismatch(
                    "requested curve bls12381 does not match input curve metadata".to_string(),
                ));
            }
            "bls12381".to_string()
        }
    };

    if args.prepared {
        return Err(Error::PreparedNotImplemented);
    }

    let adapter = create_adapter(&requested_curve)?;
    validate_point_format(adapter.as_ref(), &requested_curve, &args)?;

    if !args.skip_local_verify {
        let ok = local_verify(adapter.as_ref(), &inputs)?;
        if !ok {
            return Err(Error::LocalProofVerificationFailed(
                "local arkworks verification returned false".to_string(),
            ));
        }
    }

    generate_move_package(
        &args.out,
        adapter.as_ref(),
        &inputs,
        &GenerateMovePackageOptions {
            package_name: &args.package_name,
            module_name: &args.module_name,
            account_address: &args.account_address,
            mode: args.mode.into_move_mode(),
            force: args.force,
        },
    )?;

    if args.run_aptos_test {
        run_aptos_test(&args.out)?;
    }

    Ok(())
}

fn validate_point_format(
    adapter: &dyn export_aptos_verifier_core::curves::CurveAdapter,
    requested_curve: &str,
    args: &GenerateArgs,
) -> Result<()> {
    let normalized = canonicalize_curve(requested_curve);
    if normalized == "bn254" {
        let expected = adapter.default_point_format();
        if expected != map_point_format(&args.bn254_format) {
            return Err(Error::UnsupportedCurve(format!(
                "unsupported BN254 format, expected {:?}",
                expected
            )));
        }
    }
    if normalized == "bls12381" {
        let expected = adapter.default_point_format();
        if expected != map_point_format(&args.bls_format) {
            return Err(Error::UnsupportedCurve(format!(
                "unsupported BLS12-381 format, expected {:?}",
                expected
            )));
        }
    }
    Ok(())
}

impl fmt::Display for CurveArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Auto => "auto",
            Self::Bn254 => "bn254",
            Self::Bls12381 => "bls12381",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for InputFormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Auto => "auto",
            Self::SnarkjsJson => "snarkjs-json",
            Self::ArkworksCompact => "arkworks-compact",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for PointFormatArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Compressed => "compressed",
            Self::Uncompressed => "uncompressed",
        };
        write!(f, "{s}")
    }
}

impl fmt::Display for ModeArg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Library => "library",
            Self::Entry => "entry",
            Self::Test => "test",
        };
        write!(f, "{s}")
    }
}

fn canonicalize_curve(name: &str) -> String {
    name.to_lowercase().replace(['-', '_'], "")
}

fn map_point_format(value: &PointFormatArg) -> PointFormat {
    match value {
        PointFormatArg::Compressed => PointFormat::Compressed,
        PointFormatArg::Uncompressed => PointFormat::Uncompressed,
    }
}

fn validate_names(value: &str, field: &str) -> Result<()> {
    let re = Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
    if !re.is_match(value) {
        if field == "module_name" {
            return Err(Error::InvalidModuleName(format!(
                "{field} must match [A-Za-z_][A-Za-z0-9_]*"
            )));
        }
        return Err(Error::InvalidPackageName(format!(
            "{field} must match [A-Za-z_][A-Za-z0-9_]*"
        )));
    }
    Ok(())
}

fn validate_account_address(value: &str) -> Result<()> {
    let re = Regex::new(r"^0[xX][0-9a-fA-F]{1,64}$").unwrap();
    if !re.is_match(value) {
        return Err(Error::InvalidAccountAddress(
            "account_address must match 0x[0-9a-fA-F]{1,64}".to_string(),
        ));
    }
    Ok(())
}

fn run_aptos_test(out_dir: &std::path::Path) -> Result<()> {
    let aptos = ProcessCommand::new("aptos")
        .arg("move")
        .arg("test")
        .arg("--package-dir")
        .arg(out_dir)
        .output();

    match aptos {
        Ok(out) => {
            if !out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(Error::AptosTestFailed(format!(
                    "ERR_APTOS_TEST_FAILED: {}\nstdout:\n{}\nstderr:\n{}",
                    out.status, stdout, stderr
                )));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::AptosTestFailed(
                "ERR_APTOS_CLI_NOT_FOUND: install Aptos CLI or run without --run-aptos-test"
                    .to_string(),
            ));
        }
        Err(err) => {
            return Err(Error::AptosTestFailed(err.to_string()));
        }
    }

    Ok(())
}

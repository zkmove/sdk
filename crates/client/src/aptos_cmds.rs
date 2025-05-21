use crate::constants::*;
use anyhow::Result;
use clap::{value_parser, Parser, Subcommand, ValueEnum};
use halo2_proofs::{halo2curves::bn256::Bn256, poly::kzg::commitment::ParamsKZG};
use move_core_types::{identifier::Identifier, language_storage::ModuleId};
use move_package::{
    compilation::{
        compiled_package::{CompiledPackage, OnDiskCompiledPackage},
        package_layout::CompiledPackageLayout,
    },
    source_package::layout::SourcePackageLayout,
};
use std::{fs, path::PathBuf, str::FromStr};

#[derive(Parser)]
#[command(about = " generate aptos txns for verify proof on aptos")]
pub struct AptosCommands {
    #[arg(long = "zkmove-address")]
    zkmove_address: String,
    #[arg(long, default_value = VK_REGISTRY_MODULE)]
    vk_registry_module: String,
    #[arg(long, default_value = VK_REGISTRY_FUNC)]
    vk_registry_func: String,
    #[arg(long, default_value = VERIFICATION_MODULE)]
    verification_module: String,
    #[arg(long, default_value = VERIFICATION_FUNC)]
    verification_func: String,
    #[arg(long = "verifier-module", default_value = VERIFIER_API)]
    onchain_verifier_module: String,
    #[arg(long = "publish-vk-func", default_value = PUBLISH_CIRCUIT)]
    onchain_publish_circuit_func: String,
    #[arg(long, default_value = VERIFY)]
    onchain_verify_func: String,
    #[arg(long = "package_dir", short = 'p', value_parser = value_parser! (PathBuf))]
    package_dir: PathBuf,
    #[arg(short = 'd', long = "debug", help = "debug mode")]
    debug: bool,
    #[command(subcommand)]
    command: AptosSubcommands,
}
impl AptosCommands {
    pub fn run(&self, params: &ParamsKZG<Bn256>) -> Result<()> {
        // Always root ourselves to the package root, and then compile relative to that.
        let rooted_path = SourcePackageLayout::try_find_root(&self.package_dir.canonicalize()?)?;
        let manifest = {
            let manifest_string =
                fs::read_to_string(rooted_path.join(SourcePackageLayout::Manifest.path()))?;
            let toml_manifest =
                move_package::source_package::manifest_parser::parse_move_manifest_string(
                    manifest_string,
                )?;
            move_package::source_package::manifest_parser::parse_source_manifest(toml_manifest)?
        };

        let package_name = manifest.package.name.to_string();
        let build_path = rooted_path
            .join(CompiledPackageLayout::Root.path())
            .join(&package_name);
        let package = OnDiskCompiledPackage::from_path(build_path.as_path())?;
        let package = package.into_compiled_package()?;

        match &self.command {
            AptosSubcommands::BuildRegisterVkAptosTxn(cmd) => cmd.run(
                &package,
                &self.zkmove_address,
                &self.vk_registry_module,
                &self.vk_registry_func,
                params,
            ),
            AptosSubcommands::BuildSubmitAttestationAptosTxn(cmd) => cmd.run(
                &package,
                &self.zkmove_address,
                &self.verification_module,
                &self.verification_func,
                params,
            ),
            AptosSubcommands::BuildPublishCircuitAptosTxn(cmd) => cmd.run(
                &package,
                &self.zkmove_address,
                &self.onchain_verifier_module,
                &self.onchain_publish_circuit_func,
                params,
            ),
            AptosSubcommands::BuildVerifyProofAptosTxn(cmd) => cmd.run(
                &self.zkmove_address,
                &self.onchain_verifier_module,
                &self.onchain_verify_func,
            ),
        }
    }
}

#[derive(Subcommand)]
enum AptosSubcommands {
    BuildRegisterVkAptosTxn(BuildRegisterVkAptosTxn),
    BuildSubmitAttestationAptosTxn(BuildSubmitAttestationTxn),
    BuildPublishCircuitAptosTxn(BuildPublishCircuitAptosTxn),
    BuildVerifyProofAptosTxn(BuildVerifyProofTxn),
}

#[derive(Parser)]
struct BuildRegisterVkAptosTxn {
    // TODO
}
impl BuildRegisterVkAptosTxn {
    pub fn run(
        &self,
        package: &CompiledPackage,
        zkmove_address: &str,
        vk_registry_module: &str,
        vk_registry_func: &str,
        params: &ParamsKZG<Bn256>,
    ) -> Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Parser)]
struct BuildSubmitAttestationTxn {
    // TODO
}

impl BuildSubmitAttestationTxn {
    pub fn run(
        &self,
        package: &CompiledPackage,
        zkmove_address: &str,
        verification_module: &str,
        verification_func: &str,
        params: &ParamsKZG<Bn256>,
    ) -> Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Parser)]
struct BuildPublishCircuitAptosTxn {
    #[arg(long = "entry_module", value_parser = value_parser!(ModuleIdWrapper))]
    entry_module: ModuleIdWrapper,
    #[arg(long = "function_name", value_parser = value_parser!(Identifier))]
    function_name: Identifier,
    #[arg(long = "output", short = 'o', value_parser = value_parser!(PathBuf))]
    output_dir: Option<PathBuf>,
    #[arg(long = "max_rows", default_value = "1024")]
    max_num_rows: usize,
}
impl BuildPublishCircuitAptosTxn {
    pub fn run(
        &self,
        package: &CompiledPackage,
        zkmove_address: &str,
        onchain_verifier_module: &str,
        onchain_publish_circuit_func: &str,
        params: &ParamsKZG<Bn256>,
    ) -> Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum KZGVariant {
    GWC,
    SHPLONK,
}

#[derive(Parser)]
struct BuildVerifyProofTxn {
    #[arg(long = "proof", short = 'p', value_parser = value_parser ! (PathBuf))]
    proof_path: PathBuf,
    #[arg(long = "output", short = 'o', value_parser = value_parser ! (PathBuf))]
    output_dir: Option<PathBuf>,
    #[arg(long)]
    param_address: String,
    #[arg(long)]
    circuit_address: String,

    #[arg(long = "kzg", value_enum)]
    variant: KZGVariant,
}
impl BuildVerifyProofTxn {
    pub fn run(
        &self,
        zkmove_address: &str,
        onchain_verifier_module: &str,
        onchain_verify_func: &str,
    ) -> Result<()> {
        // TODO
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ModuleIdWrapper(ModuleId);

impl FromStr for ModuleIdWrapper {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split("::").collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid module id format. Expected 'address::name'");
        }
        Ok(ModuleIdWrapper(ModuleId::new(
            parts[0].parse()?,
            Identifier::new(parts[1])?,
        )))
    }
}

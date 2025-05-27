use anyhow::{Context, Result};
use clap::{value_parser, Parser, Subcommand};
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr},
    plonk::VerifyingKey,
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
    SerdeFormat,
};
use logger::*;
use move_package::{
    compilation::{
        compiled_package::{CompiledPackage, OnDiskCompiledPackage},
        package_layout::CompiledPackageLayout,
    },
    source_package::layout::SourcePackageLayout,
};
use std::path::{Path, PathBuf};
use toml::Value;
#[cfg(feature = "test-circuits")]
use vm_circuit::mock_prove_circuit;
use vm_circuit::{
    best_k, prove_circuit, setup_circuit, verify_circuit, CircuitConfigV2,
    Footprints, InstanceFields, SubCircuit, VmCircuit, NUM_INSTANCE_COLUMNS,
};

#[derive(Parser)]
#[command(about = "Command for proving and verification.")]
pub struct VmCommands {
    #[command(subcommand)]
    command: Subcommands,
}
impl VmCommands {
    pub fn run(&self, params: &ParamsKZG<Bn256>) -> Result<()> {
        match &self.command {
            Subcommands::Prove(prove_command) => prove_command.run(params),
            Subcommands::Verify(verify_command) => verify_command.run(params),
        }
    }
}

#[derive(Subcommand)]
enum Subcommands {
    #[command(about = "Generate proof based on witness")]
    Prove(ProveCommand),
    #[command(about = "Verify proof")]
    Verify(VerifyCommand),
}

#[derive(Parser)]
#[command(about = "Generate proof based on witness")]
pub struct ProveCommand {
    #[arg(
        short = 'w',
        long = "witness",
        help = "path to .json file containing witness"
    )]
    witness: PathBuf,
    #[arg(
        long = "pubs-indices",
        help = "Indices of arguments to be treated as public inputs (e.g., --pubs-indices 0 1)",
        value_parser = clap::value_parser!(usize),
        num_args = 0..,
    )]
    pubs_indices: Vec<usize>,
    #[arg(short = 'o', long = "output-dir", help = "directory to save the proof")]
    output_dir: Option<PathBuf>,
    #[arg(short = 'd', long = "debug", help = "debug with mock prover")]
    debug: bool,
}

impl ProveCommand {
    pub fn run(&self, params: &ParamsKZG<Bn256>) -> Result<()> {
        debug!("Loading witness from {:?}", self.witness.display());
        let traces = Footprints::load(&self.witness)
            .with_context(|| format!("Failed to load witness from {:?}", self.witness))?;

        let rooted_path = self.find_package_root()?;
        let package = self.load_package(&rooted_path)?;

        let circuit_config =
            Self::get_circuit_config_from_move_toml(&rooted_path.join("Move.toml"));
        let circuit = VmCircuit::<Fr>::new(&package, &traces, &self.pubs_indices, circuit_config);

        let k = best_k(&circuit);
        debug!("Optimal k = {}", k);

        let mut params = params.clone();
        if k < params.k() {
            params.downsize(k);
        }

        let args = traces.args().expect("Args not found");
        let instances = InstanceFields::<_, NUM_INSTANCE_COLUMNS>::new(&args, &self.pubs_indices);

        #[cfg(feature = "test-circuits")]
        mock_prove_circuit(&circuit, instances.0, k)?;

        #[cfg(not(feature = "test-circuits"))]
        self.generate_and_save_proof(circuit, &instances, &params, &rooted_path)?;

        Ok(())
    }

    fn find_package_root(&self) -> Result<PathBuf> {
        SourcePackageLayout::try_find_root(&self.witness.canonicalize()?)
            .context("Failed to find root path for the package")
    }

    fn load_package(&self, rooted_path: &Path) -> Result<CompiledPackage> {
        let manifest_path = rooted_path.join(SourcePackageLayout::Manifest.path());
        let manifest_string = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("Failed to read manifest at {:?}", manifest_path))?;
        let toml_manifest =
            move_package::source_package::manifest_parser::parse_move_manifest_string(
                manifest_string,
            )?;
        let manifest =
            move_package::source_package::manifest_parser::parse_source_manifest(toml_manifest)?;

        let package_name = manifest.package.name.to_string();
        let build_path = rooted_path
            .join(CompiledPackageLayout::Root.path())
            .join(&package_name);
        let package = OnDiskCompiledPackage::from_path(build_path.as_path())
            .with_context(|| format!("Failed to load package at {:?}", build_path))?;
        Ok(package.into_compiled_package()?)
    }

    fn get_circuit_config_from_move_toml(toml_path: &Path) -> CircuitConfigV2 {
        let toml_content = std::fs::read_to_string(toml_path).expect("Failed to read Move.toml");
        let parsed_toml: Value = toml_content
            .parse::<Value>()
            .expect("Failed to parse Move.toml");

        if let Some(circuit) = parsed_toml.get("circuit") {
            let max_rows = circuit
                .get("max_rows")
                .and_then(|max_rows| max_rows.as_integer())
                .map(|v| v as usize);

            CircuitConfigV2 { max_rows }
        } else {
            CircuitConfigV2::default()
        }
    }

    fn save_to_file<P: AsRef<Path>, D: AsRef<[u8]>>(
        &self,
        dir: P,
        file_name: &str,
        data: D,
    ) -> Result<()> {
        let file_path = dir.as_ref().join(file_name);
        std::fs::write(&file_path, data)
            .with_context(|| format!("Failed to save file to {:?}", file_path))?;
        debug!("File saved to {:?}", file_path.display());
        Ok(())
    }

    #[cfg(not(feature = "test-circuits"))]
    fn generate_and_save_proof(
        &self,
        circuit: VmCircuit<Fr>,
        instances: &InstanceFields<Fr, NUM_INSTANCE_COLUMNS>,
        params: &ParamsKZG<Bn256>,
        rooted_path: &Path,
    ) -> Result<()> {
        debug!("Get proving and verifying keys");
        let (vk, pk) = setup_circuit(&circuit, params)?;

        debug!("Generating zk proof");
        let proof = prove_circuit(circuit, &instances.as_ref(), params, &pk)
            .context("Proof generation failed")?;

        let output_dir = self
            .output_dir
            .clone()
            .unwrap_or_else(|| rooted_path.join("proofs"));
        std::fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create output directory at {:?}", output_dir))?;

        let file_stem = self
            .witness
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid witness file name"))?;

        self.save_to_file(&output_dir, &format!("{}.proof", file_stem), &proof)?;
        self.save_to_file(
            &output_dir,
            &format!("{}.instance", file_stem),
            &instances.to_bytes(),
        )?;
        self.save_to_file(
            &output_dir,
            &format!("{}.vk", file_stem),
            &vk.to_bytes(SerdeFormat::Processed),
        )?;

        Ok(())
    }
}

#[derive(Parser)]
#[command(about = "Verify the proof")]
pub struct VerifyCommand {
    #[arg(short = 'k', help = "k for kzg params")]
    k: u32,
    #[arg(long = "pubs-path", value_parser = value_parser!(PathBuf))]
    pubs_path: PathBuf,
    #[arg(long = "proof-path", short = 'p', value_parser = value_parser!(PathBuf))]
    proof_path: PathBuf,
    #[arg(long = "vk-path", short = 'v', value_parser = value_parser!(PathBuf))]
    vk_path: PathBuf,
    #[arg(long = "output-dir", short = 'o', value_parser = value_parser!(PathBuf))]
    output_dir: Option<PathBuf>,
    #[arg(short = 'd', long = "debug", help = "debug with mock prover")]
    debug: bool,
}

impl VerifyCommand {
    pub fn run(&self, params: &ParamsKZG<Bn256>) -> Result<()> {
        let mut params = params.clone();
        if self.k < params.k() {
            params.downsize(self.k);
        }
        let vk = VerifyingKey::from_bytes::<VmCircuit<Fr>>(
            &std::fs::read(&self.vk_path)
                .with_context(|| format!("Failed to read vk from {:?}", self.vk_path))?,
            SerdeFormat::Processed,
        )?;
        let proof = std::fs::read(&self.proof_path)
            .with_context(|| format!("Failed to read proof from {:?}", self.proof_path))?;
        let pubs = std::fs::read(&self.pubs_path)
            .with_context(|| format!("Failed to read pubs from {:?}", self.pubs_path))?;
        let instances = InstanceFields::<Fr, NUM_INSTANCE_COLUMNS>::from_bytes(&pubs);

        verify_circuit(&instances.as_ref(), &params, &vk, &proof)
            .expect("verify proof should be ok");

        debug!("Proof verified.");
        Ok(())
    }
}

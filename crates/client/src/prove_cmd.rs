use anyhow::Result;
use clap::Parser;
use halo2_proofs::{
    halo2curves::bn256::{Bn256, Fr},
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use logger::*;
use move_package::{
    compilation::{compiled_package::OnDiskCompiledPackage, package_layout::CompiledPackageLayout},
    source_package::layout::SourcePackageLayout,
};
use std::path::PathBuf;
#[cfg(feature = "test-circuits")]
use vm_circuit::mock_prove_circuit;
use vm_circuit::{
    best_k, print_cs_info, prove_circuit, setup_circuit, verify_circuit, CircuitConfigV2,
    Footprints, InstanceFields, SubCircuit, VmCircuit, NUM_INSTANCE_COLUMNS,
};

#[derive(Parser)]
#[command(about = "Run the full sequence of setup, proving, and verification")]
pub struct ProveCommand {
    #[arg(
        short = 'w',
        long = "witness",
        help = "path to .json file containing witness"
    )]
    witness: PathBuf,
    #[arg(
        short = 'p',
        long = "pubs-indices",
        help = "Indices of arguments to be treated as public inputs (e.g., --pubs-indices 0 1)",
        value_parser = clap::value_parser!(usize),
        num_args = 0..,
    )]
    pubs_indices: Vec<usize>,
    #[arg(long = "proof-output-dir", help = "directory to save the proof")]
    proof_output_dir: Option<PathBuf>,
    #[arg(short = 'd', long = "debug", help = "debug with mock prover")]
    debug: bool,
}

impl ProveCommand {
    pub fn run(&self, params: &ParamsKZG<Bn256>) -> Result<()> {
        debug!("witness {:?}", self.witness.display());

        let traces = Footprints::load(&self.witness)?;
        let entry = traces.entry().expect("Entry not found");

        let rooted_path = SourcePackageLayout::try_find_root(&self.witness.canonicalize()?)?;
        let manifest = {
            let manifest_string =
                std::fs::read_to_string(rooted_path.join(SourcePackageLayout::Manifest.path()))?;
            let toml_manifest =
                move_package::source_package::manifest_parser::parse_move_manifest_string(
                    manifest_string,
                )?;
            move_package::source_package::manifest_parser::parse_source_manifest(toml_manifest)?
        };
        let package = {
            let package_name = manifest.package.name.to_string();
            let build_path = rooted_path
                .join(CompiledPackageLayout::Root.path())
                .join(&package_name);
            let package = OnDiskCompiledPackage::from_path(build_path.as_path())?;
            package.into_compiled_package()?
        };
        let circuit = VmCircuit::<Fr>::new(
            &package,
            &traces,
            &self.pubs_indices,
            CircuitConfigV2::default(),
        );

        let k = best_k(&circuit);
        debug!("k = {}", k);

        let mut params = params.clone();
        if k < params.k() {
            params.downsize(k);
        }

        debug!("Setup pk/vk");
        let (vk, pk) = setup_circuit(&circuit, &params)?;
        if self.debug {
            print_cs_info(vk.cs());
        }
        debug!("Generate zk proof");
        let instances =
            InstanceFields::<_, NUM_INSTANCE_COLUMNS>::new(&entry.args, &self.pubs_indices);

        #[cfg(feature = "test-circuits")]
        {
            if debug {
                debug!("Mock prove");
                mock_prove_circuit(&circuit, instances.0, k)?;
            }
        }
        #[cfg(not(feature = "test-circuits"))]
        {
            let proof = prove_circuit(circuit, &instances.as_ref(), &params, &pk)
                .expect("proof generation should not fail");
            verify_circuit(&instances.as_ref(), &params, &vk, &proof)
                .expect("verify proof should be ok");

            let proof_output_dir = self
                .proof_output_dir
                .clone()
                .unwrap_or_else(|| rooted_path.join("proofs"));
            std::fs::create_dir_all(proof_output_dir.as_path())?;
            let proof_output_path = proof_output_dir.join(format!(
                "{}.proof.hex",
                self.witness.file_stem().unwrap().to_str().unwrap()
            ));
            std::fs::write(proof_output_path.clone(), hex::encode(proof.clone()))?;
            debug!("Save proof to {:?}", proof_output_path.display());
        }
        Ok(())
    }
}

use clap::{Parser, Subcommand};
use halo2_proofs::{
    halo2curves::bn256::Bn256,
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use std::path::PathBuf;
use zkmove_cli::{aptos_cmds::AptosCommands, prove_cmd::ProveCommand, util_cmd::UtilCommand};

#[derive(Parser)]
#[command(name = "zkmove", about = "CLI for zkMove")]
pub struct Cli {
    #[arg(long, help = "param file used for prove/verify in kzg")]
    param_path: PathBuf,
    #[arg(short, help = "set k for kzg, if not set, use the k in param file")]
    k: Option<u8>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Util(UtilCommand),
    Prove(ProveCommand),
    Aptos(AptosCommands),
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut param_file = std::fs::File::open(args.param_path.as_path())?;

    let mut params = ParamsKZG::<Bn256>::read(&mut param_file)?;
    if let Some(k) = args.k {
        params.downsize(k as u32);
    }
    match args.command {
        Commands::Util(c) => c.run(&params),
        Commands::Prove(prove_command) => prove_command.run(&params),
        Commands::Aptos(aptos_command) => aptos_command.run(&params),
    }
}

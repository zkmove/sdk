use clap::{Parser, Subcommand};
use halo2_proofs::{
    halo2curves::bn256::Bn256,
    poly::{commitment::Params, kzg::commitment::ParamsKZG},
};
use std::path::PathBuf;
use zkmove_cli::{aptos_cmds::AptosCommands, prove_cmd::ProveCommand};

#[derive(Parser)]
#[command(name = "zkmove", about = "CLI for zkMove")]
pub struct Cli {
    #[arg(long, help = "param file used for prove/verify in kzg")]
    param_path: PathBuf,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Prove(ProveCommand),
    Aptos(AptosCommands),
}

fn main() -> anyhow::Result<()> {
    logger::init_for_main(true);
    let args = Cli::parse();
    let mut param_file = std::fs::File::open(args.param_path.as_path())?;
    let params = ParamsKZG::<Bn256>::read(&mut param_file)?;

    match args.command {
        Commands::Prove(prove_command) => prove_command.run(&params),
        Commands::Aptos(aptos_command) => aptos_command.run(&params),
    }
}

use clap::{clap_derive::ArgEnum, Parser};
use tracing_subscriber::EnvFilter;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ArgEnum, Debug, PartialEq, Clone)]
pub enum NearNetwork {
    Testnet,
    Mainnet,
}

#[derive(Parser, Debug)]
#[clap(
    version = VERSION,
    author = "Tonic Foundation <hello@tonic.foundation>"
)]
pub(crate) struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum SubCommand {
    Run(RunConfigArgs),
}

#[derive(Parser, Debug)]
pub(crate) struct RunConfigArgs {
    #[clap(long)]
    pub contract_id: String,

    #[clap(long)]
    pub from_blockheight: Option<u64>,

    #[clap(short, long, arg_enum)]
    pub network: NearNetwork,
}

pub(crate) fn init_logging() {
    let env_filter = EnvFilter::new(
        "nearcore=info,tonic=info,tonic-tps=info,tokio_reactor=info,near=info,stats=info,telemetry=info,indexer=info,near-performance-metrics=info",
    );
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();
}

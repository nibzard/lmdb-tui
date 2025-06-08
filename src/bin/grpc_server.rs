use std::net::SocketAddr;
use std::path::PathBuf;

use clap::Parser;
use lmdb_tui::{db::env as dbenv, grpc};

#[derive(Debug, Parser)]
#[command(author, version, about = "Run gRPC server for lmdb-tui automation")]
struct Args {
    /// Path to the LMDB environment directory
    path: PathBuf,
    /// Address to listen on
    #[arg(long, default_value = "127.0.0.1:50051")]
    addr: SocketAddr,
    /// Open environment read-only
    #[arg(long)]
    read_only: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    env_logger::init();
    let env = dbenv::open_env(&args.path, args.read_only)?;
    grpc::serve(env, args.addr).await?;
    Ok(())
}

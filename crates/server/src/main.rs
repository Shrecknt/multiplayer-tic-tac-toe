use clap::Parser;
use server::server::start_server;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Interface and port to listen on
    #[arg(short = 'H', long, default_value = "0.0.0.0:21552")]
    hostname: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting server.");

    let args = Args::parse();
    start_server(&args.hostname).await?;

    Ok(())
}

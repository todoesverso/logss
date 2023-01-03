use clap::Parser;

/// Simple program to show logs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Matching line
    #[arg(short, long)]
    pub contains: Vec<String>,
}

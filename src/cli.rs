
#[derive(clap::Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[arg(short, long)]
    pub uri: String,
}
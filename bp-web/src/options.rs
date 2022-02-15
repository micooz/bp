use clap::Parser;

#[derive(Parser)]
#[clap(name = "bp-web", version, about)]
pub struct Options {
    /// Web server launch address
    #[clap(long)]
    pub bind: Option<String>,
}

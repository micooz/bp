use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "bp", version = clap::crate_version!())]
pub struct Options {
    #[clap(short, long)]
    pub bind: String,

    #[clap(short, long)]
    pub config: String,
}

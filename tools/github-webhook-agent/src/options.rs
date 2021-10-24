use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "bp", version = clap::crate_version!())]
pub struct Options {
    /// Listen address
    #[clap(short, long)]
    pub bind: String,

    /// Config file in yaml format
    #[clap(short, long)]
    pub config: String,
}

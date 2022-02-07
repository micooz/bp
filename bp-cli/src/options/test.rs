use bp_core::Address;

#[derive(clap::Args, Default)]
pub struct TestOptions {
    /// Client configuration file path
    #[clap(long, default_value = "config.json")]
    pub config: String,

    /// Fire a HTTP GET request to this address, [default: <empty>]
    #[clap(long)]
    pub http: Option<Address>,
}

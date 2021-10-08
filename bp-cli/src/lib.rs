mod bootstrap;
mod options;
pub mod logging;
pub mod test_utils;

pub use bootstrap::bootstrap;
pub use options::{check_options, Options};

#[derive(Debug)]
pub struct ServiceContext {
    pub bind_addr: String,
    // pub handle: JoinHandle<()>,
}

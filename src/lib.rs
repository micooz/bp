type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, Error>;

mod bootstrap;
mod net;
mod options;
mod protocols;
mod utils;

pub use bootstrap::boot as bootstrap;
pub use options::Options;

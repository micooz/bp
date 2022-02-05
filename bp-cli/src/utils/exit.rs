use std::process;

pub fn exit(err: ExitError) -> ! {
    process::exit(err.into());
}

pub enum ExitError {
    ArgumentsError,
    BootstrapError,
}

impl From<ExitError> for i32 {
    fn from(v: ExitError) -> Self {
        match v {
            ExitError::ArgumentsError => 100,
            ExitError::BootstrapError => 200,
        }
    }
}

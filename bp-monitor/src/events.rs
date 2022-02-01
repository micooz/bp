use serde::Serialize;

pub trait Event {
    fn name() -> String;
}

#[derive(Serialize)]
pub struct NewConnectionEvent {
    pub inner: usize,
}

impl Event for NewConnectionEvent {
    fn name() -> String {
        "NewConnectionEvent".to_string()
    }
}

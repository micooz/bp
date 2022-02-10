use serde::Serialize;

use super::Event;

#[derive(Serialize)]
pub struct NewIncomingConnectionEvent {
    pub inner: usize,
}

impl Event for NewIncomingConnectionEvent {
    fn name() -> String {
        "NewIncomingConnectionEvent".to_string()
    }
}

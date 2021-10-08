use crate::context::SharedData;
use std::sync::Arc;

lazy_static::lazy_static! {
  pub static ref SHARED_DATA: Arc<SharedData> = Arc::new(SharedData::default());
}

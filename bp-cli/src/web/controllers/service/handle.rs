use bp_core::ServiceInfo;
use parking_lot::Mutex;
use tokio::sync::mpsc::Sender;

type Data = (Sender<()>, Vec<ServiceInfo>);

#[derive(Default)]
pub struct ServiceHandle {
    inner: Mutex<Option<Data>>,
}

impl ServiceHandle {
    pub fn info(&self) -> Option<Vec<ServiceInfo>> {
        let inner = self.inner.lock();
        (*inner).as_ref().map(|(_, info)| info.clone())
    }

    pub fn running(&self) -> bool {
        self.sender().is_some()
    }

    pub async fn abort(&self) {
        if self.running() {
            self.sender().unwrap().send(()).await.unwrap();
            self.set(None);
        }
    }

    pub fn set(&self, value: Option<Data>) {
        let mut inner = self.inner.lock();
        *inner = value;
    }

    fn sender(&self) -> Option<Sender<()>> {
        let inner = self.inner.lock();
        (*inner).as_ref().map(|(sender, _)| sender.clone())
    }
}

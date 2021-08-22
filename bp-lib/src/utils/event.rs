use std::{collections::HashMap, hash::Hash};

pub type Event<P, R> = Box<dyn (Fn(Option<P>) -> R) + Send + Sync>;

pub struct EventEmitter<E, P, R> {
    event_map: HashMap<E, Event<P, R>>,
}

impl<E, P, R> EventEmitter<E, P, R>
where
    E: Eq + Hash + ToString,
{
    pub fn on<F>(&mut self, event: E, callback: F)
    where
        F: (Fn(Option<P>) -> R) + Send + Sync + 'static,
    {
        self.event_map.insert(event, Box::new(callback));
    }

    pub async fn emit(&self, event: E, param: Option<P>) -> Result<R, String> {
        match self.event_map.get(&event) {
            Some(func) => Ok(func(param)),
            None => Err(format!("no listener found for {}", event.to_string())),
        }
    }
}

impl<E, P, R> Default for EventEmitter<E, P, R> {
    fn default() -> Self {
        Self {
            event_map: HashMap::new(),
        }
    }
}

#[test]
fn test_event_emitter() {
    #[derive(Debug, Default)]
    struct Param {
        inner: Option<HashMap<String, String>>,
    }

    #[derive(Debug)]
    enum Value {
        V1,
        V2,
    }

    let mut emitter = EventEmitter::<&str, Param, Value>::default();

    emitter.on("event_1", |v| {
        println!("event_1 triggered with param {:?}", v);
        Value::V1
    });

    emitter.on("event_2", |v| {
        println!("event_2 triggered with param {:?}", v);
        Value::V2
    });

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create tokio runtime");

    runtime.block_on(async {
        emitter.emit("event_1", Some(Param { inner: None })).await.unwrap();
        emitter.emit("event_2", Some(Param { inner: None })).await.unwrap();

        let handle = tokio::spawn(async move {
            let ret = emitter.emit("event_2", Some(Param { inner: None }));
            println!("event_2 returns value: {:?}", ret.await);
        });
        handle.await.unwrap();
    });
}

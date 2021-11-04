use std::collections::HashMap;

use bp_core::utils::event::*;

#[test]
fn test_event_emitter() {
    #[derive(Debug, Default)]
    struct Param {
        #[allow(dead_code)]
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

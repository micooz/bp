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

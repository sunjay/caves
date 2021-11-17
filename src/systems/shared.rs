use std::sync::{Arc, Mutex};

use specs::System;

#[derive(Debug, Default)]
pub struct SharedSystem<S> {
    system: Arc<Mutex<S>>,
}

impl<S> Clone for SharedSystem<S> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
        }
    }
}

impl<'a, S: System<'a>> SharedSystem<S> {
    pub fn new(system: S) -> Self {
        Self {
            system: Arc::new(Mutex::new(system)),
        }
    }
}

impl<'a, S: System<'a>> System<'a> for SharedSystem<S> {
    type SystemData = <S as System<'a>>::SystemData;

    fn run(&mut self, data: Self::SystemData) {
        self.system
            .lock()
            .expect("bug: system lock has been poisoned (another thread panicked holding the lock)")
            .run(data);
    }
}

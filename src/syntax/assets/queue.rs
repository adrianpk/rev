use super::Entry;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(super) struct Queue(Mutex<State>);
#[derive(Default)]
struct State {
    pending: VecDeque<Arc<Entry>>,
    running: bool,
}

impl Queue {
    pub fn schedule(self: &Arc<Self>, entry: Arc<Entry>) {
        let mut state = self.0.lock().expect("language asset queue");
        state.pending.push_back(entry);
        if state.running {
            return;
        }
        state.running = true;
        drop(state);
        let queue = std::sync::Arc::clone(self);
        if let Err(error) = std::thread::Builder::new()
            .name("language-assets".into())
            .spawn(move || queue.run())
        {
            let mut state = self.0.lock().expect("language asset queue");
            state.running = false;
            for entry in state.pending.drain(..) {
                let _ = entry
                    .outcome
                    .set(Err(format!("start language asset worker: {error}")));
            }
        }
    }

    fn next(&self) -> Option<Arc<Entry>> {
        let mut state = self.0.lock().expect("language asset queue");
        let entry = state.pending.pop_front();
        if entry.is_none() {
            state.running = false;
        }
        entry
    }
    fn run(&self) {
        while let Some(entry) = self.next() {
            // Native code is trusted. This catches Rust admission panics, not
            // arbitrary faults in a user-supplied shared library.
            let outcome = std::panic::catch_unwind(|| super::admission::load(&entry))
                .unwrap_or_else(|_| Err("language asset admission panicked".into()));
            let _ = entry.outcome.set(outcome);
        }
    }
}

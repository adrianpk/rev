use super::fixture;
use rev::git::{Repositories, Update};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

#[path = "repository_fixture.rs"]
mod repository_fixture;
pub use repository_fixture::Repository;

impl Repository {
    pub fn owner(&self, extra: &str) -> Repositories {
        Repositories::new(&self.snapshot(extra), Arc::new(|| {}))
    }

    pub fn open(&self, owner: &mut Repositories, directory: &Path) -> Update {
        let operation = owner.open(directory).unwrap();
        let update = finish(owner);
        assert_eq!(update.operation, operation);
        update
    }
}

pub fn finish(owner: &mut Repositories) -> Update {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut updates = owner.poll();
        if !updates.is_empty() {
            return updates.remove(0);
        }
        assert!(
            Instant::now() < deadline,
            "Git work did not finish: {:?}",
            owner.progress().collect::<Vec<_>>()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

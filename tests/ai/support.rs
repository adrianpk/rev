use crate::fixture::Fixture;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

pub fn peer(fixture: &Fixture, mode: &str) -> std::path::PathBuf {
    let root = &fixture.0;
    let path = root.join("pi-peer");
    fs::write(&path, include_str!("peer.py")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    fixture.write("mode", mode);
    path
}

pub fn until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(12);
    while !ready() {
        assert!(Instant::now() < deadline, "controlled Pi fixture deadline");
        std::thread::sleep(Duration::from_millis(5));
    }
}

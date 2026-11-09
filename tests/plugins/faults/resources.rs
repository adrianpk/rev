use super::super::{fixture::Fixture, lifecycle::peer};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    time::{Duration, Instant},
};

#[allow(dead_code)]
#[path = "../../../examples/completion_acceptance/support.rs"]
mod support;
#[allow(dead_code)]
#[path = "../../../examples/completion_acceptance/terminal.rs"]
mod terminal;
pub(super) use terminal::Terminal;
pub(super) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Reuse the existing 8-second/1-MiB PTY owner and isolated native roots. Keep
// Terminal first so its owned editor is reaped before fixture files disappear.
pub(super) struct Native {
    pub terminal: Terminal,
    pub fixture: Fixture,
    pub launch: Duration,
}
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_str().unwrap().replace('\'', "'\\''"))
}
impl Native {
    pub fn start(profile: &str, mode: Option<&str>, startup: bool, large: bool) -> Result<Self> {
        let fixture = Fixture::new();
        for path in [
            "pty/home",
            "pty/config/rev",
            "pty/data",
            "pty/state",
            "pty/cache",
            "workspace",
            "plugin",
        ] {
            fs::create_dir_all(fixture.0.join(path))?;
        }
        let mut config = format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\n"
        );
        if let Some(mode) = mode {
            fixture.write("plugin/mode", mode);
            fixture.write("plugin/profile", profile);
            let manifest = fixture.write(
                "plugin.toml",
                &format!(
                    r#"[plugin]
id='field.notes'
version='0.1.0'
api='rev.plugins/0.1'
[process]
program={}
args={}
[[commands]]
name='collect'
description='Collect source into a result buffer.'
parameters=[]
prefix='none'
recording='reject'
"#,
                    serde_json::to_string(peer())?,
                    json!([fixture.0.join("plugin"), "field.notes"])
                ),
            );
            config += &format!(
                "[plugins]\nenabled=true\nmanifests={}\nstartup={}\n",
                json!([manifest]),
                if startup { "['field.notes']" } else { "[]" }
            );
        }
        fixture.write("pty/config/rev/config.toml", &config);
        let text = if large {
            ("l".repeat(63) + "\n").repeat(8192)
        } else {
            "source text\n".into()
        };
        fixture.write("workspace/source.txt", &text);
        let wrapper = fixture.write(
            "native-editor",
            &format!(
                "#!/bin/sh\nprintf '%s' \"$$\" > {}\nexec {} \"$@\"\n",
                quote(&fixture.0.join("editor.pid")),
                quote(Path::new(env!("CARGO_BIN_EXE_rev")))
            ),
        );
        fs::set_permissions(&wrapper, fs::Permissions::from_mode(0o700))?;
        let began = Instant::now();
        let terminal = Terminal::start(
            &fixture.0,
            &wrapper,
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/check-completion.sh"),
            "source.txt",
        )?;
        Ok(Self {
            terminal,
            fixture,
            launch: began.elapsed(),
        })
    }
    pub fn peer_path(&self, name: &str) -> std::path::PathBuf {
        self.fixture.0.join("plugin").join(name)
    }
    pub fn editor_pid(&self) -> Result<String> {
        Ok(fs::read_to_string(self.fixture.0.join("editor.pid"))?)
    }
    pub fn request(&mut self, sequence: usize, method: &str, params: Value) -> Result<()> {
        let temporary = self.peer_path("request-tmp");
        fs::write(
            &temporary,
            format!(
                "{}\n",
                json!({"jsonrpc":"2.0", "id":format!("buffer-request-{sequence}"), "method":method, "params":params})
            ),
        )?;
        fs::rename(temporary, self.peer_path(&format!("request-{sequence}")))?;
        Ok(())
    }
    pub fn response(&self, sequence: usize) -> Result<Value> {
        wait(|| self.peer_path(&format!("response-{sequence}")).exists())
            .map_err(|e| format!("plugin RPC response {sequence}: {e}"))?;
        let value: Value =
            serde_json::from_slice(&fs::read(self.peer_path(&format!("response-{sequence}")))?)?;
        assert!(value.get("error").is_none(), "{value}");
        assert_eq!(value["id"], format!("buffer-request-{sequence}"));
        Ok(value["result"].clone())
    }
    pub fn command(&mut self, profile: &str, command: &str) -> Result<()> {
        let mut bytes = if profile == "modal" {
            b":".to_vec()
        } else {
            b"\x1bx".to_vec()
        };
        bytes.extend_from_slice(command.as_bytes());
        bytes.push(b'\r');
        self.terminal.send(&bytes)
    }
}
pub(super) fn wait(mut predicate: impl FnMut() -> bool) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !predicate() {
        if Instant::now() >= deadline {
            return Err("owned plugin fixture exceeded five seconds".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
fn memory(pid: &str, key: &str) -> Result<u64> {
    let text = fs::read_to_string(format!("/proc/{pid}/status"))?;
    Ok(text
        .lines()
        .find_map(|line| line.strip_prefix(key))
        .ok_or("missing process memory")?
        .split_whitespace()
        .next()
        .ok_or("missing memory value")?
        .parse()?)
}
fn descriptors(pid: &str) -> Result<usize> {
    Ok(fs::read_dir(format!("/proc/{pid}/fd"))?.count())
}

// Five fresh native processes separate launch/idle costs from two bounded work
// loads. These observations are not benchmarks or production latency promises.
#[test]
fn resource_profiles() -> Result<()> {
    for (profile, mode, startup, large) in [
        ("baseline", None, false, false),
        ("installed-unused", Some("echo"), false, false),
        ("active-idle", Some("echo"), true, false),
        ("large-read", Some("events"), true, true),
        ("event-load", Some("events"), true, false),
    ] {
        let mut native = Native::start("non-modal", mode, startup, large)?;
        let pid = native.editor_pid()?;
        let peer = if startup {
            wait(|| native.peer_path("offers").exists())
                .map_err(|e| format!("{profile} startup: {e}"))?;
            Some(
                fs::read_to_string(native.peer_path("starts"))?
                    .trim()
                    .to_owned(),
            )
        } else {
            None
        };
        let mut work = Duration::ZERO;
        let mut delivered = 0;
        if large {
            native.request(
                0,
                "buffers.list",
                json!({"page":{"cursor":null,"limit":256},"kind":"text"}),
            )?;
            let list = native.response(0)?;
            let source = list["buffers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|v| v["name"].as_str().unwrap().ends_with("source.txt"))
                .unwrap();
            let start = Instant::now();
            for n in 1..=4 {
                native.request(n,"buffers.read",json!({"buffer":source["stamp"]["buffer"],"range":{"start":0,"end":524288},"expected_revision":null}))?;
            }
            native.terminal.send(b"!")?;
            native.terminal.frame(|f| f.contains("!llll"))?;
            work = start.elapsed();
            for n in 1..=4 {
                assert_eq!(native.response(n)?["text"].as_str().unwrap().len(), 524288);
            }
        } else if profile == "event-load" {
            native.request(
                0,
                "events.subscribe",
                json!({"kinds":["changed"],"buffers":[]}),
            )?;
            native.response(0)?;
            let start = Instant::now();
            native.terminal.send(&[b'e'; 128])?;
            native.terminal.frame(|f| f.contains("eeeeeeee"))?;
            work = start.elapsed();
            wait(|| {
                fs::read_to_string(native.peer_path("events"))
                    .is_ok_and(|s| s.lines().count() >= 128)
            })?;
            delivered = fs::read_to_string(native.peer_path("events"))?
                .lines()
                .count();
            assert_eq!(delivered, 128);
        }
        let before = support::cpu_ticks(&pid)?;
        let peer_before = peer.as_ref().map(|p| support::cpu_ticks(p)).transpose()?;
        native.terminal.idle()?;
        let ticks = support::cpu_ticks(&pid)?.saturating_sub(before);
        let peer_ticks = peer
            .as_ref()
            .zip(peer_before)
            .map(|(p, b)| support::cpu_ticks(p).map(|a| a.saturating_sub(b)))
            .transpose()?
            .unwrap_or(0);
        let peer_rss = peer
            .as_ref()
            .map(|p| memory(p, "VmRSS:"))
            .transpose()?
            .unwrap_or(0);
        let peer_fds = peer
            .as_ref()
            .map(|p| descriptors(p))
            .transpose()?
            .unwrap_or(0);
        if !startup {
            assert!(!native.peer_path("starts").exists());
        }
        println!(
            "MEASURE plugins profile={profile} launch_us={} rss_kib={} peak_kib={} fds={} idle_ticks={ticks} peer_rss_kib={peer_rss} peer_fds={peer_fds} peer_idle_ticks={peer_ticks} work_input_us={} events={delivered} tty_bytes={}",
            native.launch.as_micros(),
            memory(&pid, "VmRSS:")?,
            memory(&pid, "VmHWM:")?,
            descriptors(&pid)?,
            work.as_micros(),
            native.terminal.bytes()
        );
        if large || profile == "event-load" {
            native.terminal.send(b"\x18\x13")?;
            let saved = native.fixture.0.join("workspace/source.txt");
            native.terminal.frame(|_| {
                fs::read_to_string(&saved)
                    .is_ok_and(|text| text.starts_with(if large { "!" } else { "eeeeeeee" }))
            })?;
        }
        native.terminal.finish()?;
        assert!(!Path::new(&format!("/proc/{pid}")).exists());
        if let Some(peer) = peer {
            assert!(!Path::new(&format!("/proc/{peer}")).exists());
        }
    }
    Ok(())
}

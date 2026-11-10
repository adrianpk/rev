//! The supervisor owns one child, captured root and retained session boundary.
use super::{
    Answer, Control, Profile, Purpose, Request, directory, history, io, protocol, resource,
};
use crate::config::Pi;
use serde_json::json;
use std::fs::{File, Metadata};
use std::os::unix::fs::MetadataExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::{Arc, atomic::Ordering};
use std::time::{Duration, Instant};

pub(super) struct Launch {
    pub policy: Pi,
    pub root: PathBuf,
    pub root_handle: Option<File>,
    pub request: Request,
    pub profile: Profile,
}

struct Root {
    path: PathBuf,
    metadata: Metadata,
}
impl Root {
    fn capture(path: &Path, held: Option<&File>, directory: bool) -> Result<Self, String> {
        let canonical = path
            .canonicalize()
            .map_err(|_| "Pi working directory unavailable")?;
        if directory && canonical.as_os_str() != path.as_os_str() {
            return Err("Pi canonical root changed before launch".into());
        }
        let metadata = std::fs::metadata(&canonical).map_err(|_| "Pi root observation failed")?;
        if !metadata.is_dir() {
            return Err("Pi working directory is not a directory".into());
        }
        if let Some(handle) = held {
            let held = handle.metadata().map_err(|_| "Pi root handle failed")?;
            if (held.dev(), held.ino()) != (metadata.dev(), metadata.ino()) {
                return Err("Pi directory root changed before launch".into());
            }
        }
        Ok(Self {
            path: canonical,
            metadata,
        })
    }
}

impl Launch {
    pub fn run(mut self, control: &Arc<Control>) -> Result<Option<Answer>, String> {
        let root = Root::capture(
            &self.root,
            self.root_handle.as_ref(),
            self.profile.directory,
        )?;
        probe(&self.policy, &root.path, control, self.request.deadline)?;
        let store = open_store(&self.policy, &self.profile)?;
        let candidate = store.as_ref().map(history::Store::candidate).transpose()?;
        let resource = extraction(&self.profile)?;
        if control.canceled() {
            return Err("Pi invocation canceled".into());
        }
        let (pipes, mut child) =
            self.child(&root.path, candidate.as_deref(), resource.as_ref(), control)?;
        let output = pipes.drive(&mut child, |inbox, input, _child| {
            let private_roots = private_roots(&self.profile, candidate.as_deref())?;
            let mut supervisor = Supervisor {
                policy: &self.policy,
                profile: &self.profile,
                root: &root,
                control,
                private_roots,
                candidate,
                store,
                lease: None,
                history_ready: false,
                model: String::new(),
                reused: false,
            };
            supervisor.run(&mut self.request, inbox, input)
        });
        // The extension file remains owned until stream draining and child
        // cleanup finish, including an early protocol or reader failure.
        drop(resource);
        output
    }

    fn child(
        &self,
        root: &Path,
        candidate: Option<&Path>,
        resource: Option<&resource::Resource>,
        control: &Arc<Control>,
    ) -> Result<(io::Pipes, io::Owned), String> {
        let (pipes, [stdin, stdout, stderr]) = io::Pipes::new()?;
        let mut command = self.command(candidate, resource)?;
        let child = io::Owned::new(
            command
                .current_dir(root)
                .stdin(stdin)
                .stdout(stdout)
                .stderr(stderr)
                .spawn()
                .map_err(|_| "Pi RPC launch failed; check ai.pi.command")?,
            Arc::clone(control),
        );
        if candidate.is_some() {
            child.limit_history()?;
        }
        Ok((pipes, child))
    }

    fn command(
        &self,
        candidate: Option<&Path>,
        resource: Option<&resource::Resource>,
    ) -> Result<Command, String> {
        let mut command = Command::new(&self.policy.command);
        command.args([
            "--mode",
            "rpc",
            "--no-extensions",
            "--no-mcp",
            "--no-skills",
            "--no-prompt-templates",
            "--no-context-files",
            "--no-themes",
            "--no-approve",
            "--offline",
        ]);
        if let Some(path) = candidate {
            command.arg("--session").arg(path).arg("--session-dir").arg(
                path.parent()
                    .ok_or("Pi private session directory missing")?,
            );
        } else {
            command.arg("--no-session");
        }
        if let Some(resource) = resource {
            resource
                .check()
                .map_err(|_| "Pi internal resource changed")?;
            command
                .args(["--tools", "rev_read,rev_list,rev_search", "--extension"])
                .arg(&resource.path);
        } else {
            command.arg("--no-tools");
        }
        if !matches!(self.request.purpose, Purpose::Models)
            && let Some(model) = &self.policy.model
        {
            command.arg("--model").arg(model);
        }
        Ok(command)
    }
}

fn open_store(policy: &Pi, profile: &Profile) -> Result<Option<history::Store>, String> {
    if !profile.retained || !policy.persist_sessions {
        return Ok(None);
    }
    let state = profile.state.clone().map(Ok).unwrap_or_else(|| {
        crate::config::Roots::from_env()
            .map(|r| r.state)
            .map_err(|_| "Pi state root unavailable".to_string())
    })?;
    Ok(Some(history::Store::open(&state)?))
}

fn extraction(profile: &Profile) -> Result<Option<resource::Resource>, String> {
    if profile.directory {
        Ok(Some(resource::Resource::new()?))
    } else {
        Ok(None)
    }
}

fn private_roots(profile: &Profile, candidate: Option<&Path>) -> Result<Vec<PathBuf>, String> {
    let mut roots = if profile.directory {
        directory::private_roots()?
    } else {
        Vec::new()
    };
    if let Some(path) = candidate
        && let Some(parent) = path.parent().and_then(|p| p.parent())
    {
        roots.push(parent.to_owned());
    }
    Ok(roots)
}

struct Supervisor<'a> {
    policy: &'a Pi,
    profile: &'a Profile,
    root: &'a Root,
    control: &'a Arc<Control>,
    private_roots: Vec<PathBuf>,
    candidate: Option<PathBuf>,
    store: Option<history::Store>,
    lease: Option<history::Lease>,
    history_ready: bool,
    model: String,
    reused: bool,
}

impl Supervisor<'_> {
    fn run(
        &mut self,
        request: &mut Request,
        inbox: &io::Inbox,
        input: &mut UnixStream,
    ) -> Result<Option<Answer>, String> {
        loop {
            let answer = {
                let mut exchange = protocol::Exchange {
                    inbox,
                    input,
                    control: self.control,
                    request,
                };
                match self.answer(&mut exchange) {
                    Ok(answer) => answer,
                    Err(error) => {
                        exchange.cancel();
                        return Err(error);
                    }
                }
            };
            if !self.profile.retained || self.policy.idle_timeout_ms == 0 {
                return Ok(Some(answer));
            }
            let until = Instant::now() + Duration::from_millis(self.policy.idle_timeout_ms as u64);
            self.publish(answer, until);
            let Some(next) = self.idle(inbox, until)? else {
                return Ok(None);
            };
            *request = next;
        }
    }

    fn answer(&mut self, exchange: &mut protocol::Exchange<'_>) -> Result<Answer, String> {
        let request = exchange.request;
        match &request.purpose {
            Purpose::Models => {
                let data = exchange.request("get_available_models", json!({}))?;
                Ok(Answer::Models(protocol::models(&data)?))
            }
            Purpose::Generate(message) => self.generate(exchange, message),
        }
    }

    fn generate(
        &mut self,
        exchange: &mut protocol::Exchange<'_>,
        message: &str,
    ) -> Result<Answer, String> {
        self.prepare(exchange)?;
        let provenance = self.provenance().to_owned();
        let output = exchange.generate(protocol::Generation {
            policy: self.policy,
            message,
            model: self.model.clone(),
            directory: self.profile.directory,
            history: provenance,
            lease: self.lease.as_mut(),
        })?;
        if let Some(lease) = &mut self.lease {
            lease.touch()?;
        }
        self.reused = true;
        Ok(Answer::Generated(output))
    }

    fn prepare(&mut self, exchange: &mut protocol::Exchange<'_>) -> Result<(), String> {
        let before = self.profile.directory && (self.history_ready || self.candidate.is_none());
        if before {
            self.admit(exchange)?;
        }
        self.observe(exchange)?;
        self.bind(exchange)?;
        if self.profile.directory && !before {
            self.admit(exchange)?;
        }
        Ok(())
    }

    fn observe(&mut self, exchange: &mut protocol::Exchange<'_>) -> Result<(), String> {
        let state = exchange.request("get_state", json!({}))?;
        let selected = self
            .lease
            .as_ref()
            .map(|l| l.path.as_path())
            .or(self.candidate.as_deref());
        let observed = protocol::state(&state, selected, !self.history_ready)?;
        if self.history_ready && observed != self.model {
            return Err("Pi model changed across an owned session boundary".into());
        }
        self.model = observed;
        Ok(())
    }

    fn bind(&mut self, exchange: &mut protocol::Exchange<'_>) -> Result<(), String> {
        if self.history_ready {
            return Ok(());
        }
        if let Some(store) = self.store.take() {
            self.bind_history(store, exchange)?;
        }
        self.reset_applied();
        self.history_ready = true;
        Ok(())
    }

    fn bind_history(
        &mut self,
        store: history::Store,
        exchange: &mut protocol::Exchange<'_>,
    ) -> Result<(), String> {
        let fingerprint = fingerprint(self.policy, &self.root.metadata, &self.private_roots)?;
        let chosen = store.bind(&history::Selection {
            root: &self.root.path,
            fingerprint: &fingerprint,
            model: &self.model,
            candidate: self.candidate.as_deref().ok_or("Pi candidate missing")?,
            reset: self.profile.reset,
        })?;
        self.reused = Some(&chosen.path) != self.candidate.as_ref();
        self.reset_applied();
        if self.reused {
            self.switch_session(exchange, &chosen)?;
        }
        self.lease = Some(chosen);
        Ok(())
    }

    fn reset_applied(&self) {
        if self.profile.reset {
            self.control.reset_applied.store(true, Ordering::Release);
        }
    }

    fn switch_session(
        &self,
        exchange: &mut protocol::Exchange<'_>,
        chosen: &history::Lease,
    ) -> Result<(), String> {
        let switched = exchange.request("switch_session", json!({"sessionPath":chosen.path}))?;
        if switched["cancelled"] != false {
            return Err("Pi history switch was canceled".into());
        }
        let state = exchange.request("get_state", json!({}))?;
        if protocol::state(&state, Some(&chosen.path), false)? != self.model {
            return Err(
                "Pi restored a different model/provider; preserve history and reset it".into(),
            );
        }
        Ok(())
    }

    fn admit(&self, exchange: &mut protocol::Exchange<'_>) -> Result<(), String> {
        let timeout = exchange
            .request
            .deadline
            .saturating_duration_since(Instant::now())
            .as_millis()
            .min(1_800_000);
        let timeout = u64::try_from(timeout).expect("Pi grant timeout is capped at 1800000 ms");
        let grant = json!({"version":1,"nonce":exchange.request.epoch,"root":self.root.path,
            "dev":self.root.metadata.dev().to_string(),"ino":self.root.metadata.ino().to_string(),
            "timeout_ms":timeout,"max_input_bytes":self.policy.max_input_bytes,"private_roots":self.private_roots});
        exchange.admit(&grant)
    }

    fn provenance(&self) -> &'static str {
        if self.lease.is_some() {
            if self.reused {
                "private directory history; resumed"
            } else {
                "private directory history; new"
            }
        } else if self.profile.directory {
            if self.reused {
                "in-memory directory history; reused"
            } else {
                "fresh ephemeral directory history"
            }
        } else {
            "fresh input-only history"
        }
    }

    fn publish(&self, answer: Answer, until: Instant) {
        let mut mailbox = self.control.mailbox.lock().expect("Pi mailbox");
        mailbox.result = Some(answer);
        mailbox.idle_until = Some(until);
    }

    fn idle(&self, inbox: &io::Inbox, until: Instant) -> Result<Option<Request>, String> {
        loop {
            {
                let mut mailbox = self.control.mailbox.lock().expect("Pi mailbox");
                if let Some(next) = mailbox.pending.take() {
                    return Ok(Some(next));
                }
                if self.control.canceled() || Instant::now() >= until {
                    mailbox.closed = true;
                    mailbox.idle_until = None;
                    return Ok(None);
                }
                // Pending admission and retirement are atomic; release their
                // shared owner before waiting for another protocol record.
                drop(mailbox);
            }
            if inbox
                .next(until.saturating_duration_since(Instant::now()))?
                .is_some()
            {
                return Err(
                    "Pi emitted work after settlement; the owned connection was retired".into(),
                );
            }
        }
    }
}

fn fingerprint(
    policy: &Pi,
    root: &std::fs::Metadata,
    private: &[PathBuf],
) -> Result<String, String> {
    // Literal canonical policy data, not a hash treated as proof of equivalence.
    serde_json::to_string(&json!({"grant":"rev-pi-read-list-search-v2","pi":"1.3.0","root_dev":root.dev().to_string(),"root_ino":root.ino().to_string(),"command":policy.command,"model_selection":policy.model,"persist":policy.persist_sessions,"idle_ms":policy.idle_timeout_ms,"request_ms":policy.request_timeout_ms,"input":policy.max_input_bytes,"result":policy.max_result_bytes,"tools":["rev_read","rev_list","rev_search"],"private":private,"links":"reject","turns":8,"calls":32,"returned":4194304,"file":1048576})).map_err(|_| "Pi grant fingerprint encoding failed".into())
}

fn probe(
    policy: &Pi,
    root: &Path,
    control: &Arc<Control>,
    deadline: Instant,
) -> Result<(), String> {
    let (pipes, [stdin, stdout, stderr]) = io::Pipes::new()?;
    let mut child = io::Owned::new(
        Command::new(&policy.command)
            .arg("--version")
            .env("PI_OFFLINE", "1")
            .current_dir(root)
            .stdin(stdin)
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .map_err(|_| "Pi is unavailable; install Pi 1.3.0 or set ai.pi.command")?,
        Arc::clone(control),
    );
    pipes.drive(&mut child, |inbox, _input, child| {
        version(inbox, child, control, deadline)
    })
}

fn version(
    inbox: &io::Inbox,
    child: &mut Child,
    control: &Control,
    deadline: Instant,
) -> Result<(), String> {
    let deadline = deadline.min(Instant::now() + Duration::from_secs(5));
    loop {
        if control.canceled() {
            return Err("Pi invocation canceled".into());
        }
        if Instant::now() >= deadline {
            return Err("Pi version probe exceeded its deadline".into());
        }
        if let Some(bytes) = inbox.next(Duration::from_millis(20))? {
            if bytes != b"1.3.0" {
                return Err(
                    "Unsupported Pi version; this adapter requires checked Pi 1.3.0".into(),
                );
            }
            // Version output admits the same executable used for the RPC child.
            return version_exit(child);
        }
    }
}

fn version_exit(child: &mut Child) -> Result<(), String> {
    let exit = Instant::now() + Duration::from_secs(1);
    while Instant::now() < exit {
        if let Some(status) = child.try_wait().map_err(|_| "Pi version wait failed")? {
            return if status.success() {
                Ok(())
            } else {
                Err("Pi version probe failed".into())
            };
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("Pi version probe did not exit".into())
}

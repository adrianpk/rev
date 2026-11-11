use super::support::{self, synchronized};
use crate::support::{Result, complete, cpu_ticks, idle, key, open, select, settle, source};
use crossterm::event::KeyCode;
use rev::{
    command::{BufferCommand, Command, CompletionCommand as C, History, LspCommand, Motion, Split},
    session::{CompletionFallback as F, CompletionSource as S, Session},
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DOCUMENT: &str = "fn cfg_native() {}\nfn main() { cfg| }\n";
struct Journey {
    root: PathBuf,
    session: Session,
    original: String,
}

pub(crate) fn run(root: &Path) -> Result<()> {
    let root = root.join("controlled");
    let config = support::peer(&root)?;
    std::fs::create_dir(root.join("workspace"))?;
    std::fs::write(root.join("config.toml"), config)?;
    let settings = rev::config::prepare(&root.join("config.toml"));
    assert!(settings.diagnostic.is_none());
    let session = open(&root, "main.rs", DOCUMENT, &settings.snapshot)?;
    let mut journey = Journey {
        root,
        session,
        original: DOCUMENT.replace('|', ""),
    };
    journey.switch_sources()?;
    journey.shared_root()?;
    journey.empty_and_failed()?;
    journey.retention_and_exit()
}
impl Journey {
    fn switch_sources(&mut self) -> Result<()> {
        complete(&mut self.session)?;
        assert!(matches!(
            self.session.completion_source(),
            Some(S::Native(_))
        ));
        select(&mut self.session, "cfg_native");
        idle(&mut self.session);
        assert!(!self.root.join("peer.pid").exists());
        self.result(&json!([{"label":"cfg_server","data":{"selected":1}}]))?;
        std::fs::write(self.root.join("peer.resolved"), json!({"label":"cfg_server","additionalTextEdits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"use demo::Thing;\n"}]}).to_string())?;
        self.session.execute(Command::Lsp(LspCommand::Mode));
        synchronized(&mut self.session)?;
        self.import()?;
        self.session.execute(Command::Lsp(LspCommand::Disconnect));
        assert_eq!(self.session.lsp_status(), rev::lsp::Status::Excluded);
        self.native(None)?;
        self.session.execute(Command::Lsp(LspCommand::Connect));
        synchronized(&mut self.session)?;
        complete(&mut self.session)?;
        assert!(matches!(
            self.session.completion_source(),
            Some(S::Semantic { .. })
        ));
        Ok(())
    }
    fn import(&mut self) -> Result<()> {
        complete(&mut self.session)?;
        assert!(matches!(
            self.session.completion_source(),
            Some(S::Semantic { .. })
        ));
        select(&mut self.session, "cfg_server");
        assert_eq!(support::count(&self.root, "completionItem/resolve"), 0);
        let history = self.session.buffer()?.history().len();
        self.session.execute(Command::Completion(C::Accept));
        settle(&mut self.session)?;
        assert_eq!(
            source(&self.session),
            "use demo::Thing;\n".to_owned() + &self.original.replace("{ cfg }", "{ cfg_server }")
        );
        assert_eq!(self.session.buffer()?.history().len(), history + 1);
        assert_eq!(support::count(&self.root, "completionItem/resolve"), 1);
        self.session.execute(Command::History(History::Undo));
        assert_eq!(source(&self.session), self.original);
        Ok(())
    }
    fn shared_root(&mut self) -> Result<()> {
        let first = synchronized(&mut self.session)?;
        self.session.execute(Command::Split(Split::Right));
        assert_eq!(synchronized(&mut self.session)?, first);
        let other = self.root.join("workspace/other.rs");
        std::fs::write(&other, "fn other() {}\n")?;
        self.session.load(&other)?;
        assert_eq!(
            synchronized(&mut self.session)?.connection,
            first.connection
        );
        self.close();
        self.session.load(&self.root.join("workspace/main.rs"))?;
        self.session.execute(Command::Motion(Motion::BufferEnd));
        for _ in 0..3 {
            self.session.execute(Command::Motion(Motion::BackwardChar));
        }
        assert_eq!(synchronized(&mut self.session)?, first);
        assert_eq!(support::count(&self.root, "initialize"), 1);
        Ok(())
    }
    fn empty_and_failed(&mut self) -> Result<()> {
        self.result(&json!([]))?;
        self.native(Some(F::Empty))?;
        assert!(matches!(
            self.session.lsp_status(),
            rev::lsp::Status::Ready(_)
        ));
        let pid = support::pid(&self.root)?;
        assert!(
            std::process::Command::new("kill")
                .args(["-TERM", "--", &pid])
                .status()?
                .success()
        );
        support::wait(|| support::failed(&self.session))?;
        support::reaped(&pid)?;
        for _ in 0..3 {
            self.native(Some(F::Unavailable))?;
        }
        assert_eq!(support::count(&self.root, "initialize"), 1);
        self.session.execute(Command::Lsp(LspCommand::Restart));
        synchronized(&mut self.session)?;
        assert_eq!(support::count(&self.root, "initialize"), 2);
        Ok(())
    }
    fn native(&mut self, reason: Option<F>) -> Result<()> {
        complete(&mut self.session)?;
        assert!(matches!(
            self.session.completion_source(),
            Some(S::Native(_))
        ));
        if let Some(reason) = reason {
            assert_eq!(self.session.completion_source(), Some(S::Native(reason)));
        }
        select(&mut self.session, "cfg_native");
        let history = self.session.buffer()?.history().len();
        self.session.execute(Command::Completion(C::Accept));
        assert_eq!(self.session.buffer()?.history().len(), history + 1);
        self.session.execute(Command::History(History::Undo));
        assert_eq!(source(&self.session), self.original);
        Ok(())
    }
    fn retention_and_exit(mut self) -> Result<()> {
        self.close();
        settle(&mut self.session)?;
        idle(&mut self.session);
        let pid = support::pid(&self.root)?;
        observe_idle(&pid)?;
        let pid = self.disable_and_reopen(&pid)?;
        assert_eq!(
            std::fs::read_to_string(self.root.join("workspace/main.rs"))?,
            self.original
        );
        let start = Instant::now();
        drop(self.session);
        support::reaped(&pid)?;
        assert!(start.elapsed() < Duration::from_secs(2));
        println!(
            "PASS controlled source journey: disabled/native, enable/semantic, selected resolve/import/undo, exclusion/restore, shared buffers/views, valid-empty fallback, EOF/native, explicit restart, retained idle <=1 CPU tick/350ms, disable and exit/reap"
        );
        Ok(())
    }
    fn disable_and_reopen(&mut self, pid: &str) -> Result<String> {
        self.session.execute(Command::Lsp(LspCommand::Mode));
        settle(&mut self.session)?;
        support::reaped(pid)?;
        self.session.execute(Command::Lsp(LspCommand::Mode));
        self.session.load(&self.root.join("workspace/main.rs"))?;
        synchronized(&mut self.session)?;
        support::pid(&self.root)
    }
    fn result(&self, value: &serde_json::Value) -> Result<()> {
        std::fs::write(self.root.join("peer.result"), value.to_string())?;
        Ok(())
    }
    fn close(&mut self) {
        self.session.execute(Command::Buffer(BufferCommand::Kill));
        key(&mut self.session, KeyCode::Enter);
    }
}

fn observe_idle(pid: &str) -> Result<()> {
    let ticks = (cpu_ticks("self")?, cpu_ticks(pid)?);
    std::thread::sleep(Duration::from_millis(350));
    assert!(cpu_ticks("self")?.saturating_sub(ticks.0) <= 1);
    assert!(cpu_ticks(pid)?.saturating_sub(ticks.1) <= 1);
    Ok(())
}

use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use rev::{
    buffer::Buffers,
    command::Command,
    session::Session,
    term::{SessionId, State},
    view::{Size, View},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Process,
    time::{Duration, Instant},
};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) const ORIGINAL: &str = "* Protected source\nordinary native text\n";

pub(crate) struct Editor {
    pub session: Session,
    pub directory: PathBuf,
}
impl Drop for Editor {
    fn drop(&mut self) {
        self.session
            .shutdown_terminals()
            .expect("owned terminal cleanup");
    }
}
impl Editor {
    pub fn open(root: &Path, profile: &str) -> Result<Self> {
        let directory = repository(root, profile)?;
        let config = settings(&directory, profile)?;
        let startup = rev::config::prepare(&config);
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut buffers = Buffers::with_settings(startup.snapshot.clone())?;
        let id = buffers.visit(&directory.join("source.org"))?;
        let view = View::with_settings(buffers.get(id)?, &startup.snapshot);
        let session = Session::new(
            buffers,
            view,
            Size {
                width: 100,
                height: 40,
            },
        )?;
        Ok(Self { session, directory })
    }
    pub fn selected(&self) -> SessionId {
        self.session
            .terminal_buffer()
            .expect("selected terminal")
            .terminal()
            .expect("terminal identity")
    }
    pub fn wait(&mut self, ready: impl Fn(&rev::term::Session) -> bool) -> Result<()> {
        let id = self.selected();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.session.expire_input(Instant::now());
            let session = self.session.terminals().get(id).ok_or("lost terminal")?;
            assert!(
                !matches!(session.state(), State::Failed(_)),
                "{:?}",
                session.state()
            );
            if ready(session) {
                return Ok(());
            }
            assert!(
                Instant::now() < deadline,
                "native terminal deadline: {}",
                screen(session)
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
    pub fn line(&mut self, line: &str) {
        for c in line.chars() {
            key(&mut self.session, KeyCode::Char(c), KeyModifiers::NONE);
        }
        key(&mut self.session, KeyCode::Enter, KeyModifiers::NONE);
    }
}
pub(crate) fn repository(root: &Path, profile: &str) -> Result<PathBuf> {
    let path = root.join("workspace").join(profile);
    fs::create_dir_all(&path)?;
    fs::write(path.join("source.org"), ORIGINAL)?;
    git(&path, &["init", "--quiet", "--initial-branch=main"])?;
    git(&path, &["add", "source.org"])?;
    git(
        &path,
        &[
            "-c",
            "user.name=Terminal Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "-m",
            "Fixture",
        ],
    )?;
    fs::write(path.join("changed.txt"), "untracked fixture\n")?;
    fs::write(
        path.join("lazygit.yml"),
        "disableStartupPopups: true\nnotARepository: skip\npromptToReturnFromSubprocess: false\ngui:\n  mouseEvents: true\n  showRandomTip: false\n  showBottomLine: false\nupdate:\n  method: never\ngit:\n  autoFetch: false\n  autoRefresh: false\n",
    )?;
    fs::write(
        path.join("bashrc"),
        "PS1='SHELL_READY> '\nHISTFILE=/dev/null\n",
    )?;
    Ok(path)
}
fn git(path: &Path, args: &[&str]) -> Result<()> {
    let output = Process::new("git").args(args).current_dir(path).output()?;
    assert!(
        output.status.success(),
        "git fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
pub(crate) fn settings(path: &Path, profile: &str) -> Result<PathBuf> {
    let config = path.join("config.toml");
    let bashrc = serde_json::to_string(&path.join("bashrc").to_string_lossy())?;
    fs::write(
        &config,
        format!(
            "editor.interaction_profile='{profile}'\ntree.visible=false\ncompletion.automatic=false\nterminal.program='/bin/bash'\nterminal.args=['--noprofile','--rcfile',{bashrc},'-i']\nterminal.panel_height=15\nterminal.scrollback_lines=64\n"
        ),
    )?;
    Ok(config)
}
pub(crate) fn key(session: &mut Session, code: KeyCode, modifiers: KeyModifiers) {
    session.handle(Event::Key(KeyEvent::new(code, modifiers)));
}
pub(crate) fn named(session: &mut Session, profile: &str, name: &str) {
    // Colon is host entry only in a compatible editor/copy context; M-x is
    // the protected route while the live child owns printable input.
    if profile == "modal" && !session.terminal_focused() {
        key(session, KeyCode::Char(':'), KeyModifiers::NONE);
    } else {
        key(session, KeyCode::Char('x'), KeyModifiers::ALT);
    }
    session.handle(Event::Paste(name.into()));
    key(session, KeyCode::Enter, KeyModifiers::NONE);
}
pub(crate) fn screen(session: &rev::term::Session) -> String {
    let grid = session.screen().grid();
    (0..grid.screen_lines())
        .map(|row| {
            (0..grid.columns())
                .map(|column| grid[Point::new(Line(row as i32), Column(column))].c)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
pub(crate) fn source(session: &Session) -> String {
    let buffer = session.buffer().expect("source buffer");
    buffer
        .slice(&buffer.span(0..buffer.len_chars()).expect("source span"))
        .expect("source text")
        .to_string()
}
pub(crate) fn pid_file(path: &Path) -> Result<u32> {
    Ok(fs::read_to_string(path)?.trim().parse()?)
}

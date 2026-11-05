use crate::{repository::Repository, support::*};
use crossterm::event::{KeyCode, KeyModifiers};
use rev::{
    command::Command,
    session::{Action, Session},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

// A real delayed Git process and its descendant remain owned on q/declined
// exit. Only explicit termination consent triggers the existing bounded reaper.
pub(crate) fn run() -> Result<()> {
    let descriptors = fs::read_dir("/proc/self/fd")?.count();
    let threads = crate::metrics::threads()?;
    for profile in ["non-modal", "modal"] {
        journey(profile)?;
    }
    assert_eq!(
        fs::read_dir("/proc/self/fd")?.count(),
        descriptors,
        "guarded exit leaked a descriptor"
    );
    assert_eq!(
        crate::metrics::threads()?,
        threads,
        "guarded exit leaked a worker"
    );
    println!("PASS Git exit cleanup: descriptors={descriptors} threads={threads}");
    Ok(())
}

fn journey(profile: &str) -> Result<()> {
    let repository = Repository::new("sha1");
    fs::write(repository.root.join("source.txt"), "disk\n")?;
    repository.commit();
    let head = repository.git(["rev-parse", "HEAD"]);
    let mut owned = editor(&repository, profile)?;
    let editor = &mut owned.0;
    named(editor, "git-status");
    wait(editor, |s| s.git_observation().is_some());
    let (leader, descendant) = start_owned(editor, &repository)?;
    retained(editor, leader, descendant)?;
    let elapsed = terminate(editor, leader, descendant)?;
    assert_eq!(repository.git(["rev-parse", "HEAD"]), head);
    assert_eq!(fs::read(repository.root.join("source.txt"))?, b"disk\n");
    println!(
        "PASS owned Git lifetime profile={profile}: q/decline retain leader+descendant; accepted exit reaps in {}us without repository rollback",
        elapsed
    );
    Ok(())
}

fn hold(repository: &Repository) -> Result<()> {
    fs::write(
        &repository.program,
        "#!/bin/sh\nexport GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null\nif [ \"$2\" = status ]; then\n trap '' TERM\n echo $$ > \"$0.leader\"\n sleep 60 &\n echo $! > \"$0.descendant\"\n wait\nfi\nexec /usr/bin/git \"$@\"\n",
    )?;
    Ok(())
}

fn pid(path: &Path) -> Result<u32> {
    Ok(fs::read_to_string(path)?.trim().parse()?)
}

fn live(pid: u32) -> Result<bool> {
    match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => Ok(!stat
            .rsplit_once(')')
            .ok_or("invalid process stat")?
            .1
            .trim_start()
            .starts_with('Z')),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

fn decline(editor: &mut Session) {
    editor.execute(Command::Quit);
    assert!(editor.echo().contains("terminate owned Git operations"));
    key(editor, KeyCode::Enter, KeyModifiers::NONE);
    plain(editor, 'n');
    assert_ne!(editor.expire_input(Instant::now()), Action::Exit);
}

fn exiting(editor: &mut Session) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while editor.expire_input(Instant::now()) != Action::Exit {
        assert!(
            Instant::now() < deadline,
            "guarded exit failed: {}",
            editor.echo()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn start_owned(editor: &mut Session, repository: &Repository) -> Result<(u32, u32)> {
    hold(repository)?;
    named(editor, "git-refresh");
    wait(editor, |_| {
        repository.program.with_extension("descendant").exists()
    });
    let leader = pid(&repository.program.with_extension("leader"))?;
    let descendant = pid(&repository.program.with_extension("descendant"))?;
    Ok((leader, descendant))
}

fn retained(editor: &mut Session, leader: u32, descendant: u32) -> Result<()> {
    named(editor, "git-quit");
    assert!(!editor.git_visible());
    assert!(live(leader)? && live(descendant)?);
    decline(editor);
    assert!(live(leader)? && live(descendant)?);
    Ok(())
}

fn terminate(editor: &mut Session, leader: u32, descendant: u32) -> Result<u128> {
    let started = Instant::now();
    editor.execute(Command::Quit);
    plain(editor, 'y');
    exiting(editor);
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(!live(leader)? && !live(descendant)?);
    Ok(started.elapsed().as_micros())
}

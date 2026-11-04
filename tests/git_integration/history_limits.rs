use super::*;
use crate::status::presentation::{entry, plain, render, wait};
use rev::{
    buffer::Buffers,
    command::{Command, GitCommand},
    session::Session,
    view::{Size, View},
};
use std::{
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command as Process, Stdio},
};

// Small configured bounds admit only complete reflog/blame records and an
// explicit historical byte prefix. Full object, index and disk facts persist.
#[test]
fn bounded_inspection() {
    let r = Repository::new("sha256");
    let source = (0..20)
        .map(|i| format!("line {i}: {}\n", "x".repeat(200)))
        .collect::<String>();
    fs::write(r.root.join("lines"), &source).unwrap();
    r.commit();
    let subject = "subject".repeat(300);
    r.git(["commit", "--allow-empty", "--message", &subject]);
    let large = vec![b'x'; 10000];
    fs::write(r.root.join("large"), &large).unwrap();
    r.git(["add", "--all"]);
    r.git(["commit", "--message", &subject]);
    let head = object(&r, "HEAD");
    let index = r.git(["ls-files", "--stage", "-z"]);
    let mut owner=r.owner("history_page_size=1\nmax_status_paths=2\nmax_output_bytes=65536\nmax_detail_bytes=4096\nmax_diagnostic_bytes=1024\nmax_process_record_bytes=131072\n");
    r.open(&mut owner, &r.root);
    let id = owner.contexts()[0].id;
    owner.reflog(id).unwrap();
    assert!(finish(&mut owner).published);
    let reflog = owner.context(id).unwrap().reflog.as_ref().unwrap();
    assert_eq!(reflog.entries.len(), 1);
    assert!(matches!(
        reflog.completeness,
        Completeness::Truncated { .. }
    ));
    owner.blame(id, std::path::Path::new("lines")).unwrap();
    let u = finish(&mut owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    let b = owner.context(id).unwrap().blame.as_ref().unwrap();
    assert!(!b.lines.is_empty() && b.lines.len() < 20);
    for (i, line) in b.lines.iter().enumerate() {
        assert_eq!(line.final_line, i + 1);
        assert_eq!(line.text, source.lines().nth(i).unwrap().as_bytes());
    }
    assert!(matches!(b.completeness, Completeness::Truncated { .. }));
    let h = history(&mut owner);
    let revision = h.revisions().next().unwrap();
    owner
        .show_revision(
            h.clone(),
            revision.object,
            revision.parents.first().copied(),
        )
        .unwrap();
    let u = finish(&mut owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    let detail = owner
        .context(id)
        .unwrap()
        .revision
        .as_ref()
        .unwrap()
        .clone();
    owner
        .revision_file(detail, std::path::Path::new("large"))
        .unwrap();
    let u = finish(&mut owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    let file = owner.context(id).unwrap().revision_file.as_ref().unwrap();
    assert_eq!(file.bytes, &large[..4096]);
    assert!(matches!(file.completeness, Completeness::Truncated { .. }));
    assert!(owner.admitted_bytes() <= rev::git::SESSION_BYTES);
    assert_eq!(object(&r, "HEAD"), head);
    assert_eq!(r.git(["ls-files", "--stage", "-z"]), index);
    assert_eq!(fs::read(r.root.join("large")).unwrap(), large);
}

// Deletions read the old-side blob; root and binary content remain typed and
// read-only. Configured diff/text-conversion helpers never execute on these reads.
#[test]
fn deleted_binary_and_helpers() {
    let r = Repository::new("sha1");
    let helper = r.fixture.write(
        "helper",
        "#!/bin/sh\nprintf invoked > helper-ran\nexit 99\n",
    );
    fs::set_permissions(&helper, fs::Permissions::from_mode(0o700)).unwrap();
    r.git(["config", "diff.external", helper.to_str().unwrap()]);
    r.git(["config", "diff.fixture.textconv", helper.to_str().unwrap()]);
    fs::write(r.root.join(".gitattributes"), "*.txt diff=fixture\n").unwrap();
    fs::write(r.root.join("old.txt"), b"old bytes\n").unwrap();
    fs::write(r.root.join("stable.txt"), b"stable\n").unwrap();
    fs::write(r.root.join("binary.bin"), b"\0\xffbinary\n").unwrap();
    r.commit();
    let initial = object(&r, "HEAD");
    r.git(["rm", "old.txt"]);
    r.commit();
    let head = object(&r, "HEAD");
    let index = r.git(["ls-files", "--stage", "-z"]);
    let mut owner = r.owner("");
    r.open(&mut owner, &r.root);
    let id = owner.contexts()[0].id;
    let h = history(&mut owner);
    let current = h.revisions().next().unwrap();
    owner
        .show_revision(h.clone(), current.object, current.parents.first().copied())
        .unwrap();
    assert!(finish(&mut owner).published);
    let d = owner
        .context(id)
        .unwrap()
        .revision
        .as_ref()
        .unwrap()
        .clone();
    owner
        .revision_file(d, std::path::Path::new("old.txt"))
        .unwrap();
    assert!(finish(&mut owner).published);
    let file = owner.context(id).unwrap().revision_file.as_ref().unwrap();
    assert_eq!(file.object.hex(), initial);
    assert_eq!(file.bytes, b"old bytes\n");
    let root = h.revisions().find(|r| r.object.hex() == initial).unwrap();
    owner.show_revision(h.clone(), root.object, None).unwrap();
    assert!(finish(&mut owner).published);
    let d = owner
        .context(id)
        .unwrap()
        .revision
        .as_ref()
        .unwrap()
        .clone();
    owner
        .revision_file(d, std::path::Path::new("binary.bin"))
        .unwrap();
    assert!(finish(&mut owner).published);
    let file = owner.context(id).unwrap().revision_file.as_ref().unwrap();
    assert!(file.binary);
    assert_eq!(file.bytes, b"\0\xffbinary\n");
    owner.blame(id, std::path::Path::new("stable.txt")).unwrap();
    let u = finish(&mut owner);
    assert!(u.published, "{:?}", owner.record(u.operation));
    assert!(!r.root.join("helper-ran").exists());
    assert!(!r.root.join("old.txt").exists());
    assert_eq!(object(&r, "HEAD"), head);
    assert_eq!(r.git(["ls-files", "--stage", "-z"]), index);
}

// Oversized real Git author metadata fails admission without replacing retained
// history. Equal numeric identities cannot revive a previous Status generation.
#[test]
fn metadata_admission() {
    let r = Repository::new("sha1");
    r.commit();
    let mut owner = r.owner("");
    r.open(&mut owner, &r.root);
    let h = history(&mut owner);
    r.git(["config", "user.name", &"a".repeat(4097)]);
    r.commit();
    let head = object(&r, "HEAD");
    owner.refresh(h.observation.context).unwrap();
    assert!(finish(&mut owner).published);
    owner.log(h.observation.context, None).unwrap();
    let u = finish(&mut owner);
    assert!(!u.published);
    assert!(
        owner
            .record(u.operation)
            .unwrap()
            .diagnostic
            .as_deref()
            .is_some_and(|s| s.contains("metadata exceeds"))
    );
    assert!(Arc::ptr_eq(
        owner
            .context(h.observation.context)
            .unwrap()
            .history
            .as_ref()
            .unwrap(),
        &h
    ));
    let revision = h.revisions().next().unwrap();
    assert!(
        owner
            .show_revision(h.clone(), revision.object, None)
            .is_err()
    );
    assert_eq!(object(&r, "HEAD"), head);
}

// Fast-import prepares 257 real commits in one owned fixture process. Native
// commands then exercise the actual 256-target ceiling without hundreds of
// fixture setup subprocesses or injected application state.
#[test]
fn ordered_mark_capacity() {
    use std::fmt::Write as _;
    let r = Repository::new("sha1");
    let mut import = "blob\nmark :1\ndata 5\nbase\n".to_owned();
    for n in 0..257 {
        let message = format!("Revision {n}\n");
        write!(import,"commit refs/heads/main\nmark :{}\ncommitter Example Author <author@example.invalid> {} +0000\ndata {}\n{}",n+2,n+1,message.len(),message).unwrap();
        if n > 0 {
            writeln!(import, "from :{}", n + 1).unwrap();
        }
        import.push_str("M 100644 :1 source.txt\n\n");
    }
    import.push_str("done\n");
    let mut child = Process::new(&r.program)
        .args(["fast-import", "--quiet"])
        .current_dir(&r.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(import.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    r.git(["reset", "--hard", "HEAD"]);
    let snapshot = r.snapshot("history_page_size=512\n");
    let mut buffers = Buffers::with_settings(snapshot.clone()).unwrap();
    let id = buffers.visit(&r.root.join("source.txt")).unwrap();
    let view = View::with_settings(buffers.get(id).unwrap(), &snapshot);
    let mut editor = crate::status::presentation::Editor(
        Session::new(
            buffers,
            view,
            Size {
                width: 80,
                height: 24,
            },
        )
        .unwrap(),
    );
    let s = &mut editor.0;
    entry(s);
    wait(s, |s| s.git_observation().is_some());
    plain(s, 'h');
    wait(s, |s| s.echo().contains("Git history inspected"));
    plain(s, '*');
    for n in 0..256 {
        plain(s, 'a');
        if n < 255 {
            plain(s, 'n');
        }
    }
    plain(s, 'a');
    assert!(s.echo().contains("already"));
    plain(s, 'n');
    plain(s, 'a');
    assert!(s.echo().contains("limit"));
    plain(s, '*');
    let base = object(&r, "HEAD~256");
    let frame = render(s).join("\n");
    assert!(frame.contains("256 ordered"), "{frame}");
    assert!(frame.contains(&format!("Base: {base}")), "{frame}");
    plain(s, 'p');
    plain(s, 'U');
    assert!(render(s).join("\n").contains("255 ordered"));
    s.execute(Command::Git(GitCommand::UnmarkAll));
    assert!(render(s).join("\n").contains("0 ordered"));
    assert_eq!(fs::read(r.root.join("source.txt")).unwrap(), b"base\n");
}

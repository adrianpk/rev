use super::links::editor;
use super::*;
use rev::org::attachment::Effect;
use std::os::unix::fs::PermissionsExt;
fn source() -> &'static str {
    "* Task\n:PROPERTIES:\n:DIR: assets\n:END:\nbody\n"
}
// Symlink inputs/parents/targets, unreadable sources and read-only replacements
// reject with exact original source/history and no unowned file replacement.
#[test]
fn attachments_path_rejections() {
    for scenario in [
        "input symlink",
        "parent symlink",
        "target symlink",
        "unreadable",
        "read-only target",
    ] {
        let (f, mut s) = editor("", source(), "");
        let input = f.write("input.txt", "input");
        let target = f.write("assets/input.txt", "target");
        match scenario {
            "input symlink" => {
                std::fs::remove_file(&input).unwrap();
                std::os::unix::fs::symlink(&target, &input).unwrap();
            }
            "parent symlink" => {
                std::fs::rename(f.0.join("assets"), f.0.join("outside")).unwrap();
                std::os::unix::fs::symlink(f.0.join("outside"), f.0.join("assets")).unwrap();
            }
            "target symlink" => {
                std::fs::remove_file(&target).unwrap();
                std::os::unix::fs::symlink(&input, &target).unwrap();
            }
            "unreadable" => {
                std::fs::set_permissions(&input, std::fs::Permissions::from_mode(0o000)).unwrap()
            }
            _ => std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o444)).unwrap(),
        }
        let target_bytes = std::fs::read(&target).unwrap();
        let view = s.view().clone();
        action(&mut s, C::AttachCopy);
        argument(&mut s, "input.txt");
        if s.echo().contains("replace existing file") {
            argument(&mut s, "yes");
        }
        assert!(
            !s.echo().contains("heading Changed"),
            "{scenario}: {}",
            s.echo()
        );
        assert_eq!(std::fs::read(&target).unwrap(), target_bytes, "{scenario}");
        assert_eq!(text(&s), source());
        assert_eq!(s.view(), &view);
        assert!(s.buffer().unwrap().history().is_empty());
        if let Some(result) = s.attachment_results().front() {
            assert_eq!(result.file.effect, Effect::Rejected, "{scenario}");
        }
        action(&mut s, C::Attach);
        cancel(&mut s);
        settle(&mut s);
        assert!(s.remaining_input(Instant::now()).is_none());
    }
}
// Opening requires an exact listed name. Unsafe ID components and finite file
// limits reject without creating attachment directories or modifying text.
#[test]
fn attachments_admission() {
    let (f, mut s) = editor("", source(), "");
    f.write("assets/note.txt", "owned");
    action(&mut s, C::AttachOpenNative);
    argument(&mut s, "../destination.org");
    assert!(s.echo().contains("exact attachment"), "{}", s.echo());
    assert_eq!(text(&s), source());
    let (f, mut s) = editor("", "* Task\n:PROPERTIES:\n:ID: ../outside\n:END:\n", "");
    f.write("input.txt", "owned");
    action(&mut s, C::AttachCopy);
    argument(&mut s, "input.txt");
    assert!(s.echo().contains("safe directory"), "{}", s.echo());
    assert!(!f.0.join("data").exists());
    let (f, mut s) = editor("org.limits.agenda_file_bytes=2", source(), "");
    f.write("input.txt", "long");
    action(&mut s, C::AttachCopy);
    argument(&mut s, "input.txt");
    assert!(s.echo().contains("byte limit"), "{}", s.echo());
    assert!(!f.0.join("assets").exists());
    assert_eq!(text(&s), source());
}
// A canonical read-only heading can list attachments, but failed mutation
// admission must release the dispatcher without acquiring file edit rights.
#[test]
fn attachments_readonly() {
    let f = Fixture::new();
    f.write("assets/note.txt", "owned");
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.visit(&f.write("source.org", source())).unwrap();
    buffers.get_mut(id).unwrap().set_readonly(true);
    let view = View::new(buffers.get(id).unwrap());
    let mut s = Session::new(
        buffers,
        view,
        Size {
            width: 100,
            height: 14,
        },
    )
    .unwrap();
    action(&mut s, C::AttachCopy);
    assert!(s.echo().contains("read-only"), "{}", s.echo());
    action(&mut s, C::AttachList);
    assert!(s.echo().contains("1 attachments"), "{}", s.echo());
    assert_eq!(text(&s), source());
    assert!(s.buffer().unwrap().history().is_empty());
    assert_eq!(
        std::fs::read_to_string(f.0.join("assets/note.txt")).unwrap(),
        "owned"
    );
}
// Source visiting/indexing and moving point leave actual resource access times
// untouched. Explicit listing then reads that owned directory, proving the
// fixture observes resource reads rather than merely checking source bytes.
#[test]
fn attachments_visit_inert() {
    use std::fs::{File, FileTimes};
    let f = Fixture::new();
    let file = f.write("assets/note.txt", "owned resource");
    let directory = f.0.join("assets");
    let old = UNIX_EPOCH + Duration::from_secs(1000000000);
    let times = FileTimes::new()
        .set_accessed(old)
        .set_modified(old + Duration::from_secs(60));
    File::open(&file).unwrap().set_times(times).unwrap();
    File::open(&directory).unwrap().set_times(times).unwrap();
    let path = f.write("source.org", source());
    let mut s = host(&f, Some(&path), false);
    settle(&mut s);
    super::links::line(&mut s, "3");
    settle(&mut s);
    assert_eq!(std::fs::metadata(&file).unwrap().accessed().unwrap(), old);
    assert_eq!(
        std::fs::metadata(&directory).unwrap().accessed().unwrap(),
        old
    );
    action(&mut s, C::AttachList);
    assert!(s.echo().contains("1 attachments"), "{}", s.echo());
    assert!(std::fs::metadata(&directory).unwrap().accessed().unwrap() > old);
    assert_eq!(std::fs::metadata(file).unwrap().accessed().unwrap(), old);
    assert!(s.remaining_input(Instant::now()).is_none());
}

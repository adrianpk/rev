use super::links::editor;
use super::*;
use rev::{command::History, org::attachment::Effect};
use std::os::unix::fs::MetadataExt;
// Copy/move/hard-link use real binary files, native ID directories and one
// ordinary metadata undo group. Source undo never claims to undo file effects.
#[test]
fn attachments_methods() {
    for (name, command, effect, original, newline) in [
        (
            "copy",
            C::AttachCopy,
            Effect::Published,
            "* Task\r\nbody EOF",
            "\r\n",
        ),
        (
            "move",
            C::AttachMove,
            Effect::Removed,
            "* Task\r\nbody EOF",
            "\r\n",
        ),
        (
            "hard link",
            C::AttachLink,
            Effect::Published,
            "* Task\r\nbody EOF",
            "\r\n",
        ),
        (
            "EOF heading",
            C::AttachCopy,
            Effect::Published,
            "* Task",
            "\n",
        ),
    ] {
        let (f, mut s) = editor("", original, "");
        let input = f.write("input one.bin", "");
        let bytes = [0, 255, 10, 42];
        std::fs::write(&input, bytes).unwrap();
        let inode = std::fs::metadata(&input).unwrap().ino();
        action(&mut s, command);
        assert!(!f.0.join("data").exists(), "{name}");
        argument(&mut s, "input one.bin");
        assert!(
            s.echo().contains("create new attachment"),
            "{name}: {}",
            s.echo()
        );
        assert!(!f.0.join("data").exists());
        argument(&mut s, "yes");
        assert!(s.echo().contains("heading Changed"), "{name}: {}", s.echo());
        let result = s.attachment_results().front().unwrap();
        assert_eq!(
            result.file.effect, effect,
            "{name}: {:?}",
            result.file.error
        );
        assert!(
            result.file.error.is_none(),
            "{name}: {:?}",
            result.file.error
        );
        let id = result.file.id.as_ref().unwrap();
        let target =
            f.0.join("data")
                .join(&id[..2])
                .join(&id[2..])
                .join("input one.bin");
        assert_eq!(std::fs::read(&target).unwrap(), bytes);
        assert!(
            text(&s).contains(&format!(":ID: {id}{newline}")),
            "{}",
            text(&s)
        );
        assert!(text(&s).contains(":ATTACH:"));
        assert_eq!(input.exists(), name != "move");
        assert_eq!(
            std::fs::metadata(&target).unwrap().ino() == inode,
            name == "hard link"
        );
        assert_eq!(s.buffer().unwrap().history().len(), 1);
        s.execute(Command::History(History::Undo));
        assert_eq!(text(&s), original);
        assert_eq!(std::fs::read(target).unwrap(), bytes);
        assert_eq!(
            std::fs::read_to_string(f.0.join("source.org")).unwrap(),
            original
        );
        assert!(s.remaining_input(Instant::now()).is_none());
    }
}
// Dispatcher operations use the effective DIR instead of the local ID. Lists
// and native opens inspect existing files; setting DIR never creates folders.
#[test]
fn attachments_dispatcher() {
    let source = "* Home\n:PROPERTIES:\n:ID: abcdef\n:DIR: assets\n:END:\nbody\n";
    let (f, mut s) = editor("", source, "");
    f.write("assets/note one.org", "* Attached 界\n");
    action(&mut s, C::Attach);
    argument(&mut s, "f");
    assert!(s.echo().contains("1 attachments"), "{}", s.echo());
    assert!(s.echo().contains("note one.org"));
    assert!(!f.0.join("data").exists());
    action(&mut s, C::AttachOpenNative);
    argument(&mut s, "1");
    assert_eq!(text(&s), "* Attached 界\n", "{}", s.echo());
    action(&mut s, C::LinkBack);
    assert_eq!(text(&s), source);
    action(&mut s, C::AttachDirectory);
    argument(&mut s, "future notes");
    assert!(text(&s).contains(f.0.join("future notes").to_str().unwrap()));
    assert!(!f.0.join("future notes").exists());
    s.execute(Command::History(History::Undo));
    assert_eq!(text(&s), source);
}
// Existing files are shown before the explicit replacement decision. Declining
// or cancelling changes neither participant and leaves the dispatcher reusable.
#[test]
fn attachments_collision() {
    let source = "* Home\n:PROPERTIES:\n:DIR: assets\n:END:\n";
    let (f, mut s) = editor("", source, "");
    f.write("report.txt", "new");
    let target = f.write("assets/report.txt", "old");
    for answer in ["no", "cancel"] {
        action(&mut s, C::AttachCopy);
        argument(&mut s, "report.txt");
        assert!(s.echo().contains("replace existing file"), "{}", s.echo());
        if answer == "no" {
            argument(&mut s, "no");
        } else {
            cancel(&mut s);
            settle(&mut s);
        }
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");
        assert_eq!(text(&s), source);
        assert!(s.buffer().unwrap().history().is_empty());
    }
    action(&mut s, C::AttachCopy);
    argument(&mut s, "report.txt");
    argument(&mut s, "yes");
    assert_eq!(
        std::fs::read_to_string(target).unwrap(),
        "new",
        "{}",
        s.echo()
    );
    assert!(text(&s).contains(":ATTACH:"));
}
// Opening media uses the shared explicit viewer confirmation and re-resolves
// the selected heading's attachment data without inventing an inline link.
#[test]
fn attachments_viewer() {
    let source = "* Home\n:PROPERTIES:\n:DIR: assets\n:END:\n";
    let (f, mut s) = editor("org.tools.viewer=['/bin/sh','viewer.sh']", source, "");
    let path = f.write("assets/picture one.png", "owned media");
    f.write("viewer.sh", "printf '%s' \"$1\" > called\n");
    action(&mut s, C::AttachOpen);
    argument(&mut s, "1");
    assert!(!f.0.join("called").exists());
    argument(&mut s, "yes");
    assert!(s.echo().contains("Viewer completed"), "{}", s.echo());
    assert_eq!(
        std::fs::read_to_string(f.0.join("called")).unwrap(),
        path.to_str().unwrap()
    );
    assert_eq!(text(&s), source);
    assert!(s.buffer().unwrap().history().is_empty());
}

use super::*;
use rev::file::Publication;
fn project(extra: &str) -> (Fixture, Session) {
    let configuration = format!(
        r#"
org.time.zone = "UTC"
{extra}
[[org.publish.projects]]
name = "docs"
roots = ["."]
include = ["source.org"]
output = "site"
formats = ["html", "markdown"]
assets = ["assets/**"]
"#
    );
    let (f, s) = super::links::editor(
        &configuration,
        "\u{feff}* Heading 界\r\nBody\r\n",
        "* Other\n",
    );
    (f, s.with_org_clock(clock))
}
fn publish(s: &mut Session) {
    action(s, C::Publish);
    assert!(s.echo().contains("Select"), "{}", s.echo());
    argument(s, "docs");
}
// Publishing observes canonical unsaved source and copies selected binary assets,
// while BOM/CRLF disk bytes, history and unowned output files remain untouched.
#[test]
fn project_native_outputs() {
    let (f, mut s) = project("");
    f.write("assets/picture.bin", "asset");
    std::fs::write(f.0.join("assets/picture.bin"), [0, 255, 13, 10]).unwrap();
    f.write("site/unowned", "keep");
    let disk = std::fs::read(f.0.join("source.org")).unwrap();
    s.handle(Event::Paste("unsaved ".into()));
    let before = text(&s);
    let stamp = s.buffer().unwrap().stamp();
    publish(&mut s);
    assert_eq!(
        s.publishing_results().len(),
        3,
        "{} {:?}",
        s.echo(),
        s.publishing_results()
    );
    assert!(
        s.publishing_results()
            .iter()
            .all(|r| r.publication == Publication::Complete),
        "{:?}",
        s.publishing_results()
    );
    let output = std::fs::read_to_string(f.0.join("site/source.md")).unwrap();
    assert!(output.contains("unsaved"));
    assert_eq!(
        std::fs::read(f.0.join("site/assets/picture.bin")).unwrap(),
        [0, 255, 13, 10]
    );
    assert_eq!(
        std::fs::read_to_string(f.0.join("site/unowned")).unwrap(),
        "keep"
    );
    assert_eq!(std::fs::read(f.0.join("source.org")).unwrap(), disk);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(text(&s), before);
    assert!(s.buffer().unwrap().is_modified());
    s.execute(Command::History(rev::command::History::Undo));
    assert!(!text(&s).contains("unsaved"));
}
// Replacement consent belongs to an observed file. Decline, cancellation,
// external modification and symlink swaps must retain the actual destination.
#[test]
fn project_replacement_evidence() {
    for mode in ["decline", "cancel", "changed", "symlink", "accept"] {
        let (f, mut s) = project("");
        let target = f.write("site/source.html", "previous");
        let other = f.write("unowned", "other");
        publish(&mut s);
        assert!(s.echo().contains("Select"), "{mode}: {}", s.echo());
        match mode {
            "decline" => argument(&mut s, "no"),
            "cancel" => {
                cancel(&mut s);
                settle(&mut s);
            }
            "changed" => {
                std::fs::write(&target, "changed").unwrap();
                argument(&mut s, "yes");
            }
            "symlink" => {
                std::fs::remove_file(&target).unwrap();
                std::os::unix::fs::symlink(&other, &target).unwrap();
                argument(&mut s, "yes");
            }
            "accept" => argument(&mut s, "yes"),
            _ => unreachable!(),
        }
        let actual = std::fs::read_to_string(&target).unwrap();
        let expected = match mode {
            "changed" => "changed",
            "symlink" => "other",
            _ => "previous",
        };
        if mode == "accept" {
            assert!(actual.contains("<h1"));
        } else {
            assert_eq!(actual, expected, "{mode}");
        }
        assert_eq!(std::fs::read_to_string(&other).unwrap(), "other");
        assert_eq!(
            s.publishing_results().len(),
            2,
            "{mode}: {:?}",
            s.publishing_results()
        );
        let failed = s
            .publishing_results()
            .iter()
            .filter(|r| r.publication != Publication::Complete)
            .count();
        assert_eq!(
            failed,
            if mode == "accept" {
                0
            } else if mode == "cancel" {
                2
            } else {
                1
            },
            "{mode}: {:?}",
            s.publishing_results()
        );
    }
}
// A selected missing source remains a visible failure beside successful files;
// duplicated outputs and aliases cannot overwrite any project input.
#[test]
fn project_partial_sources() {
    for mode in [
        "missing",
        "collision",
        "hardlink",
        "input",
        "source symlink",
    ] {
        let f = Fixture::new();
        let roots = if mode == "missing" {
            "['notes', 'missing.org']"
        } else if mode == "collision" {
            "['notes', 'other']"
        } else {
            "['notes']"
        };
        let format = if mode == "input" { "org" } else { "markdown" };
        let output = if mode == "input" { "notes" } else { "site" };
        let config = format!(
            "org.time.zone='UTC'\n[[org.publish.projects]]\nname='docs'\nroots={roots}\noutput='{output}'\nformats=['{format}']"
        );
        let startup = prepare(&f.write("config.toml", &config));
        assert!(startup.diagnostic.is_none());
        let source = f.write("notes/item.org", "* Selected\n");
        f.write("other/item.org", "* Other\n");
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers.visit(&source).unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 100,
                height: 14,
            },
        )
        .unwrap()
        .with_org_clock(clock);
        if mode == "hardlink" {
            std::fs::create_dir(f.0.join("site")).unwrap();
            std::fs::hard_link(&source, f.0.join("site/item.md")).unwrap();
        }
        if mode == "source symlink" {
            std::os::unix::fs::symlink(f.0.join("other/item.org"), f.0.join("notes/link.org"))
                .unwrap();
        }
        publish(&mut s);
        if mode == "hardlink" {
            argument(&mut s, "yes");
        }
        assert_eq!(
            std::fs::read_to_string(&source).unwrap(),
            "* Selected\n",
            "{mode}"
        );
        if mode == "input" {
            assert!(!f.0.join("site").exists());
            continue;
        }
        let complete = s
            .publishing_results()
            .iter()
            .filter(|r| r.publication == Publication::Complete)
            .count();
        assert_eq!(
            complete,
            usize::from(matches!(mode, "missing" | "source symlink")),
            "{mode}: {:?}",
            s.publishing_results()
        );
        assert!(
            s.publishing_results()
                .iter()
                .any(|r| r.publication == Publication::NotPublished),
            "{mode}: {:?}",
            s.publishing_results()
        );
    }
}
// Confirmed upload uses literal argv and the existing bounded process group;
// no tool launches before consent, and failures retain completed native files.
#[test]
fn project_upload_consent() {
    for mode in [
        "yes", "no", "cancel", "missing", "failure", "timeout", "output",
    ] {
        let f = Fixture::new();
        let script = match mode {
            "failure" => "exit 7",
            "timeout" => "printf started > started; sleep 5; printf escaped > escaped",
            "output" => "printf 123456789",
            _ => "printf uploaded > marker; printf '%s' \"$1\" > argument",
        };
        let program = f.write("tool.sh", &format!("#!/bin/sh\n{script}\n"));
        let executable = upload_executable(mode);
        let limit = if mode == "output" { 2 } else { 1024 };
        let config = format!(
            "org.time.zone='UTC'\norg.tools.upload=['{executable}', '{}', '{{output_dir}}']\norg.limits.job_seconds=1\norg.limits.job_output_bytes={limit}\n[[org.publish.projects]]\nname='docs'\nroots=['source.org']\noutput='site space;literal'\nformats=['markdown']\nupload=true",
            program.display()
        );
        let startup = prepare(&f.write("config.toml", &config));
        assert!(startup.diagnostic.is_none(), "{:?}", startup.diagnostic);
        let mut buffers = Buffers::with_settings(startup.snapshot).unwrap();
        let id = buffers
            .visit(&f.write("source.org", "* Heading\n"))
            .unwrap();
        let view = View::new(buffers.get(id).unwrap());
        let mut s = Session::new(
            buffers,
            view,
            Size {
                width: 100,
                height: 14,
            },
        )
        .unwrap()
        .with_org_clock(clock);
        publish(&mut s);
        let output = f.0.join("site space;literal");
        assert!(output.join("source.md").exists());
        assert!(!output.join("marker").exists());
        if mode == "cancel" {
            cancel(&mut s);
            settle(&mut s);
        } else {
            argument(&mut s, if mode == "no" { "no" } else { "yes" });
        }
        assert_eq!(s.publishing_results().len(), 1);
        assert_eq!(s.publishing_results()[0].publication, Publication::Complete);
        assert_eq!(
            output.join("marker").exists(),
            mode == "yes",
            "{mode}: {}",
            s.echo()
        );
        assert!(!output.join("escaped").exists(), "{mode}");
        if mode == "yes" {
            assert_eq!(
                std::fs::read_to_string(output.join("argument")).unwrap(),
                output.to_str().unwrap()
            );
        }
        if matches!(mode, "missing" | "failure" | "timeout" | "output") {
            assert!(
                !s.echo().contains("upload completed"),
                "{mode}: {}",
                s.echo()
            );
        }
    }
}

fn upload_executable(mode: &str) -> &'static str {
    if mode == "missing" {
        "absent-upload"
    } else {
        "/bin/sh"
    }
}

use super::*;
use std::os::unix::fs::PermissionsExt;
// Failed real-file observations/permissions reject without removing source or
// publishing a target buffer. External replacement bytes remain authoritative.
#[test]
fn refile_file_rejections() {
    for scenario in [
        "readonly",
        "source replacement",
        "target replacement",
        "stale target",
        "missing",
    ] {
        let (fixture, mut session) = refile::editor("* Note\n", "* Target\n");
        let source = fixture.0.join("source.org");
        let target = fixture.0.join("destination.org");
        if scenario == "readonly" {
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o444)).unwrap();
        }
        if scenario == "missing" {
            std::fs::remove_file(&target).unwrap();
        }
        action(&mut session, C::Refile);
        match scenario {
            "source replacement" => {
                std::fs::rename(fixture.write("replacement.org", "* Note\n"), &source).unwrap()
            }
            "target replacement" => {
                std::fs::rename(fixture.write("replacement.org", "* Target\n"), &target).unwrap()
            }
            "stale target" => std::fs::write(&target, "* Target\nexternal\n").unwrap(),
            _ => {}
        }
        let expected = std::fs::read(&target).ok();
        if scenario != "missing" {
            argument(&mut session, "1");
        }
        assert!(
            session.echo().contains("nothing moved"),
            "{scenario}: {}",
            session.echo()
        );
        assert_eq!(text(&session), "* Note\n");
        assert!(session.buffer().unwrap().history().is_empty());
        assert!(session.refile_results().is_empty());
        assert_eq!(std::fs::read(&target).ok(), expected, "{scenario}");
        assert_eq!(std::fs::read_to_string(&source).unwrap(), "* Note\n");
    }
}
// Duplicate titles remain separate exact locators. Partial text rejects; an
// explicit displayed number selects the second location without arbitrary match.
#[test]
fn refile_duplicate_targets() {
    let (fixture, mut session) = refile::editor("* Note\n", "* Target\n* Target\n");
    action(&mut session, C::Refile);
    assert!(session.echo().contains("2 targets"));
    argument(&mut session, "Target");
    assert!(session.echo().contains("complete refile target"));
    assert_eq!(text(&session), "* Note\n");
    cancel(&mut session);
    settle(&mut session);
    action(&mut session, C::Refile);
    argument(&mut session, "2");
    session.load(&fixture.0.join("destination.org")).unwrap();
    assert_eq!(text(&session), "* Target\n* Target\n** Note\n");
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("destination.org")).unwrap(),
        "* Target\n* Target\n"
    );
}

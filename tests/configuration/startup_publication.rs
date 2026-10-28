use super::*;

use rev::config::{default_template, prepare};

// Only native session startup provisions. The resulting complete file is loaded
// through ordinary validation; repeated startup leaves its entry and bytes alone.
#[test]
fn first_start() {
    if isolated() {
        let root = Roots::from_env().unwrap().config_file();
        assert!(!root.exists());
        assert_eq!(Startup::from_env(), Startup::default());
        assert!(!root.exists());
        let mut session = launch(None);
        assert_eq!(session.echo(), "C-x C-c: exit");
        assert_eq!(std::fs::read(&root).unwrap(), default_template().as_bytes());
        assert_eq!(
            session.buffer().unwrap().policy(),
            &prepare(&root).snapshot.settings().editor
        );
        session.handle(key(KeyCode::Tab));
        assert_eq!(text(&session), "    ");
        assert_eq!(launch(None).echo(), "C-x C-c: exit");
        return;
    }
    for fallback in [false, true] {
        let fixture = Fixture::new();
        fixture.write("project/kept.txt", "kept");
        let mut command = process("startup::publication::first_start", &fixture);
        if fallback {
            command.env_remove("XDG_CONFIG_HOME");
        }
        success(command.output().unwrap());
        let relative = if fallback {
            "home/.config/rev/config.toml"
        } else {
            "user/rev/config.toml"
        };
        let root = fixture.0.join(relative);
        assert_eq!(std::fs::read(&root).unwrap(), default_template().as_bytes());
        assert_eq!(
            std::fs::read_dir(root.parent().unwrap()).unwrap().count(),
            1
        );
        for name in ["data", "state", "cache"] {
            assert!(!fixture.0.join(name).exists(), "{name}");
        }
    }
}

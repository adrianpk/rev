use crate::command::OrgCommand;

use super::super::{Action, Session};

impl Session {
    pub(super) fn indentation_command(&mut self, command: OrgCommand) -> Option<Action> {
        use OrgCommand::{IndentDisable, IndentEnable, IndentMode};
        if !matches!(command, IndentMode | IndentEnable | IndentDisable) {
            return None;
        }
        let id = self.view().point.stamp.buffer;
        let buffer = match self.buffers.get_mut(id) {
            Ok(buffer) => buffer,
            Err(error) => return Some(self.error(&error.to_string())),
        };
        let enabled = match command {
            IndentMode => !buffer.org_indentation(),
            IndentEnable => true,
            IndentDisable => false,
            _ => unreachable!("checked indentation command"),
        };
        buffer.set_org_indentation(enabled);
        Some(self.message(if enabled {
            "Org indentation enabled"
        } else {
            "Org indentation disabled"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        buffer::Buffers,
        config::Snapshot,
        view::{Size, View},
    };

    mod fixture {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/configuration/fixture.rs"
        ));
    }

    // Actual readonly transient Org results use the shared handler in both
    // profiles. Reapplying startup policy cannot erase an explicit equal choice.
    #[test]
    fn readonly_results() {
        for profile in ["non-modal", "modal"] {
            let f = fixture::Fixture::new();
            let startup = crate::config::prepare(&f.write(
                "config.toml",
                &format!("editor.interaction_profile='{profile}'"),
            ))
            .snapshot;
            let mut buffers = Buffers::with_settings(startup.clone()).unwrap();
            let source = buffers.create("* Source\nbody\n").unwrap();
            buffers
                .get_mut(source)
                .unwrap()
                .select_mode(crate::buffer::Mode::Org)
                .unwrap();
            let id = buffers
                .org_result("*Result*", "** Heading\nbody\n")
                .unwrap();
            let view = View::with_settings(buffers.get(id).unwrap(), &startup);
            let mut session = Session::new(
                buffers,
                view,
                Size {
                    width: 40,
                    height: 12,
                },
            )
            .unwrap();
            let buffer = session.buffers.get_mut(id).unwrap();
            let scan = crate::org::ColorScan::new(buffer, crate::org::Limits::default()).unwrap();
            buffer
                .install_org_coloring(scan.run(&crate::org::Cancellation::default()))
                .unwrap();
            let stale = session
                .view()
                .wrapped_rows(session.buffer().unwrap(), 40, 5)
                .unwrap();
            let point = session.view().point;
            for (command, enabled) in [
                (OrgCommand::IndentDisable, false),
                (OrgCommand::IndentDisable, false),
                (OrgCommand::IndentMode, true),
                (OrgCommand::IndentEnable, true),
            ] {
                session.execute(crate::command::Command::Org(command));
                let buffer = session.buffer().unwrap();
                assert!(buffer.is_org() && buffer.is_transient() && buffer.is_readonly());
                assert_eq!(buffer.org_indentation(), enabled);
                if !enabled {
                    assert!(stale[1].hit(buffer, 40, 0).is_err());
                }
                assert_eq!(session.view().point, point);
                assert!(buffer.history().is_empty());
                assert!(!buffer.is_modified());
            }
            let incoming =
                crate::config::prepare(&f.write("other.toml", "org.indent=false")).snapshot;
            session
                .buffers
                .get_mut(id)
                .unwrap()
                .apply_settings(&incoming);
            assert!(session.buffer().unwrap().org_indentation());
            session.execute(crate::command::Command::Org(OrgCommand::IndentDisable));
            session
                .buffers
                .get_mut(id)
                .unwrap()
                .apply_settings(&Snapshot::default());
            assert!(!session.buffer().unwrap().org_indentation());
            assert!(session.buffers.get(source).unwrap().org_indentation());
            assert_eq!(session.buffers.get(source).unwrap().len_chars(), 14);
            assert_eq!(session.buffers.get(id).unwrap().len_chars(), 16);
            assert!(session.buffers.get(id).unwrap().history().is_empty());
        }
    }
}

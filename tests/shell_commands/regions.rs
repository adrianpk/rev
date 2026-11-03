use super::*;
use rev::command::{Motion, RectangleCommand, RegionCommand};

pub(super) fn whole(editor: &mut Session) {
    editor.execute(Command::Motion(Motion::BufferStart));
    editor.execute(Command::Region(RegionCommand::Set));
    editor.execute(Command::Motion(Motion::BufferEnd));
}
fn region_command(editor: &mut Session, command: &str) {
    editor.execute(Command::ShellCommandOnRegion);
    answer(editor, command);
}

// Region stdin is the unsaved canonical snapshot, including its literal newline
// and Unicode bytes. Shell-looking input remains data and is never interpolated.
#[test]
fn region_snapshot() {
    for (name, input, command, expected) in [
        ("sort", "c\nb\na\n", "LC_ALL=C sort", "a\nb\nc\n"),
        (
            "tr unsaved",
            "café\nz\n",
            "LC_ALL=C tr a-z A-Z",
            "CAFé\nZ\n",
        ),
        ("unicode crlf", "界é\r\né", "cat", "界é\r\né"),
        (
            "shell syntax",
            "$(touch marker)\n'quoted'",
            "cat",
            "$(touch marker)\n'quoted'",
        ),
    ] {
        let f = Fixture::new();
        let (mut editor, source) = editor(&f, "");
        editor
            .replace(0..editor.buffer().unwrap().len_chars(), input)
            .unwrap();
        whole(&mut editor);
        let history = editor.buffer().unwrap().history().len();
        region_command(&mut editor, command);
        assert_eq!(output(&mut editor), expected, "{name}");
        editor.load(&source).unwrap();
        assert_eq!(text(editor.buffer().unwrap()), input, "{name}");
        assert_eq!(editor.buffer().unwrap().history().len(), history);
        assert!(editor.buffer().unwrap().is_modified());
        assert!(!source.parent().unwrap().join("marker").exists());
        assert_ne!(fs::read_to_string(source).unwrap(), input);
    }
}

// Admission measures UTF-8 bytes before snapshot/staging, requires a nonempty
// linear selection and rejects rectangles instead of flattening their data.
#[test]
fn region_admission() {
    for (name, input, selection, accepted) in [
        ("exact byte bound", "ééé", "linear", true),
        ("excess byte bound", "éééa", "linear", false),
        ("absent selection", "text", "none", false),
        ("empty selection", "text", "empty", false),
        ("rectangle", "a\nb", "rectangle", false),
    ] {
        let f = Fixture::new();
        let (mut editor, _) = editor(&f, "max_input_bytes=6");
        editor
            .replace(0..editor.buffer().unwrap().len_chars(), input)
            .unwrap();
        editor.execute(Command::Motion(Motion::BufferStart));
        match selection {
            "linear" => whole(&mut editor),
            "empty" => {
                editor.execute(Command::Region(RegionCommand::Set));
            }
            "rectangle" => {
                editor.execute(Command::Rectangle(RectangleCommand::Mark));
                editor.execute(Command::Motion(Motion::BufferEnd));
            }
            _ => {}
        }
        editor.execute(Command::ShellCommandOnRegion);
        if accepted {
            answer(&mut editor, "cat");
            assert_eq!(output(&mut editor), input, "{name}");
        } else {
            assert!(
                editor.echo().contains(if selection == "linear" {
                    "max_input_bytes"
                } else {
                    "linear selection"
                }),
                "{name}: {}",
                editor.echo()
            );
            editor.execute(Command::ShellCommandCancel);
            assert!(editor.echo().contains("No shell command is active"));
            assert_eq!(text(editor.buffer().unwrap()), input);
        }
    }
}

// A real command emits more than pipe capacity to both streams before reading
// a large region. Anonymous finite stdin staging avoids write/read deadlock.
#[test]
fn duplex_input() {
    let f = Fixture::new();
    let (mut editor, _) = editor(&f, "");
    let input = "input\n".repeat(32768);
    editor
        .replace(0..editor.buffer().unwrap().len_chars(), &input)
        .unwrap();
    whole(&mut editor);
    region_command(
        &mut editor,
        "printf '%131072s' X; printf '%131072s' Y >&2; cat",
    );
    let result = output(&mut editor);
    assert_eq!(result, format!("{}X{input}", " ".repeat(131071)));
    switch(&mut editor, "*Shell Diagnostics*");
    let diagnostics = text(editor.buffer().unwrap());
    assert_eq!(
        diagnostics.split_once('\n').unwrap().1,
        format!("{}Y", " ".repeat(131071))
    );
}

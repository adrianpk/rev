use super::*;
use rev::buffer::edit::Point;
use rev::file::{Output, Publication};
struct Uncertain;
impl Output for Uncertain {
    fn sync_directory(&mut self, _: &File) -> io::Result<()> {
        Err(io::Error::other("injected directory sync failure"))
    }
}
// Real file replacement precedes canonical publication. A rejected write keeps
// source/history/points; unresolved completion retains actual publication truth.
#[test]
fn capture_saved_edit() {
    for state in ["complete", "readonly", "uncertain"] {
        let fixture = Fixture::new();
        let path = fixture.0.join("capture.org");
        fs::write(&path, "\u{feff}* Old\r\n").unwrap();
        let mut buffers = Buffers::new().unwrap();
        let id = buffers.visit(&path).unwrap();
        let buffer = buffers.get_mut(id).unwrap();
        let before = buffer.stamp();
        let mut points = vec![Point {
            stamp: before,
            position: Scalar(0),
        }];
        let edit = Replacement {
            source: buffer.span(buffer.len_chars()..buffer.len_chars()).unwrap(),
            text: "* New\r\n".into(),
        };
        let prepared = buffer.prepare(&[edit], &points, None).unwrap();
        if state == "readonly" {
            fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap()
        }
        let outcome = if state == "uncertain" {
            buffer.publish_saved_edit(prepared, &mut points, &mut Uncertain)
        } else {
            buffer.publish_saved_edit(prepared, &mut points, &mut file::Files)
        }
        .unwrap();
        if state == "readonly" {
            assert_eq!(outcome.publication(), Publication::NotPublished);
            assert_eq!(buffer.stamp(), before);
            assert!(buffer.history().is_empty());
            assert_eq!(fs::read(&path).unwrap(), "\u{feff}* Old\r\n".as_bytes());
        } else {
            assert_eq!(outcome.is_complete(), state == "complete");
            assert_eq!(buffer.history().len(), 1);
            assert_eq!(buffer.is_modified(), state == "uncertain");
            assert_eq!(
                fs::read(&path).unwrap(),
                "\u{feff}* Old\r\n* New\r\n".as_bytes()
            );
            buffer.navigate(History::Undo, &mut points, None).unwrap();
            assert_eq!(text(buffer), "* Old\r\n");
        }
    }
}

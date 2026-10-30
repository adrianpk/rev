use super::{execution::configured, *};
const PNG: &[u8] = include_bytes!("../fixtures/org-preview.png");
const SOURCE: &str =
    "\u{feff}* Images\r\n[[file:image.png]]\r\n[[https://example.invalid/image.png]]\r\nkeep EOF";
// Explicit preview jobs preserve BOM/CRLF, point, canonical revision and history.
// Rendering carries a separate image rectangle; clearing one split is view-local.
#[test]
fn preview_source() {
    let (f, mut s) = configured(SOURCE, "");
    std::fs::write(f.0.join("image.png"), PNG).unwrap();
    let before = text(&s);
    let point = s.view().point;
    let stamp = s.buffer().unwrap().stamp();
    assert!(s.frame().unwrap().previews().is_empty());
    settle(&mut s);
    action(&mut s, C::LinkPreviewRegion);
    let frame = s.frame().unwrap();
    assert_eq!(frame.previews().len(), 1, "{}", s.echo());
    let area = frame.previews()[0].area;
    assert!(area.column() >= 6 && area.width() <= 40 && area.row() + area.height() < 15);
    assert_eq!(s.view().point, point);
    assert_eq!(s.buffer().unwrap().stamp(), stamp);
    assert_eq!(s.buffer().unwrap().history().len(), 0);
    assert_eq!(text(&s), before);
    assert!(!s.buffer().unwrap().is_modified());
    s.execute(Command::Split(rev::window::Split::Right));
    settle(&mut s);
    assert_eq!(s.frame().unwrap().previews().len(), 2);
    action(&mut s, C::LinkPreviewClear);
    assert!(s.view().preview.images(stamp).is_empty());
    assert_eq!(s.frame().unwrap().previews().len(), 1);
    assert_eq!(
        std::fs::read(f.0.join("source.org")).unwrap(),
        SOURCE.as_bytes()
    );
}
// Read-only source can preview without gaining edit authority. Failed local
// resources and URI links retain ordinary text and an explicit viewer path.
#[test]
fn preview_fallback() {
    let (f, mut s) = super::execution::configured_readonly(SOURCE, "", true);
    std::fs::write(f.0.join("image.png"), b"bad PNG").unwrap();
    let before = text(&s);
    action(&mut s, C::LinkPreviewRegion);
    assert!(s.frame().unwrap().previews().is_empty());
    assert!(s.echo().contains("2 fallback"), "{}", s.echo());
    assert_eq!(text(&s), before);
    assert!(s.buffer().unwrap().is_readonly());
    assert_eq!(s.buffer().unwrap().history().len(), 0);
    assert!(!f.0.join("called").exists());
    action(&mut s, C::LinkPreviewClear);
    action(&mut s, C::LinkPreviewRefresh);
    assert!(s.echo().contains("disabled"));
}
// Source changes and C-g invalidate owned observations before any disposable
// result is attached. Stopped/cancelled previews leave no idle deadline.
#[test]
fn preview_invalidation() {
    for mode in ["edit", "cancel"] {
        let (f, mut s) = configured(SOURCE, "");
        std::fs::write(f.0.join("image.png"), PNG).unwrap();
        settle(&mut s);
        s.execute(Command::Org(C::LinkPreviewRegion));
        if mode == "edit" {
            s.handle(Event::Paste("changed".into()));
        } else {
            cancel(&mut s);
        }
        settle(&mut s);
        assert!(s.frame().unwrap().previews().is_empty(), "{mode}");
        assert_eq!(
            text(&s),
            if mode == "edit" {
                format!("changed{}", SOURCE.trim_start_matches('\u{feff}'))
            } else {
                SOURCE.trim_start_matches('\u{feff}').into()
            }
        );
        assert_eq!(s.remaining_input(Instant::now()), None);
        assert_eq!(
            std::fs::read(f.0.join("source.org")).unwrap(),
            SOURCE.as_bytes()
        );
    }
}

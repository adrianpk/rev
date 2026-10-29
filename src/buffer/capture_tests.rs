use super::*;
use std::path::Path;
// Transient publication rejects at backend admission, including explicit
// write-file requests. Selecting a major mode cannot turn this owner into Org.
#[test]
fn capture_draft_admission() {
    let mut buffers = Buffers::new().unwrap();
    let id = buffers.capture_draft("* Draft").unwrap();
    for target in [None, Some(Path::new("unwritten-capture.org"))] {
        let error = buffers.prepare_save(id, target).err().unwrap();
        assert!(error.to_string().contains("Transient capture draft"));
        let error = buffers.request_save(id, target).err().unwrap();
        assert!(error.to_string().contains("Transient capture draft"));
    }
    let draft = buffers.get_mut(id).unwrap();
    assert!(draft.select_mode(Mode::Org).is_err());
    assert_eq!(draft.mode(), "Capture");
    assert!(!draft.is_org());
    assert!(draft.file().is_none());
}

use super::*;
use crate::{
    fixture::Fixture,
    org::{
        Cancellation,
        preview::{Image, Png},
    },
};
const PNG: &[u8] = include_bytes!("../../../tests/fixtures/org-preview.png");
fn placement(f: &Fixture) -> Placement {
    let path = f.0.join("image.png");
    std::fs::write(&path, PNG).unwrap();
    Placement {
        image: Arc::new(Image {
            row: 0,
            png: Png::parse(PNG.to_vec(), &Cancellation::default()).unwrap(),
            source: crate::file::resolve(&path).unwrap(),
        }),
        area: Area::new(6, 1, 30, 10).unwrap(),
    }
}
// Initial transfer and geometry changes use separate owned operations. Stable
// frames do not retransmit; removal frees exactly the assigned image ID.
#[test]
fn graphics_lifetime() {
    let f = Fixture::new();
    let mut place = placement(&f);
    let mut images = Images::default();
    let mut bytes = Vec::new();
    images
        .present(&mut bytes, std::slice::from_ref(&place))
        .unwrap();
    let id = images.slots[0].id;
    let first = bytes.clone();
    assert!(bytes.windows(5).any(|b| b == b"a=t,t"));
    assert!(bytes.windows(10).any(|b| b == b"C=1,z=-1,q"));
    images
        .present(&mut bytes, std::slice::from_ref(&place))
        .unwrap();
    assert_eq!(bytes, first);
    place.area = Area::new(7, 2, 12, 4).unwrap();
    bytes.clear();
    images
        .present(&mut bytes, std::slice::from_ref(&place))
        .unwrap();
    let wire = String::from_utf8(bytes.clone()).unwrap();
    assert!(!wire.contains("a=t"));
    assert!(wire.contains(&format!("d=i,i={id}")));
    assert!(wire.contains("c=12,r=4"));
    images.clear(&mut bytes).unwrap();
    assert!(
        String::from_utf8(bytes.clone())
            .unwrap()
            .contains(&format!("d=I,i={id}"))
    );
    let cleared = bytes.clone();
    images.clear(&mut bytes).unwrap();
    assert_eq!(bytes, cleared);
}
#[derive(Default)]
struct Fault {
    bytes: Vec<u8>,
    budget: Option<usize>,
    flush: bool,
}
impl Write for Fault {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.budget == Some(0) {
            self.budget = None;
            return Err(io::Error::other("owned graphics write failure"));
        }
        let count = bytes.len().min(self.budget.unwrap_or(usize::MAX));
        if let Some(budget) = &mut self.budget {
            *budget -= count;
        }
        self.bytes.extend_from_slice(&bytes[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if std::mem::take(&mut self.flush) {
            Err(io::Error::other("owned graphics flush failure"))
        } else {
            Ok(())
        }
    }
}
// Partial transfer, failed flush and failed deletion retain the exact cleanup
// obligation. A retry closes incomplete APC state and clears the same ID once.
#[test]
fn graphics_failures() {
    for mode in ["upload", "flush", "delete"] {
        let f = Fixture::new();
        let place = placement(&f);
        let mut images = Images::default();
        let mut out = Fault {
            budget: (mode == "upload").then_some(8),
            flush: mode == "flush",
            ..Fault::default()
        };
        let result = images.present(&mut out, &[place]);
        if mode == "delete" {
            assert!(result.is_ok());
            out.budget = Some(8);
            assert!(images.clear(&mut out).is_err());
        } else {
            assert!(result.is_err(), "{mode}");
        }
        let id = images.slots[0].id;
        assert!(images.pending, "{mode}");
        images.clear(&mut out).unwrap();
        assert!(images.slots.is_empty());
        assert!(!images.pending);
        assert!(String::from_utf8_lossy(&out.bytes).contains(&format!("d=I,i={id}")));
        let before = out.bytes.clone();
        images.clear(&mut out).unwrap();
        assert_eq!(out.bytes, before);
    }
}
// Whole base64 quartets and the final padding stay within Kitty's 4096-byte
// payload ceiling; chunking never reintroduces arbitrary image bytes as ANSI.
#[test]
fn graphics_chunks() {
    assert_eq!(base64(b"f"), b"Zg==");
    assert_eq!(base64(b"fo"), b"Zm8=");
    assert_eq!(base64(b"foo"), b"Zm9v");
    let mut bytes = Vec::new();
    upload(&mut bytes, 7, &vec![255; 3073]).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains(&"/".repeat(4096)));
    assert!(text.ends_with("\x1b_Gm=0;/w==\x1b\\"));
    assert_eq!(text.matches("\x1b_G").count(), 2);
}

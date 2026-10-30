use super::*;
use crate::fixture;
const PNG: &[u8] = include_bytes!("../../../tests/fixtures/org-preview.png");
fn work(f: &fixture::Fixture, source: &str) -> Work {
    let p = f.write("config.toml", "");
    let settings = crate::config::prepare(&p).snapshot;
    let mut buffers = crate::buffer::Buffers::with_settings(settings).unwrap();
    let id = buffers.visit(&f.write("source.org", source)).unwrap();
    let buffer = buffers.get(id).unwrap();
    let point = crate::view::View::new(buffer).point;
    Work::new(
        Snapshot::new(buffer, point, &crate::config::Org::default()).unwrap(),
        true,
        false,
        PNG.len(),
    )
}
// Real PNG ingress stays outside canonical buffers, preserves source/disk and
// rechecks resource identity. URI and unsupported formats remain viewer paths.
#[test]
fn preview_resources() {
    let f = fixture::Fixture::new();
    let image = f.0.join("image.png");
    std::fs::write(&image, PNG).unwrap();
    let source =
        "[[file:image.png]]\r\n[[https://example.invalid/image.png]]\r\n[[file:other.svg]]\r\n";
    let out = work(&f, source).run(&Cancellation::default()).unwrap();
    assert_eq!(out.images.len(), 1);
    assert_eq!(out.fallback, 2);
    assert_eq!(out.images[0].png.width, 2);
    assert_eq!(out.images[0].png.bytes, PNG);
    assert!(out.current().is_ok());
    assert_eq!(
        std::fs::read_to_string(f.0.join("source.org")).unwrap(),
        source
    );
    std::fs::write(image, b"changed").unwrap();
    assert!(out.current().is_err());
}
// Byte ceilings and containment reject a resource instead of reading a prefix
// as an admitted image. Selection count, cancellation and file types are finite.
#[test]
fn preview_bounds() {
    let f = fixture::Fixture::new();
    std::fs::write(f.0.join("image.png"), PNG).unwrap();
    let mut selected = work(&f, "[[file:image.png]]");
    selected.limit = PNG.len() - 1;
    let out = selected.run(&Cancellation::default()).unwrap();
    assert!(out.images.is_empty());
    assert_eq!(out.fallback, 1);
    let selected = work(&f, &"[[file:image.png]]\n".repeat(33));
    assert!(
        selected
            .run(&Cancellation::default())
            .err()
            .unwrap()
            .contains("32 resources")
    );
    let cancel = Cancellation::default();
    cancel.cancel();
    assert!(work(&f, "[[file:image.png]]").run(&cancel).is_err());
    std::fs::remove_file(f.0.join("image.png")).unwrap();
    std::fs::create_dir(f.0.join("image.png")).unwrap();
    assert!(
        work(&f, "[[file:image.png]]")
            .run(&Cancellation::default())
            .unwrap()
            .images
            .is_empty()
    );
}
// The independent fixture has complete CRCs and pixel dimensions. Truncation,
// corruption, trailing bytes, animation and huge decoded extents are rejected.
#[test]
fn png_admission() {
    assert!(Png::parse(PNG.to_vec(), &Cancellation::default()).is_ok());
    for name in ["truncated", "crc", "extent", "animation", "trailing"] {
        let mut bytes = PNG.to_vec();
        match name {
            "truncated" => {
                bytes.pop();
            }
            "crc" => bytes[45] ^= 1,
            "extent" => {
                bytes[16..20].copy_from_slice(&8193u32.to_be_bytes());
                let crc = png::crc(&bytes[12..29], &Cancellation::default()).unwrap();
                bytes[29..33].copy_from_slice(&crc.to_be_bytes());
            }
            "animation" => {
                bytes[37..41].copy_from_slice(b"acTL");
                let end = 41 + u32::from_be_bytes(bytes[33..37].try_into().unwrap()) as usize;
                let crc = png::crc(&bytes[37..end], &Cancellation::default()).unwrap();
                bytes[end..end + 4].copy_from_slice(&crc.to_be_bytes());
            }
            "trailing" => bytes.push(0),
            _ => unreachable!(),
        }
        assert!(
            Png::parse(bytes, &Cancellation::default()).is_err(),
            "{name}"
        );
    }
}

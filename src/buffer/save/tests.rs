use super::*;
use crate::{buffer::edit::Replacement, fixture::Fixture};

// A prepared file cannot publish after source/association/read-only/disk
// changes. This exercises the gap between real staging and native admission.
#[test]
fn staged_rechecks() {
    for variant in [
        "revision",
        "readonly",
        "disk",
        "association",
        "appeared",
        "closed",
    ] {
        let fixture = Fixture::new();
        let path = fixture.0.join("target.txt");
        let mut buffers = Buffers::new().unwrap();
        let id = if variant == "appeared" {
            buffers.create("captured").unwrap()
        } else {
            std::fs::write(&path, "original").unwrap();
            buffers.visit(&path).unwrap()
        };
        let target = if variant == "appeared" {
            Some(path.as_path())
        } else {
            None
        };
        let save = buffers.prepare_save(id, target).unwrap();
        let staged = file::stage(&save, &mut file::Files).unwrap();
        match variant {
            "revision" => {
                let buffer = buffers.get_mut(id).unwrap();
                let edit = Replacement {
                    source: buffer.span(0..0).unwrap(),
                    text: "new ".into(),
                };
                let prepared = buffer.prepare(&[edit], &[], None).unwrap();
                buffer.publish(prepared, &mut []).unwrap();
            }
            "readonly" => buffers.get_mut(id).unwrap().set_readonly(true),
            "disk" | "appeared" => std::fs::write(&path, "external").unwrap(),
            "association" => {
                let alternate = fixture.0.join("alternate.txt");
                let other = buffers.prepare_save(id, Some(&alternate)).unwrap();
                assert!(buffers.publish_save(other).is_complete());
            }
            "closed" => {
                let kill = buffers.prepare_kill(id).unwrap();
                buffers.kill(kill).unwrap();
            }
            _ => unreachable!(),
        }
        let outcome = buffers.publish_staged_save(save, staged);
        assert_eq!(
            outcome.publication(),
            Publication::NotPublished,
            "{variant}"
        );
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            if matches!(variant, "disk" | "appeared") {
                "external"
            } else {
                "original"
            },
            "{variant}"
        );
        assert!(std::fs::read_dir(&fixture.0).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".rev-")
        }));
    }
}

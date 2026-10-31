use rev::tree::Tree;

use super::config_fixture::Fixture;

// A retained hit cannot substitute a sorted row, changed kind, collapsed slot
// or another root. Rejection leaves the current valid selection unchanged.
#[test]
fn retained_identity() {
    for change in ["shift", "type", "collapse", "root"] {
        let fixture = Fixture::new();
        let choice = fixture.write("root/choice", "choice");
        fixture.write("root/other", "other");
        let mut tree = Tree::new(fixture.0.join("root"));
        tree.expand().unwrap();
        let hit = tree.hit(1).unwrap();
        match change {
            "shift" => {
                fixture.write("root/aaa", "unrelated");
                tree.refresh().unwrap();
            }
            "type" => {
                std::fs::remove_file(&choice).unwrap();
                std::fs::create_dir(&choice).unwrap();
                tree.refresh().unwrap();
            }
            "collapse" => tree.collapse(),
            _ => {
                fixture.write("replacement/entry", "unrelated");
                tree = Tree::new(fixture.0.join("replacement"));
                tree.expand().unwrap();
            }
        }
        let before = tree.selected().unwrap().path().to_owned();
        assert!(!tree.select_hit(&hit), "{change}");
        assert_eq!(tree.selected().unwrap().path(), before);
        let fresh = tree.hit(0).unwrap();
        assert!(tree.select_hit(&fresh));
        assert_eq!(tree.selected().unwrap().path(), tree.root().unwrap());
        assert!(tree.hit(u16::MAX).is_none());
    }
}

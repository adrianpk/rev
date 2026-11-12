use crate::support::Result;
use rev::buffer::Buffers;
use rev::config::{Settings, prepare};
use rev::display::Frame;
use rev::panel::Composition;
use rev::tree::Tree;
use rev::view::{Size, View};
use rev::window::Windows;
use std::path::Path;
use unicode_width::UnicodeWidthStr;

// Admission is atomic across includes. Explicit false, root overrides and
// invalid source leaves must behave the same before any tree is constructed.
pub(super) fn run(root: &Path) -> Result<()> {
    let fixture = root.join("configuration");
    std::fs::create_dir_all(fixture.join("workspace"))?;
    let base = fixture.join("base.toml");
    let config = fixture.join("config.toml");
    for (name, inherited, explicit, expected, origin) in [
        ("omitted", "", "", true, None),
        (
            "included false",
            "tree.decorated=false",
            "",
            false,
            Some(base.as_path()),
        ),
        (
            "root false",
            "tree.decorated=true",
            "tree.decorated=false",
            false,
            Some(config.as_path()),
        ),
        (
            "root true",
            "tree.decorated=false",
            "tree.decorated=true",
            true,
            Some(config.as_path()),
        ),
    ] {
        std::fs::write(&base, inherited)?;
        let text = format!("include=['base.toml']\n{explicit}");
        std::fs::write(&config, &text)?;
        let startup = prepare(&config);
        assert!(
            startup.diagnostic.is_none(),
            "{name}: {:?}",
            startup.diagnostic
        );
        let settings = startup.snapshot.settings();
        assert_eq!(settings.tree.decorated, expected, "{name}");
        assert!(!settings.tree.visible && !settings.gutter.nerd_fonts);
        assert_eq!(startup.snapshot.origin(&["tree", "decorated"]), origin);
        assert_eq!(std::fs::read_to_string(&config)?, text);
    }
    rejected_settings(&base, &config)?;
    geometry(&fixture)?;
    println!(
        "PASS configuration: omission/include/root precedence and origins, invalid booleans/RGB/roles reject whole candidate; four settings at 1/5/8/9/13/24 cells and hidden/zero/restored surfaces"
    );
    Ok(())
}

fn rejected_settings(base: &Path, config: &Path) -> Result<()> {
    for invalid in [
        "tree.decorated='false'",
        "gutter.nerd_fonts=1",
        "palette.tree_entries.source='#xyzxyz'",
        "palette.tree_entries.unknown='#010203'",
    ] {
        std::fs::write(base, invalid)?;
        std::fs::write(
            config,
            "include=['base.toml']\ntree.visible=true\ntree.decorated=true\n",
        )?;
        let startup = prepare(config);
        assert!(startup.diagnostic.is_some(), "accepted {invalid}");
        assert_eq!(startup.snapshot.settings(), &Settings::default());
    }
    Ok(())
}

fn geometry(root: &Path) -> Result<()> {
    std::fs::write(root.join("workspace/界é.txt"), "retained")?;
    for decorated in [false, true] {
        for nerd_fonts in [false, true] {
            for width in [1, 5, 8, 9, 13, 24] {
                configured_geometry(root, decorated, nerd_fonts, width)?;
            }
        }
    }
    Ok(())
}

fn configured_geometry(root: &Path, decorated: bool, nerd_fonts: bool, width: i32) -> Result<()> {
    let config = root.join("geometry.toml");
    std::fs::write(
        &config,
        format!(
            "tree.visible=true\ntree.auto_width=false\ntree.width={width}\ntree.decorated={decorated}\ngutter.nerd_fonts={nerd_fonts}"
        ),
    )?;
    let startup = prepare(&config);
    assert!(startup.diagnostic.is_none());
    let buffers = Buffers::with_settings(startup.snapshot.clone())?;
    let view = View::with_settings(buffers.get(buffers.initial())?, &startup.snapshot);
    let windows = Windows::with_settings(view, &startup.snapshot)?;
    let tree = Tree::with_settings(root.join("workspace"), &startup.snapshot);
    let mut surface = Surface {
        buffers,
        windows,
        tree,
    };
    surface.tree.expand()?;
    let unicode_row = surface.unicode_row()?;
    let identity = surface.tree.hit(unicode_row);
    for columns in [100, 31, 0, 100] {
        if !surface.check(columns, width, decorated, unicode_row)? {
            continue;
        }
        assert_eq!(surface.tree.hit(unicode_row), identity);
    }
    Ok(())
}
struct Surface {
    buffers: Buffers,
    windows: Windows,
    tree: Tree,
}
impl Surface {
    fn unicode_row(&self) -> Result<u16> {
        Ok(u16::try_from(
            self.tree
                .entries()
                .iter()
                .position(|entry| entry.path().ends_with("界é.txt"))
                .ok_or("missing Unicode entry")?,
        )?)
    }

    fn check(&self, columns: u16, width: i32, decorated: bool, unicode_row: u16) -> Result<bool> {
        let buffers = &self.buffers;
        let windows = &self.windows;
        let tree = &self.tree;
        let size = Size {
            width: columns,
            height: if columns == 0 { 0 } else { 8 },
        };
        let Some(composition) = Composition::workspace(windows, tree, size, 0) else {
            assert_eq!(columns, 0);
            return Ok(false);
        };
        let frame = Frame::compose_workspace(buffers, windows, tree, &composition, "")?;
        assert!(
            frame
                .rows()
                .iter()
                .all(|row| row.text().width() == usize::from(columns))
        );
        assert_eq!(composition.tree().is_some(), columns >= 32);
        if columns >= 32 {
            let label = frame.rows()[usize::from(unicode_row)]
                .text()
                .split('│')
                .next()
                .ok_or("missing tree divider")?;
            assert_eq!(label.width(), usize::try_from(width)?);
            if width == 9 {
                assert!(label.ends_with(if decorated { "界é" } else { "界é.t" }));
            }
        }
        Ok(true)
    }
}

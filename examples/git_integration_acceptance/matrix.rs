use crate::{repository::Repository, support::Result};
use rev::{
    command::Binding,
    input::keymap::{Context, Sequence},
};

// Frozen expectations cover the reviewed direct keys and complete family
// sequences. Semantic effects are established by journeys and family fixtures;
// catalog availability alone is deliberately not treated as execution evidence.
pub(crate) fn run() -> Result<()> {
    let repository = Repository::new("sha1");
    for profile in ["non-modal", "modal"] {
        let snapshot = repository.snapshot(&format!("[editor]\ninteraction_profile='{profile}'\n"));
        let mut rows = 0;
        for line in include_str!("bindings.tsv")
            .lines()
            .filter(|l| !l.starts_with('#'))
        {
            let parts: Vec<_> = line.splitn(3, '\t').collect();
            let context = Context::ALL
                .into_iter()
                .find(|c| c.name() == parts[0])
                .ok_or("unknown matrix context")?;
            let sequence = Sequence::parse(parts[1])?;
            let expected = Binding::named(parts[2]).ok_or("unknown matrix command")?;
            assert_eq!(
                snapshot.keymaps().map(context).lookup(&sequence),
                Some(expected),
                "{profile}: {line}"
            );
            rows += 1;
        }
        assert_eq!(rows, 289);
        println!("PASS native matrix profile={profile}: {rows} frozen contextual bindings");
    }
    Ok(())
}

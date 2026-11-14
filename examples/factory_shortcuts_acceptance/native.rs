use crate::support::{Result, SIZE, editor, send, text};
use crossterm::event::Event;
use rev::{
    command::{Command, Split},
    display::Band,
    session::Session,
};

// The rendered effective hint must route to the same source-preserving balance
// operation, including explicit replacement of a newly accepted factory chord.
pub(super) fn run() -> Result<()> {
    for (profile, context, factory_hint) in [
        ("non-modal", "editor", "C-x +"),
        ("modal", "vim_normal", "C-w ="),
    ] {
        for overridden in [false, true] {
            let config = if overridden {
                format!("[keys.{context}]\n'M-m'='balance-windows'\n")
            } else {
                String::new()
            };
            let hint = if overridden { "M-m" } else { factory_hint };
            let mut session = editor(profile, &config);
            check(&mut session, hint)?;
        }
    }
    println!(
        "PASS native effective hints: rendered factory and overridden routes dispatch identical balance effects, retain source/history/point/focus and cancel selector input in both profiles"
    );
    Ok(())
}

fn check(session: &mut Session, hint: &str) -> Result<()> {
    for _ in 0..2 {
        session.execute(Command::Split(Split::Right));
    }
    let id = session.windows().selected_id();
    let point = session.view().point;
    let stamp = session.buffer()?.stamp();
    let source = text(session);
    let before = widths(session);
    assert!(before.iter().max().unwrap() - before.iter().min().unwrap() > 1);
    hinted_row(session, hint)?;
    send(session, hint)?;
    let after = widths(session);
    assert!(after.iter().max().unwrap() - after.iter().min().unwrap() <= 1);
    assert_eq!(session.windows().selected_id(), id);
    assert_eq!(session.view().point, point);
    assert_eq!(session.buffer()?.stamp(), stamp);
    assert_eq!(text(session), source);
    assert!(session.buffer()?.history().is_empty());
    Ok(())
}

fn hinted_row(session: &mut Session, hint: &str) -> Result<()> {
    session.execute(Command::ExecuteExtended);
    session.handle(Event::Paste("balance-windows".into()));
    let frame = session.frame()?;
    let (index, row) = frame
        .rows()
        .iter()
        .enumerate()
        .find(|(_, r)| r.band() == Band::Completion && r.text().starts_with("balance-windows"))
        .ok_or("missing balance candidate")?;
    assert!(row.text().contains(&format!("[{hint}]")), "{}", row.text());
    let cells = crate::surface::draw(&frame);
    let painted: String = (0..row.text().chars().count())
        .map(|column| cells[&(index, column)].character)
        .collect();
    assert_eq!(painted, row.text());
    send(session, "C-g")
}

fn widths(session: &Session) -> Vec<u16> {
    session
        .windows()
        .geometry(SIZE)
        .unwrap()
        .panes()
        .iter()
        .map(|(_, area)| area.width())
        .collect()
}

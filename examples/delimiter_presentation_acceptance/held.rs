use crate::{
    journey, observe,
    observe::Observer,
    support::Result,
    terminal::{Terminal, row},
};
use std::time::{Duration, Instant};

// Enhanced arrow event bytes enter the real crossterm/native repeat owner.
// The first and reversed key are Press; subsequent events are Repeat, then
// Release. Never manufacture a release by repeatedly invoking a named command.
pub fn run(t: &mut Terminal, o: &mut Observer, profile: &str) -> Result<()> {
    prepare(t, o, profile)?;
    let first = down(t, o)?;
    let release = reverse(t, o)?;
    retired(t, o)?;
    println!(
        "MEASURE held PTY lines=1000 down=96 up=12 final_line=85 first_us={} release_us={} idle_ms=350 idle_cpu_ticks_max=3; native deadline-zero assertion runs in the presentation batch",
        first.as_micros(),
        release.as_micros()
    );
    Ok(())
}

fn prepare(t: &mut Terminal, o: &mut Observer, profile: &str) -> Result<()> {
    t.send(b"\x18\x06navigation.txt\r")?;
    journey::wait(t, o, |f| f.contains("navigation.txt -- RW"))?;
    // Visiting a new source starts modal Normal; this workload owns Insert.
    if profile == "modal" {
        t.send(b"i")?;
        journey::wait(t, o, |f| f.contains("INSERT"))?;
    }
    journey::choose(t, o, "pulse")?;
    t.send(b"\x06\x06\x06\x06\x06\x06\x06\x06\x06\x06")?;
    journey::wait(t, o, |f| observe::navigation(f, 1))?;
    Ok(())
}

fn retired(t: &mut Terminal, o: &mut Observer) -> Result<()> {
    // An unmarked frame before asynchronous confirmation is not expiry proof.
    journey::wait(t, o, |f| observe::navigation(f, 85) && observe::pulsing(f))?;
    let frame = journey::wait(t, o, |f| observe::navigation(f, 85) && observe::unmarked(f))?;
    let top = row(&frame, 1);
    t.settle_updates(|f| {
        o.record(f);
        assert!(observe::navigation(f, 85) && observe::unmarked(f));
        assert_eq!(row(f, 1), top, "idle viewport moved");
    })?;
    t.idle()?;
    Ok(())
}

fn down(t: &mut Terminal, o: &mut Observer) -> Result<Duration> {
    let input = [b"\x1b[1;1:1B".to_vec(), b"\x1b[1;1:2B".repeat(95)].concat();
    let started = Instant::now();
    let mut first = None;
    t.send(&input)?;
    t.updates(
        |f| observe::navigation(f, 97),
        |f| {
            o.record(f);
            if !observe::position(f, 1, 11) {
                first.get_or_insert_with(|| started.elapsed());
            }
        },
    )?;
    let first = first.ok_or("no completed progress frame")?;
    assert!(first <= Duration::from_millis(250));
    Ok(first)
}

fn reverse(t: &mut Terminal, o: &mut Observer) -> Result<Duration> {
    let input = [
        b"\x1b[1;1:1A".to_vec(),
        b"\x1b[1;1:2A".repeat(11),
        b"\x1b[1;1:3A".to_vec(),
    ]
    .concat();
    let started = Instant::now();
    t.send(&input)?;
    journey::wait(t, o, |f| observe::navigation(f, 85))?;
    let elapsed = started.elapsed();
    assert!(
        elapsed <= Duration::from_millis(250),
        "release-to-final frame took {} us",
        elapsed.as_micros()
    );
    Ok(elapsed)
}

use rustix::process::Signal;
use std::time::{Duration, Instant};

pub(super) fn stop(
    exited: bool,
    mut signal: impl FnMut(Signal) -> Result<(), String>,
    mut running: impl FnMut(Instant) -> Result<bool, String>,
) -> Result<(), String> {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(2);
    signal(if exited { Signal::KILL } else { Signal::TERM })?;
    // An unreaped leader pins the group until its final observation. Preserve
    // that complete shutdown witness instead of rescanning unrelated tasks.
    if !exited && wait(start + Duration::from_millis(500), deadline, &mut running)? {
        return Ok(());
    }
    if !exited {
        signal(Signal::KILL)?;
    }
    if wait(deadline, deadline, &mut running)? {
        Ok(())
    } else {
        Err("Git process-group cleanup timed out".into())
    }
}

fn wait(
    until: Instant,
    deadline: Instant,
    running: &mut impl FnMut(Instant) -> Result<bool, String>,
) -> Result<bool, String> {
    while Instant::now() < until {
        if !running(deadline)? {
            return Ok(true);
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Once a complete observation establishes shutdown, a later process-table
    // timeout cannot revoke it. Cover both cancellation and an exited leader.
    #[test]
    fn shutdown_observed() {
        for exited in [false, true] {
            let mut observations = 0;
            let mut signals = Vec::new();
            let result = stop(
                exited,
                |signal| {
                    signals.push(signal);
                    Ok(())
                },
                |_| {
                    observations += 1;
                    match observations {
                        1 => Ok(true),
                        2 => Ok(false),
                        _ => Err("redundant process-table observation".into()),
                    }
                },
            );
            assert!(result.is_ok(), "exited={exited}: {result:?}");
            assert_eq!(observations, 2);
            assert_eq!(signals, [if exited { Signal::KILL } else { Signal::TERM }]);
        }
    }

    // Signalling alone is not shutdown. Observation failures before a complete
    // no-live-member result must keep cleanup owned for either entry state.
    #[test]
    fn observation_failure() {
        for exited in [false, true] {
            let result = stop(exited, |_| Ok(()), |_| Err("observation failed".into()));
            assert_eq!(result.unwrap_err(), "observation failed");
        }
    }
}

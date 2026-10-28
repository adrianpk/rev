use rev::org::time::{
    ClockInterval, OperationTime, ZoneEnvironment, ZoneSelection, ZonedOperation,
};
use std::time::{Duration, UNIX_EPOCH};

fn operation(seconds: u64, zone: &str) -> ZonedOperation {
    let environment = ZoneEnvironment {
        localtime: "unused".into(),
        zoneinfo: "unused".into(),
        tz: Some(zone.into()),
    };
    ZonedOperation::capture(&ZoneSelection::Local, &environment, || {
        Ok(UNIX_EPOCH + Duration::from_secs(seconds))
    })
    .unwrap()
}
// Source precision is minutes, while subtraction uses captured UTC instants:
// midnight and both offset transitions retain independently expected durations.
#[test]
fn elapsed_records() {
    for (name, start, end, zone, expected) in [
        (
            "minute boundary",
            59,
            60,
            "UTC0",
            "CLOCK: [1970-01-01 Thu 00:00]--[1970-01-01 Thu 00:01] =>  0:01",
        ),
        (
            "midnight",
            86340,
            90000,
            "UTC0",
            "CLOCK: [1970-01-01 Thu 23:59]--[1970-01-02 Fri 01:00] =>  1:01",
        ),
        (
            "spring",
            1711845000,
            1711848600,
            "CET-1CEST,M3.5.0,M10.5.0/3",
            "CLOCK: [2024-03-31 Sun 01:30]--[2024-03-31 Sun 03:30] =>  1:00",
        ),
        (
            "autumn",
            1729989000,
            1729992600,
            "CET-1CEST,M3.5.0,M10.5.0/3",
            "CLOCK: [2024-10-27 Sun 02:30]--[2024-10-27 Sun 02:30] =>  1:00",
        ),
        (
            "zero",
            10,
            59,
            "UTC0",
            "CLOCK: [1970-01-01 Thu 00:00]--[1970-01-01 Thu 00:00] =>  0:00",
        ),
    ] {
        let value = ClockInterval::new(operation(start, zone));
        let stop = OperationTime::capture(|| Ok(UNIX_EPOCH + Duration::from_secs(end))).unwrap();
        assert_eq!(value.close(stop).unwrap().source, expected, "{name}");
    }
}
// A backward operation clock is diagnosed without consuming the active value.
#[test]
fn backward_clock() {
    let interval = ClockInterval::new(operation(100, "UTC0"));
    let before = interval.open_source();
    let stop = OperationTime::capture(|| Ok(UNIX_EPOCH + Duration::from_secs(99))).unwrap();
    assert!(interval.close(stop).unwrap_err().contains("before"));
    assert_eq!(interval.open_source(), before);
}

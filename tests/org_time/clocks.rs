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

// Recovery accepts the selected zone instant only when it identifies the stored
// inactive start. Source spelling is retained; recurrence and forged offsets fail.
#[test]
fn clock_resume_source() {
    use rev::org::time::{CivilDateTime, CivilTime, Date, Zone};
    let zone = Zone::utc();
    let civil = CivilDateTime {
        date: Date::new(2024, 3, 1).unwrap(),
        time: CivilTime::new(14, 0, 0).unwrap(),
    };
    let choice = zone.resolve(civil).unwrap().candidates()[0];
    for (source, valid) in [
        ("  CLOCK: [2024-03-01 raw 14:00]", true),
        ("CLOCK: <2024-03-01 Fri 14:00>", false),
        ("CLOCK: [2024-03-01 Fri]", false),
        ("CLOCK: [2024-03-01 Fri 14:01]", false),
        ("CLOCK: [2024-03-01 Fri 14:00 +1d]", false),
        ("CLOCK: [2024-03-01 Fri 14:00 -1d]", false),
    ] {
        let operation = ZonedOperation::from_resolved(choice, zone.clone()).unwrap();
        let value = ClockInterval::resume(source, operation);
        assert_eq!(value.is_ok(), valid, "{source}");
        if let Ok(value) = value {
            assert_eq!(value.open_source(), source);
        }
    }
    let mut forged = choice;
    forged.offset_seconds = 3600;
    assert!(ZonedOperation::from_resolved(forged, zone).is_err());
}

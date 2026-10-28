use rev::org::time::{CivilDateTime, CivilTime, Date, Timestamp, TimestampRange};
fn now() -> CivilDateTime {
    CivilDateTime {
        date: Date::new(2006, 6, 13).unwrap(),
        time: CivilTime::new(14, 20, 45).unwrap(),
    }
}
fn original() -> TimestampRange {
    TimestampRange::parse("<2006-06-13 Tue>").unwrap()
}

// Source observations retain localized or incorrect weekdays and whitespace.
// Their semantic fields remain checked; formatting alone cannot normalize them.
#[test]
fn source_roundtrip() {
    for (name, text) in [
        ("date only", "<2024-02-29>"),
        ("localized", "[2024-02-29  木  9:05 +1w -2d]"),
        ("wrong weekday", "<2000-02-29 Sun>"),
        ("clock range", "<2024-02-29 Thu 23:45-0:15>"),
        ("warning only", "<2024-02-29   --2d>"),
        ("no weekday habit", "<2024-02-29 .+1d/3d>"),
        ("habit spelling", "<2024-02-29 Thu .+1d/3d>"),
        ("date range", "[2024-02-29 Thu]--[2024-03-01 Fri]"),
    ] {
        assert_eq!(
            TimestampRange::parse(text).unwrap().format(),
            text,
            "{name}"
        );
    }
    let stamp = Timestamp::parse("<2024-02-29 Thu 23:45-0:15>").unwrap();
    assert_eq!(
        stamp.end_civil().unwrap().unwrap().date,
        Date::new(2024, 3, 1).unwrap()
    );
}

// Editing one field retains other source spelling, including suffix spacing.
// Explicit normalization alone repairs an incorrect weekday on an unchanged date.
#[test]
fn field_preservation() {
    let mut stamp = Timestamp::parse("<2024-02-29  木  9:05 +1w -2d>").unwrap();
    stamp.set_active(false);
    assert_eq!(stamp.format(), "[2024-02-29  木  9:05 +1w -2d]");
    stamp
        .set_time(Some(CivilTime::new(10, 15, 0).unwrap()))
        .unwrap();
    assert_eq!(stamp.format(), "[2024-02-29  木  10:15 +1w -2d]");
    stamp.set_date(Date::new(2024, 3, 1).unwrap());
    assert_eq!(stamp.format(), "[2024-03-01 Fri 10:15 +1w -2d]");
    let mut stamp = Timestamp::parse("<2024-02-29   +1w>").unwrap();
    stamp.set_date(Date::new(2024, 3, 1).unwrap());
    assert_eq!(stamp.format(), "<2024-03-01 Fri +1w>");
    let mut stamp = Timestamp::parse("<2024-02-29  Sun>").unwrap();
    stamp.normalize_weekday();
    assert_eq!(stamp.format(), "<2024-02-29 Thu>");
}

// Malformed UTF-8 field boundaries, overflow, duplicates and unsupported source
// must fail without panics or partial values. Oversized input is rejected first.
#[test]
fn invalid_source() {
    for (name, text) in [
        ("non leap", "<2023-02-29 Wed>"),
        ("year zero", "<0000-01-01>"),
        ("unicode digits", "<é00-01-01>"),
        ("hour", "<2024-01-01 Mon 24:00>"),
        ("minute", "<2024-01-01 Mon 10:60>"),
        ("seconds", "<2024-01-01 Mon 10:00:01>"),
        ("control", "<2024-01-01\nMon>"),
        ("tail", "<2024-01-01>garbage"),
        ("duplicate suffix", "<2024-01-01 +1w ++2d>"),
        ("zero repeat", "<2024-01-01 +0d>"),
        ("unicode unit", "<2024-01-01 +1日>"),
        ("mismatched range", "<2024-01-01>--[2024-01-02]"),
        ("backward range", "<2024-01-02>--<2024-01-01>"),
        ("end outside source years", "<9999-12-31 Fri 23:30-00:30>"),
    ] {
        assert!(TimestampRange::parse(text).is_err(), "{name}");
    }
    assert!(TimestampRange::parse(&"<".repeat(515)).is_err());
    assert!(Timestamp::new(now().date, Some(now().time), true).is_err());
}

// These documented prompt examples use one fixed default, independent of the
// host calendar, locale and TZ. Week dates enforce the ISO week-year boundary.
#[test]
fn date_entry_forms() {
    for (name, text, expected) in [
        ("short ISO", "3-2-5", "2003-02-05"),
        ("slash year", "2/5/3", "2003-02-05"),
        ("next day", "14", "2006-06-14"),
        ("next month", "12", "2006-07-12"),
        ("next year", "2/5", "2007-02-05"),
        ("weekday", "Fri", "2006-06-16"),
        ("short year window", "2/5/40", "1940-02-05"),
        ("named month", "sep 15", "2006-09-15"),
        ("compact month", "22sept", "2006-09-22"),
        ("past month", "feb 15", "2007-02-15"),
        ("named year", "sep 12 9", "2009-09-12"),
        ("week", "w4", "2006-01-23"),
        ("week weekday", "2012w4fri", "2012-01-27"),
        ("ISO week", "2012-w04-5", "2012-01-27"),
        ("today", ".", "2006-06-13"),
    ] {
        assert_eq!(
            original()
                .enter(text, now())
                .unwrap()
                .start
                .date()
                .to_string(),
            expected,
            "{name}"
        );
    }
}

// Clock ranges and relative inputs resolve against the captured default/now.
// Calendar days do not copy the current time; explicit hours do, at minute precision.
#[test]
fn clock_and_relative_entry() {
    for (name, text, expected) in [
        ("plain clock", "12:45", "<2006-06-13 Tue 12:45>"),
        ("compact date clock", "22sept0:34", "<2006-09-22 Fri 00:34>"),
        ("AM PM range", "11am-1:15pm", "<2006-06-13 Tue 11:00-13:15>"),
        ("h range", "11h-13h15", "<2006-06-13 Tue 11:00-13:15>"),
        ("duration", "11am+2:15", "<2006-06-13 Tue 11:00-13:15>"),
        ("overnight", "23:30--0:30", "<2006-06-13 Tue 23:30-00:30>"),
        ("relative day", "+4", "<2006-06-17 Sat>"),
        ("previous weekday", "-Tue", "<2006-06-06 Tue>"),
        ("relative week", "+2w", "<2006-06-27 Tue>"),
        ("relative hour", "+12h", "<2006-06-14 Wed 02:20>"),
        ("relative weekday", "+2tue", "<2006-06-27 Tue>"),
        ("relative with time", "+2d 10:30", "<2006-06-15 Thu 10:30>"),
        (
            "date range",
            "2006-06-14--2006-06-16",
            "<2006-06-14 Wed>--<2006-06-16 Fri>",
        ),
    ] {
        assert_eq!(
            original().enter(text, now()).unwrap().format(),
            expected,
            "{name}"
        );
    }
    let source = TimestampRange::parse("[2007-01-10 Wed]").unwrap();
    assert_eq!(
        source.enter("++5", now()).unwrap().format(),
        "[2007-01-15 Mon]"
    );
    assert_eq!(
        source.enter("--5", now()).unwrap().format(),
        "[2007-01-05 Fri]"
    );
}

// Rejected drafts leave the original intact, including Unicode spelling. Counts
// and ISO weeks are checked rather than wrapping; unknown text cannot mean today.
#[test]
fn invalid_date_entry() {
    let original = TimestampRange::parse("[2006-06-13 火]").unwrap();
    for (name, text) in [
        ("invalid date", "2023-02-29"),
        ("invalid time", "25:00"),
        ("clock junk", "10:20:30"),
        ("unknown", "来週"),
        ("invalid week", "2014-w53-1"),
        ("empty range", "2006-06-14--"),
        ("overflow weekday", "+9223372036854775807tue"),
        ("overflow count", "+999999999999999999999999d"),
        ("negative duration", "11am+-2"),
        ("control", "2024-01-01\n"),
    ] {
        assert!(original.enter(text, now()).is_err(), "{name}");
    }
    assert!(original.enter(&"é".repeat(129), now()).is_err());
    assert_eq!(
        original.enter("", now()).unwrap().format(),
        "[2006-06-13 火]"
    );
}

// Double-sign hour arithmetic retains the original prompt clock, rather than
// replacing it with the operation's different wall time on each interpretation.
#[test]
fn relative_default_clock() {
    let original = TimestampRange::parse("[2007-01-10  Mer. 9:05]").unwrap();
    assert_eq!(
        original.enter("++2h", now()).unwrap().format(),
        "[2007-01-10  Mer. 11:05]"
    );
    assert_eq!(
        original.enter("+2h", now()).unwrap().format(),
        "[2006-06-13 Tue 16:20]"
    );
}

// Partial dates prefer the captured present while explicit double-sign offsets
// use the original timestamp. Editing an old source cannot reread the host clock.
#[test]
fn partial_date_defaults() {
    let original = TimestampRange::parse("[2007-01-10 Wed]").unwrap();
    for (name, input, expected) in [
        ("day", "14", "2006-06-14"),
        ("month", "sep12", "2006-09-12"),
        ("weekday", "Tue", "2006-06-20"),
        ("default shift", "++5", "2007-01-15"),
    ] {
        assert_eq!(
            original
                .enter(input, now())
                .unwrap()
                .start
                .date()
                .to_string(),
            expected,
            "{name}"
        );
    }
}

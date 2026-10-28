use rev::org::time::{Timestamp, TimestampField as F};

// Selected fields use source byte boundaries, including a localized weekday.
// Suffix digits cannot be mistaken for civil date/time fields.
#[test]
fn timestamp_fields() {
    let text = "<2024-02-29 木 9:05-10:30 +1w>";
    let stamp = Timestamp::parse(text).unwrap();
    for (name, needle, offset, expected) in [
        ("open", "<", 0, F::Active),
        ("year", "2024", 2, F::Year),
        ("month", "-02", 1, F::Month),
        ("day", "29", 1, F::Day),
        ("weekday", "木", 0, F::Day),
        ("hour", "9:", 0, F::Hour),
        ("minute", ":05", 1, F::Minute),
        ("end hour", "10:", 1, F::EndHour),
        ("end minute", ":30", 2, F::EndMinute),
        ("close", ">", 0, F::Active),
    ] {
        assert_eq!(
            stamp.field_at(text.find(needle).unwrap() + offset).unwrap(),
            expected,
            "{name}"
        );
    }
    assert!(stamp.field_at(text.find("+1w").unwrap() + 1).is_err());
    assert!(stamp.field_at(text.find('木').unwrap() + 1).is_err());
    assert!(stamp.field_at(text.len() + 1).is_err());
}

// Civil adjustments retain exact untouched spelling. The first clock moves its
// end equally, while editing the end changes only the stored clock range length.
#[test]
fn field_adjustments() {
    for (name, source, field, up, expected) in [
        (
            "year",
            "[2024-02-29  木 9:05 +1w]",
            F::Year,
            true,
            "[2025-03-01 Sat 9:05 +1w]",
        ),
        (
            "month",
            "<2024-01-31 Wed>",
            F::Month,
            true,
            "<2024-03-02 Sat>",
        ),
        ("day", "<2024-02-29 Thu>", F::Day, true, "<2024-03-01 Fri>"),
        (
            "hour",
            "[2024-02-29  木 9:05-10:30 +1w]",
            F::Hour,
            true,
            "[2024-02-29  木 10:05-11:30 +1w]",
        ),
        (
            "midnight",
            "<2024-02-29 Thu 23:45-0:15>",
            F::Hour,
            true,
            "<2024-03-01 Fri 00:45-01:15>",
        ),
        (
            "minute",
            "<2024-02-29 Thu 9:05-10:30>",
            F::Minute,
            false,
            "<2024-02-29 Thu 9:04-10:29>",
        ),
        (
            "end hour",
            "<2024-02-29 Thu 9:05-10:30>",
            F::EndHour,
            true,
            "<2024-02-29 Thu 9:05-11:30>",
        ),
        (
            "end minute",
            "<2024-02-29 Thu 9:05-10:30>",
            F::EndMinute,
            false,
            "<2024-02-29 Thu 9:05-10:29>",
        ),
        (
            "active",
            "[2024-02-29  木 9:05 +1w]",
            F::Active,
            true,
            "<2024-02-29  木 9:05 +1w>",
        ),
    ] {
        let mut stamp = Timestamp::parse(source).unwrap();
        stamp.step(field, up).unwrap();
        assert_eq!(stamp.format(), expected, "{name}");
    }
}

// Rejected field changes remain atomic, including source-year bounds and absent
// clocks. The public value is unchanged when the checked candidate cannot exist.
#[test]
fn field_adjustment_errors() {
    for (name, source, field, up) in [
        ("upper year", "<9999-12-31 Fri>", F::Day, true),
        ("lower year", "[0001-01-01 Mon]", F::Year, false),
        ("missing time", "<2024-02-29 Thu>", F::Hour, true),
        ("missing end", "<2024-02-29 Thu 09:05>", F::EndMinute, true),
    ] {
        let mut stamp = Timestamp::parse(source).unwrap();
        assert!(stamp.step(field, up).is_err(), "{name}");
        assert_eq!(stamp.format(), source, "{name}");
    }
}

use super::zone_fixture::tzif;
use rev::org::time::Zone;

// Explicit TZif transitions select their new offsets at the exact UTC boundary.
// V2+ footer recurrence supplies future rules instead of freezing the last type.
#[test]
fn tzif_transitions() {
    for version in [b'2', b'3', b'4'] {
        let bytes = tzif(
            version,
            &[(1711846800, 1), (1729990800, 0)],
            &[(3600, 0, 0), (7200, 1, 4)],
            b"CET\0CEST\0",
            "CET-1CEST,M3.5.0,M10.5.0/3",
        );
        let zone = Zone::from_tzif(&bytes).unwrap();
        for (at, expected) in [
            (1711846799, 3600),
            (1711846800, 7200),
            (1729990799, 7200),
            (1729990800, 3600),
            (4102444800, 3600),
            (4118083200, 7200),
        ] {
            assert_eq!(zone.offset_at(at).unwrap(), expected, "{version}: {at}");
        }
    }
}

// Version 1 and an empty future footer do not manufacture unrecorded coverage;
// a -00 type represents unknown time, not a guessed zero offset.
#[test]
fn unknown_coverage() {
    let bytes = tzif(
        0,
        &[(0, 1), (100, 0)],
        &[(3600, 0, 0), (7200, 1, 4)],
        b"CET\0CEST\0",
        "",
    );
    let zone = Zone::from_tzif(&bytes).unwrap();
    for (at, expected) in [(-1, 3600), (0, 7200), (99, 7200)] {
        assert_eq!(zone.offset_at(at).unwrap(), expected)
    }
    assert!(zone.offset_at(100).is_err());
    let bytes = tzif(b'2', &[], &[(0, 0, 0)], b"-00\0", "");
    assert!(Zone::from_tzif(&bytes).unwrap().offset_at(0).is_err());
    let bytes = tzif(b'2', &[], &[(19800, 0, 0)], b"FIX\0", "");
    assert_eq!(
        Zone::from_tzif(&bytes)
            .unwrap()
            .offset_at(4102444800)
            .unwrap(),
        19800
    );
}

// POSIX offsets use the sign opposite to UTC offsets. Both hemispheres, quoted
// names, fractional offsets and rule-time extensions have independent fixtures.
#[test]
fn posix_offsets() {
    for (text, seconds, expected) in [
        ("UTC0", 0, 0),
        ("JST-9", 0, 32400),
        ("ABC5:30:15", 0, -19815),
        ("EST5EDT,M3.2.0,M11.1.0", 1710053999, -18000),
        ("EST5EDT,M3.2.0,M11.1.0", 1710054000, -14400),
        ("EST5EDT,M3.2.0,M11.1.0", 1730613599, -14400),
        ("EST5EDT,M3.2.0,M11.1.0", 1730613600, -18000),
        ("AEST-10AEDT,M10.1.0,M4.1.0/3", 1704067200, 39600),
        ("AEST-10AEDT,M10.1.0,M4.1.0/3", 1719792000, 36000),
        ("<-03>3<-02>,M3.5.0/-2,M10.5.0/-1", 1711846800, -7200),
        ("XXX3EDT4,0/0,J365/23", 1704067200, -14400),
    ] {
        assert_eq!(
            Zone::from_posix(text).unwrap().offset_at(seconds).unwrap(),
            expected,
            "{text}: {seconds}"
        );
    }
}

// Julian rules ignore February 29; ordinal rules count it. No implicit DST
// defaults, trailing data or out-of-profile offsets are silently admitted.
#[test]
fn posix_rules() {
    let julian = Zone::from_posix("STD0DST,J60/0,J300/0").unwrap();
    let ordinal = Zone::from_posix("STD0DST,59/0,300/0").unwrap();
    assert_eq!(julian.offset_at(1709164800).unwrap(), 0);
    assert_eq!(ordinal.offset_at(1709164800).unwrap(), 3600);
    for text in [
        "",
        "UT0",
        "UTC25",
        "UTC24:01",
        "STD0DST",
        "STD0DST,M13.1.0,M10.1.0",
        "STD0DST,M3.0.0,M10.1.0",
        "STD0DST,J0,J300",
        "STD0DST,366,300",
        "STD0DST,M3.1.0/168,M10.1.0",
        "STD0DST,M3.1.0/2u,M10.1.0",
        "UTC0junk",
        "界0",
    ] {
        assert!(Zone::from_posix(text).is_err(), "{text}");
    }
}

// A POSIX unknown designation cannot claim a real zero offset through either a
// direct TZ value or a TZif footer. No valid civil instant may be fabricated.
#[test]
fn unknown_posix_designation() {
    assert!(Zone::from_posix("<-00>0").is_err());
    assert!(Zone::from_posix("STD0<-00>,M3.1.0,M10.1.0").is_err());
    let bytes = tzif(b'3', &[], &[(0, 0, 0)], b"UTC\0", "<-00>0");
    assert!(Zone::from_tzif(&bytes).is_err());
}

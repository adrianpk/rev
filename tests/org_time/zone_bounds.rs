use super::fixture::Fixture;
use super::zone_fixture::tzif;
use super::zone_observation::environment;
use rev::org::time::{Zone, ZoneSelection, ZonedOperation};

// Corrupt independent format fields, including unused types and indicator
// tables. Every rejection occurs at the public parser without a panic or guess.
#[test]
fn invalid_tzif_fields() {
    let base = tzif(0, &[], &[(0, 0, 0)], b"UTC\0", "");
    for (name, at, replacement) in [
        ("magic", 0, b'X'),
        ("version", 4, b'9'),
        ("reserved", 5, 1),
        ("leap", 31, 1),
        ("zero types", 39, 0),
        ("zero chars", 43, 0),
        ("DST flag", 48, 2),
        ("name index", 49, 4),
        ("designation bytes", 50, 255),
        ("termination", 53, b'X'),
    ] {
        let mut bytes = base.clone();
        bytes[at] = replacement;
        assert!(Zone::from_tzif(&bytes).is_err(), "{name}");
    }
    for (name, at, count) in [
        ("transitions", 32, 65537_u32),
        ("types", 36, 257),
        ("designations", 40, 4097),
        ("UT count", 20, 2),
        ("standard count", 24, 2),
    ] {
        let mut bytes = base.clone();
        bytes[at..at + 4].copy_from_slice(&count.to_be_bytes());
        assert!(Zone::from_tzif(&bytes).is_err(), "{name}");
    }
    let mut bytes = base.clone();
    bytes.extend([0]);
    assert!(Zone::from_tzif(&bytes).is_err());
}

// Ordering, offset and footer agreement protect the whole observation, rather
// than only records selected by the current lookup. Version 2 cannot use v3 rules.
#[test]
fn invalid_tzif_records() {
    for (name, bytes) in [
        (
            "unordered",
            tzif(0, &[(1, 0), (0, 0)], &[(0, 0, 0)], b"UTC\0", ""),
        ),
        (
            "duplicate",
            tzif(0, &[(0, 0), (0, 0)], &[(0, 0, 0)], b"UTC\0", ""),
        ),
        ("type index", tzif(0, &[(0, 1)], &[(0, 0, 0)], b"UTC\0", "")),
        ("offset", tzif(0, &[], &[(86401, 0, 0)], b"UTC\0", "")),
        (
            "unused type",
            tzif(0, &[], &[(0, 0, 0), (i32::MIN, 0, 0)], b"UTC\0", ""),
        ),
        (
            "footer mismatch",
            tzif(b'2', &[(0, 0)], &[(3600, 0, 0)], b"UTC\0", "UTC0"),
        ),
        (
            "v2 signed rule",
            tzif(
                b'2',
                &[],
                &[(0, 0, 0)],
                b"UTC\0",
                "STD0DST,M3.5.0/-2,M10.5.0",
            ),
        ),
        (
            "v2 long rule",
            tzif(
                b'2',
                &[],
                &[(0, 0, 0)],
                b"UTC\0",
                "STD0DST,M3.5.0/26,M10.5.0",
            ),
        ),
    ] {
        assert!(Zone::from_tzif(&bytes).is_err(), "{name}");
    }
    let mut bytes = tzif(0, &[], &[(0, 0, 0)], b"UTC\0", "");
    bytes[23] = 1;
    bytes[27] = 1;
    bytes.extend([0, 1]);
    assert!(Zone::from_tzif(&bytes).is_err());
}

// Every truncated prefix rejects. Exact type/transition/designation bounds are
// exercised with real bytes; one-over counts and the 1 MiB ceiling reject.
#[test]
fn tzif_limits() {
    let base = tzif(b'3', &[(0, 0)], &[(0, 0, 0)], b"UTC\0", "UTC0");
    for length in 0..base.len() {
        assert!(Zone::from_tzif(&base[..length]).is_err(), "prefix {length}");
    }
    let types = vec![(0, 0, 0); 256];
    let transitions: Vec<_> = (0..65536).map(|i| (i64::from(i), 255)).collect();
    let mut names = vec![0; 4096];
    names[..4].copy_from_slice(b"UTC\0");
    let bytes = tzif(b'4', &transitions, &types, &names, "UTC0");
    assert_eq!(
        Zone::from_tzif(&bytes).unwrap().offset_at(65535).unwrap(),
        0
    );
    assert!(Zone::from_tzif(&vec![0; 1048577]).is_err());
    let mut mismatch = base.clone();
    mismatch[55] = b'2';
    assert!(Zone::from_tzif(&mismatch).is_err());
}

// Explicit observation IO rejects missing/invalid/directory/oversized fixtures.
// A previously captured operation remains usable after replacement by bad data.
#[test]
fn zone_io_failures() {
    let fixture = Fixture::new();
    let mut env = environment(&fixture);
    let retained =
        ZonedOperation::capture(&ZoneSelection::System, &env, || Ok(std::time::UNIX_EPOCH))
            .unwrap();
    std::fs::write(&env.localtime, b"not a zone").unwrap();
    assert!(Zone::observe(&ZoneSelection::System, &env).is_err());
    assert_eq!(retained.now().time.hour(), 1);
    env.localtime = fixture.0.clone();
    assert!(Zone::observe(&ZoneSelection::System, &env).is_err());
    env.localtime = fixture.0.join("missing");
    assert!(Zone::observe(&ZoneSelection::System, &env).is_err());
    env.localtime = fixture.0.join("oversized");
    std::fs::write(&env.localtime, vec![0; 1048577]).unwrap();
    assert!(Zone::observe(&ZoneSelection::System, &env).is_err());
    env.tz = Some("Missing/Fixture".into());
    assert!(Zone::observe(&ZoneSelection::Local, &env).is_err());
    assert!(Zone::observe(&ZoneSelection::Iana("../escape".into()), &env).is_err());
}

// Name/string bounds reject traversal, malformed environment data and absent
// effective coverage. Empty local TZ is explicit UTC, even without local files.
#[test]
fn selection_limits() {
    let fixture = Fixture::new();
    let mut env = environment(&fixture);
    for name in [
        "",
        ".",
        "..",
        "Area/../Place",
        "Area//Place",
        "Area/Place/",
        "/Area/Place",
        "Area\\Place",
        "Area/界",
    ] {
        assert!(ZoneSelection::parse(name).is_err(), "{name}");
    }
    assert!(ZoneSelection::parse(&"a".repeat(255)).is_ok());
    assert!(ZoneSelection::parse(&"a".repeat(256)).is_err());
    env.tz = Some("x".repeat(1025).into());
    assert!(Zone::observe(&ZoneSelection::Local, &env).is_err());
    env.tz = Some("".into());
    std::fs::remove_file(&env.localtime).unwrap();
    assert_eq!(
        Zone::observe(&ZoneSelection::Local, &env)
            .unwrap()
            .offset_at(0)
            .unwrap(),
        0
    );
    let failed = ZonedOperation::capture(&ZoneSelection::Utc, &env, || {
        Err("owned clock failure".into())
    });
    assert_eq!(failed.unwrap_err(), "owned clock failure");
}

// A future rule must agree with the complete last time type, including DST and
// designation. Matching only the numerical offset does not establish agreement.
#[test]
fn footer_type_agreement() {
    for (name, types, designation) in [
        ("name", vec![(0, 0, 0)], b"BAD\0".as_slice()),
        ("DST", vec![(0, 1, 0)], b"UTC\0".as_slice()),
    ] {
        let bytes = tzif(b'3', &[(0, 0)], &types, designation, "UTC0");
        assert!(Zone::from_tzif(&bytes).is_err(), "{name}");
    }
}

// Abbreviation limits apply to native TZif and POSIX input. Oversized public
// input rejects directly, without relying on the filesystem observation boundary.
#[test]
fn abbreviation_limits() {
    let exact = "A".repeat(64);
    assert_eq!(
        Zone::from_posix(&format!("{exact}0"))
            .unwrap()
            .offset_at(0)
            .unwrap(),
        0
    );
    assert!(Zone::from_posix(&format!("A{exact}0")).is_err());
    let names = format!("{exact}\0");
    assert!(Zone::from_tzif(&tzif(b'3', &[], &[(0, 0, 0)], names.as_bytes(), "")).is_ok());
    let names = format!("A{exact}\0");
    assert!(Zone::from_tzif(&tzif(b'3', &[], &[(0, 0, 0)], names.as_bytes(), "")).is_err());
    assert!(Zone::from_posix(&"A".repeat(1025)).is_err());
}

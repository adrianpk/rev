use rev::org::time::{CivilDateTime, CivilTime, Date, Resolution, ResolvedTime, Zone};

fn local(date: (u16, u8, u8), time: (u8, u8, u8)) -> CivilDateTime {
    CivilDateTime {
        date: Date::new(date.0, date.1, date.2).unwrap(),
        time: CivilTime::new(time.0, time.1, time.2).unwrap(),
    }
}
fn zone() -> Zone {
    Zone::from_posix("CET-1CEST,M3.5.0,M10.5.0/3").unwrap()
}

// Folds retain both ordered UTC choices and require a valid observed offset.
// Gaps offer no manufactured instant, including when an offset is forced.
#[test]
fn explicit_fold_gap() {
    let zone = zone();
    let fold = local((2024, 10, 27), (2, 30, 0));
    let result = zone.resolve(fold).unwrap();
    let expected = [
        ResolvedTime {
            unix_seconds: 1729989000,
            offset_seconds: 7200,
        },
        ResolvedTime {
            unix_seconds: 1729992600,
            offset_seconds: 3600,
        },
    ];
    assert_eq!(result, Resolution::Ambiguous(expected.to_vec()));
    for choice in expected {
        assert_eq!(result.choose_offset(choice.offset_seconds).unwrap(), choice);
        assert_eq!(zone.civil_at(choice.unix_seconds).unwrap(), fold);
    }
    assert!(result.choose_offset(0).is_err());
    let gap = zone.resolve(local((2024, 3, 31), (2, 30, 0))).unwrap();
    assert_eq!(gap, Resolution::Nonexistent);
    assert!(gap.choose_offset(3600).is_err());
    assert!(gap.choose_offset(7200).is_err());
}

// Exact civil transition edges resolve and project through one observation.
// The first and last source dates remain valid despite offset/year spill.
#[test]
fn unique_edges() {
    for (value, seconds, offset) in [
        (local((2024, 3, 31), (1, 59, 59)), 1711846799, 3600),
        (local((2024, 3, 31), (3, 0, 0)), 1711846800, 7200),
        (local((1, 1, 1), (0, 0, 0)), -62135600400, 3600),
        (local((9999, 12, 31), (23, 59, 59)), 253402297199, 3600),
    ] {
        let zone = zone();
        let result = zone.resolve(value).unwrap();
        assert_eq!(
            result,
            Resolution::Unique(ResolvedTime {
                unix_seconds: seconds,
                offset_seconds: offset
            })
        );
        assert_eq!(zone.civil_at(seconds).unwrap(), value);
    }
}

// Unspecified coverage must not become a guessed gap simply because there are
// no known candidate offsets, nor may an exhausted rule yield a unique result.
#[test]
fn unresolved_coverage() {
    use super::zone_fixture::tzif;
    let unknown = Zone::from_tzif(&tzif(b'2', &[], &[(0, 0, 0)], b"-00\0", "")).unwrap();
    assert!(unknown.resolve(local((2024, 1, 1), (0, 0, 0))).is_err());
    let exhausted = Zone::from_tzif(&tzif(b'2', &[(0, 0)], &[(0, 0, 0)], b"UTC\0", "")).unwrap();
    assert!(exhausted.resolve(local((2024, 1, 1), (0, 0, 0))).is_err());
}

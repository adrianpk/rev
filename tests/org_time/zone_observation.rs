use super::fixture::Fixture;
use super::zone_fixture::tzif;
use rev::org::time::{Zone, ZoneEnvironment, ZoneSelection};

pub(super) fn environment(fixture: &Fixture) -> ZoneEnvironment {
    let localtime = fixture.write("localtime", "");
    let named = fixture.write("zoneinfo/Area/Place", "");
    std::fs::write(&localtime, tzif(b'2', &[], &[(3600, 0, 0)], b"LOC\0", "")).unwrap();
    std::fs::write(named, tzif(b'2', &[], &[(19800, 0, 0)], b"NAM\0", "")).unwrap();
    ZoneEnvironment {
        localtime,
        zoneinfo: fixture.0.join("zoneinfo"),
        tz: None,
    }
}

// System, local TZ, explicit UTC and named selections use real owned files.
// An explicit selection overrides TZ, and no test changes process environment.
#[test]
fn observed_selection() {
    let fixture = Fixture::new();
    let mut env = environment(&fixture);
    env.tz = Some("JST-9".into());
    for (name, offset) in [
        ("system", 3600),
        ("local", 32400),
        ("UTC", 0),
        ("Area/Place", 19800),
    ] {
        let choice = ZoneSelection::parse(name).unwrap();
        assert_eq!(
            Zone::observe(&choice, &env).unwrap().offset_at(0).unwrap(),
            offset,
            "{name}"
        );
    }
    env.tz = Some(":Area/Place".into());
    assert_eq!(
        Zone::observe(&ZoneSelection::Local, &env)
            .unwrap()
            .offset_at(0)
            .unwrap(),
        19800
    );
    env.tz = Some(env.localtime.clone().into_os_string());
    assert_eq!(
        Zone::observe(&ZoneSelection::Local, &env)
            .unwrap()
            .offset_at(0)
            .unwrap(),
        3600
    );
}

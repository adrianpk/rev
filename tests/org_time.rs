#![forbid(unsafe_code)]

#[path = "org_time/civil.rs"]
mod civil;

#[path = "org_time/durations.rs"]
mod durations;

#[path = "org_time/calendar.rs"]
mod calendar;

#[path = "org_time/bounds.rs"]
mod bounds;
#[path = "org_time/operation.rs"]
mod operation;

#[path = "org_time/zone_fixture.rs"]
mod zone_fixture;
#[path = "org_time/zones.rs"]
mod zones;

#[path = "configuration/fixture.rs"]
mod fixture;
#[path = "org_time/zone_observation.rs"]
mod zone_observation;

#[path = "org_time/zone_resolution.rs"]
mod zone_resolution;

#[path = "org_time/zone_bounds.rs"]
mod zone_bounds;

#[path = "org_time/timestamps.rs"]
mod timestamps;

#[path = "org_time/timestamp_fields.rs"]
mod timestamp_fields;

#[path = "org_time/repeaters.rs"]
mod repeaters;

#[path = "org_time/clock_tables.rs"]
mod clock_tables;
#[path = "org_time/clock_totals.rs"]
mod clock_totals;
#[path = "org_time/clocks.rs"]
mod clocks;
#[path = "org_time/habit_graphs.rs"]
mod habit_graphs;
#[path = "org_time/habit_history.rs"]
mod habit_history;
#[path = "org_time/habits.rs"]
mod habits;
#[path = "org_time/timers.rs"]
mod timers;

use super::{Result, observe, require};
use rev::{
    buffer::Buffers,
    config::{ExportBackend, Org},
    org::{
        Cancellation, Coverage, Limits, Scan, StructureScan,
        export::{Request, Scope, Work},
        formula::{
            self,
            eval::{Grid, Policy},
        },
        time::{ZoneEnvironment, ZoneSelection, ZonedOperation},
    },
};
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(super) fn source(root: &Path) -> Result<()> {
    let mut buffers = Buffers::new()?;
    let id = buffers.visit(&root.join("work/large.org"))?;
    let buffer = buffers.get(id)?;
    let maximum_chunk = std::cell::Cell::new(Duration::ZERO);
    observe(
        "org_source",
        || {
            let scan = indexed(buffer, &maximum_chunk)?;
            let structure =
                StructureScan::new(buffer, Limits::default())?.run(&Cancellation::default());
            require(
                structure.coverage() == Coverage::Complete,
                "large structure incomplete",
            )?;
            require(
                structure.len() == 10000,
                "large structure heading count changed",
            )?;
            Ok((scan.into_index(), structure))
        },
        || {
            format!(
                "\"bytes\":1048576,\"headings\":10000,\"chunk_limit_ms\":10,\"max_chunk_us\":{:.3}",
                maximum_chunk.get().as_secs_f64() * 1_000_000.0
            )
        },
    )
}

fn indexed(
    buffer: &rev::buffer::Buffer,
    maximum_chunk: &std::cell::Cell<Duration>,
) -> Result<Scan> {
    let mut scan = Scan::new(buffer, Limits::default())?;
    let mut maximum = Duration::ZERO;
    let mut steps = 0;
    loop {
        let start = Instant::now();
        let coverage = scan.advance(&Cancellation::default());
        maximum = maximum.max(start.elapsed());
        steps += 1;
        if coverage != Coverage::Pending {
            break;
        }
    }
    require(
        scan.index().coverage() == Coverage::Complete
            && scan.index().covered_bytes() == rev::coordinates::Byte(1048576),
        "large index incomplete",
    )?;
    require(
        steps >= 256 && maximum <= Duration::from_millis(10),
        "index compute chunk exceeded 10ms",
    )?;
    maximum_chunk.set(maximum_chunk.get().max(maximum));
    Ok(scan)
}

pub(super) fn formulas() -> Result<()> {
    let grid = Grid::new(vec![vec!["old".into()]; 10000], 0)?;
    let cancel = Cancellation::default();
    let program = formula::Program::parse("$1=@-1+1::@1$1=1", formula::Limits::default(), &cancel)?;
    observe(
        "org_formulas",
        || {
            let result = program.evaluate(&grid, Policy::default(), &cancel)?;
            require(result.len() == 10000, "formula graph lost cells")?;
            require(
                result
                    .iter()
                    .enumerate()
                    .all(|(n, c)| c.value == (n + 1) as f64 && c.text == (n + 1).to_string()),
                "formula chain result changed",
            )?;
            Ok(result)
        },
        || "\"cells\":10000,\"last_value\":10000".into(),
    )
}

pub(super) fn clock() -> Result<ZonedOperation> {
    Ok(ZonedOperation::capture(
        &ZoneSelection::Utc,
        &ZoneEnvironment {
            localtime: "unused".into(),
            zoneinfo: "unused".into(),
            tz: None,
        },
        || Ok(std::time::UNIX_EPOCH + Duration::from_secs(1709251200)),
    )?)
}

fn output(snapshot: &rev::org::export::Snapshot, backend: ExportBackend) -> Result<Vec<u8>> {
    let cancel = Cancellation::default();
    let bytes = if backend == ExportBackend::Odt {
        snapshot.package(&cancel)?.bytes().to_vec()
    } else {
        snapshot.render(&cancel)?.text().as_bytes().to_vec()
    };
    require(!bytes.is_empty(), "native export returned empty output")?;
    Ok(bytes)
}

pub(super) fn exports(root: &Path) -> Result<()> {
    let mut buffers = Buffers::new()?;
    let id = buffers.visit(&root.join("work/export.org"))?;
    let policy = Org::default();
    observe(
        "org_exports",
        || {
            let mut outputs = Vec::new();
            for backend in ExportBackend::ALL {
                outputs.push(exported(&buffers, id, backend, &policy)?);
            }
            verified_exports(&outputs)?;
            Ok(outputs)
        },
        || "\"backends\":11".into(),
    )
}

fn exported(
    buffers: &Buffers,
    id: rev::buffer::BufferId,
    backend: ExportBackend,
    policy: &Org,
) -> Result<Vec<u8>> {
    let request = Request {
        backend,
        scope: Scope::Buffer,
        options: None,
        time: clock()?,
    };
    let snapshot = Work::new(buffers, id, request, policy)?.run(&Cancellation::default())?;
    output(&snapshot, backend)
}

fn verified_exports(outputs: &[Vec<u8>]) -> Result<()> {
    require(outputs.len() == 11, "native export backend omitted")?;
    require(
        outputs[0].windows(6).any(|s| s == b"Native"),
        "text export lost title",
    )?;
    require(
        outputs[7].starts_with(b"PK\x03\x04"),
        "ODT package signature missing",
    )?;
    require(
        outputs[10].windows(15).any(|s| s == b"BEGIN:VCALENDAR"),
        "calendar component missing",
    )
}

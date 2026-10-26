use std::ops::ControlFlow;
use std::path::Path;
use std::time::{Duration, Instant};

use rev::buffer::edit::Replacement;
use rev::buffer::{Buffer, Buffers};
use rev::syntax::Syntax;
use tree_sitter::{ParseOptions, ParseState};

use super::{Result, require, statistics};

fn cold(buffer: &Buffer) -> Result<(Duration, usize)> {
    let mut syntax = Syntax::rust(Some(tree_sitter_rust::HIGHLIGHTS_QUERY));
    let mut calls = 0;
    let mut progress = |_: &ParseState| {
        calls += 1;
        ControlFlow::Continue(())
    };
    let start = Instant::now();
    syntax.update_with(
        buffer.slice(&buffer.span(0..buffer.len_chars())?)?,
        buffer.stamp(),
        ParseOptions::new().progress_callback(&mut progress),
    );
    let elapsed = start.elapsed();
    require(syntax.mode(buffer.stamp()) == "Rust", "cold parser failed")?;
    require(calls > 0, "cold parse supplied no work evidence")?;
    Ok((elapsed, calls))
}

fn refresh(buffer: &mut Buffer) -> Result<(Duration, usize)> {
    let mut calls = 0;
    let mut progress = |_: &ParseState| {
        calls += 1;
        ControlFlow::Continue(())
    };
    let start = Instant::now();
    buffer.refresh_syntax_with(ParseOptions::new().progress_callback(&mut progress));
    let elapsed = start.elapsed();
    require(buffer.mode() == "Rust", "incremental parser failed")?;
    Ok((elapsed, calls))
}

fn change(buffer: &mut Buffer, position: usize, inserted: bool) -> Result<()> {
    let replacement = Replacement {
        source: buffer.span(position..position + usize::from(!inserted))?,
        text: if inserted { "x" } else { "" }.into(),
    };
    let prepared = buffer.prepare(&[replacement], &[], None)?;
    buffer.publish(prepared, &mut [])?;
    Ok(())
}

#[derive(Default)]
struct Work {
    samples: Vec<Duration>,
    total: usize,
    maximum: usize,
    unchanged: usize,
}

impl Work {
    fn edits(&mut self, buffer: &mut Buffer, position: usize) -> Result<()> {
        for _ in 0..8 {
            for inserted in [true, false] {
                change(buffer, position, inserted)?;
                let (elapsed, calls) = refresh(buffer)?;
                require(
                    buffer.syntax().is_incremental(),
                    "accepted edit discarded its parse seed",
                )?;
                self.samples.push(elapsed);
                self.total += calls;
                self.maximum = self.maximum.max(calls);
                self.unchanged += refresh(buffer)?.1;
            }
        }
        Ok(())
    }

    fn report(&mut self, cold_time: Duration, cold_calls: usize) -> Result<()> {
        require(
            self.unchanged == 0,
            "unchanged revision executed parser work",
        )?;
        let (median, maximum_us) = statistics(&mut self.samples);
        println!(
            "{{\"workload\":\"syntax_work\",\"samples\":{},\"median_us\":{median:.3},\"max_us\":{maximum_us:.3},\"cold_parse_us\":{:.3},\"cold_callbacks\":{cold_calls},\"incremental_callbacks\":{},\"max_incremental_callbacks\":{},\"unchanged_callbacks\":{}}}",
            self.samples.len(),
            cold_time.as_secs_f64() * 1_000_000.0,
            self.total,
            self.maximum,
            self.unchanged
        );
        // Compare actual parser progress with a cold parse of this same input.
        // This detects full reparse work, not a hardware latency guarantee.
        require(
            self.maximum * 4 < cold_calls,
            "local edit repeats substantial full-parse work",
        )?;
        Ok(())
    }
}

fn positions(buffer: &Buffer) -> Result<[usize; 2]> {
    let whole = buffer.slice(&buffer.span(0..buffer.len_chars())?)?;
    let row = buffer.len_lines() / 2;
    let start = whole.line_to_char(row);
    // A valid identifier edit and an unfinished parameter match the two real
    // editing contexts; do not infer incomplete-code behavior from valid code.
    Ok([start + 3, start + format!("fn item_{row}(").chars().count()])
}

pub(super) fn run(path: &Path) -> Result<()> {
    let mut buffers = Buffers::new()?;
    let id = buffers.visit(path)?;
    let buffer = buffers.get_mut(id)?;
    let (cold_time, cold_calls) = cold(buffer)?;
    let mut work = Work::default();
    for position in positions(buffer)? {
        work.edits(buffer, position)?;
    }
    work.report(cold_time, cold_calls)
}

use super::read::{Read, Stop};
use super::{InlineContext, InlineKind, InlineObservation, LinkKind, Raw, candidate};
use crate::org::{Coverage, Limit, Limits};
use std::ops::Range;

struct Frame {
    range: Range<usize>,
    position: usize,
    context: InlineContext,
    parent: Option<usize>,
    depth: usize,
    failed_until: [usize; 8],
}

impl Frame {
    fn new(
        range: Range<usize>,
        context: InlineContext,
        parent: Option<usize>,
        depth: usize,
    ) -> Self {
        Self {
            position: range.start,
            range,
            context,
            parent,
            depth,
            failed_until: [0; 8],
        }
    }
}

pub(super) fn scan(
    read: &mut Read<'_>,
    result: &mut InlineObservation,
    context: InlineContext,
    limits: Limits,
) -> Result<(), Stop> {
    read.tick()?;
    if context == InlineContext::Literal {
        return Ok(());
    }
    let mut frames = vec![Frame::new(read.start..read.end, context, None, 0)];
    while let Some(mut frame) = frames.pop() {
        read.start = frame.range.start;
        read.end = frame.range.end;
        read.depth = frame.depth;
        read.failed_until = frame.failed_until;
        process(read, result, &mut frames, &mut frame, limits)?;
    }
    Ok(())
}

fn process(
    read: &mut Read<'_>,
    result: &mut InlineObservation,
    frames: &mut Vec<Frame>,
    frame: &mut Frame,
    limits: Limits,
) -> Result<(), Stop> {
    while frame.position < frame.range.end {
        let Some(mut node) = candidate(read, frame.position, frame.context)? else {
            frame.position += 1;
            continue;
        };
        if result.nodes.len() == limits.nodes {
            return Err(Stop(Coverage::Incomplete(Limit::SourceNodes)));
        }
        if frame.depth >= limits.depth {
            return Err(Stop(Coverage::Incomplete(Limit::SourceDepth)));
        }
        node.parent = frame.parent;
        frame.position = node.source.end;
        let child = child(&node, frame.context);
        let parent = result.nodes.len();
        result.nodes.push(node);
        if let Some((range, context)) = child {
            descend(
                frames,
                frame,
                (range, context, parent),
                read.failed_until,
                limits.depth,
            )?;
            return Ok(());
        }
    }
    Ok(())
}

fn descend(
    frames: &mut Vec<Frame>,
    frame: &Frame,
    child: (Range<usize>, InlineContext, usize),
    failed_until: [usize; 8],
    limit: usize,
) -> Result<(), Stop> {
    if frame.depth + 1 > limit {
        return Err(Stop(Coverage::Incomplete(Limit::SourceDepth)));
    }
    let resume = Frame {
        range: frame.range.clone(),
        position: frame.position,
        context: frame.context,
        parent: frame.parent,
        depth: frame.depth,
        failed_until,
    };
    frames.push(resume);
    frames.push(Frame::new(child.0, child.1, Some(child.2), frame.depth + 1));
    Ok(())
}

fn child(node: &Raw, context: InlineContext) -> Option<(Range<usize>, InlineContext)> {
    if node.content.is_empty() && node.detail.as_ref().is_none_or(Range::is_empty) {
        return None;
    }
    match node.kind {
        InlineKind::Bold | InlineKind::Italic | InlineKind::Underline | InlineKind::Strike => {
            Some((node.content.clone(), context))
        }
        InlineKind::Link(LinkKind::Bracket) => node
            .detail
            .clone()
            .map(|range| (range, InlineContext::Minimal)),
        InlineKind::RadioTarget => Some((node.content.clone(), InlineContext::Minimal)),
        InlineKind::FootnoteInline => node
            .detail
            .clone()
            .map(|range| (range, InlineContext::Standard)),
        _ => None,
    }
}

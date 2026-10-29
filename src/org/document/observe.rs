use super::*;
impl Document {
    pub(super) fn observe(text: Rope, cancel: &Cancellation) -> Result<Self, String> {
        let raw = structure::observe(text.clone(), Limits::default(), cancel);
        if raw.coverage != Coverage::Complete
            || raw.nodes.iter().any(|n| n.kind == StructureKind::Malformed)
        {
            return Err("Document structure is incomplete or malformed; source unchanged".into());
        }
        let mut protected = literal_ranges(&raw.nodes, &text);
        macros(&text, &mut protected, cancel)?;
        normalize(&mut protected);
        let mut objects = inline_nodes(&text, &raw.nodes, &protected, cancel)?;
        radio_profile(&objects)?;
        objects.sort_by_key(|o| o.source.start);
        objects.dedup_by(|a, b| a.source == b.source && a.kind == b.kind);
        Ok(Self {
            text,
            nodes: raw.nodes,
            objects,
            protected,
        })
    }
    pub(super) fn standard(&self, range: &Range<usize>) -> Result<(), String> {
        if protected_range(&self.protected, range) {
            return Err(
                "Literal block, macro or unsupported source form; edit source explicitly".into(),
            );
        }
        let admitted = self.nodes.iter().any(|n| {
            n.context == Some(InlineContext::Standard)
                && n.content.start <= range.start
                && range.end <= n.content.end
        });
        if !admitted && !(self.text.len_bytes() == 0 && range.is_empty()) {
            return Err("Action requires ordinary Org text; literal or unsupported context".into());
        }
        if self
            .objects
            .iter()
            .any(|o| opaque(o.kind) && o.source != *range && overlaps(&o.source, range))
        {
            return Err("Action crosses a literal inline object or link; source unchanged".into());
        }
        Ok(())
    }
}
pub(super) fn overlaps(a: &Range<usize>, b: &Range<usize>) -> bool {
    if b.is_empty() {
        a.start < b.start && b.start < a.end
    } else {
        a.start < b.end && b.start < a.end
    }
}
fn opaque(kind: InlineKind) -> bool {
    matches!(
        kind,
        InlineKind::Code
            | InlineKind::Verbatim
            | InlineKind::Link(_)
            | InlineKind::Target
            | InlineKind::RadioTarget
            | InlineKind::Entity
    )
}

fn literal_ranges(nodes: &[structure::Raw], text: &Rope) -> Vec<Range<usize>> {
    nodes
        .iter()
        .filter(|n| literal(n, text))
        .map(|n| n.source.clone())
        .collect()
}
fn literal(node: &structure::Raw, text: &Rope) -> bool {
    use super::super::BlockKind as B;
    match node.kind {
        StructureKind::Block(B::Source | B::Example | B::Export | B::Comment | B::Dynamic) => true,
        StructureKind::Block(B::Greater) => !node.name.as_ref().is_some_and(|r| {
            matches!(
                text.byte_slice(r.clone())
                    .to_string()
                    .to_ascii_uppercase()
                    .as_str(),
                "QUOTE" | "CENTER"
            )
        }),
        StructureKind::FixedWidth | StructureKind::Comment => true,
        _ => false,
    }
}
fn macros(
    text: &Rope,
    ranges: &mut Vec<Range<usize>>,
    cancel: &Cancellation,
) -> Result<(), String> {
    let mut scan = Macros {
        start: None,
        previous: [' '; 2],
        ranges: std::mem::take(ranges),
    };
    for (index, character) in text.chars().enumerate() {
        if index % 256 == 0 {
            check(cancel)?;
        }
        scan.character(text, index, character)?;
    }
    if let Some(at) = scan.start {
        scan.ranges.push(at..text.len_bytes());
    }
    *ranges = scan.ranges;
    Ok(())
}
struct Macros {
    start: Option<usize>,
    previous: [char; 2],
    ranges: Vec<Range<usize>>,
}
impl Macros {
    fn character(&mut self, text: &Rope, index: usize, character: char) -> Result<(), String> {
        if character == '{' && self.previous == ['{', '{'] && self.start.is_none() {
            self.start = Some(text.char_to_byte(index - 2));
        }
        if character == '}'
            && self.previous == ['}', '}']
            && let Some(at) = self.start.take()
        {
            if self.ranges.len() >= Limits::default().nodes {
                return Err("Document protected node limit reached".into());
            }
            self.ranges.push(at..text.char_to_byte(index + 1));
        }
        self.previous = [self.previous[1], character];
        Ok(())
    }
}

pub(super) fn protected_range(ranges: &[Range<usize>], range: &Range<usize>) -> bool {
    let index = ranges.partition_point(|r| r.end <= range.start);
    ranges
        .get(index)
        .is_some_and(|r| r.start <= range.start || r.start < range.end)
}
fn normalize(ranges: &mut Vec<Range<usize>>) {
    ranges.sort_by_key(|r| r.start);
    let mut count = 0;
    for i in 0..ranges.len() {
        if count > 0 && ranges[count - 1].end >= ranges[i].start {
            ranges[count - 1].end = ranges[count - 1].end.max(ranges[i].end);
        } else {
            ranges[count] = ranges[i].clone();
            count += 1;
        }
    }
    ranges.truncate(count);
}

fn radio_profile(objects: &[inline::Raw]) -> Result<(), String> {
    let mut count = 0;
    let mut bytes = 0;
    for o in objects.iter().filter(|o| o.kind == InlineKind::RadioTarget) {
        count += 1;
        bytes += o.content.len();
        if count > 4096 || bytes > 1048576 || o.content.len() > 1024 {
            return Err("Radio targets exceed native record/name profile".into());
        }
    }
    Ok(())
}

fn protected_contains(ranges: &[Range<usize>], range: &Range<usize>) -> bool {
    let index = ranges.partition_point(|r| r.end <= range.start);
    ranges
        .get(index)
        .is_some_and(|r| r.start <= range.start && range.end <= r.end)
}

fn inline_nodes(
    text: &Rope,
    nodes: &[structure::Raw],
    protected: &[Range<usize>],
    cancel: &Cancellation,
) -> Result<Vec<inline::Raw>, String> {
    let mut objects = Vec::new();
    for node in nodes {
        check(cancel)?;
        if protected_contains(protected, &node.content) {
            continue;
        }
        if let Some(context) = node.context.filter(|c| *c != InlineContext::Literal) {
            objects.extend(inline::objects(
                text,
                node.content.clone(),
                context,
                cancel,
            )?);
        }
        if objects.len() > Limits::default().nodes {
            return Err("Document inline node limit reached".into());
        }
    }
    objects.retain(|o| !protected_range(protected, &o.source));

    Ok(objects)
}

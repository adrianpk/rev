use std::ops::Range;

use super::Request;
use crate::{ai::pi, config::AiOutput, coordinates::Scalar};

pub(super) const EXPLAIN: &str = "Explain the supplied material using software engineering as the default perspective, including code, configuration, logs, diffs and technical documentation. Explain purpose, behavior, relationships and non-obvious details rather than merely paraphrasing. If the material is clearly outside software engineering, infer the relevant domain and adapt the explanation without announcing a persona. Match depth to the input: start with an overview for a whole file; keep a small selection focused. Use examples only when they help understanding.";

pub(super) fn guidance(output: AiOutput) -> String {
    match output {
        AiOutput::Explanation => {
            format!("{EXPLAIN} Write the explanation as Org body text, not Markdown. Use Org lists, =inline code=, subsections starting at **, and #+begin_src language / #+end_src blocks where useful. Start with the explanation itself. Do not wrap the entire answer in a block or repeat a document title. The editor supplies the source heading and request metadata; do not invent them. An explicit user instruction about perspective, content and length takes precedence over this default guidance.")
        }
        _ => {
            "Return only the exact text to insert or replace, without explanatory prose, display fences, a document title or request metadata unless those characters are explicitly requested as target text. The editor supplies a separate Org proposal preview; its markup is never part of the accepted text.".into()
        }
    }
}

pub(super) struct Document {
    pub text: String,
    pub body: Range<usize>,
    pub drawer: Scalar,
    pub drawer_body: Range<usize>,
}

pub(super) fn document(request: &Request, output: &pi::Output, condition: &str) -> Document {
    let target = &request.target;
    let operation = match target.output {
        AiOutput::Explanation => "explanation",
        AiOutput::Insert => "insertion proposal",
        AiOutput::Replace => "replacement proposal",
    };
    let mut text = format!("* {} — {operation}\n\n", line(&request.source_name));
    if !condition.is_empty() {
        text.push_str(&format!("Source warning: {}\n\n", line(condition)));
    }
    let start = text.chars().count();
    if request.conflict.is_some() {
        text.push_str("Captured Git conflict/source context; historical explanation only. Its native span retains return ownership. No write, stage or continue authority.\n\n");
    }
    if target.output.edits() {
        proposal(&mut text, &output.text, &request.language);
    } else {
        text.push_str(&output.text);
    }
    let end = text.chars().count();
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push('\n');
    if target.output.edits() {
        text.push_str("Proposal only: ai-apply-result accepts; ai-discard-result discards.\n\n");
    } else {
        text.push_str("Read-only explanation. ai-discard-result returns to source.\n\n");
    }
    let drawer = Scalar(text.chars().count());
    let drawer_start = text.len() + ":REQUEST:\n".len();
    text.push_str(&format!(
        ":REQUEST:\nBackend: Pi\nModel: {}\nSource: {}\nInput: {}\nContext: {}\nHistory: {}\nSnapshot: buffer {} revision {}, scalars {}..{}\n",
        line(&output.model),
        line(&request.source_name),
        request.definition.input.name(),
        if output.directory {
            "guarded directory read/list/search; captured native snapshots override disk"
        } else {
            "input only; directory discovery disabled"
        },
        line(&output.history),
        target.source.stamp.buffer,
        target.source.stamp.revision,
        target.source.range.start,
        target.source.range.end,
    ));
    if output.directory {
        text.push_str(&format!(
            "Directory root: {}\n",
            line(&request.root.to_string_lossy())
        ));
    }
    let drawer_end = text.len();
    text.push_str(":END:\n");
    Document {
        text,
        body: start..end,
        drawer,
        drawer_body: drawer_start..drawer_end,
    }
}

fn line(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

fn proposal(text: &mut String, generated: &str, language: &str) {
    let source = language != "plain"
        && !language.is_empty()
        && language
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-+_".contains(&b));
    if source {
        text.push_str(&format!("#+begin_src {language}\n"));
    } else {
        text.push_str("#+begin_example\n");
    }
    // Org comma escapes protect literal headings and block delimiters in the
    // display only. Acceptance uses the separately retained original payload.
    for row in generated.split_inclusive('\n') {
        let row = row.strip_suffix('\n').unwrap_or(row);
        let row = row.strip_suffix('\r').unwrap_or(row);
        let body = row.trim_start_matches([' ', '\t']);
        let prefix = row.len() - body.len();
        let unescaped = body.trim_start_matches(',');
        text.push_str(&row[..prefix]);
        if unescaped.starts_with('*') || unescaped.starts_with("#+") {
            text.push(',');
        }
        text.push_str(body);
        text.push('\n');
    }
    text.push_str(if source {
        "#+end_src\n"
    } else {
        "#+end_example\n"
    });
}

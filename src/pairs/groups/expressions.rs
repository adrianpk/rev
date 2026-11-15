use super::super::{Context, Profile};

/// Syntax-class units share the group's reader, stack, facts and request budget.
/// These are source ranges, never typed pair facts or parser-node spans.
#[derive(Default)]
pub(super) struct Expressions {
    atom: Option<(usize, usize)>,
    pub tag: Option<usize>,
}

impl Expressions {
    pub fn boundary(
        &mut self,
        at: usize,
        next: Option<char>,
        syntax: (Profile, Context, usize),
    ) -> Option<(usize, usize)> {
        let (profile, context, depth) = syntax;
        let continues = next.is_some_and(|c| atom(profile, c))
            && matches!(context, Context::Code | Context::String(_));
        if continues && self.atom.is_none_or(|(_, level)| level == depth) {
            return None;
        }
        self.take(at)
    }

    pub fn take(&mut self, end: usize) -> Option<(usize, usize)> {
        self.atom.take().filter(|(start, _)| *start < end)
    }

    pub fn ordinary(&mut self, at: usize, c: char, syntax: (Profile, Context, usize)) {
        let (profile, context, depth) = syntax;
        if matches!(context, Context::Code | Context::String(_)) && (atom(profile, c) || c == '\\')
        {
            self.atom.get_or_insert((at, depth));
        }
    }
}

fn atom(profile: Profile, c: char) -> bool {
    if word(c) {
        return true;
    }
    match profile {
        Profile::Plain | Profile::Org | Profile::Markdown => {
            matches!(
                c,
                '-' | '+' | '*' | '/' | '$' | '\'' | '=' | '<' | '>' | '&'
            )
        }
        Profile::Css => matches!(c, '-' | '.' | '@' | '#'),
        Profile::Html => matches!(c, '-' | '+' | '*' | '/' | '.' | '$' | ':' | '=' | '&'),
        _ => c == '$',
    }
}

fn word(c: char) -> bool {
    c.is_alphanumeric()
        || c == '_'
        || unicode_width::UnicodeWidthChar::width(c) == Some(0)
            && !c.is_control()
            && !c.is_whitespace()
}

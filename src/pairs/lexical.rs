use std::ops::Range;

use ropey::RopeSlice;

use super::{
    Context, Kind, Limit, MAX_DEPTH, Profile, Reason,
    reader::{Pause, Reader},
};

pub(super) enum Action {
    Ignore,
    Open(Kind, bool),
    Close(Kind),
    Excluded(Range<usize>, Context),
    Refuse(Reason),
}

#[derive(Clone, Copy)]
enum Mode {
    Code,
    Line(usize),
    LiteralLine(usize),
    Escape(usize),
    Block(usize, usize),
    HtmlComment(usize),
    Quote {
        kind: Kind,
        start: usize,
        raw: bool,
        escaped: bool,
    },
    Marker {
        marker: char,
        start: usize,
    },
    Fence {
        start: usize,
        end: &'static str,
    },
}

#[derive(Clone, Copy)]
pub(super) struct Lexical {
    pub profile: Profile,
    mode: Mode,
    html_tag: bool,
    html_equals: bool,
    pub maximum_depth: usize,
}

impl Lexical {
    pub fn new(profile: Profile) -> Self {
        Self {
            profile,
            mode: Mode::Code,
            html_tag: false,
            html_equals: false,
            maximum_depth: 0,
        }
    }

    pub fn context(&self) -> Context {
        match self.mode {
            Mode::Code => Context::Code,
            Mode::Line(_) | Mode::Block(..) | Mode::HtmlComment(_) => Context::Comment,
            Mode::LiteralLine(_) | Mode::Escape(_) => Context::Literal,
            Mode::Quote { kind, .. } => Context::String(kind),
            Mode::Marker { .. } | Mode::Fence { .. } => Context::Literal,
        }
    }

    pub fn start(&self) -> Option<usize> {
        match self.mode {
            Mode::Code => None,
            Mode::Line(start)
            | Mode::LiteralLine(start)
            | Mode::Escape(start)
            | Mode::Block(start, _)
            | Mode::HtmlComment(start)
            | Mode::Quote { start, .. }
            | Mode::Marker { start, .. }
            | Mode::Fence { start, .. } => Some(start),
        }
    }

    pub fn comment_depth(&self) -> usize {
        if let Mode::Block(_, depth) = self.mode {
            depth
        } else {
            0
        }
    }

    pub fn step(&mut self, r: &mut Reader, text: RopeSlice<'_>) -> Result<Action, Pause> {
        let Some(c) = r.peek(text, 0)? else {
            return Ok(Action::Ignore);
        };
        match self.mode {
            Mode::Code => self.code(r, text, c),
            Mode::Quote { .. } => Ok(self.quoted(r, c)),
            _ => self.trivia(r, text, c),
        }
    }

    fn trivia(&mut self, r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Action, Pause> {
        match self.mode {
            Mode::Line(start) => self.line(r, text, (c, start, Context::Comment)),
            Mode::LiteralLine(start) => self.line(r, text, (c, start, Context::Literal)),
            Mode::Block(start, depth) => self.block(r, text, (start, depth)),
            Mode::HtmlComment(start) => self.html_comment(r, text, start),
            Mode::Marker { marker, start } => Ok(self.marker(r, c, (marker, start))),
            Mode::Fence { start, end } => self.fence(r, text, (start, end)),
            Mode::Escape(_) => {
                r.consume(1);
                self.mode = Mode::Code;
                Ok(Action::Ignore)
            }
            _ => unreachable!("literal pair mode"),
        }
    }

    fn code(&mut self, r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Action, Pause> {
        if let Some(action) = self.comment_start(r, text, c)? {
            return Ok(action);
        }
        if let Some(action) = self.prose_marker(r, text, c)? {
            return Ok(action);
        }
        if triple_quote(r, text, c)? {
            return Ok(Action::Refuse(Reason::AmbiguousQuote));
        }
        if let Some(reason) = self.ambiguous(r, text, c)? {
            return Ok(Action::Refuse(reason));
        }
        self.ordinary(r, text, c)
    }

    fn comment_start(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        if c == '/' && self.profile.block_comments() {
            let next = r.peek(text, 1)?;
            if next == Some('*') {
                self.mode = Mode::Block(r.at, 1);
                self.maximum_depth = self.maximum_depth.max(1);
                r.consume(2);
                return Ok(Some(Action::Ignore));
            }
            if next == Some('/') && self.profile.slash_comments() {
                self.mode = Mode::Line(r.at);
                r.consume(2);
                return Ok(Some(Action::Ignore));
            }
        }
        if c == '#' && self.profile == Profile::Toml {
            self.mode = Mode::Line(r.at);
            r.consume(1);
            return Ok(Some(Action::Ignore));
        }
        self.html_start(r, text, c)
    }

    fn html_start(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        if self.profile != Profile::Html || c != '<' {
            return Ok(None);
        }
        if r.prefix(text, "<!--")? {
            self.mode = Mode::HtmlComment(r.at);
            r.consume(4);
            return Ok(Some(Action::Ignore));
        }
        if r.prefix(text, "<script")? || r.prefix(text, "<style")? {
            return Ok(Some(Action::Refuse(Reason::LiteralContext)));
        }
        self.html_tag = true;
        Ok(None)
    }

    fn prose_marker(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        if !self.profile.prose() {
            return Ok(None);
        }
        if let Some(action) = self.block_marker(r, text, c)? {
            return Ok(Some(action));
        }
        let org_marker = self.profile == Profile::Org && matches!(c, '=' | '~');
        if c == '`' || org_marker && r.previous.is_none_or(|p| !p.is_alphanumeric()) {
            self.mode = Mode::Marker {
                marker: c,
                start: r.at,
            };
            r.consume(1);
            return Ok(Some(Action::Ignore));
        }
        Ok(None)
    }

    fn block_marker(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        match self.profile {
            Profile::Org => self.org_marker(r, text, c),
            Profile::Markdown => self.markdown_marker(r, text, c),
            _ => Ok(None),
        }
    }

    fn markdown_marker(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        if r.line_start && markdown_indent(r, text)? {
            self.mode = Mode::LiteralLine(r.at);
            r.consume(1);
            return Ok(Some(Action::Ignore));
        }
        if !matches!(c, '`' | '~') {
            return Ok(None);
        }
        let end = if c == '`' { "```" } else { "~~~" };
        if !r.prefix(text, end)? {
            return Ok(None);
        }
        if r.peek(text, 3)? == Some(c) {
            return Ok(Some(Action::Refuse(Reason::AmbiguousQuote)));
        }
        self.mode = Mode::Fence { start: r.at, end };
        r.consume(3);
        Ok(Some(Action::Ignore))
    }

    fn org_marker(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Action>, Pause> {
        if let Some(action) = inline_org(r, text, c)? {
            return Ok(Some(action));
        }
        if !r.line_start || c != '#' || !r.prefix(text, "#+begin_")? {
            return Ok(None);
        }
        for (begin, end) in [
            ("#+begin_src", "#+end_src"),
            ("#+begin_example", "#+end_example"),
        ] {
            if r.prefix(text, begin)? {
                self.mode = Mode::Fence { start: r.at, end };
                r.consume(begin.len());
                return Ok(Some(Action::Ignore));
            }
        }
        Ok(Some(Action::Refuse(Reason::LiteralContext)))
    }

    fn ambiguous(
        &self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        c: char,
    ) -> Result<Option<Reason>, Pause> {
        match self.profile {
            Profile::Script => Ok(match c {
                '/' => Some(Reason::AmbiguousSlash),
                '`' => Some(Reason::AmbiguousQuote),
                '<' => Some(Reason::LiteralContext),
                _ => None,
            }),
            Profile::Rust => rust_ambiguity(r, text, c),
            Profile::Toml => toml_ambiguity(r, text, c),
            _ => Ok(None),
        }
    }

    fn ordinary(&mut self, r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Action, Pause> {
        if c == '\\' {
            return self.escape(r, text);
        }
        if let Some(kind) = quote(c) {
            return Ok(self.open_quote(r, kind));
        }
        if self.html_punctuation(c) {
            return Ok(Action::Refuse(Reason::LiteralContext));
        }
        r.consume(1);
        Ok(Kind::structural(c).map_or(Action::Ignore, |(kind, open)| {
            if open {
                Action::Open(kind, true)
            } else {
                Action::Close(kind)
            }
        }))
    }

    fn escape(&mut self, r: &mut Reader, text: RopeSlice<'_>) -> Result<Action, Pause> {
        if self.profile == Profile::C {
            return Ok(Action::Refuse(Reason::LiteralContext));
        }
        let count = if r.peek(text, 1)?.is_some() { 2 } else { 1 };
        if count == 1 {
            self.mode = Mode::Escape(r.at);
        }
        r.consume(count);
        Ok(Action::Ignore)
    }

    fn html_punctuation(&mut self, c: char) -> bool {
        if self.profile != Profile::Html {
            return false;
        }
        self.html_equals = self.html_equals && c.is_whitespace() || c == '=';
        self.html_tag &= c != '>';
        self.html_tag && Kind::structural(c).is_some()
    }

    fn open_quote(&mut self, r: &mut Reader, kind: Kind) -> Action {
        let word = r.previous.is_some_and(|c| c.is_alphanumeric() || c == '_');
        if self.profile.prose() && kind == Kind::SingleQuote {
            r.consume(1);
            return Action::Ignore;
        }
        let mut admitted = self.profile.quote(kind) && !word;
        if self.profile == Profile::Html {
            admitted &= self.html_tag && self.html_equals;
            self.html_equals = false;
        }
        let raw =
            kind == Kind::Backtick || self.profile == Profile::Toml && kind == Kind::SingleQuote;
        self.mode = Mode::Quote {
            kind,
            start: r.at,
            raw,
            escaped: false,
        };
        r.consume(1);
        Action::Open(kind, admitted)
    }

    fn line(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        state: (char, usize, Context),
    ) -> Result<Action, Pause> {
        let (c, start, context) = state;
        if self.profile == Profile::C && c == '\\' && matches!(r.peek(text, 1)?, Some('\r' | '\n'))
        {
            let crlf = r.peek(text, 1)? == Some('\r') && r.peek(text, 2)? == Some('\n');
            r.consume(if crlf { 3 } else { 2 });
            return Ok(Action::Ignore);
        }
        r.consume(1);
        if matches!(c, '\n' | '\r') {
            self.mode = Mode::Code;
            Ok(Action::Excluded(start..r.at, context))
        } else {
            Ok(Action::Ignore)
        }
    }

    fn block(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        state: (usize, usize),
    ) -> Result<Action, Pause> {
        let (start, depth) = state;
        if self.profile == Profile::Rust && r.prefix(text, "/*")? {
            if depth == MAX_DEPTH {
                return Err(Pause::Limit(Limit::Depth));
            }
            self.mode = Mode::Block(start, depth + 1);
            self.maximum_depth = self.maximum_depth.max(depth + 1);
            r.consume(2);
            return Ok(Action::Ignore);
        }
        if r.prefix(text, "*/")? {
            r.consume(2);
            self.mode = if depth == 1 {
                Mode::Code
            } else {
                Mode::Block(start, depth - 1)
            };
            return Ok(if depth == 1 {
                Action::Excluded(start..r.at, Context::Comment)
            } else {
                Action::Ignore
            });
        }
        r.consume(1);
        Ok(Action::Ignore)
    }

    fn html_comment(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        start: usize,
    ) -> Result<Action, Pause> {
        if r.prefix(text, "-->")? {
            r.consume(3);
            self.mode = Mode::Code;
            return Ok(Action::Excluded(start..r.at, Context::Comment));
        }
        r.consume(1);
        Ok(Action::Ignore)
    }

    fn quoted(&mut self, r: &mut Reader, c: char) -> Action {
        let Mode::Quote {
            kind,
            start,
            raw,
            escaped,
        } = self.mode
        else {
            unreachable!("quoted pair mode")
        };
        r.consume(1);
        if invalid_newline(self.profile, (kind, raw, escaped), c) {
            return Action::Refuse(Reason::AmbiguousQuote);
        }
        if !escaped && c == kind.characters().1 {
            self.mode = Mode::Code;
            return Action::Close(kind);
        }
        self.mode = Mode::Quote {
            kind,
            start,
            raw,
            escaped: continued_cr(self.profile, escaped, c) || !raw && !escaped && c == '\\',
        };
        Action::Ignore
    }

    fn marker(&mut self, r: &mut Reader, c: char, state: (char, usize)) -> Action {
        let (marker, start) = state;
        r.consume(1);
        if c == marker {
            self.mode = Mode::Code;
            Action::Excluded(start..r.at, Context::Literal)
        } else {
            Action::Ignore
        }
    }

    fn fence(
        &mut self,
        r: &mut Reader,
        text: RopeSlice<'_>,
        state: (usize, &'static str),
    ) -> Result<Action, Pause> {
        let (start, end) = state;
        if r.line_start
            && r.prefix(text, end)?
            && r.peek(text, end.len())?
                .is_none_or(|c| c.is_whitespace() || c == '`' || c == '~')
        {
            r.consume(end.len());
            self.mode = Mode::Code;
            return Ok(Action::Excluded(start..r.at, Context::Literal));
        }
        r.consume(1);
        Ok(Action::Ignore)
    }
}

fn quote(c: char) -> Option<Kind> {
    match c {
        '"' => Some(Kind::DoubleQuote),
        '\'' => Some(Kind::SingleQuote),
        '`' => Some(Kind::Backtick),
        _ => None,
    }
}

fn rust_character(r: &mut Reader, text: RopeSlice<'_>) -> Result<bool, Pause> {
    let escaped = r.peek(text, 1)? == Some('\\');
    let end = if escaped { 3 } else { 2 };
    Ok(r.peek(text, end)? == Some('\''))
}

fn inline_org(r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Option<Action>, Pause> {
    if r.previous.is_none_or(|p| !p.is_alphanumeric()) && c == 's' && r.prefix(text, "src_")? {
        Ok(Some(Action::Refuse(Reason::LiteralContext)))
    } else {
        Ok(None)
    }
}

fn rust_ambiguity(r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Option<Reason>, Pause> {
    if c == 'r' && matches!(r.peek(text, 1)?, Some('"' | '#')) {
        return Ok(Some(Reason::AmbiguousQuote));
    }
    if c == '\'' && !rust_character(r, text)? {
        return Ok(Some(Reason::AmbiguousQuote));
    }
    Ok(None)
}

fn toml_ambiguity(r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<Option<Reason>, Pause> {
    if matches!(c, '\'' | '"') && r.prefix(text, &c.to_string().repeat(3))? {
        Ok(Some(Reason::AmbiguousQuote))
    } else {
        Ok(None)
    }
}

fn markdown_indent(r: &mut Reader, text: RopeSlice<'_>) -> Result<bool, Pause> {
    for offset in 0..4 {
        match r.peek(text, offset)? {
            Some(' ') => (),
            Some('\t') => return Ok(true),
            _ => return Ok(false),
        }
    }
    Ok(true)
}

fn triple_quote(r: &mut Reader, text: RopeSlice<'_>, c: char) -> Result<bool, Pause> {
    Ok(matches!(c, '\'' | '"') && r.prefix(text, &c.to_string().repeat(3))?)
}

fn invalid_newline(profile: Profile, policy: (Kind, bool, bool), c: char) -> bool {
    let (kind, raw, escaped) = policy;
    matches!(c, '\r' | '\n')
        && !raw
        && profile.single_line(kind)
        && !(escaped && profile.continued_line())
}

fn continued_cr(profile: Profile, escaped: bool, c: char) -> bool {
    escaped && c == '\r' && profile.continued_line()
}

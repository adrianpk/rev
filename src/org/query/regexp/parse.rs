use super::*;
struct Fragment {
    start: usize,
    exits: Vec<(usize, bool)>,
}
pub(super) struct Parser {
    input: Vec<char>,
    at: usize,
    depth: usize,
    maximum: usize,
    states: Vec<State>,
}
impl Parser {
    pub(super) fn new(source: &str, maximum: usize) -> Result<Self, String> {
        if source.is_empty() || source.len() > 65536 || !(1..=128).contains(&maximum) {
            return Err("Regexp requires 1..65536 bytes and depth 1..128".into());
        }
        // Both native and escaped Org grouping/alternation spellings are data.
        let source = normalize(source);
        Ok(Self {
            input: source.chars().collect(),
            at: 0,
            depth: 0,
            maximum,
            states: Vec::new(),
        })
    }
    pub(super) fn compile(mut self) -> Result<Regexp, String> {
        let root = self.alternative()?;
        if self.at != self.input.len() {
            return Err("Unexpected regexp closing group".into());
        }
        let end = self.push(State::Accept)?;
        self.patch(root.exits, end);
        Ok(Regexp {
            states: self.states,
            start: root.start,
        })
    }
    fn push(&mut self, state: State) -> Result<usize, String> {
        if self.states.len() == 512 {
            return Err("Regexp exceeds 512 states".into());
        }
        let index = self.states.len();
        self.states.push(state);
        Ok(index)
    }
    fn patch(&mut self, exits: Vec<(usize, bool)>, next: usize) {
        for (index, second) in exits {
            match &mut self.states[index] {
                State::Consume(_, n)
                | State::Start(n)
                | State::End(n)
                | State::WordStart(n)
                | State::WordEnd(n) => *n = next,
                State::Split(a, b) => {
                    if second {
                        *b = next
                    } else {
                        *a = next
                    }
                }
                State::Accept => unreachable!("accept has no exits"),
            }
        }
    }
    fn alternative(&mut self) -> Result<Fragment, String> {
        let mut left = self.sequence()?;
        while self.take('|') {
            let right = self.sequence()?;
            let start = self.push(State::Split(left.start, right.start))?;
            left.exits.extend(right.exits);
            left = Fragment {
                start,
                exits: left.exits,
            };
        }
        Ok(left)
    }
    fn sequence(&mut self) -> Result<Fragment, String> {
        let mut result: Option<Fragment> = None;
        while self.peek().is_some_and(|c| !matches!(c, '|' | ')')) {
            let next = self.repeated()?;
            if let Some(old) = result {
                self.patch(old.exits, next.start);
                result = Some(Fragment {
                    start: old.start,
                    exits: next.exits,
                });
            } else {
                result = Some(next);
            }
        }
        result.ok_or_else(|| "Empty regexp branch/group".into())
    }
    fn repeated(&mut self) -> Result<Fragment, String> {
        let fragment = self.primary()?;
        let Some(op) = self.peek().filter(|c| "*+?".contains(*c)) else {
            return Ok(fragment);
        };
        self.at += 1;
        let split = self.push(State::Split(fragment.start, usize::MAX))?;
        let result = match op {
            '*' | '+' => {
                self.patch(fragment.exits, split);
                Fragment {
                    start: if op == '*' { split } else { fragment.start },
                    exits: vec![(split, true)],
                }
            }
            '?' => {
                let mut exits = fragment.exits;
                exits.push((split, true));
                Fragment {
                    start: split,
                    exits,
                }
            }
            _ => unreachable!("quantifier"),
        };
        if self.peek().is_some_and(|c| "*+?".contains(c)) {
            return Err("Repeated regexp quantifier".into());
        }
        Ok(result)
    }
    fn primary(&mut self) -> Result<Fragment, String> {
        if self.take('(') {
            return self.group();
        }
        let c = self.next().ok_or("Unfinished regexp")?;
        let state = self.state(c)?;
        let start = self.push(state)?;
        Ok(Fragment {
            start,
            exits: vec![(start, false)],
        })
    }
    fn state(&mut self, c: char) -> Result<State, String> {
        if "*+?{}".contains(c) {
            return Err("Unsupported or misplaced regexp operator".into());
        }
        match c {
            '^' => Ok(State::Start(usize::MAX)),
            '$' => Ok(State::End(usize::MAX)),
            '\\' => self.escaped_state(),
            _ => Ok(State::Consume(self.atom(c)?, usize::MAX)),
        }
    }
    fn atom(&mut self, c: char) -> Result<Atom, String> {
        match c {
            '.' => Ok(Atom::Any),
            '[' => self.class(),
            c => Ok(Atom::Literal(c)),
        }
    }
    fn group(&mut self) -> Result<Fragment, String> {
        self.depth += 1;
        if self.depth > self.maximum {
            return Err("Regexp exceeds query depth".into());
        }
        let result = self.alternative()?;
        self.depth -= 1;
        if !self.take(')') {
            return Err("Unclosed regexp group".into());
        }
        Ok(result)
    }
    fn escaped_state(&mut self) -> Result<State, String> {
        if self.take('<') {
            return Ok(State::WordStart(usize::MAX));
        }
        if self.take('>') {
            return Ok(State::WordEnd(usize::MAX));
        }
        Ok(State::Consume(self.escape()?, usize::MAX))
    }
    fn escape(&mut self) -> Result<Atom, String> {
        match self.next().ok_or("Unfinished regexp escape")? {
            'n' => Ok(Atom::Literal('\n')),
            'r' => Ok(Atom::Literal('\r')),
            't' => Ok(Atom::Literal('\t')),
            's' => Ok(Atom::Space),
            'w' => Ok(Atom::Word),
            c if c.is_ascii_alphanumeric() => {
                Err("Unsupported regexp escape or backreference".into())
            }
            c => Ok(Atom::Literal(c)),
        }
    }
    fn class(&mut self) -> Result<Atom, String> {
        let negative = self.take('^');
        let mut ranges = Vec::new();
        while self.peek().is_some_and(|c| c != ']') {
            let first = self.class_char()?;
            let last = if self.peek() == Some('-') && self.input.get(self.at + 1) != Some(&']') {
                self.at += 1;
                self.class_char()?
            } else {
                first
            };
            if first > last {
                return Err("Inverted regexp class range".into());
            }
            ranges.push((first, last));
        }
        if ranges.is_empty() || !self.take(']') {
            return Err("Empty or unclosed regexp class".into());
        }
        Ok(Atom::Class { ranges, negative })
    }
    fn class_char(&mut self) -> Result<char, String> {
        let c = self.next().ok_or("Unfinished regexp class")?;
        if c == '\\' {
            match self.escape()? {
                Atom::Literal(c) => Ok(c),
                _ => Err("Regexp classes require literal endpoints".into()),
            }
        } else {
            Ok(c)
        }
    }
    fn peek(&self) -> Option<char> {
        self.input.get(self.at).copied()
    }
    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.at += 1;
        Some(c)
    }
    fn take(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.at += 1;
            true
        } else {
            false
        }
    }
}

fn normalize(source: &str) -> String {
    let mut chars = source.chars();
    let mut result = String::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next() {
            Some(c) if "()|".contains(c) => result.push(c),
            Some(c) => result.extend(['\\', c]),
            None => result.push('\\'),
        }
    }
    result
}

use super::{Comparison, MatchQuery, Node, Operator};

pub(super) struct Parser<'a> {
    pub(super) source: &'a str,
    pub(super) position: usize,
    depth: usize,
    nodes: Vec<Node>,
}
impl<'a> Parser<'a> {
    pub(super) fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            depth: 0,
            nodes: Vec::new(),
        }
    }
    pub(super) fn query(mut self) -> Result<MatchQuery, String> {
        self.space();
        let mut root = if self.peek() == Some('/') {
            self.push(Node::All)?
        } else {
            self.or(false)?
        };
        root = self.todo_suffix(root)?;
        if self.peek().is_some() {
            return self.fail("unexpected trailing match input");
        }
        Ok(MatchQuery {
            nodes: self.nodes,
            root,
        })
    }
    fn todo_suffix(&mut self, mut root: usize) -> Result<usize, String> {
        if self.eat('/') {
            let unfinished = self.eat('!');
            let mut todo = if self.peek().is_none() && unfinished {
                self.push(Node::All)?
            } else {
                self.or(true)?
            };
            if unfinished {
                let state = self.push(Node::Unfinished)?;
                todo = self.push(Node::And(todo, state))?;
            }
            root = self.push(Node::And(root, todo))?;
        }
        Ok(root)
    }
    fn push(&mut self, node: Node) -> Result<usize, String> {
        if self.nodes.len() == 512 {
            return self.fail("match exceeds 512 nodes");
        }
        let index = self.nodes.len();
        self.nodes.push(node);
        Ok(index)
    }
    fn or(&mut self, todo: bool) -> Result<usize, String> {
        let mut left = self.and(todo)?;
        while self.eat('|') {
            let right = self.and(todo)?;
            left = self.push(Node::Or(left, right))?;
        }
        Ok(left)
    }
    fn and(&mut self, todo: bool) -> Result<usize, String> {
        let mut left = self.unary(todo)?;
        while self.and_next() {
            let right = self.unary(todo)?;
            left = self.push(Node::And(left, right))?;
        }
        Ok(left)
    }
    fn and_next(&mut self) -> bool {
        self.eat('&') || matches!(self.peek(), Some('+' | '-' | '!'))
    }
    fn unary(&mut self, todo: bool) -> Result<usize, String> {
        if self.depth > 32 {
            return self.fail("match exceeds depth 32");
        }
        self.depth += 1;
        let result = self.signed(todo);
        self.depth -= 1;
        result
    }
    fn signed(&mut self, todo: bool) -> Result<usize, String> {
        if self.eat('-') || self.eat('!') {
            let node = self.unary(todo)?;
            return self.push(Node::Not(node));
        }
        if self.eat('+') {
            return self.unary(todo);
        }
        if self.eat('(') {
            let node = self.or(todo)?;
            if !self.eat(')') {
                return self.fail("missing closing parenthesis");
            }
            return Ok(node);
        }
        self.atom(todo)
    }
    fn atom(&mut self, todo: bool) -> Result<usize, String> {
        let name = self.name()?;
        if let Some(operator) = self.operator() {
            if todo {
                return self.fail("TODO suffix accepts states, not property comparisons");
            }
            let key = crate::org::property_name(&name)?;
            let present = self.eat('*');
            let value = self.value()?;
            return self.push(Node::Compare(Comparison {
                key,
                operator,
                present,
                value,
            }));
        }
        if !crate::org::visibility::tag_name(&name) {
            return self.fail("invalid tag or TODO name");
        }
        self.push(if todo {
            Node::Todo(name)
        } else {
            Node::Tag(name)
        })
    }
    fn operator(&mut self) -> Option<Operator> {
        self.space();
        for (text, operator) in [
            ("<=", Operator::LessEqual),
            (">=", Operator::GreaterEqual),
            ("!=", Operator::NotEqual),
            ("<>", Operator::NotEqual),
            ("/=", Operator::NotEqual),
            ("==", Operator::Equal),
            ("=", Operator::Equal),
            ("<", Operator::Less),
            (">", Operator::Greater),
        ] {
            if self.source[self.position..].starts_with(text) {
                self.position += text.len();
                return Some(operator);
            }
        }
        None
    }
    pub(super) fn eat(&mut self, character: char) -> bool {
        self.space();
        if self.peek() != Some(character) {
            return false;
        }
        self.position += character.len_utf8();
        true
    }
    pub(super) fn peek(&mut self) -> Option<char> {
        self.space();
        self.source[self.position..].chars().next()
    }
    pub(super) fn space(&mut self) {
        while self.source[self.position..].starts_with(char::is_whitespace) {
            self.position += self.source[self.position..]
                .chars()
                .next()
                .expect("whitespace")
                .len_utf8();
        }
    }
    pub(super) fn fail<T>(&self, reason: &str) -> Result<T, String> {
        Err(format!("Org match at byte {}: {reason}", self.position))
    }
}

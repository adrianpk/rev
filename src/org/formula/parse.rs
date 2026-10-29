use super::{
    lex::{Budget, Kind, Lexer, Token},
    *,
};
mod expression;
mod format;
pub(super) struct Parser<'a> {
    text: &'a str,
    tokens: Vec<Token>,
    at: usize,
    program: Program,
    budget: Budget<'a>,
}
impl<'a> Parser<'a> {
    pub(super) fn new(
        text: &'a str,
        limits: Limits,
        cancel: &'a Cancellation,
    ) -> Result<Self, Error> {
        let mut lexer = Lexer::new(text, limits, cancel)?;
        let tokens = lexer.scan()?;
        Ok(Self {
            text,
            tokens,
            at: 0,
            program: Program {
                nodes: Vec::new(),
                assignments: Vec::new(),
            },
            budget: lexer.budget,
        })
    }
    pub(super) fn program(mut self) -> Result<Program, Error> {
        loop {
            let assignment = self.assignment()?;
            self.program.assignments.push(assignment);
            match self.token().kind {
                Kind::End => return Ok(self.program),
                Kind::Assignments => self.at += 1,
                _ => return Err(self.error(ErrorKind::Syntax, "expected :: between assignments")),
            }
        }
    }
    fn assignment(&mut self) -> Result<Assignment, Error> {
        let target = self.field()?;
        if !target.target() {
            return Err(Error::new(
                ErrorKind::Syntax,
                target.source,
                "assignment requires an absolute field or column target",
            ));
        }
        if !matches!(self.token().kind, Kind::Equal) {
            return Err(self.error(ErrorKind::Syntax, "expected = after assignment target"));
        }
        self.at += 1;
        let expression = self.expression(0, 0)?;
        let decimals = self.format()?;
        let source = target.source.start..self.tokens[self.at - 1].source.end;
        Ok(Assignment {
            target,
            expression,
            decimals,
            source,
        })
    }
    fn field(&mut self) -> Result<Reference, Error> {
        let Kind::Reference(reference) = self.token().kind.clone() else {
            return Err(self.error(ErrorKind::Syntax, "expected an Org field reference"));
        };
        self.at += 1;
        Ok(reference)
    }
    fn push(&mut self, expression: Expression, source: Range<usize>) -> Result<usize, Error> {
        self.budget.step(source.start)?;
        if self.program.nodes.len() >= 100000 {
            return Err(self.error(ErrorKind::Limit, "formula exceeds 100000 syntax nodes"));
        }
        let id = self.program.nodes.len();
        self.program.nodes.push(Node { expression, source });
        Ok(id)
    }
    fn token(&self) -> &Token {
        &self.tokens[self.at]
    }
    fn error(&self, kind: ErrorKind, message: &str) -> Error {
        Error::new(kind, self.token().source.clone(), message)
    }
}
impl Reference {
    fn target(&self) -> bool {
        self.column.as_ref().is_some_and(Component::absolute)
            && self.row.as_ref().is_none_or(Component::absolute)
    }
}
impl Component {
    fn absolute(&self) -> bool {
        matches!(self.coordinate, Coordinate::Absolute(_))
    }
}

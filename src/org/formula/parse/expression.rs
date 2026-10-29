use super::*;
impl Parser<'_> {
    pub(super) fn expression(&mut self, minimum: u8, depth: usize) -> Result<usize, Error> {
        self.budget.step(self.token().source.start)?;
        if depth > self.budget.limits.depth {
            return Err(self.error(
                ErrorKind::Limit,
                &format!("formula exceeds depth {}", self.budget.limits.depth),
            ));
        }
        let mut left = self.value(depth)?;
        while let Kind::Operator(operator) = self.token().kind {
            let (precedence, next) = operator.binding();
            if precedence < minimum {
                break;
            }
            self.at += 1;
            let right = self.expression(next, depth + 1)?;
            let source =
                self.program.nodes[left].source.start..self.program.nodes[right].source.end;
            left = self.push(Expression::Binary(operator, left, right), source)?;
        }
        Ok(left)
    }
    fn value(&mut self, depth: usize) -> Result<usize, Error> {
        let token = self.token().clone();
        match token.kind {
            Kind::Number(value) => {
                self.at += 1;
                self.push(Expression::Number(value), token.source)
            }
            Kind::Reference(_) => self.range(),
            Kind::Operator(Operator::Add | Operator::Subtract) => self.unary(depth),
            Kind::Open => self.group(depth),
            Kind::Name => self.call(depth),
            _ => Err(self.error(ErrorKind::Syntax, "expected a formula value")),
        }
    }
    fn unary(&mut self, depth: usize) -> Result<usize, Error> {
        let token = self.token().clone();
        let Kind::Operator(operator) = token.kind else {
            unreachable!("classified unary operator")
        };
        self.at += 1;
        let child = self.expression(40, depth + 1)?;
        self.push(
            Expression::Unary(operator, child),
            token.source.start..self.program.nodes[child].source.end,
        )
    }
    fn group(&mut self, depth: usize) -> Result<usize, Error> {
        let start = self.token().source.start;
        self.at += 1;
        let child = self.expression(0, depth + 1)?;
        let end = self.close()?;
        self.push(Expression::Group(child), start..end)
    }
    fn range(&mut self) -> Result<usize, Error> {
        let start = self.field()?;
        if !matches!(self.token().kind, Kind::Range) {
            let source = start.source.clone();
            return self.push(Expression::Field(start), source);
        }
        self.at += 1;
        let end = self.field()?;
        let source = start.source.start..end.source.end;
        self.push(Expression::Range(start, end), source)
    }
    fn call(&mut self, depth: usize) -> Result<usize, Error> {
        let source = self.token().source.clone();
        let function = Function::named(&self.text[source.clone()]).ok_or_else(|| {
            self.error(
                ErrorKind::Unsupported,
                "unsupported function or named reference",
            )
        })?;
        self.at += 1;
        if !matches!(self.token().kind, Kind::Open) {
            return Err(self.error(ErrorKind::Syntax, "expected function argument list"));
        }
        self.at += 1;
        let arguments = self.arguments(depth)?;
        let end = self.close()?;
        if !function.arity(arguments.len()) {
            return Err(Error::new(
                ErrorKind::Syntax,
                source.start..end,
                "invalid function argument count",
            ));
        }
        self.push(Expression::Call(function, arguments), source.start..end)
    }
    fn arguments(&mut self, depth: usize) -> Result<Vec<usize>, Error> {
        let mut arguments = Vec::new();
        if matches!(self.token().kind, Kind::Close) {
            return Ok(arguments);
        }
        loop {
            arguments.push(self.expression(0, depth + 1)?);
            if !matches!(self.token().kind, Kind::Comma) {
                return Ok(arguments);
            }
            self.at += 1;
        }
    }
    fn close(&mut self) -> Result<usize, Error> {
        if !matches!(self.token().kind, Kind::Close) {
            return Err(self.error(ErrorKind::Syntax, "expected closing parenthesis"));
        }
        let end = self.token().source.end;
        self.at += 1;
        Ok(end)
    }
}
impl Operator {
    fn binding(self) -> (u8, u8) {
        match self {
            Self::Add | Self::Subtract => (10, 11),
            Self::Divide => (20, 21),
            Self::Multiply => (30, 31),
            Self::Power => (50, 50),
        }
    }
}

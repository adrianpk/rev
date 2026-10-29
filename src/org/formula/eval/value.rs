use super::*;
impl Engine<'_> {
    pub(super) fn expression(&mut self, cell: usize, root: usize) -> Result<f64, Error> {
        // Postorder visits only the selected expression. A flat left-associated
        // tree needs no recursive Rust frames or repeated prefix evaluation.
        let mut pending = vec![(root, false)];
        let mut values = Vec::new();
        while let Some((id, done)) = pending.pop() {
            self.work.charge(1)?;
            let expression = &self.program.nodes[id].expression;
            if done {
                self.reduce(cell, expression, &mut values).map_err(|e| {
                    e.located(
                        self.grid.location(cell),
                        self.program.nodes[id].source.clone(),
                    )
                })?;
            } else {
                schedule(id, expression, &mut pending);
            }
        }
        scalar(values.pop().expect("parsed expression value"))
    }
    fn reduce(
        &mut self,
        cell: usize,
        expression: &Expression,
        values: &mut Vec<Value>,
    ) -> Result<(), Error> {
        let value = match expression {
            Expression::Number(number) => Value::Number(*number),
            Expression::Field(reference) => self.input(self.grid.resolve(reference, cell)?)?,
            Expression::Range(start, end) => Value::Range(self.grid.rectangle(start, end, cell)?),
            _ => self.operation(expression, values)?,
        };
        values.push(value);
        Ok(())
    }
    fn operation(
        &mut self,
        expression: &Expression,
        values: &mut Vec<Value>,
    ) -> Result<Value, Error> {
        match expression {
            Expression::Unary(operator, _) => {
                let value = scalar(values.pop().expect("unary operand"))?;
                numeric::finite(if *operator == Operator::Subtract {
                    -value
                } else {
                    value
                })
                .map(Value::Number)
            }
            Expression::Binary(operator, _, _) => {
                let right = scalar(values.pop().expect("right operand"))?;
                let left = scalar(values.pop().expect("left operand"))?;
                numeric::binary(*operator, left, right).map(Value::Number)
            }
            Expression::Group(_) => Ok(values.pop().expect("group value")),
            Expression::Call(function, arguments) => {
                let start = values.len() - arguments.len();
                let result = self.call(*function, &values[start..])?;
                values.truncate(start);
                Ok(Value::Number(result))
            }
            _ => unreachable!("classified expression operation"),
        }
    }
    pub(super) fn input(&mut self, cell: usize) -> Result<Value, Error> {
        self.work.charge(1)?;
        if let Some(value) = self.values[cell] {
            return Ok(Value::Number(value));
        }
        if self.owners[cell].is_some() {
            unreachable!("topologically ordered formula input");
        }
        if self.inputs[cell].is_none() {
            let text = &self.grid.cells[cell];
            self.work.charge(text.len())?;
            self.inputs[cell] = Some(parse_input(text.trim())?);
        }
        Ok(match self.inputs[cell].expect("captured input") {
            Input::Number(n) => Value::Number(n),
            Input::Text => Value::Text,
        })
    }
}
fn parse_input(text: &str) -> Result<Input, Error> {
    if text.is_empty() {
        return Ok(Input::Number(0.0));
    }
    match text.parse::<f64>() {
        Ok(number) => numeric::finite(number).map(Input::Number),
        Err(_) => Ok(Input::Text),
    }
}
fn schedule(id: usize, expression: &Expression, pending: &mut Vec<(usize, bool)>) {
    pending.push((id, true));
    match expression {
        Expression::Unary(_, child) | Expression::Group(child) => pending.push((*child, false)),
        Expression::Binary(_, left, right) => pending.extend([(*right, false), (*left, false)]),
        Expression::Call(_, arguments) => {
            pending.extend(arguments.iter().rev().map(|a| (*a, false)))
        }
        _ => {}
    }
}
pub(super) fn scalar(value: Value) -> Result<f64, Error> {
    match value {
        Value::Number(number) => Ok(number),
        _ => Err(Error::new(
            ErrorKind::Numeric,
            "text or range used as a numeric scalar",
        )),
    }
}

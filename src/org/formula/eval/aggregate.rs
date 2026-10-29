use super::*;
use value::scalar;
struct Aggregate {
    count: usize,
    sum: f64,
    mean: f64,
    minimum: f64,
    maximum: f64,
}
impl Engine<'_> {
    pub(super) fn call(&mut self, function: Function, values: &[Value]) -> Result<f64, Error> {
        match function {
            Function::Abs => numeric::finite(scalar(values[0])?.abs()),
            Function::Round => {
                let places = values
                    .get(1)
                    .copied()
                    .map(scalar)
                    .transpose()?
                    .unwrap_or(0.0);
                numeric::round(scalar(values[0])?, places, self.policy.decimal_places)
            }
            _ => self.aggregate(function, values),
        }
    }
    fn aggregate(&mut self, function: Function, values: &[Value]) -> Result<f64, Error> {
        let mut aggregate = Aggregate {
            count: 0,
            sum: 0.0,
            mean: 0.0,
            minimum: f64::INFINITY,
            maximum: f64::NEG_INFINITY,
        };
        for value in values {
            match *value {
                Value::Number(number) => aggregate.push(number, function)?,
                Value::Range(range) => self.aggregate_range(&mut aggregate, range, function)?,
                Value::Text => {}
            }
        }
        aggregate.result(function)
    }
    fn aggregate_range(
        &mut self,
        aggregate: &mut Aggregate,
        range: grid::Rectangle,
        function: Function,
    ) -> Result<(), Error> {
        for cell in range.cells(self.grid.columns) {
            if let Value::Number(number) = self.input(cell)? {
                aggregate.push(number, function)?;
            }
        }
        Ok(())
    }
}
impl Aggregate {
    fn push(&mut self, number: f64, function: Function) -> Result<(), Error> {
        self.count += 1;
        match function {
            Function::Sum => self.sum = numeric::finite(self.sum + number)?,
            Function::Mean => {
                let n = self.count as f64;
                self.mean = numeric::finite(self.mean * ((n - 1.0) / n) + number / n)?;
            }
            Function::Minimum => self.minimum = self.minimum.min(number),
            Function::Maximum => self.maximum = self.maximum.max(number),
            _ => unreachable!("aggregate function"),
        }
        Ok(())
    }
    fn result(&self, function: Function) -> Result<f64, Error> {
        if self.count == 0 && function != Function::Sum {
            return Err(Error::new(
                ErrorKind::EmptyAggregate,
                "aggregate has no numeric values",
            ));
        }
        Ok(match function {
            Function::Sum => self.sum,
            Function::Mean => self.mean,
            Function::Minimum => self.minimum,
            Function::Maximum => self.maximum,
            _ => unreachable!("aggregate function"),
        })
    }
}

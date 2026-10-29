use super::*;
mod format;
pub(super) use format::format;
pub(super) fn finite(number: f64) -> Result<f64, Error> {
    if number.is_finite() {
        Ok(number)
    } else {
        Err(Error::new(
            ErrorKind::NonFinite,
            "non-finite numeric input or result",
        ))
    }
}
pub(super) fn binary(operator: Operator, left: f64, right: f64) -> Result<f64, Error> {
    if operator == Operator::Divide && right == 0.0 {
        return Err(Error::new(ErrorKind::DivisionByZero, "division by zero"));
    }
    finite(match operator {
        Operator::Add => left + right,
        Operator::Subtract => left - right,
        Operator::Multiply => left * right,
        Operator::Divide => left / right,
        Operator::Power => left.powf(right),
    })
}
pub(super) fn round(number: f64, places: f64, maximum: usize) -> Result<f64, Error> {
    if places.fract() != 0.0 || places.abs() > maximum as f64 {
        return Err(Error::new(
            ErrorKind::Numeric,
            "round precision must be a bounded integer",
        ));
    }
    if number.abs() >= 4503599627370496.0 && places >= 0.0 {
        return Ok(number);
    }
    let factor = 10f64.powi(places as i32);
    finite(finite(number * factor)?.round() / factor)
}

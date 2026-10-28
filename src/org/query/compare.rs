use super::{Comparison, Operator, Value};
use std::cmp::Ordering;
impl Comparison {
    pub(super) fn matches(
        &self,
        actual: Option<&String>,
        units: &crate::config::DurationUnits,
    ) -> bool {
        if self.present && actual.is_none() {
            return false;
        }
        let order = match &self.value {
            Value::Text(wanted) => Some(actual.map_or("", String::as_str).cmp(wanted)),
            Value::Number(wanted) => self
                .number(actual, units)
                .and_then(|n| n.partial_cmp(wanted)),
        };
        order.is_some_and(|order| self.operator.accepts(order))
    }
    fn number(&self, value: Option<&String>, units: &crate::config::DurationUnits) -> Option<f64> {
        if self.key == "EFFORT" {
            return value.map_or(Some(0.0), |s| {
                crate::org::time::Duration::parse(s, units)
                    .ok()
                    .map(|v| v.minutes())
            });
        }
        number(value)
    }
}
fn number(value: Option<&String>) -> Option<f64> {
    let Some(value) = value else {
        return Some(0.0);
    };
    value.trim().parse::<f64>().ok().filter(|n| n.is_finite())
}
impl Operator {
    fn accepts(self, order: Ordering) -> bool {
        match self {
            Self::Equal => order == Ordering::Equal,
            Self::NotEqual => order != Ordering::Equal,
            Self::Less => order == Ordering::Less,
            Self::LessEqual => order != Ordering::Greater,
            Self::Greater => order == Ordering::Greater,
            Self::GreaterEqual => order != Ordering::Less,
        }
    }
}

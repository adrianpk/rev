use super::*;
pub(in crate::org::formula::eval) fn format(
    value: f64,
    decimals: Option<usize>,
    policy: Policy,
) -> Result<String, Error> {
    let text = if let Some(places) = decimals {
        if places > policy.decimal_places {
            return Err(Error::new(
                ErrorKind::Limit,
                "fixed precision exceeds numeric policy",
            ));
        }
        let rounded = round(value, places as f64, policy.decimal_places)?;
        let normalized = if rounded == 0.0 { 0.0 } else { rounded };
        format!("{normalized:.places$}")
    } else {
        significant(value, policy.significant_digits)?
    };
    finite(
        text.parse()
            .map_err(|_| Error::new(ErrorKind::Numeric, "formatted value is not numeric"))?,
    )?;
    Ok(text)
}
fn significant(value: f64, digits: usize) -> Result<String, Error> {
    if value == 0.0 {
        return Ok("0".into());
    }
    let scientific = format!("{:e}", value.abs());
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("native scientific spelling");
    let mut exponent: i32 = exponent.parse().expect("native decimal exponent");
    let places = digits - 1;
    let mut mantissa = round(
        mantissa.parse().expect("native mantissa"),
        places as f64,
        15,
    )?;
    if mantissa >= 10.0 {
        mantissa /= 10.0;
        exponent += 1;
    }
    let coefficient = format!("{mantissa:.places$}").replace('.', "");
    let coefficient = coefficient.trim_end_matches('0');
    let body = if exponent >= -4 && exponent < digits as i32 {
        fixed(coefficient, exponent)
    } else {
        scientific_text(coefficient, exponent)
    };
    Ok(if value.is_sign_negative() {
        format!("-{body}")
    } else {
        body
    })
}
fn fixed(coefficient: &str, exponent: i32) -> String {
    let point = exponent + 1;
    if point <= 0 {
        return format!("0.{}{coefficient}", "0".repeat((-point) as usize));
    }
    let point = point as usize;
    if point >= coefficient.len() {
        return format!("{coefficient}{}", "0".repeat(point - coefficient.len()));
    }
    format!("{}.{}", &coefficient[..point], &coefficient[point..])
}
fn scientific_text(coefficient: &str, exponent: i32) -> String {
    if coefficient.len() == 1 {
        format!("{coefficient}e{exponent}")
    } else {
        format!("{}.{}e{exponent}", &coefficient[..1], &coefficient[1..])
    }
}

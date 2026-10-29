pub(super) fn name(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(
            "Org macro name requires 1..64 ASCII letters, digits, underscores or hyphens".into(),
        );
    }
    Ok(())
}
pub(super) fn invocation(value: &str) -> Result<(&str, Vec<String>), String> {
    if value.len() > 4096 {
        return Err("Org macro invocation exceeds 4096 bytes".into());
    }
    let (key, args) = match value.split_once('(') {
        None => (value, Vec::new()),
        Some((key, args)) => (
            key,
            arguments(
                args.strip_suffix(')')
                    .ok_or("Unclosed Org macro arguments")?,
            )?,
        ),
    };
    name(key)?;
    Ok((key, args))
}
fn arguments(value: &str) -> Result<Vec<String>, String> {
    let mut args = vec![String::new()];
    let mut escaped = false;
    for c in value.chars() {
        if escaped {
            push_escaped(&mut args, c)?;
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            ',' => next_argument(&mut args)?,
            '(' | ')' => return Err("Nested Org macro argument syntax is unsupported".into()),
            _ => args.last_mut().expect("argument").push(c),
        }
    }
    if escaped {
        return Err("Incomplete Org macro argument escape".into());
    }
    Ok(args)
}
pub(super) fn substitute(body: &str, args: &[String], limit: usize) -> Result<String, String> {
    let mut result = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '$' {
            result.push(c);
        } else {
            parameter(&mut result, chars.next(), args)?;
        }
        if result.len() > limit {
            return Err("Org macro substitution byte limit reached".into());
        }
    }
    Ok(result)
}
fn parameter(out: &mut String, c: Option<char>, args: &[String]) -> Result<(), String> {
    match c {
        Some('$') => out.push('$'),
        Some(c @ '1'..='9') => out.push_str(
            args.get(c as usize - '1' as usize)
                .ok_or("Missing Org macro argument")?,
        ),
        _ => return Err("Unsupported Org macro parameter; use $1..$9 or $$".into()),
    }
    Ok(())
}

fn next_argument(args: &mut Vec<String>) -> Result<(), String> {
    if args.len() == 9 {
        return Err("Org macros admit at most nine arguments".into());
    }
    args.push(String::new());
    Ok(())
}

fn push_escaped(args: &mut [String], c: char) -> Result<(), String> {
    if c != ',' && c != '\\' {
        return Err("Unsupported Org macro argument escape".into());
    }
    args.last_mut().expect("argument").push(c);
    Ok(())
}

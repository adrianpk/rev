use super::Error;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    version: u64,
    commands: Vec<String>,
}

pub(super) fn validate(commands: &[String]) -> Result<(), Error> {
    if commands.len() > super::super::WINDOW
        || commands.iter().any(|name| {
            name.is_empty()
                || name.len() > 128
                || !name.as_bytes()[0].is_ascii_lowercase()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
    {
        return Err(Error::Bound);
    }
    Ok(())
}

pub(super) fn decode(bytes: Option<&[u8]>) -> Result<Vec<String>, Error> {
    let Some(bytes) = bytes else {
        return Ok(Vec::new());
    };
    if bytes.len() > super::BYTES {
        return Err(Error::Bound);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Error::Invalid)?;
    let document: Document = toml::from_str(text).map_err(|_| Error::Invalid)?;
    if document.version != 1 {
        return Err(Error::Unsupported);
    }
    validate(&document.commands)?;
    Ok(document.commands)
}

pub(super) fn encode(commands: &[String]) -> Result<Vec<u8>, Error> {
    validate(commands)?;
    let mut text = String::from("version = 1\ncommands = [\n");
    for name in commands {
        text.push_str(&format!("  \"{name}\",\n"));
    }
    text.push_str("]\n");
    if text.len() > super::BYTES {
        return Err(Error::Bound);
    }
    Ok(text.into_bytes())
}

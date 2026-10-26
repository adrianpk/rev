use std::io;
use std::path::{Component, Path};

/// Tree mutations accept one literal name, not a path expression.
pub(crate) fn validate(name: &str) -> io::Result<()> {
    let mut components = Path::new(name).components();
    let normal = matches!(components.next(), Some(Component::Normal(part)) if part == name);
    if name.trim().is_empty()
        || name.chars().any(char::is_control)
        || !normal
        || components.next().is_some()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "enter one nonempty filename without controls or path separators",
        ));
    }
    Ok(())
}

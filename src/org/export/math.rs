//! Shared bounded delimiter recognition; each backend owns its math policy.
pub(super) fn fragment(value: &str, at: usize, limit: usize) -> Option<(usize, &str, bool)> {
    let tail = &value[at..];
    let (open, close, display) = [
        ("$$", "$$", true),
        ("$", "$", false),
        (r"\(", r"\)", false),
        (r"\[", r"\]", true),
    ]
    .into_iter()
    .find(|(open, _, _)| tail.starts_with(open))?;
    let body = tail.get(open.len()..)?;
    let end = body.find(close)?;
    if end == 0 || end > limit {
        return None;
    }
    Some((at + open.len() + end + close.len(), &body[..end], display))
}

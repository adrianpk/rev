//! Recipe-specific evidence admission. Package provenance never masquerades as
//! the independent artifact digest required by a native download recipe.
pub(super) fn valid(recipe: &str, evidence: &str) -> bool {
    if recipe == "ruby-lsp" {
        return evidence
            .strip_prefix("gem-lock:sha256:")
            .is_some_and(sha256);
    }
    if recipe == "python-lsp-server" {
        return evidence
            .strip_prefix("pip-report:sha256:")
            .is_some_and(sha256);
    }
    if recipe == "gopls" {
        return evidence.strip_prefix("go-module:").is_some_and(go_sum);
    }
    if ["typescript-language-server", "vscode-langservers-extracted"].contains(&recipe) {
        return evidence
            .strip_prefix("npm-lock:sha256:")
            .is_some_and(sha256);
    }
    evidence.strip_prefix("sha256:").is_some_and(sha256)
}

pub(super) fn go_sum(value: &str) -> bool {
    let Some(digest) = value.strip_prefix("h1:") else {
        return false;
    };
    digest.len() == 44
        && digest.ends_with('=')
        && digest[..43]
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"+/".contains(&b))
}

pub(super) fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

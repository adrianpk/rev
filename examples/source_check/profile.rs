// These advisory diagnostics become blocking for selected owners during adoption.
// Baseline compiler/all-group diagnostics remain blocking across the workspace.
pub(super) const SELECTED: &[&str] = &[
    "redundant_clone",
    "needless_pass_by_value",
    "needless_collect",
    "trivially_copy_pass_by_ref",
    "large_types_passed_by_value",
    "clone_on_ref_ptr",
    "unused_async",
    "significant_drop_in_scrutinee",
    "significant_drop_tightening",
    "cast_possible_truncation",
    "cast_possible_wrap",
    "cast_sign_loss",
    "cast_precision_loss",
];

pub(super) fn selected(code: &str) -> bool {
    code.strip_prefix("clippy::")
        .is_some_and(|name| SELECTED.contains(&name))
}

pub(super) fn controlled(code: &str) -> bool {
    code.starts_with("clippy::")
        || [
            "warnings",
            "unsafe_code",
            "unfulfilled_lint_expectations",
            "clippy::all",
            "clippy::pedantic",
            "clippy::nursery",
            "clippy::restriction",
            "clippy::disallowed_methods",
            "clippy::missing_errors_doc",
            "clippy::missing_panics_doc",
        ]
        .contains(&code)
}

pub(super) fn print_flags() {
    for flag in flags() {
        println!("{flag}");
    }
}

pub(super) fn flags() -> Vec<String> {
    ["-Wclippy::all", "-Dunfulfilled_lint_expectations"]
        .into_iter()
        .map(str::to_owned)
        .chain(SELECTED.iter().map(|name| format!("-Wclippy::{name}")))
        .collect()
}

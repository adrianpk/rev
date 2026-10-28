// One typed recurrence owner validates timestamp suffixes without changing
// their exact source spelling. Completion invokes it explicitly.
pub(super) fn check(text: &str) -> Result<(), String> {
    super::super::repeat::check_suffix(text)
}

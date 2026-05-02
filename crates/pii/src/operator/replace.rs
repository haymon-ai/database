//! `Replace` operator: write a literal in place of the span.

pub(crate) fn apply(new_value: &str) -> String {
    new_value.to_owned()
}

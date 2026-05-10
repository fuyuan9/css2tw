use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

pub fn generate_unified_diff(old: &str, new: &str, file_name: &str) -> String {
    let diff = TextDiff::from_lines(old, new);
    let mut output = String::new();

    let _ = write!(&mut output, "--- {}\n+++ {}\n", file_name, file_name);

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            let _ = write!(&mut output, "{}{}", sign, change);
        }
    }

    output
}

use super::{inert, profile};
use tree_sitter::Node;

pub(super) fn check(node: Node<'_>, source: &str, issues: &mut Vec<String>) {
    if !["attribute_item", "inner_attribute_item"].contains(&node.kind()) {
        return;
    }
    let Some(attribute) = node.named_child(0) else {
        return;
    };
    let Some(name) = attribute.named_child(0) else {
        return;
    };
    let Some(arguments) = attribute.child_by_field_name("arguments") else {
        return;
    };
    let name = &source[name.byte_range()];
    if name == "cfg_attr" {
        conditional(node, arguments, source, issues);
    } else {
        policy(node, name, arguments, source, issues);
    }
}

fn conditional(owner: Node<'_>, arguments: Node<'_>, source: &str, issues: &mut Vec<String>) {
    let mut cursor = arguments.walk();
    let mut children = arguments.named_children(&mut cursor).peekable();
    while let Some(child) = children.next() {
        let name = &source[child.byte_range()];
        if ["allow", "expect"].contains(&name)
            && let Some(arguments) = children.peek().filter(|node| node.kind() == "token_tree")
        {
            policy(owner, name, *arguments, source, issues);
        }
        if child.kind() == "token_tree" {
            conditional(owner, child, source, issues);
        }
    }
}

fn policy(node: Node<'_>, name: &str, arguments: Node<'_>, source: &str, issues: &mut Vec<String>) {
    if !["allow", "expect"].contains(&name) {
        return;
    }
    let parts = parts(arguments, source);
    if !parts
        .iter()
        .any(|part| profile::controlled(&spelling(part, source)))
    {
        return;
    }
    let reason = parts.iter().any(|part| reason(part, source));
    let grouped = parts.iter().any(|part| group(&spelling(part, source)));
    if name != "expect" || !reason || grouped || broad(node) {
        issues.push(format!("{}: controlled lint exceptions require a local expect with a nonempty reason; group/module suppression is forbidden", node.start_position().row + 1));
    }
}

fn parts<'tree>(node: Node<'tree>, source: &str) -> Vec<Vec<Node<'tree>>> {
    let mut parts = vec![Vec::new()];
    for child in node.children(&mut node.walk()) {
        match &source[child.byte_range()] {
            "," => parts.push(Vec::new()),
            "(" | ")" => {}
            _ if !["line_comment", "block_comment"].contains(&child.kind()) => {
                parts.last_mut().unwrap().push(child);
            }
            _ => {}
        }
    }
    parts
}

fn spelling(part: &[Node<'_>], source: &str) -> String {
    part.iter().map(|node| &source[node.byte_range()]).collect()
}

fn reason(part: &[Node<'_>], source: &str) -> bool {
    let [name, equals, literal] = part else {
        return false;
    };
    if &source[name.byte_range()] != "reason" || &source[equals.byte_range()] != "=" {
        return false;
    }
    let text = &source[literal.byte_range()];
    match literal.kind() {
        "string_literal" => {
            serde_json::from_str::<String>(text).is_ok_and(|reason| !reason.trim().is_empty())
        }
        "raw_string_literal" => text
            .find('"')
            .zip(text.rfind('"'))
            .is_some_and(|(start, end)| !text[start + 1..end].trim().is_empty()),
        _ => false,
    }
}

fn group(name: &str) -> bool {
    [
        "warnings",
        "clippy::all",
        "clippy::pedantic",
        "clippy::nursery",
        "clippy::restriction",
        "clippy::cargo",
        "clippy::complexity",
        "clippy::correctness",
        "clippy::perf",
        "clippy::style",
        "clippy::suspicious",
    ]
    .contains(&name)
}

fn broad(node: Node<'_>) -> bool {
    if node.kind() == "inner_attribute_item" {
        return true;
    }
    let mut next = node.next_named_sibling();
    while let Some(sibling) = next {
        if sibling.kind() != "attribute_item" && !inert(sibling) {
            return ["mod_item", "impl_item", "trait_item"].contains(&sibling.kind());
        }
        next = sibling.next_named_sibling();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::super::inspect;

    // Only explicit item-level expectations may defer a selected rule; quoted
    // attributes, broad groups and empty reasons cannot act as exceptions.
    #[test]
    fn exceptions() {
        for (name, source, valid) in [
            (
                "reasoned",
                "#[expect(clippy::needless_pass_by_value, reason = \"Public ownership contract.\")] fn f() {}",
                true,
            ),
            (
                "raw reason",
                "#[expect(clippy::redundant_clone, reason = r#\"Captured owner.\"#)] fn f() {}",
                true,
            ),
            (
                "commented",
                "#[expect(clippy /* note */ :: redundant_clone, reason = \"Captured owner.\")] fn f() {}",
                true,
            ),
            (
                "missing reason",
                "#[expect(clippy::redundant_clone)] fn f() {}",
                false,
            ),
            (
                "blank reason",
                "#[expect(clippy::redundant_clone, reason = \"   \")] fn f() {}",
                false,
            ),
            (
                "escaped blank",
                "#[expect(clippy::redundant_clone, reason = \"\\n\")] fn f() {}",
                false,
            ),
            (
                "allow",
                "#[allow(clippy::redundant_clone)] fn f() {}",
                false,
            ),
            (
                "crate",
                "#![expect(clippy::redundant_clone, reason = \"Too broad.\")]",
                false,
            ),
            (
                "module",
                "#[expect(clippy::redundant_clone, reason = \"Too broad.\")] mod m {}",
                false,
            ),
            (
                "group",
                "#[expect(clippy::all, reason = \"Too broad.\")] fn f() {}",
                false,
            ),
            ("other rule", "#[allow(dead_code)] fn f() {}", true),
            ("baseline allow", "#[allow(clippy::eq_op)] fn f() {}", false),
            (
                "conditional allow",
                "#[cfg_attr(test, allow(clippy::redundant_clone))] fn f() {}",
                false,
            ),
            (
                "nested conditional",
                "#[cfg_attr(test, cfg_attr(unix, allow(clippy::redundant_clone)))] fn f() {}",
                false,
            ),
            (
                "conditional reason",
                "#[cfg_attr(test, expect(clippy::redundant_clone, reason = \"Captured owner.\"))] fn f() {}",
                true,
            ),
            (
                "quoted",
                "const TEXT: &str = r#\"#[allow(clippy::all)]\"#;",
                true,
            ),
        ] {
            let issues = inspect(source, false, false).unwrap();
            assert_eq!(issues.is_empty(), valid, "{name}: {issues:?}");
        }
    }
}

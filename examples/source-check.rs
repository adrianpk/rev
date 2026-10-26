#![forbid(unsafe_code)]

use std::error::Error;
use std::fs;

use tree_sitter::{Node, Parser};

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = std::env::args().skip(1);
    let mut failed = false;
    while let Some(kind) = arguments.next() {
        let path = arguments.next().ok_or("missing source path")?;
        let source = fs::read_to_string(&path)?;
        let issues = inspect(&source, kind == "--root")?;
        failed |= !issues.is_empty();
        for issue in issues {
            eprintln!("{path}:{issue}");
        }
    }
    if failed {
        return Err("source-quality check failed".into());
    }
    println!("Source quality passed: safe Rust, parameters <= 5, nesting <= 3, cyclomatic <= 10.");
    Ok(())
}

fn inspect(source: &str, root: bool) -> Result<Vec<String>, Box<dyn Error>> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into())?;
    let tree = parser.parse(source, None).ok_or("source parse canceled")?;
    let node = tree.root_node();
    let mut issues = Vec::new();
    if node.has_error() {
        issues.push("1: Rust syntax could not be analyzed".into());
    }
    if root && !forbids(node, source) {
        issues.push("1: crate root must declare #![forbid(unsafe_code)]".into());
    }
    visit(node, source, &mut issues);
    Ok(issues)
}

fn forbids(node: Node<'_>, source: &str) -> bool {
    node.named_children(&mut node.walk()).any(|child| {
        child.kind() == "inner_attribute_item"
            && source[child.byte_range()]
                .split_whitespace()
                .collect::<String>()
                == "#![forbid(unsafe_code)]"
    })
}

fn inert(node: Node<'_>) -> bool {
    [
        "string_literal",
        "raw_string_literal",
        "char_literal",
        "line_comment",
        "block_comment",
    ]
    .contains(&node.kind())
}

fn visit(node: Node<'_>, source: &str, issues: &mut Vec<String>) {
    if inert(node) {
        return;
    }
    let line = node.start_position().row + 1;
    if ["unsafe", "pointer_type", "foreign_mod_item", "extern"].contains(&node.kind()) {
        issues.push(format!(
            "{line}: first-party unsafe, raw pointers, and FFI are forbidden"
        ));
    }
    if let Some(parameters) = node.child_by_field_name("parameters") {
        parameters_check(parameters, line, issues);
    }
    if node.kind() == "function_item" {
        function_check(node, source, issues);
    }
    for child in node.children(&mut node.walk()) {
        visit(child, source, issues);
    }
}

fn parameters_check(node: Node<'_>, line: usize, issues: &mut Vec<String>) {
    let count = node
        .named_children(&mut node.walk())
        .filter(|child| !inert(*child) && child.kind() != "attribute_item")
        .count();
    if count > 5 {
        issues.push(format!(
            "{line}: {count} parameters exceeds 5, including self"
        ));
    }
}

fn function_check(node: Node<'_>, source: &str, issues: &mut Vec<String>) {
    let Some(body) = node.child_by_field_name("body") else {
        return;
    };
    let (score, nesting) = metrics(body);
    let line = node.start_position().row + 1;
    let name = node
        .child_by_field_name("name")
        .map(|name| &source[name.byte_range()])
        .unwrap_or("function");
    if score > 10 {
        issues.push(format!("{line}: {name} cyclomatic {score} exceeds 10"));
    }
    if nesting > 3 {
        issues.push(format!(
            "{line}: {name} logical nesting {nesting} exceeds 3"
        ));
    }
    println!("{name}: cyclomatic={score}, nesting={nesting}");
}

fn logical(node: Node<'_>) -> bool {
    [
        "if_expression",
        "for_expression",
        "while_expression",
        "loop_expression",
        "match_expression",
    ]
    .contains(&node.kind())
}

fn branches(node: Node<'_>) -> usize {
    if [
        "if_expression",
        "for_expression",
        "while_expression",
        "loop_expression",
        "&&",
        "||",
        "?",
        "or_pattern",
    ]
    .contains(&node.kind())
    {
        return 1;
    }
    if node.kind() == "match_block" {
        return node
            .named_children(&mut node.walk())
            .filter(|child| child.kind() == "match_arm")
            .count()
            .saturating_sub(1);
    }
    usize::from(node.kind() == "match_pattern" && node.child_by_field_name("condition").is_some())
}

fn metrics(node: Node<'_>) -> (usize, usize) {
    let mut pending = vec![(node, 0)];
    let mut score = 1;
    let mut nesting = 0;
    while let Some((node, depth)) = pending.pop() {
        if inert(node) {
            continue;
        }
        score += branches(node);
        let depth = depth + usize::from(logical(node));
        nesting = nesting.max(depth);
        pending.extend(node.children(&mut node.walk()).map(|child| (child, depth)));
    }
    (score, nesting)
}

#[cfg(test)]
mod tests {
    use super::*;

    // The checker must distinguish real Rust constructs from quoted fixtures.
    #[test]
    fn source_rules() {
        let cases = [
            (
                "safe root",
                "#![forbid(unsafe_code)]\nfn main() {}",
                true,
                "",
            ),
            ("missing root", "fn main() {}", true, "crate root"),
            (
                "quoted root",
                "const S: &str = r\"#![forbid(unsafe_code)]\";",
                true,
                "crate root",
            ),
            ("unsafe block", "fn f() { unsafe {} }", false, "unsafe"),
            (
                "raw pointer",
                "fn f(p: *const u8) {}",
                false,
                "raw pointers",
            ),
            ("ffi", "unsafe extern \"C\" { fn f(); }", false, "FFI"),
            (
                "parameters",
                "fn f(a:u8,b:u8,c:u8,d:u8,e:u8,f:u8) {}",
                false,
                "parameters",
            ),
            (
                "receiver",
                "impl A { fn f(&self,a:u8,b:u8,c:u8,d:u8,e:u8) {} }",
                false,
                "parameters",
            ),
            (
                "closure",
                "fn f() { let c = |a,b,c,d,e,f| 0; }",
                false,
                "parameters",
            ),
            (
                "nesting",
                "fn f() { loop { loop { loop { loop {} } } } }",
                false,
                "nesting",
            ),
            (
                "complexity",
                "fn f() { a()?;b()?;c()?;d()?;e()?;f()?;g()?;h()?;i()?;j()?; }",
                false,
                "cyclomatic",
            ),
            (
                "quoted tokens",
                "fn f() { let s = r###\"unsafe { /* fn x(a,b,c,d,e,f) */ }\"###; let c = 'x'; /* nested /* unsafe */ comment */ }",
                false,
                "",
            ),
            ("bad syntax", "fn f( {", false, "syntax"),
        ];
        for (name, source, root, expected) in cases {
            let issues = inspect(source, root).unwrap();
            if expected.is_empty() {
                assert!(issues.is_empty(), "{name}: {issues:?}");
            } else {
                assert!(
                    issues.iter().any(|issue| issue.contains(expected)),
                    "{name}: {issues:?}"
                );
            }
        }
    }
}

use tree_sitter::{Parser, Language};

extern "C" {
    fn tree_sitter_zig() -> Language;
}

fn main() {
    let mut parser = Parser::new();
    let language = unsafe { tree_sitter_zig() };
    parser.set_language(&language).expect("Error loading Zig grammar");

    let source = r#"
const std = @import("std");
const core = @import("./core/entity.zig");

pub const User = struct {
    id: u32,
    name: []const u8,
};

pub fn greet() void {
    const stdout = std.io.getStdOut().writer();
    try stdout.print("Hello, world!", .{});
}

test "sample test" {
    const x = 1;
    try std.testing.expect(x == 1);
}
"#;

    let tree = parser.parse(source, None).unwrap();
    print_node(tree.root_node(), 0, source);
}

fn print_node(node: tree_sitter::Node, depth: usize, source: &str) {
    let indent = " ".repeat(depth * 2);
    let text = node.utf8_text(source.as_bytes()).unwrap_or("");
    let preview = if text.len() > 30 { format!("{}...", &text[..27]) } else { text.to_string() };
    println!("{}{}: \"{}\"", indent, node.kind(), preview.replace("\n", " "));
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i) {
            print_node(child, depth + 1, source);
        }
    }
}

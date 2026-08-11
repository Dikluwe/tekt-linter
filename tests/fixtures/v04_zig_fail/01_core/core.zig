// @prompt 00_nucleo/prompts/core.md
// @layer L1
// @updated 2026-06-08
const std = @import("std");

pub fn read_file(path: []const u8) !void {
    _ = std.fs.cwd().openFile(path, .{});
}

test "read_file" {
    // coverage
}

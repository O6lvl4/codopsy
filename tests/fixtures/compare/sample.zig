// Realistic Zig code with various issues
// TODO: handle errors properly

const std = @import("std");

pub fn process(items: []const i32) i32 {
    var total: i32 = 0;
    for (items) |item| {
        if (item > 10) {
            if (item > 20) {
                total += item;
            }
        }
    }
    return total;
}

pub fn empty() void {}

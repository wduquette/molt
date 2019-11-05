# Test Script: list command.
#
# Note: TCL list syntax, i.e, conversion between Vec<Value> and String,
# is tested in Molt's Rust test suite.

test list-1.1 {no arguments} {
    list
} -ok {}

test list-1.2 {one argument} {
    list 1 2
} -ok {1 2}

test list-1.3 {two arguments} {
    list 1 2 3
} -ok {1 2 3}

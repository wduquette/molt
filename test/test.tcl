# Test Script: test command
#
# This script exercises the variants of the `test` command.

test test-1.1 {a simple test} {
    set a 5
} -ok 5

test test-2.1 {a fancy test} -setup {
} -body {
    set a 5
} -cleanup {
} -ok 5

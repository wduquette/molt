# Test Script: exit
#
# Test error cases only, since success would terminate the test suite.

test exit-1.1 {exit errors} {
    exit foo
} -error {expected integer but got "foo"}

test exit-1.2 {exit errors} {
    exit foo bar
} -error {wrong # args: should be "exit ?returnCode?"}
